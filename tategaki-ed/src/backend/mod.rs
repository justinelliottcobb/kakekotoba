//! Backend abstraction for rendering vertical text
//!
//! This module provides a unified interface for rendering vertical text across
//! different backends: GPU-accelerated (GPUI) and terminal-based (notcurses).

use crate::spatial::SpatialPosition;
use crate::text_engine::{TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};

pub mod keyboard;
pub mod selector;

#[cfg(feature = "gpui")]
pub mod gpui_native;

#[cfg(feature = "notcurses")]
pub mod terminal;

#[cfg(feature = "notcurses")]
pub mod adapter;

pub use keyboard::{EditorCommand, EditorMode, KeyInput, KeyboardHandler};
pub use selector::*;

/// Color representation that works across backends
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(TategakiError::Rendering(format!(
                "Invalid color hex: {}",
                hex
            )));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|e| TategakiError::Rendering(format!("Invalid red component: {}", e)))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|e| TategakiError::Rendering(format!("Invalid green component: {}", e)))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|e| TategakiError::Rendering(format!("Invalid blue component: {}", e)))?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16)
                .map_err(|e| TategakiError::Rendering(format!("Invalid alpha component: {}", e)))?
        } else {
            255
        };

        Ok(Self { r, g, b, a })
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }

    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

/// Rectangle bounds for rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

/// Font style for text rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

/// Text style configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub color: Color,
    pub background: Option<Color>,
    pub font_style: FontStyle,
    pub font_size: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            color: Color::white(),
            background: None,
            font_style: FontStyle::Normal,
            font_size: 14.0,
        }
    }
}

/// Cursor rendering information
#[derive(Debug, Clone, PartialEq)]
pub struct CursorInfo {
    pub position: SpatialPosition,
    pub color: Color,
    pub style: CursorStyle,
}

/// Cursor visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

/// Main rendering backend trait
///
/// This trait abstracts over different rendering backends (GPUI, notcurses, etc.)
/// to provide a unified interface for the vertical text editor.
pub trait RenderBackend: Send + Sync {
    /// Initialize the backend
    fn init(&mut self) -> Result<()>;

    /// Shutdown the backend cleanly
    fn shutdown(&mut self) -> Result<()>;

    /// Get the current viewport size in pixels or cells
    fn viewport_size(&self) -> (u32, u32);

    /// Clear the entire viewport
    fn clear(&mut self, color: Color) -> Result<()>;

    /// Render text at a specific position
    ///
    /// For vertical text, this should handle proper character rotation
    /// and spacing according to the text direction.
    fn render_text(
        &mut self,
        text: &str,
        position: (f32, f32),
        style: &TextStyle,
        direction: TextDirection,
    ) -> Result<()>;

    /// Render the cursor
    fn render_cursor(&mut self, cursor: &CursorInfo) -> Result<()>;

    /// Render a selection rectangle
    fn render_selection(&mut self, bounds: Rect, color: Color) -> Result<()>;

    /// Render a line (for UI chrome like borders)
    fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        color: Color,
        thickness: f32,
    ) -> Result<()>;

    /// Render a rectangle (for UI chrome like boxes)
    fn render_rect(&mut self, bounds: Rect, color: Color, filled: bool) -> Result<()>;

    /// Present/flush the frame to the display
    fn present(&mut self) -> Result<()>;

    /// Check if the backend is still active (window not closed, etc.)
    fn is_active(&self) -> bool;

    /// Handle a resize event
    fn handle_resize(&mut self, width: u32, height: u32) -> Result<()>;
}

/// Backend type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    #[cfg(feature = "gpui")]
    Gpui,
    #[cfg(feature = "notcurses")]
    Notcurses,
    #[cfg(feature = "ratatui")]
    Ratatui,
}

impl BackendType {
    /// Check if this backend is available (feature-gated)
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "gpui")]
            BackendType::Gpui => true,
            #[cfg(feature = "notcurses")]
            BackendType::Notcurses => true,
            #[cfg(feature = "ratatui")]
            BackendType::Ratatui => true,
        }
    }

    /// Get a human-readable name for this backend
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "gpui")]
            BackendType::Gpui => "GPUI (GPU-accelerated)",
            #[cfg(feature = "notcurses")]
            BackendType::Notcurses => "Notcurses (Terminal)",
            #[cfg(feature = "ratatui")]
            BackendType::Ratatui => "Ratatui (Terminal)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#ff00ff").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 255);
        assert_eq!(color.a, 255);

        let color_alpha = Color::from_hex("#ff00ff80").unwrap();
        assert_eq!(color_alpha.a, 128);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 30.0));
        assert!(!rect.contains(5.0, 30.0));
        assert!(!rect.contains(50.0, 5.0));
    }
}
