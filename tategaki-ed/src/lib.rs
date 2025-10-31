//! Tategaki-ed: Vertical Text Editor for Japanese Programming Languages
//!
//! A comprehensive text editor designed for vertically-oriented programming languages,
//! with dual interfaces supporting both rich graphical editing (GPUI) and terminal
//! console editing (Ratatui).
//!
//! # Features
//!
//! - **Vertical Text Support**: Native 縦書き (tategaki) text editing
//! - **Dual Interfaces**: GPUI for graphics, Ratatui for terminal
//! - **Japanese Language Integration**: Full Unicode support with IME integration
//! - **Spatial Programming**: Layout-aware editing for spatial programming languages
//! - **Mixed Script Support**: Seamless Japanese/ASCII code integration

pub mod text_engine;
pub mod spatial;
pub mod japanese;
pub mod programming;
pub mod formats;
pub mod backend;

// Conditional interface modules
#[cfg(feature = "gpui")]
pub mod gpui_interface;

#[cfg(feature = "ratatui")]
pub mod ratatui_interface;

// Re-export core types for convenience
pub use text_engine::{VerticalTextBuffer, TextDirection, LayoutEngine};
pub use spatial::{SpatialPosition, CoordinateSystem};
pub use japanese::{JapaneseInputMethod, CharacterHandler};
pub use backend::{RenderBackend, BackendType, BackendSelector, Color, Rect, TextStyle};
pub use formats::{FileManager, FileFormat, FileMetadata, FileHandler};

/// Error types for the tategaki editor
#[derive(Debug, thiserror::Error)]
pub enum TategakiError {
    #[error("Text buffer error: {0}")]
    TextBuffer(String),

    #[error("Spatial positioning error: {0}")]
    Spatial(String),

    #[error("Japanese input error: {0}")]
    Japanese(String),

    #[error("File format error: {0}")]
    Format(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Input/Output error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unicode error: {0}")]
    Unicode(String),

    #[error("Integration error with kakekotoba: {0}")]
    Integration(String),
}

/// Result type for tategaki operations
pub type Result<T> = std::result::Result<T, TategakiError>;

/// Core configuration for the vertical text editor
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditorConfig {
    /// Primary text direction
    pub text_direction: TextDirection,
    /// Enable Japanese input method
    pub enable_ime: bool,
    /// Font configuration
    pub font_config: FontConfig,
    /// Color scheme
    pub color_scheme: ColorScheme,
    /// Keyboard shortcuts
    pub keybindings: KeyBindings,
    /// Programming language features
    pub programming_features: ProgrammingFeatures,
}

/// Font configuration for vertical text
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FontConfig {
    /// Japanese font family
    pub japanese_font: String,
    /// ASCII font family
    pub ascii_font: String,
    /// Base font size
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Character spacing
    pub character_spacing: f32,
}

/// Color scheme configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColorScheme {
    /// Background color
    pub background: String,
    /// Foreground text color
    pub foreground: String,
    /// Selection background
    pub selection_bg: String,
    /// Cursor color
    pub cursor: String,
    /// Syntax highlighting colors
    pub syntax: SyntaxColors,
}

/// Syntax highlighting colors
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyntaxColors {
    /// Keywords (Japanese or ASCII)
    pub keywords: String,
    /// String literals
    pub strings: String,
    /// Comments
    pub comments: String,
    /// Numbers
    pub numbers: String,
    /// Operators
    pub operators: String,
}

/// Key binding configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyBindings {
    /// Movement keys for vertical navigation
    pub movement: MovementKeys,
    /// Editing commands
    pub editing: EditingKeys,
    /// View management
    pub view: ViewKeys,
}

/// Movement key configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MovementKeys {
    /// Move up in vertical text (previous row)
    pub up: String,
    /// Move down in vertical text (next row)
    pub down: String,
    /// Move left in vertical text (next column)
    pub left: String,
    /// Move right in vertical text (previous column)
    pub right: String,
    /// Move to line start
    pub line_start: String,
    /// Move to line end
    pub line_end: String,
}

/// Editing key configuration  
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditingKeys {
    /// Delete character
    pub delete: String,
    /// Backspace
    pub backspace: String,
    /// New line
    pub new_line: String,
    /// Copy
    pub copy: String,
    /// Cut
    pub cut: String,
    /// Paste
    pub paste: String,
    /// Undo
    pub undo: String,
    /// Redo
    pub redo: String,
}

/// View management keys
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ViewKeys {
    /// Scroll up
    pub scroll_up: String,
    /// Scroll down
    pub scroll_down: String,
    /// Zoom in
    pub zoom_in: String,
    /// Zoom out
    pub zoom_out: String,
    /// Toggle direction
    pub toggle_direction: String,
}

/// Programming language feature configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgrammingFeatures {
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Enable code folding
    pub code_folding: bool,
    /// Enable auto-indentation
    pub auto_indent: bool,
    /// Enable bracket matching
    pub bracket_matching: bool,
    /// Enable error indicators
    pub error_indicators: bool,
    /// Integration with kakekotoba compiler
    pub kakekotoba_integration: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            text_direction: TextDirection::VerticalTopToBottom,
            enable_ime: true,
            font_config: FontConfig::default(),
            color_scheme: ColorScheme::default(),
            keybindings: KeyBindings::default(),
            programming_features: ProgrammingFeatures::default(),
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            japanese_font: "Noto Sans CJK JP".to_string(),
            ascii_font: "JetBrains Mono".to_string(),
            font_size: 14.0,
            line_height: 1.4,
            character_spacing: 0.0,
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            background: "#1e1e1e".to_string(),
            foreground: "#d4d4d4".to_string(),
            selection_bg: "#264f78".to_string(),
            cursor: "#ffffff".to_string(),
            syntax: SyntaxColors::default(),
        }
    }
}

impl Default for SyntaxColors {
    fn default() -> Self {
        Self {
            keywords: "#569cd6".to_string(),
            strings: "#ce9178".to_string(),
            comments: "#6a9955".to_string(),
            numbers: "#b5cea8".to_string(),
            operators: "#d4d4d4".to_string(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            movement: MovementKeys::default(),
            editing: EditingKeys::default(),
            view: ViewKeys::default(),
        }
    }
}

impl Default for MovementKeys {
    fn default() -> Self {
        Self {
            up: "k".to_string(),
            down: "j".to_string(),
            left: "h".to_string(),
            right: "l".to_string(),
            line_start: "0".to_string(),
            line_end: "$".to_string(),
        }
    }
}

impl Default for EditingKeys {
    fn default() -> Self {
        Self {
            delete: "Delete".to_string(),
            backspace: "Backspace".to_string(),
            new_line: "Enter".to_string(),
            copy: "Ctrl+c".to_string(),
            cut: "Ctrl+x".to_string(),
            paste: "Ctrl+v".to_string(),
            undo: "Ctrl+z".to_string(),
            redo: "Ctrl+y".to_string(),
        }
    }
}

impl Default for ViewKeys {
    fn default() -> Self {
        Self {
            scroll_up: "Ctrl+u".to_string(),
            scroll_down: "Ctrl+d".to_string(),
            zoom_in: "Ctrl+Plus".to_string(),
            zoom_out: "Ctrl+Minus".to_string(),
            toggle_direction: "Ctrl+t".to_string(),
        }
    }
}

impl Default for ProgrammingFeatures {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            code_folding: true,
            auto_indent: true,
            bracket_matching: true,
            error_indicators: true,
            kakekotoba_integration: true,
        }
    }
}