//! File format support for vertical text editor
//!
//! This module provides support for reading and writing various file formats
//! with spatial metadata preservation for vertical text and programming features.

use crate::{Result, TategakiError};
use crate::text_engine::VerticalTextBuffer;
use crate::spatial::SpatialPosition;
use std::path::Path;

pub mod plain_text;
pub mod spatial_format;
pub mod markdown;
pub mod json;

pub use plain_text::*;
pub use spatial_format::*;
pub use markdown::*;
pub use json::*;

/// Supported file formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileFormat {
    /// Plain text (.txt)
    PlainText,
    /// Spatial text format with metadata (.spatial)
    Spatial,
    /// Markdown with vertical text extensions (.md)
    Markdown,
    /// JSON with spatial metadata (.json)
    Json,
    /// Auto-detect from file extension
    Auto,
}

impl FileFormat {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => FileFormat::PlainText,
            Some("spatial") => FileFormat::Spatial,
            Some("md") | Some("markdown") => FileFormat::Markdown,
            Some("json") => FileFormat::Json,
            _ => FileFormat::PlainText, // Default fallback
        }
    }

    /// Get default file extension for format
    pub fn default_extension(&self) -> &'static str {
        match self {
            FileFormat::PlainText => "txt",
            FileFormat::Spatial => "spatial",
            FileFormat::Markdown => "md",
            FileFormat::Json => "json",
            FileFormat::Auto => "txt",
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            FileFormat::PlainText => "Plain Text",
            FileFormat::Spatial => "Spatial Text Format",
            FileFormat::Markdown => "Markdown with Vertical Extensions",
            FileFormat::Json => "JSON with Spatial Metadata",
            FileFormat::Auto => "Auto-detect",
        }
    }
}

/// File metadata preserved during save/load operations
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Original file format
    pub format: FileFormat,
    /// Text direction when saved
    pub text_direction: crate::text_engine::TextDirection,
    /// Cursor position when saved
    pub cursor_position: Option<SpatialPosition>,
    /// Character encoding
    pub encoding: String,
    /// Creation timestamp
    pub created_at: Option<std::time::SystemTime>,
    /// Last modified timestamp
    pub modified_at: Option<std::time::SystemTime>,
    /// Custom properties
    pub properties: std::collections::HashMap<String, String>,
}

impl Default for FileMetadata {
    fn default() -> Self {
        Self {
            format: FileFormat::PlainText,
            text_direction: crate::text_engine::TextDirection::VerticalTopToBottom,
            cursor_position: None,
            encoding: "UTF-8".to_string(),
            created_at: None,
            modified_at: None,
            properties: std::collections::HashMap::new(),
        }
    }
}

/// File operations trait for different formats
pub trait FileHandler: Send + Sync {
    /// Load text buffer from file
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)>;

    /// Save text buffer to file
    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()>;

    /// Check if format supports spatial metadata
    fn supports_spatial_metadata(&self) -> bool;

    /// Get supported file extensions
    fn file_extensions(&self) -> Vec<&'static str>;

    /// Validate file content before loading
    fn validate(&self, path: &Path) -> Result<()>;
}

/// Main file manager for all supported formats
pub struct FileManager {
    handlers: std::collections::HashMap<FileFormat, Box<dyn FileHandler>>,
}

impl FileManager {
    /// Create new file manager with all supported formats
    pub fn new() -> Self {
        let mut handlers: std::collections::HashMap<FileFormat, Box<dyn FileHandler>> = std::collections::HashMap::new();
        
        handlers.insert(FileFormat::PlainText, Box::new(PlainTextHandler::new()));
        handlers.insert(FileFormat::Spatial, Box::new(SpatialFormatHandler::new()));
        handlers.insert(FileFormat::Markdown, Box::new(MarkdownHandler::new()));
        handlers.insert(FileFormat::Json, Box::new(JsonHandler::new()));
        
        Self { handlers }
    }

    /// Load file with automatic format detection
    pub fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let format = FileFormat::from_extension(path);
        self.load_with_format(path, format)
    }

    /// Load file with specific format
    pub fn load_with_format(&self, path: &Path, format: FileFormat) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let actual_format = if format == FileFormat::Auto {
            FileFormat::from_extension(path)
        } else {
            format
        };

        if let Some(handler) = self.handlers.get(&actual_format) {
            handler.validate(path)?;
            handler.load(path)
        } else {
            Err(TategakiError::UnsupportedFormat(format!("Unsupported format: {:?}", actual_format)))
        }
    }

    /// Save file with automatic format detection
    pub fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        let format = if metadata.format == FileFormat::Auto {
            FileFormat::from_extension(path)
        } else {
            metadata.format
        };

        if let Some(handler) = self.handlers.get(&format) {
            handler.save(buffer, metadata, path)
        } else {
            Err(TategakiError::UnsupportedFormat(format!("Unsupported format: {:?}", format)))
        }
    }

    /// Save file with specific format
    pub fn save_with_format(
        &self, 
        buffer: &VerticalTextBuffer, 
        metadata: &FileMetadata, 
        path: &Path,
        format: FileFormat
    ) -> Result<()> {
        if let Some(handler) = self.handlers.get(&format) {
            handler.save(buffer, metadata, path)
        } else {
            Err(TategakiError::UnsupportedFormat(format!("Unsupported format: {:?}", format)))
        }
    }

    /// Get list of supported formats
    pub fn supported_formats(&self) -> Vec<FileFormat> {
        self.handlers.keys().copied().collect()
    }

    /// Check if format is supported
    pub fn is_format_supported(&self, format: FileFormat) -> bool {
        self.handlers.contains_key(&format)
    }

    /// Get file extensions for format
    pub fn extensions_for_format(&self, format: FileFormat) -> Vec<&'static str> {
        if let Some(handler) = self.handlers.get(&format) {
            handler.file_extensions()
        } else {
            Vec::new()
        }
    }

    /// Check if format supports spatial metadata
    pub fn supports_spatial_metadata(&self, format: FileFormat) -> bool {
        if let Some(handler) = self.handlers.get(&format) {
            handler.supports_spatial_metadata()
        } else {
            false
        }
    }

    /// Create backup of file before saving
    pub fn create_backup(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(()); // No file to backup
        }

        let backup_path = path.with_extension(
            format!("{}.backup", path.extension().unwrap_or_default().to_string_lossy())
        );

        std::fs::copy(path, &backup_path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to create backup: {}", e))))?;

        Ok(())
    }

    /// Restore from backup
    pub fn restore_backup(&self, path: &Path) -> Result<()> {
        let backup_path = path.with_extension(
            format!("{}.backup", path.extension().unwrap_or_default().to_string_lossy())
        );

        if !backup_path.exists() {
            return Err(TategakiError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No backup file found"
            )));
        }

        std::fs::copy(&backup_path, path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to restore backup: {}", e))))?;

        // Optionally remove backup after successful restore
        std::fs::remove_file(&backup_path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to remove backup: {}", e))))?;

        Ok(())
    }

    /// List recent files (placeholder implementation)
    pub fn recent_files(&self) -> Vec<std::path::PathBuf> {
        // In a real implementation, this would read from a config file
        Vec::new()
    }

    /// Register custom file handler
    pub fn register_handler(&mut self, format: FileFormat, handler: Box<dyn FileHandler>) {
        self.handlers.insert(format, handler);
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for file operations
pub mod utils {
    use super::*;
    use std::io::Read;

    /// Detect file encoding
    pub fn detect_encoding(path: &Path) -> Result<String> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to open file: {}", e))))?;
        
        let mut buffer = [0; 1024];
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to read file: {}", e))))?;

        // Simple UTF-8 validation
        if std::str::from_utf8(&buffer[..bytes_read]).is_ok() {
            Ok("UTF-8".to_string())
        } else {
            // Could implement more sophisticated encoding detection here
            Ok("Unknown".to_string())
        }
    }

    /// Check if file is binary
    pub fn is_binary_file(path: &Path) -> Result<bool> {
        let mut file = std::fs::File::open(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to open file: {}", e))))?;
        
        let mut buffer = [0; 512];
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to read file: {}", e))))?;

        // Simple heuristic: if more than 30% of bytes are non-printable, consider binary
        let non_printable = buffer[..bytes_read]
            .iter()
            .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13) // Allow tab, LF, CR
            .count();

        Ok(non_printable as f64 / bytes_read as f64 > 0.3)
    }

    /// Get file size
    pub fn file_size(path: &Path) -> Result<u64> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to get file metadata: {}", e))))?;
        Ok(metadata.len())
    }

    /// Check if file is readable
    pub fn is_readable(path: &Path) -> bool {
        std::fs::File::open(path).is_ok()
    }

    /// Check if file is writable
    pub fn is_writable(path: &Path) -> bool {
        if path.exists() {
            std::fs::OpenOptions::new().write(true).open(path).is_ok()
        } else {
            // Check if parent directory is writable
            path.parent()
                .map(|dir| std::fs::create_dir_all(dir).is_ok())
                .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_detection() {
        assert_eq!(FileFormat::from_extension(Path::new("test.txt")), FileFormat::PlainText);
        assert_eq!(FileFormat::from_extension(Path::new("test.spatial")), FileFormat::Spatial);
        assert_eq!(FileFormat::from_extension(Path::new("test.md")), FileFormat::Markdown);
        assert_eq!(FileFormat::from_extension(Path::new("test.json")), FileFormat::Json);
        assert_eq!(FileFormat::from_extension(Path::new("test")), FileFormat::PlainText);
    }

    #[test]
    fn test_file_manager_creation() {
        let manager = FileManager::new();
        assert!(manager.is_format_supported(FileFormat::PlainText));
        assert!(manager.is_format_supported(FileFormat::Spatial));
        assert!(manager.is_format_supported(FileFormat::Markdown));
        assert!(manager.is_format_supported(FileFormat::Json));
    }

    #[test]
    fn test_file_metadata_default() {
        let metadata = FileMetadata::default();
        assert_eq!(metadata.format, FileFormat::PlainText);
        assert_eq!(metadata.encoding, "UTF-8");
        assert!(metadata.properties.is_empty());
    }

    #[test]
    fn test_format_extensions() {
        assert_eq!(FileFormat::PlainText.default_extension(), "txt");
        assert_eq!(FileFormat::Spatial.default_extension(), "spatial");
        assert_eq!(FileFormat::Markdown.default_extension(), "md");
        assert_eq!(FileFormat::Json.default_extension(), "json");
    }

    #[test]
    fn test_format_descriptions() {
        assert_eq!(FileFormat::PlainText.description(), "Plain Text");
        assert_eq!(FileFormat::Spatial.description(), "Spatial Text Format");
        assert_eq!(FileFormat::Markdown.description(), "Markdown with Vertical Extensions");
        assert_eq!(FileFormat::Json.description(), "JSON with Spatial Metadata");
    }
}