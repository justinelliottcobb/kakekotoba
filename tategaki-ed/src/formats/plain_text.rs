//! Plain text file format support

use super::{FileHandler, FileMetadata, FileFormat};
use crate::{Result, TategakiError};
use crate::text_engine::{VerticalTextBuffer, TextDirection};
use std::path::Path;

/// Plain text file handler
pub struct PlainTextHandler {
    /// Default text direction for new files
    default_direction: TextDirection,
}

impl PlainTextHandler {
    /// Create new plain text handler
    pub fn new() -> Self {
        Self {
            default_direction: TextDirection::VerticalTopToBottom,
        }
    }

    /// Set default text direction
    pub fn set_default_direction(&mut self, direction: TextDirection) {
        self.default_direction = direction;
    }

    /// Read file content as string
    fn read_file_content(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file {}: {}", path.display(), e)
            )))
    }

    /// Write string content to file
    fn write_file_content(&self, content: &str, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TategakiError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create directory {}: {}", parent.display(), e)
                )))?;
        }

        std::fs::write(path, content)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write file {}: {}", path.display(), e)
            )))
    }

    /// Detect text direction from content heuristics
    fn detect_text_direction(&self, content: &str) -> TextDirection {
        // Simple heuristic: if there are more CJK characters, assume vertical
        let cjk_count = content.chars()
            .filter(|&c| self.is_cjk_character(c))
            .count();
        
        let total_chars = content.chars().filter(|&c| !c.is_whitespace()).count();
        
        if total_chars > 0 && (cjk_count as f32 / total_chars as f32) > 0.3 {
            TextDirection::VerticalTopToBottom
        } else {
            TextDirection::HorizontalLeftToRight
        }
    }

    /// Check if character is CJK (Chinese, Japanese, Korean)
    fn is_cjk_character(&self, c: char) -> bool {
        matches!(c as u32,
            0x4E00..=0x9FFF |   // CJK Unified Ideographs
            0x3400..=0x4DBF |   // CJK Extension A
            0x20000..=0x2A6DF | // CJK Extension B
            0x2A700..=0x2B73F | // CJK Extension C
            0x2B740..=0x2B81F | // CJK Extension D
            0x3040..=0x309F |   // Hiragana
            0x30A0..=0x30FF |   // Katakana
            0xFF00..=0xFFEF |   // Halfwidth and Fullwidth Forms
            0x31F0..=0x31FF |   // Katakana Phonetic Extensions
            0x3190..=0x319F |   // Kanbun
            0x31C0..=0x31EF |   // CJK Strokes
            0xAC00..=0xD7AF     // Hangul Syllables
        )
    }

    /// Normalize line endings to Unix style
    fn normalize_line_endings(&self, content: &str) -> String {
        content.replace("\r\n", "\n").replace('\r', "\n")
    }

    /// Convert Unix line endings to platform-specific
    fn platform_line_endings(&self, content: &str) -> String {
        #[cfg(windows)]
        {
            content.replace('\n', "\r\n")
        }
        #[cfg(not(windows))]
        {
            content.to_string()
        }
    }
}

impl FileHandler for PlainTextHandler {
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let content = self.read_file_content(path)?;
        let normalized_content = self.normalize_line_endings(&content);
        
        // Detect text direction from content
        let detected_direction = self.detect_text_direction(&normalized_content);
        
        // Create buffer
        let buffer = VerticalTextBuffer::from_text(&normalized_content, detected_direction)?;
        
        // Create metadata
        let metadata = FileMetadata {
            format: FileFormat::PlainText,
            text_direction: detected_direction,
            cursor_position: None,
            encoding: "UTF-8".to_string(),
            created_at: path.metadata().ok().and_then(|m| m.created().ok()),
            modified_at: path.metadata().ok().and_then(|m| m.modified().ok()),
            properties: std::collections::HashMap::new(),
        };
        
        Ok((buffer, metadata))
    }

    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        let content = buffer.as_text();
        let platform_content = self.platform_line_endings(&content);
        self.write_file_content(&platform_content, path)
    }

    fn supports_spatial_metadata(&self) -> bool {
        false // Plain text doesn't support spatial metadata
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["txt"]
    }

    fn validate(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(TategakiError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File does not exist: {}", path.display())
            )));
        }

        // Check if file is binary
        if crate::formats::utils::is_binary_file(path)? {
            return Err(TategakiError::InvalidFormat(
                "File appears to be binary, not plain text".to_string()
            ));
        }

        // Check file size (warn for very large files)
        let size = crate::formats::utils::file_size(path)?;
        if size > 100 * 1024 * 1024 { // 100MB
            eprintln!("Warning: Large file detected ({} bytes). Loading may be slow.", size);
        }

        // Check encoding
        let encoding = crate::formats::utils::detect_encoding(path)?;
        if encoding == "Unknown" {
            eprintln!("Warning: Could not detect file encoding. Assuming UTF-8.");
        }

        Ok(())
    }
}

impl Default for PlainTextHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Extended plain text handler with more advanced features
pub struct ExtendedPlainTextHandler {
    base: PlainTextHandler,
    /// Whether to preserve original line endings
    preserve_line_endings: bool,
    /// Whether to add BOM for UTF-8
    add_bom: bool,
    /// Maximum file size to load (in bytes)
    max_file_size: u64,
}

impl ExtendedPlainTextHandler {
    pub fn new() -> Self {
        Self {
            base: PlainTextHandler::new(),
            preserve_line_endings: true,
            add_bom: false,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }

    pub fn set_preserve_line_endings(&mut self, preserve: bool) {
        self.preserve_line_endings = preserve;
    }

    pub fn set_add_bom(&mut self, add_bom: bool) {
        self.add_bom = add_bom;
    }

    pub fn set_max_file_size(&mut self, size: u64) {
        self.max_file_size = size;
    }

    /// Detect original line ending style
    fn detect_line_endings(&self, content: &str) -> LineEndingStyle {
        let crlf_count = content.matches("\r\n").count();
        let lf_count = content.matches('\n').count() - crlf_count;
        let cr_count = content.matches('\r').count() - crlf_count;

        if crlf_count > lf_count && crlf_count > cr_count {
            LineEndingStyle::Windows
        } else if cr_count > lf_count {
            LineEndingStyle::Classic
        } else {
            LineEndingStyle::Unix
        }
    }

    /// Apply line ending style
    fn apply_line_endings(&self, content: &str, style: LineEndingStyle) -> String {
        let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
        
        match style {
            LineEndingStyle::Windows => normalized.replace('\n', "\r\n"),
            LineEndingStyle::Classic => normalized.replace('\n', "\r"),
            LineEndingStyle::Unix => normalized,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineEndingStyle {
    Unix,    // \n
    Windows, // \r\n
    Classic, // \r
}

impl FileHandler for ExtendedPlainTextHandler {
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        // Check file size limit
        let size = crate::formats::utils::file_size(path)?;
        if size > self.max_file_size {
            return Err(TategakiError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("File too large ({} bytes). Maximum allowed: {} bytes", size, self.max_file_size)
            )));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file: {}", e)
            )))?;

        // Detect and store original line ending style
        let line_ending_style = if self.preserve_line_endings {
            self.detect_line_endings(&content)
        } else {
            LineEndingStyle::Unix
        };

        let normalized_content = content.replace("\r\n", "\n").replace('\r', "\n");
        let detected_direction = self.base.detect_text_direction(&normalized_content);
        
        let buffer = VerticalTextBuffer::from_text(&normalized_content, detected_direction)?;
        
        let mut properties = std::collections::HashMap::new();
        properties.insert("line_endings".to_string(), format!("{:?}", line_ending_style));
        
        let metadata = FileMetadata {
            format: FileFormat::PlainText,
            text_direction: detected_direction,
            cursor_position: None,
            encoding: "UTF-8".to_string(),
            created_at: path.metadata().ok().and_then(|m| m.created().ok()),
            modified_at: path.metadata().ok().and_then(|m| m.modified().ok()),
            properties,
        };
        
        Ok((buffer, metadata))
    }

    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        let content = buffer.as_text();
        
        // Apply original line endings if preserved
        let final_content = if self.preserve_line_endings {
            if let Some(style_str) = metadata.properties.get("line_endings") {
                match style_str.as_str() {
                    "Windows" => self.apply_line_endings(&content, LineEndingStyle::Windows),
                    "Classic" => self.apply_line_endings(&content, LineEndingStyle::Classic),
                    _ => content,
                }
            } else {
                content
            }
        } else {
            content
        };

        // Add BOM if requested
        let final_content = if self.add_bom {
            format!("\u{FEFF}{}", final_content)
        } else {
            final_content
        };

        self.base.write_file_content(&final_content, path)
    }

    fn supports_spatial_metadata(&self) -> bool {
        false
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "text"]
    }

    fn validate(&self, path: &Path) -> Result<()> {
        self.base.validate(path)?;

        // Additional validation for extended handler
        let size = crate::formats::utils::file_size(path)?;
        if size > self.max_file_size {
            return Err(TategakiError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("File exceeds maximum size limit ({} > {})", size, self.max_file_size)
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_plain_text_handler_creation() {
        let handler = PlainTextHandler::new();
        assert!(!handler.supports_spatial_metadata());
        assert_eq!(handler.file_extensions(), vec!["txt"]);
    }

    #[test]
    fn test_cjk_character_detection() {
        let handler = PlainTextHandler::new();
        assert!(handler.is_cjk_character('日'));
        assert!(handler.is_cjk_character('本'));
        assert!(handler.is_cjk_character('語'));
        assert!(!handler.is_cjk_character('a'));
        assert!(!handler.is_cjk_character('1'));
    }

    #[test]
    fn test_text_direction_detection() {
        let handler = PlainTextHandler::new();
        
        // Mostly ASCII should be horizontal
        let english_text = "Hello world! This is English text.";
        assert_eq!(handler.detect_text_direction(english_text), TextDirection::HorizontalLeftToRight);
        
        // Mostly CJK should be vertical
        let japanese_text = "こんにちは世界！これは日本語のテキストです。";
        assert_eq!(handler.detect_text_direction(japanese_text), TextDirection::VerticalTopToBottom);
    }

    #[test]
    fn test_line_ending_normalization() {
        let handler = PlainTextHandler::new();
        
        assert_eq!(handler.normalize_line_endings("line1\r\nline2"), "line1\nline2");
        assert_eq!(handler.normalize_line_endings("line1\rline2"), "line1\nline2");
        assert_eq!(handler.normalize_line_endings("line1\nline2"), "line1\nline2");
    }

    #[test]
    fn test_load_save_cycle() -> Result<()> {
        let handler = PlainTextHandler::new();
        let test_content = "Test content\nWith multiple lines\n日本語も含む";
        
        // Create temporary file
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_content).unwrap();
        
        // Load file
        let (buffer, metadata) = handler.load(temp_file.path())?;
        
        // Verify content
        assert_eq!(buffer.as_text(), test_content);
        assert_eq!(metadata.format, FileFormat::PlainText);
        
        // Save to another temporary file
        let temp_file2 = NamedTempFile::new().unwrap();
        handler.save(&buffer, &metadata, temp_file2.path())?;
        
        // Verify saved content matches
        let saved_content = std::fs::read_to_string(temp_file2.path()).unwrap();
        assert_eq!(saved_content, test_content);
        
        Ok(())
    }

    #[test]
    fn test_extended_handler() {
        let mut handler = ExtendedPlainTextHandler::new();
        handler.set_preserve_line_endings(true);
        handler.set_add_bom(false);
        handler.set_max_file_size(1024 * 1024); // 1MB
        
        assert!(!handler.supports_spatial_metadata());
        assert_eq!(handler.file_extensions(), vec!["txt", "text"]);
    }

    #[test]
    fn test_line_ending_detection() {
        let handler = ExtendedPlainTextHandler::new();
        
        assert_eq!(handler.detect_line_endings("line1\r\nline2\r\n"), LineEndingStyle::Windows);
        assert_eq!(handler.detect_line_endings("line1\nline2\n"), LineEndingStyle::Unix);
        assert_eq!(handler.detect_line_endings("line1\rline2\r"), LineEndingStyle::Classic);
    }
}