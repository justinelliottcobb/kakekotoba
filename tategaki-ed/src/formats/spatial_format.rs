//! Spatial text format with metadata support

use super::{FileHandler, FileMetadata, FileFormat};
use crate::{Result, TategakiError};
use crate::text_engine::{VerticalTextBuffer, TextDirection};
use crate::spatial::{SpatialPosition, SpatialRange};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Spatial file format handler
pub struct SpatialFormatHandler;

impl SpatialFormatHandler {
    pub fn new() -> Self {
        Self
    }
}

/// Spatial file format structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialFile {
    /// File format version
    pub version: String,
    /// Text content
    pub content: String,
    /// Spatial metadata
    pub metadata: SpatialMetadata,
}

/// Spatial metadata for the file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialMetadata {
    /// Text direction
    pub text_direction: TextDirection,
    /// Cursor position
    pub cursor_position: Option<SpatialPosition>,
    /// Selection ranges
    pub selections: Vec<SpatialRange>,
    /// Spatial annotations
    pub annotations: Vec<SpatialAnnotation>,
    /// Custom properties
    pub properties: std::collections::HashMap<String, serde_json::Value>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last modified timestamp
    pub modified_at: Option<String>,
    /// Encoding information
    pub encoding: String,
}

/// Spatial annotation for specific positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialAnnotation {
    /// Position or range
    pub range: SpatialRange,
    /// Annotation type
    pub annotation_type: AnnotationType,
    /// Annotation content
    pub content: String,
    /// Additional properties
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Types of spatial annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    /// Comment annotation
    Comment,
    /// Highlight annotation
    Highlight,
    /// Link to another position
    Link,
    /// Bookmark
    Bookmark,
    /// Code block
    CodeBlock,
    /// Custom annotation type
    Custom(String),
}

impl Default for SpatialFile {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            content: String::new(),
            metadata: SpatialMetadata::default(),
        }
    }
}

impl Default for SpatialMetadata {
    fn default() -> Self {
        Self {
            text_direction: TextDirection::VerticalTopToBottom,
            cursor_position: None,
            selections: Vec::new(),
            annotations: Vec::new(),
            properties: std::collections::HashMap::new(),
            created_at: None,
            modified_at: None,
            encoding: "UTF-8".to_string(),
        }
    }
}

impl FileHandler for SpatialFormatHandler {
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read spatial file: {}", e)
            )))?;

        let spatial_file: SpatialFile = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid spatial format: {}", e)))?;

        // Validate version
        if !self.is_version_supported(&spatial_file.version) {
            return Err(TategakiError::UnsupportedFormat(
                format!("Unsupported spatial format version: {}", spatial_file.version)
            ));
        }

        // Create buffer
        let buffer = VerticalTextBuffer::from_text(&spatial_file.content, spatial_file.metadata.text_direction)?;

        // Convert spatial metadata to file metadata
        let file_metadata = self.convert_spatial_to_file_metadata(spatial_file.metadata, path)?;

        Ok((buffer, file_metadata))
    }

    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        // Create spatial file structure
        let spatial_metadata = self.convert_file_to_spatial_metadata(metadata)?;
        
        let spatial_file = SpatialFile {
            version: "1.0".to_string(),
            content: buffer.as_text(),
            metadata: spatial_metadata,
        };

        // Serialize to JSON with pretty formatting
        let json_content = serde_json::to_string_pretty(&spatial_file)
            .map_err(|e| TategakiError::Serialization(format!("Failed to serialize spatial file: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TategakiError::Io(std::io::Error::new(
                    e.kind(),
                    format!("Failed to create directory: {}", e)
                )))?;
        }

        // Write to file
        std::fs::write(path, json_content)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write spatial file: {}", e)
            )))?;

        Ok(())
    }

    fn supports_spatial_metadata(&self) -> bool {
        true
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["spatial"]
    }

    fn validate(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(TategakiError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File does not exist: {}", path.display())
            )));
        }

        // Check if it's a valid JSON file
        let content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file: {}", e)
            )))?;

        // Try to parse as JSON
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid JSON: {}", e)))?;

        // Check required fields
        if !json_value.is_object() {
            return Err(TategakiError::InvalidFormat("Root must be an object".to_string()));
        }

        let obj = json_value.as_object().unwrap();

        if !obj.contains_key("version") {
            return Err(TategakiError::InvalidFormat("Missing 'version' field".to_string()));
        }

        if !obj.contains_key("content") {
            return Err(TategakiError::InvalidFormat("Missing 'content' field".to_string()));
        }

        if !obj.contains_key("metadata") {
            return Err(TategakiError::InvalidFormat("Missing 'metadata' field".to_string()));
        }

        // Try to deserialize completely
        let _: SpatialFile = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid spatial format: {}", e)))?;

        Ok(())
    }
}

impl SpatialFormatHandler {
    /// Check if format version is supported
    fn is_version_supported(&self, version: &str) -> bool {
        matches!(version, "1.0")
    }

    /// Convert spatial metadata to file metadata
    fn convert_spatial_to_file_metadata(&self, spatial: SpatialMetadata, path: &Path) -> Result<FileMetadata> {
        let mut properties = std::collections::HashMap::new();
        
        // Add selections as property
        if !spatial.selections.is_empty() {
            let selections_json = serde_json::to_string(&spatial.selections)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize selections: {}", e)))?;
            properties.insert("selections".to_string(), selections_json);
        }

        // Add annotations as property
        if !spatial.annotations.is_empty() {
            let annotations_json = serde_json::to_string(&spatial.annotations)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize annotations: {}", e)))?;
            properties.insert("annotations".to_string(), annotations_json);
        }

        // Add custom properties
        for (key, value) in spatial.properties {
            // If it's a JSON string, extract the actual string value
            // Otherwise, serialize the JSON value
            let string_value = match value {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            properties.insert(key, string_value);
        }

        let created_at = spatial.created_at
            .and_then(|s| s.parse::<u64>().ok())
            .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .or_else(|| path.metadata().ok().and_then(|m| m.created().ok()));

        let modified_at = spatial.modified_at
            .and_then(|s| s.parse::<u64>().ok())
            .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .or_else(|| path.metadata().ok().and_then(|m| m.modified().ok()));

        Ok(FileMetadata {
            format: FileFormat::Spatial,
            text_direction: spatial.text_direction,
            cursor_position: spatial.cursor_position,
            encoding: spatial.encoding,
            created_at,
            modified_at,
            properties,
        })
    }

    /// Convert file metadata to spatial metadata
    fn convert_file_to_spatial_metadata(&self, file_meta: &FileMetadata) -> Result<SpatialMetadata> {
        let mut spatial_meta = SpatialMetadata {
            text_direction: file_meta.text_direction,
            cursor_position: file_meta.cursor_position,
            encoding: file_meta.encoding.clone(),
            created_at: file_meta.created_at.map(|t| format!("{:?}", t)),
            modified_at: file_meta.modified_at.map(|t| format!("{:?}", t)),
            ..SpatialMetadata::default()
        };

        // Parse selections from properties
        if let Some(selections_str) = file_meta.properties.get("selections") {
            spatial_meta.selections = serde_json::from_str(selections_str)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse selections: {}", e)))?;
        }

        // Parse annotations from properties
        if let Some(annotations_str) = file_meta.properties.get("annotations") {
            spatial_meta.annotations = serde_json::from_str(annotations_str)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse annotations: {}", e)))?;
        }

        // Add other properties
        for (key, value) in &file_meta.properties {
            if key != "selections" && key != "annotations" {
                let json_value: serde_json::Value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                spatial_meta.properties.insert(key.clone(), json_value);
            }
        }

        Ok(spatial_meta)
    }

    /// Create annotation at position
    pub fn create_annotation(
        &self,
        range: SpatialRange,
        annotation_type: AnnotationType,
        content: String,
    ) -> SpatialAnnotation {
        SpatialAnnotation {
            range,
            annotation_type,
            content,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Add annotation to spatial metadata
    pub fn add_annotation(&self, spatial_meta: &mut SpatialMetadata, annotation: SpatialAnnotation) {
        spatial_meta.annotations.push(annotation);
    }

    /// Remove annotation by index
    pub fn remove_annotation(&self, spatial_meta: &mut SpatialMetadata, index: usize) -> Option<SpatialAnnotation> {
        if index < spatial_meta.annotations.len() {
            Some(spatial_meta.annotations.remove(index))
        } else {
            None
        }
    }

    /// Find annotations at position
    pub fn find_annotations_at<'a>(&self, spatial_meta: &'a SpatialMetadata, position: SpatialPosition) -> Vec<&'a SpatialAnnotation> {
        spatial_meta.annotations
            .iter()
            .filter(|ann| ann.range.contains(&position))
            .collect()
    }

    /// Get all annotations of specific type
    pub fn get_annotations_by_type<'a>(&self, spatial_meta: &'a SpatialMetadata, annotation_type: &AnnotationType) -> Vec<&'a SpatialAnnotation> {
        spatial_meta.annotations
            .iter()
            .filter(|ann| std::mem::discriminant(&ann.annotation_type) == std::mem::discriminant(annotation_type))
            .collect()
    }

    /// Validate spatial file structure
    pub fn validate_spatial_file(&self, spatial_file: &SpatialFile) -> Result<()> {
        // Check version
        if !self.is_version_supported(&spatial_file.version) {
            return Err(TategakiError::UnsupportedFormat(
                format!("Unsupported version: {}", spatial_file.version)
            ));
        }

        // Validate annotations
        for (i, annotation) in spatial_file.metadata.annotations.iter().enumerate() {
            if annotation.range.start.row > annotation.range.end.row ||
               (annotation.range.start.row == annotation.range.end.row && 
                annotation.range.start.column > annotation.range.end.column) {
                return Err(TategakiError::InvalidFormat(
                    format!("Invalid range in annotation {}: {:?}", i, annotation.range)
                ));
            }
        }

        // Validate cursor position is reasonable
        if let Some(cursor) = spatial_file.metadata.cursor_position {
            let content_lines = spatial_file.content.lines().count();
            if cursor.row >= content_lines {
                return Err(TategakiError::InvalidFormat(
                    format!("Cursor position row {} exceeds content lines {}", cursor.row, content_lines)
                ));
            }
        }

        Ok(())
    }
}

impl Default for SpatialFormatHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spatial_format_handler_creation() {
        let handler = SpatialFormatHandler::new();
        assert!(handler.supports_spatial_metadata());
        assert_eq!(handler.file_extensions(), vec!["spatial"]);
    }

    #[test]
    fn test_spatial_file_default() {
        let spatial_file = SpatialFile::default();
        assert_eq!(spatial_file.version, "1.0");
        assert!(spatial_file.content.is_empty());
        assert_eq!(spatial_file.metadata.text_direction, TextDirection::VerticalTopToBottom);
    }

    #[test]
    fn test_version_support() {
        let handler = SpatialFormatHandler::new();
        assert!(handler.is_version_supported("1.0"));
        assert!(!handler.is_version_supported("2.0"));
        assert!(!handler.is_version_supported("0.9"));
    }

    #[test]
    fn test_annotation_creation() {
        let handler = SpatialFormatHandler::new();
        let range = SpatialRange {
            start: SpatialPosition { row: 0, column: 0, byte_offset: 0 },
            end: SpatialPosition { row: 0, column: 5, byte_offset: 0 },
        };
        
        let annotation = handler.create_annotation(
            range.clone(),
            AnnotationType::Comment,
            "Test comment".to_string(),
        );

        assert_eq!(annotation.range, range);
        assert!(matches!(annotation.annotation_type, AnnotationType::Comment));
        assert_eq!(annotation.content, "Test comment");
    }

    #[test]
    fn test_save_load_cycle() -> Result<()> {
        let handler = SpatialFormatHandler::new();
        let test_content = "Test spatial content\n日本語テスト";
        
        // Create buffer and metadata
        let buffer = VerticalTextBuffer::from_text(test_content, TextDirection::VerticalTopToBottom)?;
        let mut metadata = FileMetadata {
            format: FileFormat::Spatial,
            text_direction: TextDirection::VerticalTopToBottom,
            cursor_position: Some(SpatialPosition { row: 1, column: 5, byte_offset: 0 }),
            encoding: "UTF-8".to_string(),
            ..FileMetadata::default()
        };
        
        // Add some test properties
        metadata.properties.insert("test_prop".to_string(), "test_value".to_string());
        
        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        handler.save(&buffer, &metadata, temp_file.path())?;
        
        // Load back
        let (loaded_buffer, loaded_metadata) = handler.load(temp_file.path())?;
        
        // Verify
        assert_eq!(loaded_buffer.as_text(), test_content);
        assert_eq!(loaded_metadata.format, FileFormat::Spatial);
        assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
        assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 1, column: 5, byte_offset: 0 }));
        assert_eq!(loaded_metadata.properties.get("test_prop"), Some(&"test_value".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_json_serialization() -> Result<()> {
        let mut spatial_file = SpatialFile::default();
        spatial_file.content = "Test content".to_string();
        spatial_file.metadata.cursor_position = Some(SpatialPosition { row: 0, column: 5, byte_offset: 0 });

        // Add annotation
        let annotation = SpatialAnnotation {
            range: SpatialRange {
                start: SpatialPosition { row: 0, column: 0, byte_offset: 0 },
                end: SpatialPosition { row: 0, column: 4, byte_offset: 0 },
            },
            annotation_type: AnnotationType::Highlight,
            content: "Important".to_string(),
            properties: std::collections::HashMap::new(),
        };
        spatial_file.metadata.annotations.push(annotation);
        
        // Serialize
        let json = serde_json::to_string_pretty(&spatial_file).unwrap();
        
        // Deserialize
        let deserialized: SpatialFile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.content, "Test content");
        assert_eq!(deserialized.metadata.cursor_position, Some(SpatialPosition { row: 0, column: 5, byte_offset: 0 }));
        assert_eq!(deserialized.metadata.annotations.len(), 1);
        assert!(matches!(deserialized.metadata.annotations[0].annotation_type, AnnotationType::Highlight));
        
        Ok(())
    }

    #[test]
    fn test_annotation_management() {
        let handler = SpatialFormatHandler::new();
        let mut spatial_meta = SpatialMetadata::default();
        
        // Create test annotation
        let annotation = handler.create_annotation(
            SpatialRange {
                start: SpatialPosition { row: 0, column: 0, byte_offset: 0 },
                end: SpatialPosition { row: 0, column: 5, byte_offset: 5 },
            },
            AnnotationType::Comment,
            "Test comment".to_string(),
        );

        // Add annotation
        handler.add_annotation(&mut spatial_meta, annotation);
        assert_eq!(spatial_meta.annotations.len(), 1);

        // Find annotations at position
        let position = SpatialPosition { row: 0, column: 2, byte_offset: 2 };
        let found = handler.find_annotations_at(&spatial_meta, position);
        assert_eq!(found.len(), 1);
        
        // Remove annotation
        let removed = handler.remove_annotation(&mut spatial_meta, 0);
        assert!(removed.is_some());
        assert_eq!(spatial_meta.annotations.len(), 0);
    }
}