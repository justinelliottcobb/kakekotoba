//! Comprehensive file format tests

use tategaki_ed::{
    FileManager, FileFormat, FileMetadata,
    VerticalTextBuffer, TextDirection,
    formats::{PlainTextHandler, SpatialFormatHandler, MarkdownHandler, JsonHandler, FileHandler},
    spatial::{SpatialPosition, SpatialRange},
};
use tempfile::TempDir;
use std::path::Path;

/// Test plain text format handling
#[test]
fn test_plain_text_format() -> tategaki_ed::Result<()> {
    let handler = PlainTextHandler::new();
    let temp_dir = TempDir::new().unwrap();
    
    let test_cases = vec![
        ("Simple English text", TextDirection::HorizontalLeftToRight),
        ("これは日本語のテストです。縦書きに適しています。", TextDirection::VerticalTopToBottom),
        ("Mixed content: English and 日本語 together", TextDirection::VerticalTopToBottom),
        ("Numbers 12345 and symbols !@#$%", TextDirection::HorizontalLeftToRight),
    ];

    for (content, expected_direction) in test_cases {
        let test_file = temp_dir.path().join("plain_test.txt");
        std::fs::write(&test_file, content).unwrap();
        
        // Load and verify
        let (buffer, metadata) = handler.load(&test_file)?;
        assert_eq!(buffer.as_text(), content);
        assert_eq!(metadata.format, FileFormat::PlainText);
        assert_eq!(metadata.text_direction, expected_direction);
        assert_eq!(metadata.encoding, "UTF-8");
        
        // Test save roundtrip
        handler.save(&buffer, &metadata, &test_file)?;
        let loaded_content = std::fs::read_to_string(&test_file).unwrap();
        assert_eq!(loaded_content, content);
    }

    Ok(())
}

/// Test spatial format handling
#[test]
fn test_spatial_format() -> tategaki_ed::Result<()> {
    let handler = SpatialFormatHandler::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("spatial_test.spatial");

    let content = "Test spatial content\nWith multiple lines\n日本語も含む";
    let buffer = VerticalTextBuffer::from_text(content, TextDirection::VerticalTopToBottom)?;
    
    let mut metadata = FileMetadata {
        format: FileFormat::Spatial,
        text_direction: TextDirection::VerticalTopToBottom,
        cursor_position: Some(SpatialPosition { row: 1, column: 5, byte_offset: 0 }),
        encoding: "UTF-8".to_string(),
        ..Default::default()
    };

    // Add some selections as JSON in properties
    let selections = vec![
        SpatialRange {
            start: SpatialPosition { row: 0, column: 0, byte_offset: 0 },
            end: SpatialPosition { row: 0, column: 4, byte_offset: 0 },
        }
    ];
    let selections_json = serde_json::to_string(&selections).unwrap();
    metadata.properties.insert("selections".to_string(), selections_json);
    
    // Add custom property
    metadata.properties.insert("test_property".to_string(), "test_value".to_string());

    // Save and load
    handler.save(&buffer, &metadata, &test_file)?;
    let (loaded_buffer, loaded_metadata) = handler.load(&test_file)?;

    // Verify content
    assert_eq!(loaded_buffer.as_text(), content);
    assert_eq!(loaded_metadata.format, FileFormat::Spatial);
    assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
    assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 1, column: 5, byte_offset: 0 }));
    assert!(loaded_metadata.properties.contains_key("selections"));
    assert_eq!(loaded_metadata.properties.get("test_property"), Some(&"test_value".to_string()));

    // Verify the file is valid JSON
    let file_content = std::fs::read_to_string(&test_file).unwrap();
    let _: serde_json::Value = serde_json::from_str(&file_content).unwrap();

    Ok(())
}

/// Test markdown format handling
#[test]
fn test_markdown_format() -> tategaki_ed::Result<()> {
    let handler = MarkdownHandler::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("markdown_test.md");

    let content = r#"# Test Markdown

This is a **markdown** document with *formatting*.

## Japanese Section
これは日本語のセクションです。
縦書きレイアウトに適しています。

- List item 1
- List item 2 with 日本語
- List item 3

```rust
fn main() {
    println!("Hello, world!");
}
```

The end."#;

    let buffer = VerticalTextBuffer::from_text(content, TextDirection::VerticalTopToBottom)?;
    
    let mut metadata = FileMetadata {
        format: FileFormat::Markdown,
        text_direction: TextDirection::VerticalTopToBottom,
        cursor_position: Some(SpatialPosition { row: 5, column: 10, byte_offset: 0 }),
        encoding: "UTF-8".to_string(),
        ..Default::default()
    };
    metadata.properties.insert("md_author".to_string(), "Test Author".to_string());

    // Save and load
    handler.save(&buffer, &metadata, &test_file)?;
    let (loaded_buffer, loaded_metadata) = handler.load(&test_file)?;

    // Verify content
    assert_eq!(loaded_buffer.as_text(), content);
    assert_eq!(loaded_metadata.format, FileFormat::Markdown);
    assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
    assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 5, column: 10, byte_offset: 0 }));
    assert_eq!(loaded_metadata.properties.get("md_author"), Some(&"Test Author".to_string()));

    // Verify frontmatter was added
    let file_content = std::fs::read_to_string(&test_file).unwrap();
    assert!(file_content.contains("---"));
    assert!(file_content.contains("text_direction: vertical"));
    assert!(file_content.contains("cursor_row: 5"));
    assert!(file_content.contains("cursor_column: 10"));
    assert!(file_content.contains("author: Test Author"));

    Ok(())
}

/// Test JSON format handling
#[test]
fn test_json_format() -> tategaki_ed::Result<()> {
    let handler = JsonHandler::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("json_test.json");

    let content = "Test JSON content\nWith multiple lines\n日本語テスト";
    let buffer = VerticalTextBuffer::from_text(content, TextDirection::VerticalTopToBottom)?;
    
    let mut metadata = FileMetadata {
        format: FileFormat::Json,
        text_direction: TextDirection::VerticalTopToBottom,
        cursor_position: Some(SpatialPosition { row: 2, column: 7, byte_offset: 0 }),
        encoding: "UTF-8".to_string(),
        ..Default::default()
    };
    metadata.properties.insert("title".to_string(), "Test Document".to_string());
    metadata.properties.insert("author".to_string(), "Test Author".to_string());

    // Save and load
    handler.save(&buffer, &metadata, &test_file)?;
    let (loaded_buffer, loaded_metadata) = handler.load(&test_file)?;

    // Verify content
    assert_eq!(loaded_buffer.as_text(), content);
    assert_eq!(loaded_metadata.format, FileFormat::Json);
    assert_eq!(loaded_metadata.text_direction, TextDirection::VerticalTopToBottom);
    assert_eq!(loaded_metadata.cursor_position, Some(SpatialPosition { row: 2, column: 7, byte_offset: 0 }));
    assert_eq!(loaded_metadata.properties.get("title"), Some(&"Test Document".to_string()));
    assert_eq!(loaded_metadata.properties.get("author"), Some(&"Test Author".to_string()));

    // Verify the file is valid JSON
    let file_content = std::fs::read_to_string(&test_file).unwrap();
    let json_doc: serde_json::Value = serde_json::from_str(&file_content).unwrap();
    assert!(json_doc.is_object());
    assert!(json_doc.get("version").is_some());
    assert!(json_doc.get("metadata").is_some());
    assert!(json_doc.get("content").is_some());

    Ok(())
}

/// Test format validation
#[test]
fn test_format_validation() {
    let temp_dir = TempDir::new().unwrap();

    // Test plain text validation
    let plain_handler = PlainTextHandler::new();
    let valid_text_file = temp_dir.path().join("valid.txt");
    std::fs::write(&valid_text_file, "Valid UTF-8 content").unwrap();
    assert!(plain_handler.validate(&valid_text_file).is_ok());

    let non_existent_file = temp_dir.path().join("does_not_exist.txt");
    assert!(plain_handler.validate(&non_existent_file).is_err());

    // Test spatial format validation
    let spatial_handler = SpatialFormatHandler::new();
    let valid_spatial_file = temp_dir.path().join("valid.spatial");
    let valid_spatial_content = r#"{
        "version": "1.0",
        "content": "Test content",
        "metadata": {
            "text_direction": "VerticalTopToBottom",
            "encoding": "UTF-8"
        }
    }"#;
    std::fs::write(&valid_spatial_file, valid_spatial_content).unwrap();
    assert!(spatial_handler.validate(&valid_spatial_file).is_ok());

    let invalid_spatial_file = temp_dir.path().join("invalid.spatial");
    std::fs::write(&invalid_spatial_file, "not valid json").unwrap();
    assert!(spatial_handler.validate(&invalid_spatial_file).is_err());

    // Test JSON format validation
    let json_handler = JsonHandler::new();
    let valid_json_file = temp_dir.path().join("valid.json");
    let valid_json_content = r#"{
        "version": "1.0",
        "metadata": {
            "text_direction": "VerticalTopToBottom",
            "encoding": "UTF-8"
        },
        "content": {
            "text": "Test content"
        },
        "annotations": []
    }"#;
    std::fs::write(&valid_json_file, valid_json_content).unwrap();
    assert!(json_handler.validate(&valid_json_file).is_ok());
}

/// Test file format extensions and descriptions
#[test]
fn test_format_properties() {
    let formats = vec![
        (FileFormat::PlainText, vec!["txt"], "Plain Text"),
        (FileFormat::Spatial, vec!["spatial"], "Spatial Text Format"),
        (FileFormat::Markdown, vec!["md", "markdown"], "Markdown with Vertical Extensions"),
        (FileFormat::Json, vec!["json"], "JSON with Spatial Metadata"),
    ];

    for (format, expected_extensions, expected_description) in formats {
        assert_eq!(format.description(), expected_description);
        
        let manager = FileManager::new();
        let actual_extensions = manager.extensions_for_format(format);
        
        // Check that all expected extensions are present
        for expected_ext in &expected_extensions {
            assert!(actual_extensions.contains(expected_ext), 
                "Format {:?} should support extension '{}'", format, expected_ext);
        }
    }
}

/// Test spatial metadata support
#[test]
fn test_spatial_metadata_support() {
    let handlers: Vec<(FileFormat, Box<dyn FileHandler>)> = vec![
        (FileFormat::PlainText, Box::new(PlainTextHandler::new())),
        (FileFormat::Spatial, Box::new(SpatialFormatHandler::new())),
        (FileFormat::Markdown, Box::new(MarkdownHandler::new())),
        (FileFormat::Json, Box::new(JsonHandler::new())),
    ];

    for (format, handler) in handlers {
        match format {
            FileFormat::PlainText => assert!(!handler.supports_spatial_metadata()),
            FileFormat::Spatial => assert!(handler.supports_spatial_metadata()),
            FileFormat::Markdown => assert!(handler.supports_spatial_metadata()),
            FileFormat::Json => assert!(handler.supports_spatial_metadata()),
            FileFormat::Auto => {} // Skip auto-detection
        }
    }
}

/// Test large file handling
#[test]
fn test_large_file_handling() -> tategaki_ed::Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let manager = FileManager::new();

    // Create moderately large content (100KB)
    let line = "This is a test line with some content. 日本語テストライン。\n";
    let large_content = line.repeat(1000);

    let buffer = VerticalTextBuffer::from_text(&large_content, TextDirection::VerticalTopToBottom)?;
    let metadata = FileMetadata::default();

    // Test with different formats
    let test_cases = vec![
        ("large_plain.txt", FileFormat::PlainText),
        ("large_spatial.spatial", FileFormat::Spatial),
        ("large_markdown.md", FileFormat::Markdown),
        ("large_json.json", FileFormat::Json),
    ];

    for (filename, format) in test_cases {
        let test_file = temp_dir.path().join(filename);
        
        // Save large file
        let start = std::time::Instant::now();
        manager.save_with_format(&buffer, &metadata, &test_file, format)?;
        let save_time = start.elapsed();
        
        // Load large file
        let start = std::time::Instant::now();
        let (loaded_buffer, _) = manager.load(&test_file)?;
        let load_time = start.elapsed();
        
        // Verify content
        assert_eq!(loaded_buffer.as_text(), large_content);
        
        // Performance check (generous limits)
        assert!(save_time.as_secs() < 2, "Save too slow for {}: {:?}", filename, save_time);
        assert!(load_time.as_secs() < 2, "Load too slow for {}: {:?}", filename, load_time);
        
        println!("✓ {} - Save: {:?}, Load: {:?}", filename, save_time, load_time);
    }

    Ok(())
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases() {
    let temp_dir = TempDir::new().unwrap();
    let manager = FileManager::new();

    // Empty content
    let empty_buffer = VerticalTextBuffer::from_text("", TextDirection::VerticalTopToBottom).unwrap();
    let metadata = FileMetadata::default();
    let empty_file = temp_dir.path().join("empty.txt");
    assert!(manager.save(&empty_buffer, &metadata, &empty_file).is_ok());
    assert!(manager.load(&empty_file).is_ok());

    // Very long lines
    let long_line = "a".repeat(10000) + "\n" + &"日".repeat(5000);
    let long_buffer = VerticalTextBuffer::from_text(&long_line, TextDirection::VerticalTopToBottom).unwrap();
    let long_file = temp_dir.path().join("long.txt");
    assert!(manager.save(&long_buffer, &metadata, &long_file).is_ok());
    let (loaded, _) = manager.load(&long_file).unwrap();
    assert_eq!(loaded.as_text(), long_line);

    // Special characters and Unicode
    let special_content = "Special chars: \u{0000}\u{001F}\u{007F}\u{FFFF}\nEmojis: 🌸🎌🗾\nMath: ∑∫∂∇±×÷";
    let special_buffer = VerticalTextBuffer::from_text(special_content, TextDirection::VerticalTopToBottom).unwrap();
    let special_file = temp_dir.path().join("special.txt");
    assert!(manager.save(&special_buffer, &metadata, &special_file).is_ok());
    let (loaded, _) = manager.load(&special_file).unwrap();
    assert_eq!(loaded.as_text(), special_content);
}

/// Test file format auto-detection
#[test]
fn test_format_auto_detection() {
    let test_cases = vec![
        ("test.txt", FileFormat::PlainText),
        ("document.md", FileFormat::Markdown),
        ("data.json", FileFormat::Json),
        ("spatial_doc.spatial", FileFormat::Spatial),
        ("no_extension", FileFormat::PlainText), // Default fallback
        ("multiple.dots.txt", FileFormat::PlainText),
        ("UPPERCASE.TXT", FileFormat::PlainText), // Should handle case
    ];

    for (filename, expected_format) in test_cases {
        let detected_format = FileFormat::from_extension(Path::new(filename));
        assert_eq!(detected_format, expected_format, 
            "Wrong format detected for '{}': expected {:?}, got {:?}", 
            filename, expected_format, detected_format);
    }
}

/// Test concurrent file operations
#[test]
fn test_concurrent_file_operations() -> tategaki_ed::Result<()> {
    use std::thread;
    use std::sync::Arc;
    
    let temp_dir = TempDir::new().unwrap();
    let manager = Arc::new(FileManager::new());
    
    let test_content = "Concurrent test 並行処理テスト";
    let buffer = Arc::new(VerticalTextBuffer::from_text(test_content, TextDirection::VerticalTopToBottom)?);
    
    let mut handles = vec![];
    
    // Test concurrent saves
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let buffer_clone = Arc::clone(&buffer);
        let temp_dir_path = temp_dir.path().to_path_buf();
        
        let handle = thread::spawn(move || {
            let metadata = FileMetadata {
                format: FileFormat::PlainText,
                ..Default::default()
            };
            let test_file = temp_dir_path.join(format!("concurrent_{}.txt", i));
            manager_clone.save(&buffer_clone, &metadata, &test_file)
        });
        
        handles.push(handle);
    }
    
    // Wait for all saves to complete
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    // Test concurrent loads
    let mut handles = vec![];
    
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let temp_dir_path = temp_dir.path().to_path_buf();
        
        let handle = thread::spawn(move || {
            let test_file = temp_dir_path.join(format!("concurrent_{}.txt", i));
            manager_clone.load(&test_file)
        });
        
        handles.push(handle);
    }
    
    // Verify all loads succeeded
    for handle in handles {
        let (loaded_buffer, _) = handle.join().unwrap()?;
        assert_eq!(loaded_buffer.as_text(), test_content);
    }

    Ok(())
}

/// Test backup functionality
#[test]
fn test_backup_functionality() -> tategaki_ed::Result<()> {
    let temp_dir = TempDir::new().unwrap();
    let manager = FileManager::new();
    
    let original_content = "Original content 元の内容";
    let test_file = temp_dir.path().join("backup_test.txt");
    
    // Create original file
    std::fs::write(&test_file, original_content)?;
    
    // Create backup
    manager.create_backup(&test_file)?;
    
    let backup_file = test_file.with_extension("txt.backup");
    assert!(backup_file.exists());
    assert_eq!(std::fs::read_to_string(&backup_file)?, original_content);
    
    // Modify original
    let modified_content = "Modified content 変更された内容";
    std::fs::write(&test_file, modified_content)?;
    
    // Restore from backup
    manager.restore_backup(&test_file)?;
    assert_eq!(std::fs::read_to_string(&test_file)?, original_content);
    
    // Backup should be cleaned up
    assert!(!backup_file.exists());
    
    // Test backup of non-existent file (should succeed silently)
    let non_existent = temp_dir.path().join("does_not_exist.txt");
    assert!(manager.create_backup(&non_existent).is_ok());

    Ok(())
}