//! Markdown format with vertical text extensions

use super::{FileHandler, FileMetadata, FileFormat};
use crate::{Result, TategakiError};
use crate::text_engine::{VerticalTextBuffer, TextDirection};
use crate::spatial::{SpatialPosition, SpatialRange};
use std::path::Path;

/// Markdown file handler with vertical text support
pub struct MarkdownHandler {
    /// Whether to parse vertical text directives
    parse_vertical_directives: bool,
    /// Whether to preserve original formatting
    preserve_formatting: bool,
}

impl MarkdownHandler {
    pub fn new() -> Self {
        Self {
            parse_vertical_directives: true,
            preserve_formatting: true,
        }
    }

    pub fn set_parse_vertical_directives(&mut self, parse: bool) {
        self.parse_vertical_directives = parse;
    }

    pub fn set_preserve_formatting(&mut self, preserve: bool) {
        self.preserve_formatting = preserve;
    }

    /// Parse vertical text directives from markdown content
    fn parse_vertical_directives(&self, content: &str) -> (String, MarkdownMetadata) {
        let mut cleaned_content = String::new();
        let mut metadata = MarkdownMetadata::default();
        let mut in_frontmatter = false;
        let mut frontmatter_lines = Vec::new();

        for line in content.lines() {
            // Check for YAML frontmatter
            if line.trim() == "---" {
                if !in_frontmatter {
                    in_frontmatter = true;
                    continue;
                } else {
                    // End of frontmatter
                    self.parse_frontmatter(&frontmatter_lines, &mut metadata);
                    in_frontmatter = false;
                    continue;
                }
            }

            if in_frontmatter {
                frontmatter_lines.push(line);
                continue;
            }

            // Check for vertical text directives
            if let Some(directive) = self.parse_line_directive(line) {
                match directive {
                    VerticalDirective::TextDirection(dir) => {
                        metadata.text_direction = Some(dir);
                        if !self.preserve_formatting {
                            continue; // Skip adding to content
                        }
                    }
                    VerticalDirective::Column(col_spec) => {
                        metadata.column_specs.push(col_spec);
                        if !self.preserve_formatting {
                            continue;
                        }
                    }
                    VerticalDirective::Cursor(pos) => {
                        metadata.cursor_position = Some(pos);
                        if !self.preserve_formatting {
                            continue;
                        }
                    }
                }
            }

            cleaned_content.push_str(line);
            cleaned_content.push('\n');
        }

        // Remove trailing newline
        if cleaned_content.ends_with('\n') {
            cleaned_content.pop();
        }

        (cleaned_content, metadata)
    }

    /// Parse frontmatter YAML
    fn parse_frontmatter(&self, lines: &[&str], metadata: &mut MarkdownMetadata) {
        for line in lines {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');

                match key {
                    "text_direction" | "direction" => {
                        metadata.text_direction = match value {
                            "vertical" | "vertical-tb" => Some(TextDirection::VerticalTopToBottom),
                            "horizontal" | "horizontal-lr" => Some(TextDirection::HorizontalLeftToRight),
                            _ => None,
                        };
                    }
                    "cursor_row" => {
                        if let Ok(row) = value.parse::<usize>() {
                            if let Some(ref mut pos) = metadata.cursor_position {
                                pos.row = row;
                            } else {
                                metadata.cursor_position = Some(SpatialPosition { row, column: 0, byte_offset: 0 });
                            }
                        }
                    }
                    "cursor_column" => {
                        if let Ok(column) = value.parse::<usize>() {
                            if let Some(ref mut pos) = metadata.cursor_position {
                                pos.column = column;
                            } else {
                                metadata.cursor_position = Some(SpatialPosition { row: 0, column, byte_offset: 0 });
                            }
                        }
                    }
                    _ => {
                        metadata.custom_properties.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    /// Parse a line for vertical text directives
    fn parse_line_directive(&self, line: &str) -> Option<VerticalDirective> {
        let line = line.trim();

        // HTML-style comments for directives
        if line.starts_with("<!-- tategaki:") && line.ends_with(" -->") {
            let directive_content = &line[14..line.len()-4].trim();
            return self.parse_directive_content(directive_content);
        }

        // Markdown-style directives (custom extension)
        if line.starts_with("::tategaki") {
            let directive_content = &line[10..].trim();
            return self.parse_directive_content(directive_content);
        }

        None
    }

    /// Parse directive content
    fn parse_directive_content(&self, content: &str) -> Option<VerticalDirective> {
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "direction" if parts.len() >= 2 => {
                match parts[1] {
                    "vertical" | "vertical-tb" => Some(VerticalDirective::TextDirection(TextDirection::VerticalTopToBottom)),
                    "horizontal" | "horizontal-lr" => Some(VerticalDirective::TextDirection(TextDirection::HorizontalLeftToRight)),
                    _ => None,
                }
            }
            "column" if parts.len() >= 2 => {
                // Parse column specification: "column width:20 height:40"
                let mut width = None;
                let mut height = None;
                
                for part in &parts[1..] {
                    if let Some((key, value)) = part.split_once(':') {
                        match key {
                            "width" => width = value.parse().ok(),
                            "height" => height = value.parse().ok(),
                            _ => {}
                        }
                    }
                }

                if let (Some(w), Some(h)) = (width, height) {
                    Some(VerticalDirective::Column(ColumnSpec { width: w, height: h }))
                } else {
                    None
                }
            }
            "cursor" if parts.len() >= 3 => {
                // Parse cursor position: "cursor row:5 column:10"
                let mut row = None;
                let mut column = None;
                
                for part in &parts[1..] {
                    if let Some((key, value)) = part.split_once(':') {
                        match key {
                            "row" => row = value.parse().ok(),
                            "column" => column = value.parse().ok(),
                            _ => {}
                        }
                    }
                }

                if let (Some(r), Some(c)) = (row, column) {
                    Some(VerticalDirective::Cursor(SpatialPosition { row: r, column: c, byte_offset: 0 }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Generate vertical text directives for saving
    fn generate_vertical_directives(&self, metadata: &MarkdownMetadata) -> String {
        let mut directives = String::new();

        // Add frontmatter if we have metadata
        if metadata.has_vertical_metadata() {
            directives.push_str("---\n");

            if let Some(direction) = metadata.text_direction {
                let direction_str = match direction {
                    TextDirection::VerticalTopToBottom => "vertical",
                    TextDirection::HorizontalLeftToRight => "horizontal",
                };
                directives.push_str(&format!("text_direction: {}\n", direction_str));
            }

            if let Some(cursor) = metadata.cursor_position {
                directives.push_str(&format!("cursor_row: {}\n", cursor.row));
                directives.push_str(&format!("cursor_column: {}\n", cursor.column));
            }

            for (key, value) in &metadata.custom_properties {
                directives.push_str(&format!("{}: {}\n", key, value));
            }

            directives.push_str("---\n\n");
        }

        // Add column specifications as comments
        for col_spec in &metadata.column_specs {
            directives.push_str(&format!(
                "<!-- tategaki: column width:{} height:{} -->\n",
                col_spec.width, col_spec.height
            ));
        }

        directives
    }

    /// Detect if content has CJK characters (heuristic for vertical text)
    fn has_cjk_content(&self, content: &str) -> bool {
        let cjk_count = content.chars()
            .filter(|&c| self.is_cjk_character(c))
            .count();
        let total_chars = content.chars().filter(|&c| !c.is_whitespace()).count();
        
        total_chars > 0 && (cjk_count as f32 / total_chars as f32) > 0.1
    }

    /// Check if character is CJK
    fn is_cjk_character(&self, c: char) -> bool {
        matches!(c as u32,
            0x4E00..=0x9FFF |   // CJK Unified Ideographs
            0x3400..=0x4DBF |   // CJK Extension A
            0x3040..=0x309F |   // Hiragana
            0x30A0..=0x30FF |   // Katakana
            0xFF00..=0xFFEF     // Halfwidth and Fullwidth Forms
        )
    }
}

/// Vertical text directive
#[derive(Debug, Clone)]
enum VerticalDirective {
    TextDirection(TextDirection),
    Column(ColumnSpec),
    Cursor(SpatialPosition),
}

/// Column specification for vertical layout
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColumnSpec {
    pub width: usize,
    pub height: usize,
}

/// Markdown-specific metadata
#[derive(Debug, Clone, Default)]
struct MarkdownMetadata {
    pub text_direction: Option<TextDirection>,
    pub cursor_position: Option<SpatialPosition>,
    pub column_specs: Vec<ColumnSpec>,
    pub custom_properties: std::collections::HashMap<String, String>,
}

impl MarkdownMetadata {
    fn has_vertical_metadata(&self) -> bool {
        self.text_direction.is_some() || 
        self.cursor_position.is_some() || 
        !self.column_specs.is_empty() ||
        !self.custom_properties.is_empty()
    }
}

impl FileHandler for MarkdownHandler {
    fn load(&self, path: &Path) -> Result<(VerticalTextBuffer, FileMetadata)> {
        let raw_content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to read markdown file: {}", e))))?;

        let (content, md_metadata) = if self.parse_vertical_directives {
            self.parse_vertical_directives(&raw_content)
        } else {
            (raw_content, MarkdownMetadata::default())
        };

        // Determine text direction
        let text_direction = md_metadata.text_direction
            .unwrap_or_else(|| {
                if self.has_cjk_content(&content) {
                    TextDirection::VerticalTopToBottom
                } else {
                    TextDirection::HorizontalLeftToRight
                }
            });

        // Create buffer
        let buffer = VerticalTextBuffer::from_text(&content, text_direction)?;

        // Create file metadata
        let mut properties = std::collections::HashMap::new();
        
        // Add column specs
        if !md_metadata.column_specs.is_empty() {
            let columns_json = serde_json::to_string(&md_metadata.column_specs)
                .map_err(|e| TategakiError::Serialization(format!("Failed to serialize columns: {}", e)))?;
            properties.insert("columns".to_string(), columns_json);
        }

        // Add custom properties
        for (key, value) in md_metadata.custom_properties {
            properties.insert(format!("md_{}", key), value);
        }

        let metadata = FileMetadata {
            format: FileFormat::Markdown,
            text_direction,
            cursor_position: md_metadata.cursor_position,
            encoding: "UTF-8".to_string(),
            created_at: path.metadata().ok().and_then(|m| m.created().ok()),
            modified_at: path.metadata().ok().and_then(|m| m.modified().ok()),
            properties,
        };

        Ok((buffer, metadata))
    }

    fn save(&self, buffer: &VerticalTextBuffer, metadata: &FileMetadata, path: &Path) -> Result<()> {
        let content = buffer.as_text();

        // Create markdown metadata from file metadata
        let mut md_metadata = MarkdownMetadata {
            text_direction: Some(metadata.text_direction),
            cursor_position: metadata.cursor_position,
            column_specs: Vec::new(),
            custom_properties: std::collections::HashMap::new(),
        };

        // Extract column specs from properties
        if let Some(columns_json) = metadata.properties.get("columns") {
            md_metadata.column_specs = serde_json::from_str(columns_json)
                .map_err(|e| TategakiError::Serialization(format!("Failed to parse columns: {}", e)))?;
        }

        // Extract custom properties
        for (key, value) in &metadata.properties {
            if let Some(md_key) = key.strip_prefix("md_") {
                md_metadata.custom_properties.insert(md_key.to_string(), value.clone());
            }
        }

        // Generate directives and combine with content
        let directives = self.generate_vertical_directives(&md_metadata);
        let final_content = if directives.is_empty() {
            content
        } else {
            format!("{}{}", directives, content)
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to create directory: {}", e))))?;
        }

        // Write file
        std::fs::write(path, final_content)
            .map_err(|e| TategakiError::Io(std::io::Error::new(e.kind(), format!("Failed to write markdown file: {}", e))))?;

        Ok(())
    }

    fn supports_spatial_metadata(&self) -> bool {
        true // Via frontmatter and directives
    }

    fn file_extensions(&self) -> Vec<&'static str> {
        vec!["md", "markdown"]
    }

    fn validate(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(TategakiError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File does not exist: {}", path.display()))));
        }

        // Check if file is too large
        let size = crate::formats::utils::file_size(path)?;
        if size > 10 * 1024 * 1024 { // 10MB limit for markdown
            return Err(TategakiError::Io(
                format!("Markdown file too large ({} bytes). Maximum: 10MB", size)
            ));
        }

        // Try to read as UTF-8
        let _content = std::fs::read_to_string(path)
            .map_err(|e| TategakiError::InvalidFormat(format!("Invalid UTF-8 content: {}", e)))?;

        Ok(())
    }
}

impl Default for MarkdownHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_markdown_handler_creation() {
        let handler = MarkdownHandler::new();
        assert!(handler.supports_spatial_metadata());
        assert_eq!(handler.file_extensions(), vec!["md", "markdown"]);
    }

    #[test]
    fn test_cjk_detection() {
        let handler = MarkdownHandler::new();
        assert!(handler.has_cjk_content("これは日本語のテストです"));
        assert!(!handler.has_cjk_content("This is English text"));
        assert!(handler.has_cjk_content("Mixed content with 日本語"));
    }

    #[test]
    fn test_directive_parsing() {
        let handler = MarkdownHandler::new();
        
        // Test HTML comment directive
        let directive = handler.parse_line_directive("<!-- tategaki: direction vertical -->");
        assert!(matches!(directive, Some(VerticalDirective::TextDirection(TextDirection::VerticalTopToBottom))));

        // Test column directive
        let directive = handler.parse_line_directive("<!-- tategaki: column width:20 height:40 -->");
        if let Some(VerticalDirective::Column(spec)) = directive {
            assert_eq!(spec.width, 20);
            assert_eq!(spec.height, 40);
        } else {
            panic!("Expected column directive");
        }

        // Test cursor directive
        let directive = handler.parse_line_directive("<!-- tategaki: cursor row:5 column:10 -->");
        if let Some(VerticalDirective::Cursor(pos)) = directive {
            assert_eq!(pos.row, 5);
            assert_eq!(pos.column, 10);
        } else {
            panic!("Expected cursor directive");
        }
    }

    #[test]
    fn test_frontmatter_parsing() {
        let handler = MarkdownHandler::new();
        let content = r#"---
text_direction: vertical
cursor_row: 2
cursor_column: 5
custom_prop: test_value
---

# Test Markdown

Some content here.
"#;

        let (parsed_content, metadata) = handler.parse_vertical_directives(content);
        
        assert_eq!(metadata.text_direction, Some(TextDirection::VerticalTopToBottom));
        assert_eq!(metadata.cursor_position, Some(SpatialPosition { row: 2, column: 5 }));
        assert_eq!(metadata.custom_properties.get("custom_prop"), Some(&"test_value".to_string()));
        assert!(parsed_content.contains("# Test Markdown"));
        assert!(!parsed_content.contains("---"));
    }

    #[test]
    fn test_save_load_cycle() -> Result<()> {
        let handler = MarkdownHandler::new();
        let test_content = "# Test Markdown\n\nSome **bold** text with 日本語.";
        
        // Create buffer and metadata
        let buffer = VerticalTextBuffer::from_text(test_content, TextDirection::VerticalTopToBottom)?;
        let mut metadata = FileMetadata {
            format: FileFormat::Markdown,
            text_direction: TextDirection::VerticalTopToBottom,
            cursor_position: Some(SpatialPosition { row: 1, column: 10 }),
            encoding: "UTF-8".to_string(),
            ..FileMetadata::default()
        };
        metadata.properties.insert("md_author".to_string(), "Test Author".to_string());
        
        // Save to temporary file
        let temp_file = NamedTempFile::new().unwrap();
        handler.save(&buffer, &metadata, temp_file.path())?;
        
        // Read and verify the saved content contains frontmatter
        let saved_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(saved_content.contains("---"));
        assert!(saved_content.contains("text_direction: vertical"));
        assert!(saved_content.contains("cursor_row: 1"));
        assert!(saved_content.contains("cursor_column: 10"));
        assert!(saved_content.contains("author: Test Author"));
        assert!(saved_content.contains("# Test Markdown"));
        
        // Load back
        let (loaded_buffer, loaded_metadata) = handler.load(temp_file.path())?;
        
        // Verify
        assert_eq!(loaded_buffer.as_text(), test_content);
        assert_eq!(loaded_metadata.format, FileFormat::Markdown);
        assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
        assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 1, column: 10 }));
        assert_eq!(loaded_metadata.properties.get("md_author"), Some(&"Test Author".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_directive_generation() {
        let handler = MarkdownHandler::new();
        let mut metadata = MarkdownMetadata {
            text_direction: Some(TextDirection::VerticalTopToBottom),
            cursor_position: Some(SpatialPosition { row: 5, column: 10 }),
            column_specs: vec![ColumnSpec { width: 20, height: 40 }],
            custom_properties: {
                let mut props = std::collections::HashMap::new();
                props.insert("author".to_string(), "Test".to_string());
                props
            },
        };

        let directives = handler.generate_vertical_directives(&metadata);
        
        assert!(directives.contains("---"));
        assert!(directives.contains("text_direction: vertical"));
        assert!(directives.contains("cursor_row: 5"));
        assert!(directives.contains("cursor_column: 10"));
        assert!(directives.contains("author: Test"));
        assert!(directives.contains("<!-- tategaki: column width:20 height:40 -->"));
    }
}