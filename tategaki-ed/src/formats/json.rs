//! JSON format with spatial metadata support

use super::{FileHandler, FileMetadata, FileFormat};
use crate::{Result, TategakiError};
use crate::text_engine::{VerticalTextBuffer, TextDirection};
use crate::spatial::{SpatialPosition, SpatialRange};
use std::path::Path;
use serde::{Deserialize, Serialize};

/// JSON file handler with spatial metadata
pub struct JsonHandler {
    /// Pretty print JSON output
    pretty_print: bool,
    /// Include schema information
    include_schema: bool,
}

impl JsonHandler {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
            include_schema: true,
        }
    }

    pub fn set_pretty_print(&mut self, pretty: bool) {
        self.pretty_print = pretty;
    }

    pub fn set_include_schema(&mut self, include: bool) {
        self.include_schema = include;
    }
}

/// JSON document structure for vertical text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonDocument {
    /// Schema version (optional)
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    
    /// Document format version
    pub version: String,
    
    /// Document metadata
    pub metadata: JsonMetadata,
    
    /// Text content
    pub content: JsonContent,
    
    /// Spatial annotations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<JsonAnnotation>,
    
    /// Layout specifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<JsonLayout>,
}

/// JSON metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMetadata {
    /// Document title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
    
    /// Text direction
    pub text_direction: TextDirection,
    
    /// Character encoding
    #[serde(default = "default_encoding")]
    pub encoding: String,
    
    /// Language code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    
    /// Custom properties
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// JSON content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonContent {
    /// Plain text content
    pub text: String,
    
    /// Cursor position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor_position: Option<SpatialPosition>,
    
    /// Selection ranges
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selections: Vec<SpatialRange>,
    
    /// Line break information
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line_breaks: Vec<usize>,
}

/// JSON annotation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonAnnotation {
    /// Unique identifier
    pub id: String,
    
    /// Annotation type
    pub annotation_type: String,
    
    /// Spatial range
    pub range: SpatialRange,
    
    /// Annotation content/description
    pub content: String,
    
    /// Visual styling information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<JsonStyle>,
    
    /// Additional properties
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// JSON layout specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLayout {
    /// Layout type
    pub layout_type: String,
    
    /// Column specifications
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<JsonColumn>,
    
    /// Page dimensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<JsonPageSize>,
    
    /// Margins
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margins: Option<JsonMargins>,
}

/// JSON column specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonColumn {
    /// Column width
    pub width: usize,
    
    /// Column height
    pub height: usize,
    
    /// Column position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<SpatialPosition>,
    
    /// Column properties
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// JSON page size specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPageSize {
    pub width: f32,
    pub height: f32,
    pub unit: String, // "px", "mm", "in", etc.
}

/// JSON margins specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonMargins {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
    pub unit: String,
}

/// JSON style information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonStyle {
    /// Background color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    
    /// Text color
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    
    /// Font family
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    
    /// Font size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    
    /// Font weight
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    
    /// Additional style properties
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

fn default_encoding() -> String {
    "UTF-8".to_string()
}

impl Default for JsonDocument {
    fn default() -> Self {
        Self {
            schema: Some("https://schemas.tategaki.org/document/v1.0.json".to_string()),
            version: "1.0".to_string(),
            metadata: JsonMetadata::default(),
            content: JsonContent::default(),
            annotations: Vec::new(),
            layout: None,
        }
    }
}

impl Default for JsonMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            created_at: None,
            modified_at: None,
            text_direction: TextDirection::VerticalTopToBottom,
            encoding: default_encoding(),
            language: None,
            properties: std::collections::HashMap::new(),
        }
    }
}

impl Default for JsonContent {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_position: None,
            selections: Vec::new(),
            line_breaks: Vec::new(),
        }
    }
}

impl FileHandler for JsonHandler {
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to read JSON file: {}", e))))?;

        let json_doc: JsonDocument = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid JSON format: {}", e)))?;

        // Validate version
        if !self.is_version_supported(&json_doc.version) {
            return Err(TategakiError::UnsupportedFormat(
                format!("Unsupported JSON format version: {}", json_doc.version)
            ));
        }

        // Create buffer
        let buffer = VerticalTextBuffer::from_text(&json_doc.content.text, json_doc.metadata.text_direction)?;

        // Convert JSON metadata to file metadata
        let file_metadata = self.convert_json_to_file_metadata(json_doc, path)?;

        Ok((buffer, file_metadata))
    }

    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        // Create JSON document structure
        let json_doc = self.convert_file_to_json_document(buffer, metadata)?;

        // Serialize to JSON
        let json_content = if self.pretty_print {
            serde_json::to_string_pretty(&json_doc)
        } else {
            serde_json::to_string(&json_doc)
        }.map_err(|e| TategakiError::Serialization(format!("Failed to serialize JSON: {}", e)))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to create directory: {}", e))))?;
        }

        // Write to file
        std::fs::write(path, json_content)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to write JSON file: {}", e))))?;

        Ok(())
    }

    fn supports_spatial_metadata(&self) -> bool {
        true
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["json"]
    }

    fn validate(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(TategakiError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File does not exist: {}", path.display()))));
        }

        // Try to parse as JSON
        let content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to read file: {}", e))))?;

        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid JSON: {}", e)))?;

        // Check if it's an object
        if !json_value.is_object() {
            return Err(TategakiError::InvalidFormat("JSON root must be an object".to_string()));
        }

        // Try to deserialize as JsonDocument
        let _: JsonDocument = serde_json::from_str(&content)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid document structure: {}", e)))?;

        Ok(())
    }
}

impl JsonHandler {
    /// Check if version is supported
    fn is_version_supported(&self, version: &str) -> bool {
        matches!(version, "1.0")
    }

    /// Convert JSON document to file metadata
    fn convert_json_to_file_metadata(&self, json_doc: JsonDocument, path: &Path) -> Result<FileMetadata> {
        let mut properties = std::collections::HashMap::new();

        // Add annotations as property
        if !json_doc.annotations.is_empty() {
            let annotations_json = serde_json::to_string(&json_doc.annotations)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize annotations: {}", e)))?;
            properties.insert("annotations".to_string(), annotations_json);
        }

        // Add selections as property
        if !json_doc.content.selections.is_empty() {
            let selections_json = serde_json::to_string(&json_doc.content.selections)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize selections: {}", e)))?;
            properties.insert("selections".to_string(), selections_json);
        }

        // Add layout as property
        if let Some(layout) = json_doc.layout {
            let layout_json = serde_json::to_string(&layout)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize layout: {}", e)))?;
            properties.insert("layout".to_string(), layout_json);
        }

        // Add metadata properties
        properties.insert("title".to_string(), json_doc.metadata.title.unwrap_or_default());
        properties.insert("author".to_string(), json_doc.metadata.author.unwrap_or_default());
        properties.insert("language".to_string(), json_doc.metadata.language.unwrap_or_default());

        // Add custom properties
        for (key, value) in json_doc.metadata.properties {
            properties.insert(format!("json_{}", key), value.to_string());
        }

        let created_at = json_doc.metadata.created_at
            .and_then(|s| s.parse::<u64>().ok())
            .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .or_else(|| path.metadata().ok().and_then(|m| m.created().ok()));

        let modified_at = json_doc.metadata.modified_at
            .and_then(|s| s.parse::<u64>().ok())
            .map(|secs| std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
            .or_else(|| path.metadata().ok().and_then(|m| m.modified().ok()));

        Ok(FileMetadata {
            format: FileFormat::Json,
            text_direction: json_doc.metadata.text_direction,
            cursor_position: json_doc.content.cursor_position,
            encoding: json_doc.metadata.encoding,
            created_at,
            modified_at,
            properties,
        })
    }

    /// Convert file metadata to JSON document
    fn convert_file_to_json_document(&self, buffer: &VerticalTextBuffer, file_meta: &FileMetadata) -> Result<JsonDocument> {
        let mut json_doc = JsonDocument::default();

        if !self.include_schema {
            json_doc.schema = None;
        }

        // Set metadata
        json_doc.metadata.text_direction = file_meta.text_direction;
        json_doc.metadata.encoding = file_meta.encoding.clone();
        json_doc.metadata.created_at = file_meta.created_at.map(|t| format!("{:?}", t));
        json_doc.metadata.modified_at = file_meta.modified_at.map(|t| format!("{:?}", t));

        // Extract specific properties
        json_doc.metadata.title = file_meta.properties.get("title").cloned().filter(|s| !s.is_empty());
        json_doc.metadata.author = file_meta.properties.get("author").cloned().filter(|s| !s.is_empty());
        json_doc.metadata.language = file_meta.properties.get("language").cloned().filter(|s| !s.is_empty());

        // Extract custom properties
        for (key, value) in &file_meta.properties {
            if let Some(json_key) = key.strip_prefix("json_") {
                let json_value: serde_json::Value = serde_json::from_str(value)
                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));
                json_doc.metadata.properties.insert(json_key.to_string(), json_value);
            }
        }

        // Set content
        json_doc.content.text = buffer.as_text();
        json_doc.content.cursor_position = file_meta.cursor_position;

        // Parse selections from properties
        if let Some(selections_json) = file_meta.properties.get("selections") {
            json_doc.content.selections = serde_json::from_str(selections_json)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse selections: {}", e)))?;
        }

        // Parse annotations from properties
        if let Some(annotations_json) = file_meta.properties.get("annotations") {
            json_doc.annotations = serde_json::from_str(annotations_json)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse annotations: {}", e)))?;
        }

        // Parse layout from properties
        if let Some(layout_json) = file_meta.properties.get("layout") {
            json_doc.layout = serde_json::from_str(layout_json)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse layout: {}", e)))?;
        }

        Ok(json_doc)
    }

    /// Create annotation
    pub fn create_annotation(
        &self,
        id: String,
        annotation_type: String,
        range: SpatialRange,
        content: String,
    ) -> JsonAnnotation {
        JsonAnnotation {
            id,
            annotation_type,
            range,
            content,
            style: None,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Create layout specification
    pub fn create_layout(&self, layout_type: String) -> JsonLayout {
        JsonLayout {
            layout_type,
            columns: Vec::new(),
            page_size: None,
            margins: None,
        }
    }
}

impl Default for JsonHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_handler_creation() {
        let handler = JsonHandler::new();
        assert!(handler.supports_spatial_metadata());
        assert_eq!(handler.file_extensions(), vec!["json"]);
    }

    #[test]
    fn test_json_document_default() {
        let doc = JsonDocument::default();
        assert_eq!(doc.version, "1.0");
        assert!(doc.schema.is_some());
        assert_eq!(doc.metadata.text_direction, TextDirection::VerticalTopToBottom);
        assert!(doc.content.text.is_empty());
    }

    #[test]
    fn test_version_support() {
        let handler = JsonHandler::new();
        assert!(handler.is_version_supported("1.0"));
        assert!(!handler.is_version_supported("2.0"));
    }

    #[test]
    fn test_annotation_creation() {
        let handler = JsonHandler::new();
        let range = SpatialRange {
            start: SpatialPosition { row: 0, column: 0 },
            end: SpatialPosition { row: 0, column: 5 },
        };
        
        let annotation = handler.create_annotation(
            "test_id".to_string(),
            "comment".to_string(),
            range,
            "Test annotation".to_string(),
        );
        
        assert_eq!(annotation.id, "test_id");
        assert_eq!(annotation.annotation_type, "comment");
        assert_eq!(annotation.content, "Test annotation");
    }

    #[test]
    fn test_save_load_cycle() -> Result<()> {
        let handler = JsonHandler::new();
        let test_content = "Test JSON content\n日本語テスト";
        
        // Create buffer and metadata
        let buffer = VerticalTextBuffer::from_text(test_content, TextDirection::VerticalTopToBottom)?;
        let mut metadata = FileMetadata {
            format: FileFormat::Json,
            text_direction: TextDirection::VerticalTopToBottom,
            cursor_position: Some(SpatialPosition { row: 1, column: 8 }),
            encoding: "UTF-8".to_string(),
            ..FileMetadata::default()
        };
        
        // Add some properties
        metadata.properties.insert("title".to_string(), "Test Document".to_string());
        metadata.properties.insert("author".to_string(), "Test Author".to_string());
        
        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        handler.save(&buffer, &metadata, temp_file.path())?;
        
        // Verify saved content is valid JSON
        let saved_content = std::fs::read_to_string(temp_file.path()).unwrap();
        let _: JsonDocument = serde_json::from_str(&saved_content).unwrap();
        
        // Load back
        let (loaded_buffer, loaded_metadata) = handler.load(temp_file.path())?;
        
        // Verify
        assert_eq!(loaded_buffer.as_text(), test_content);
        assert_eq!(loaded_metadata.format, FileFormat::Json);
        assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
        assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 1, column: 8 }));
        assert_eq!(loaded_metadata.properties.get("title"), Some(&"Test Document".to_string()));
        assert_eq!(loaded_metadata.properties.get("author"), Some(&"Test Author".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_json_serialization() -> Result<()> {
        let mut doc = JsonDocument::default();
        doc.content.text = "Test content".to_string();
        doc.content.cursor_position = Some(SpatialPosition { row: 0, column: 5 });
        doc.metadata.title = Some("Test Title".to_string());
        
        // Add annotation
        let annotation = JsonAnnotation {
            id: "test1".to_string(),
            annotation_type: "highlight".to_string(),
            range: SpatialRange {
                start: SpatialPosition { row: 0, column: 0 },
                end: SpatialPosition { row: 0, column: 4 },
            },
            content: "Important".to_string(),
            style: None,
            properties: std::collections::HashMap::new(),
        };
        doc.annotations.push(annotation);
        
        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let deserialized: JsonDocument = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.content.text, "Test content");
        assert_eq!(deserialized.metadata.title, Some("Test Title".to_string()));
        assert_eq!(deserialized.annotations.len(), 1);
        assert_eq!(deserialized.annotations[0].id, "test1");
        
        Ok(())
    }

    #[test]
    fn test_layout_creation() {
        let handler = JsonHandler::new();
        let layout = handler.create_layout("vertical_columns".to_string());
        assert_eq!(layout.layout_type, "vertical_columns");
        assert!(layout.columns.is_empty());
    }
}