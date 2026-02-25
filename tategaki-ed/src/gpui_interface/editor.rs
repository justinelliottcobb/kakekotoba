//! Core GPUI editor implementation

use crate::japanese::JapaneseInputMethod;
use crate::spatial::SpatialPosition;
use crate::text_engine::{TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
/// Main vertical text editor view
pub struct VerticalEditorView {
    /// Text buffer
    buffer: VerticalTextBuffer,
    /// Japanese input method
    ime: JapaneseInputMethod,
    /// Current cursor position
    cursor_position: SpatialPosition,
    /// Editor configuration
    config: crate::EditorConfig,
    /// Focus state
    focused: bool,
}

#[cfg(feature = "gpui")]
impl VerticalEditorView {
    /// Create new editor view
    pub fn new(config: crate::EditorConfig) -> Self {
        let buffer = VerticalTextBuffer::new(config.text_direction);
        let ime = JapaneseInputMethod::new();
        let cursor_position = SpatialPosition::origin();

        Self {
            buffer,
            ime,
            cursor_position,
            config,
            focused: false,
        }
    }

    /// Load text into editor
    pub fn load_text(&mut self, text: &str) -> Result<()> {
        self.buffer = VerticalTextBuffer::from_text(text, self.config.text_direction)?;
        self.cursor_position = SpatialPosition::origin();
        Ok(())
    }

    /// Get current text
    pub fn text(&self) -> String {
        self.buffer.as_text()
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Handle key input
    pub fn handle_key_input(&mut self, key: &str) -> Result<bool> {
        if self.config.enable_ime {
            match self.ime.process_key_input(key)? {
                crate::japanese::InputResult::Commit(text) => {
                    self.insert_text(&text)?;
                    return Ok(true);
                }
                crate::japanese::InputResult::Compose(_) => {
                    return Ok(true);
                }
                crate::japanese::InputResult::ShowCandidates(_) => {
                    return Ok(true);
                }
                crate::japanese::InputResult::Cancel => {
                    return Ok(true);
                }
                crate::japanese::InputResult::NoOp => {
                    // Fall through
                }
            }
        }

        // Handle regular input
        self.insert_text(key)?;
        Ok(true)
    }

    /// Insert text at cursor
    pub fn insert_text(&mut self, text: &str) -> Result<()> {
        self.buffer.insert_at(self.cursor_position, text)?;
        self.advance_cursor();
        Ok(())
    }

    /// Move cursor to position
    pub fn move_cursor_to(&mut self, position: SpatialPosition) {
        self.cursor_position = position;
    }

    /// Advance cursor
    fn advance_cursor(&mut self) {
        // Simple advancement for now
        self.cursor_position.column += 1;
    }

    /// Handle navigation
    pub fn handle_navigation(&mut self, direction: NavigationDirection) -> Result<()> {
        match direction {
            NavigationDirection::Up => {
                if self.cursor_position.row > 0 {
                    self.cursor_position.row -= 1;
                }
            }
            NavigationDirection::Down => {
                self.cursor_position.row += 1;
            }
            NavigationDirection::Left => {
                match self.config.text_direction {
                    TextDirection::VerticalTopToBottom => {
                        // Left moves to next column
                        self.cursor_position.column += 1;
                    }
                    _ => {
                        if self.cursor_position.column > 0 {
                            self.cursor_position.column -= 1;
                        }
                    }
                }
            }
            NavigationDirection::Right => {
                match self.config.text_direction {
                    TextDirection::VerticalTopToBottom => {
                        // Right moves to previous column
                        if self.cursor_position.column > 0 {
                            self.cursor_position.column -= 1;
                        }
                    }
                    _ => {
                        self.cursor_position.column += 1;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Navigation directions
#[derive(Debug, Clone, Copy)]
pub enum NavigationDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(feature = "gpui")]
impl Render for VerticalEditorView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::rgb(0x1e1e1e))
            .text_color(gpui::rgb(0xffffff))
            .child(self.render_content(cx))
    }
}

#[cfg(feature = "gpui")]
impl VerticalEditorView {
    fn render_content(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(div().id("text-content").child(self.buffer.as_text()))
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder when GPUI is disabled
pub struct VerticalEditorView {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "gpui"))]
impl VerticalEditorView {
    pub fn new(_config: crate::EditorConfig) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn load_text(&mut self, _text: &str) -> Result<()> {
        Err(TategakiError::Rendering(
            "GPUI feature not enabled".to_string(),
        ))
    }
}
