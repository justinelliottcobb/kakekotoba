//! GPUI graphical interface for vertical text editing
//!
//! This module provides a rich graphical editor interface using GPUI with advanced
//! rendering capabilities for vertical Japanese text and spatial programming features.

use crate::japanese::JapaneseInputMethod;
use crate::spatial::{CoordinateSystem, SpatialPosition};
use crate::text_engine::{LayoutEngine, TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
pub mod cursor;
#[cfg(feature = "gpui")]
pub mod editor;
#[cfg(feature = "gpui")]
pub mod renderer;
#[cfg(feature = "gpui")]
pub mod scroll;
#[cfg(feature = "gpui")]
pub mod selection;

#[cfg(feature = "gpui")]
pub use cursor::*;
#[cfg(feature = "gpui")]
pub use editor::*;
#[cfg(feature = "gpui")]
pub use renderer::*;
#[cfg(feature = "gpui")]
pub use scroll::*;
#[cfg(feature = "gpui")]
pub use selection::*;

#[cfg(feature = "gpui")]
/// Main graphical vertical editor component
pub struct GraphicalVerticalEditor {
    /// Text buffer
    buffer: VerticalTextBuffer,
    /// Layout engine for coordinate conversion
    layout_engine: LayoutEngine,
    /// Japanese input method
    ime: JapaneseInputMethod,
    /// Text renderer
    renderer: VerticalTextRenderer,
    /// Cursor management
    cursor: SpatialCursor,
    /// Selection handler
    selection: SelectionHandler,
    /// Scroll manager
    scroll: ScrollManager,
    /// Current viewport size
    viewport_size: Size<Pixels>,
    /// Editor configuration
    config: crate::EditorConfig,
}

#[cfg(feature = "gpui")]
impl GraphicalVerticalEditor {
    /// Create a new graphical vertical editor
    pub fn new(config: crate::EditorConfig) -> Self {
        let direction = config.text_direction;
        let buffer = VerticalTextBuffer::new(direction);
        let layout_engine = LayoutEngine::new(direction);
        let ime = JapaneseInputMethod::new();
        let renderer = VerticalTextRenderer::new(direction);
        let cursor = SpatialCursor::new();
        let selection = SelectionHandler::new();
        let scroll = ScrollManager::new();

        Self {
            buffer,
            layout_engine,
            ime,
            renderer,
            cursor,
            selection,
            scroll,
            viewport_size: Size::default(),
            config,
        }
    }

    /// Load text from string
    pub fn load_text(&mut self, text: &str) -> Result<()> {
        self.buffer = VerticalTextBuffer::from_text(text, self.config.text_direction)?;
        self.cursor.reset();
        self.selection.clear();
        Ok(())
    }

    /// Get current text content
    pub fn text(&self) -> String {
        self.buffer.as_text()
    }

    /// Handle keyboard input
    pub fn handle_key_event(&mut self, event: &KeyDownEvent) -> Result<bool> {
        // First try Japanese IME if enabled
        if self.config.enable_ime {
            let key_str = format!("{:?}", event.keystroke.key);
            match self.ime.process_key_input(&key_str)? {
                crate::japanese::InputResult::Commit(text) => {
                    return self.insert_text_at_cursor(&text);
                }
                crate::japanese::InputResult::Compose(_) => {
                    // Show composition in cursor
                    return Ok(true);
                }
                crate::japanese::InputResult::ShowCandidates(_) => {
                    // Show candidate window
                    return Ok(true);
                }
                crate::japanese::InputResult::Cancel => {
                    return Ok(true);
                }
                crate::japanese::InputResult::NoOp => {
                    // Fall through to regular key handling
                }
            }
        }

        // Handle regular key events
        match &event.keystroke.key {
            Key::Character(ch) => self.insert_text_at_cursor(ch),
            Key::Backspace => self.handle_backspace(),
            Key::Delete => self.handle_delete(),
            Key::Enter => self.handle_enter(),
            Key::ArrowUp => self.handle_arrow_up(),
            Key::ArrowDown => self.handle_arrow_down(),
            Key::ArrowLeft => self.handle_arrow_left(),
            Key::ArrowRight => self.handle_arrow_right(),
            _ => Ok(false),
        }
    }

    /// Insert text at cursor position
    fn insert_text_at_cursor(&mut self, text: &str) -> Result<bool> {
        // TODO: Implement text insertion
        // For now, just update cursor position
        self.cursor.advance();
        Ok(true)
    }

    /// Handle backspace key
    fn handle_backspace(&mut self) -> Result<bool> {
        // TODO: Implement backspace
        self.cursor.move_backward();
        Ok(true)
    }

    /// Handle delete key
    fn handle_delete(&mut self) -> Result<bool> {
        // TODO: Implement delete
        Ok(true)
    }

    /// Handle enter key
    fn handle_enter(&mut self) -> Result<bool> {
        // TODO: Implement newline insertion
        Ok(true)
    }

    /// Handle arrow keys for vertical navigation
    fn handle_arrow_up(&mut self) -> Result<bool> {
        match self.config.text_direction {
            TextDirection::VerticalTopToBottom => {
                // Up arrow moves up within column
                self.cursor.move_up();
            }
            TextDirection::HorizontalLeftToRight => {
                // Up arrow moves to previous line
                self.cursor.move_up();
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle down arrow
    fn handle_arrow_down(&mut self) -> Result<bool> {
        match self.config.text_direction {
            TextDirection::VerticalTopToBottom => {
                // Down arrow moves down within column
                self.cursor.move_down();
            }
            TextDirection::HorizontalLeftToRight => {
                // Down arrow moves to next line
                self.cursor.move_down();
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle left arrow
    fn handle_arrow_left(&mut self) -> Result<bool> {
        match self.config.text_direction {
            TextDirection::VerticalTopToBottom => {
                // Left arrow moves to next column (left in vertical text)
                self.cursor.move_to_next_column();
            }
            TextDirection::HorizontalLeftToRight => {
                // Left arrow moves backward in line
                self.cursor.move_backward();
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle right arrow
    fn handle_arrow_right(&mut self) -> Result<bool> {
        match self.config.text_direction {
            TextDirection::VerticalTopToBottom => {
                // Right arrow moves to previous column (right in vertical text)
                self.cursor.move_to_prev_column();
            }
            TextDirection::HorizontalLeftToRight => {
                // Right arrow moves forward in line
                self.cursor.advance();
            }
            _ => {}
        }
        Ok(true)
    }

    /// Handle mouse events for positioning and selection
    pub fn handle_mouse_event(&mut self, event: &MouseDownEvent) -> Result<bool> {
        let click_position = event.position;

        // Convert screen coordinates to logical position
        if let Ok(logical_pos) = self
            .layout_engine
            .visual_to_logical(click_position.x.0, click_position.y.0)
        {
            self.cursor.set_position(logical_pos);

            // Handle selection if shift is held
            if event.modifiers.shift {
                self.selection.extend_to_position(logical_pos);
            } else {
                self.selection.clear();
            }

            return Ok(true);
        }

        Ok(false)
    }

    /// Render the editor
    pub fn render(&mut self, cx: &mut WindowContext) -> impl IntoElement {
        div().size_full().bg(gpui::black()).child(canvas(
            move |bounds, cx| {
                self.viewport_size = bounds.size;
                self.render_text(bounds, cx);
                self.render_cursor(bounds, cx);
                self.render_selection(bounds, cx);
            },
            |_, _, _| {},
        ))
    }

    /// Render text content
    fn render_text(&mut self, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        self.renderer
            .render_buffer(&self.buffer, bounds, cx, &self.layout_engine);
    }

    /// Render cursor
    fn render_cursor(&mut self, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        self.cursor.render(bounds, cx, &self.layout_engine);
    }

    /// Render selection
    fn render_selection(&mut self, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        self.selection.render(bounds, cx, &self.layout_engine);
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder when GPUI feature is disabled
pub struct GraphicalVerticalEditor {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "gpui"))]
impl GraphicalVerticalEditor {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "gpui")]
    fn test_editor_creation() {
        let config = crate::EditorConfig::default();
        let editor = GraphicalVerticalEditor::new(config);
        assert_eq!(editor.text(), "");
    }

    #[test]
    #[cfg(not(feature = "gpui"))]
    fn test_editor_disabled() {
        let config = crate::EditorConfig::default();
        let mut editor = GraphicalVerticalEditor::new(config);
        assert!(editor.load_text("test").is_err());
    }
}
