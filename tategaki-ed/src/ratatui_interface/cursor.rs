//! Terminal cursor management

use crate::spatial::SpatialPosition;
use crate::text_engine::TextDirection;
use crate::{Result, TategakiError};

/// Terminal cursor for spatial text navigation
pub struct TerminalCursor {
    /// Current logical position
    position: SpatialPosition,
    /// Cursor visibility
    visible: bool,
    /// Preferred column (for vertical movement)
    preferred_column: usize,
    /// Cursor blink state
    blink_state: bool,
    /// Last blink toggle time (placeholder for future implementation)
    last_blink_time: u64,
    /// Cursor style
    style: TerminalCursorStyle,
}

/// Terminal cursor display style
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalCursorStyle {
    /// Block cursor (covers entire character)
    Block,
    /// Underline cursor
    Underline,
    /// Vertical bar cursor
    Bar,
}

impl Default for TerminalCursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

impl TerminalCursor {
    /// Create new terminal cursor
    pub fn new() -> Self {
        Self {
            position: SpatialPosition::origin(),
            visible: true,
            preferred_column: 0,
            blink_state: true,
            last_blink_time: 0,
            style: TerminalCursorStyle::default(),
        }
    }

    /// Get current cursor position
    pub fn position(&self) -> SpatialPosition {
        self.position
    }

    /// Set cursor position
    pub fn set_position(&mut self, position: SpatialPosition) {
        self.position = position;
        self.preferred_column = position.column;
    }

    /// Check if cursor is visible
    pub fn is_visible(&self) -> bool {
        self.visible && self.blink_state
    }

    /// Set cursor visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Set cursor style
    pub fn set_style(&mut self, style: TerminalCursorStyle) {
        self.style = style;
    }

    /// Get cursor style
    pub fn style(&self) -> TerminalCursorStyle {
        self.style
    }

    /// Toggle blink state
    pub fn toggle_blink(&mut self) {
        self.blink_state = !self.blink_state;
    }

    /// Set blink state
    pub fn set_blink_state(&mut self, blink: bool) {
        self.blink_state = blink;
    }

    /// Reset cursor to origin
    pub fn reset(&mut self) {
        self.position = SpatialPosition::origin();
        self.preferred_column = 0;
        self.visible = true;
        self.blink_state = true;
    }

    /// Move cursor up
    pub fn move_up(&mut self, text_direction: &TextDirection) {
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, "up" means previous character in column
                if self.position.row > 0 {
                    self.position.row -= 1;
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // In horizontal text, "up" means previous line
                if self.position.row > 0 {
                    self.position.row -= 1;
                    self.position.column = self.preferred_column;
                }
            }
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self, text_direction: &TextDirection) {
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, "down" means next character in column
                self.position.row += 1;
            }
            TextDirection::HorizontalLeftToRight => {
                // In horizontal text, "down" means next line
                self.position.row += 1;
                self.position.column = self.preferred_column;
            }
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self, text_direction: &TextDirection) {
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, "left" means next column (visually left)
                self.position.column += 1;
                self.position.row = 0; // Reset to top of new column
                self.preferred_column = self.position.column;
            }
            TextDirection::HorizontalLeftToRight => {
                // In horizontal text, "left" means previous character
                if self.position.column > 0 {
                    self.position.column -= 1;
                    self.preferred_column = self.position.column;
                }
            }
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, text_direction: &TextDirection) {
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, "right" means previous column (visually right)
                if self.position.column > 0 {
                    self.position.column -= 1;
                    self.position.row = 0; // Reset to top of new column
                    self.preferred_column = self.position.column;
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // In horizontal text, "right" means next character
                self.position.column += 1;
                self.preferred_column = self.position.column;
            }
        }
    }

    /// Move to start of current line/column
    pub fn move_to_line_start(&mut self) {
        match self.position.row {
            0 => {} // Already at start
            _ => {
                self.position.row = 0;
            }
        }
    }

    /// Move to end of current line/column
    pub fn move_to_line_end(&mut self) {
        // This would need buffer context to know the actual line length
        // For now, just implement as placeholder
        self.position.row = 999; // Will be clamped by buffer bounds
    }

    /// Move cursor backward in text flow
    pub fn move_backward(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        } else if self.position.row > 0 {
            self.position.row -= 1;
            // Would need buffer context to set correct column
            self.position.column = 0;
        }
        self.preferred_column = self.position.column;
    }

    /// Move cursor forward in text flow
    pub fn advance(&mut self) {
        self.position.column += 1;
        self.preferred_column = self.position.column;
        // Line wrapping would be handled by buffer context
    }

    /// Move cursor by word (left)
    pub fn move_word_left(&mut self, text_direction: &TextDirection) {
        // Simplified word movement - real implementation would need buffer context
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // Move to previous word in column or previous column
                if self.position.row >= 5 {
                    self.position.row -= 5; // Approximate word boundary
                } else if self.position.column > 0 {
                    self.position.column -= 1;
                    self.position.row = 20; // Approximate column length
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // Move to previous word
                if self.position.column >= 5 {
                    self.position.column -= 5; // Approximate word boundary
                } else if self.position.row > 0 {
                    self.position.row -= 1;
                    self.position.column = 80; // Approximate line length
                }
            }
        }
        self.preferred_column = self.position.column;
    }

    /// Move cursor by word (right)
    pub fn move_word_right(&mut self, text_direction: &TextDirection) {
        // Simplified word movement - real implementation would need buffer context
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // Move to next word in column or next column
                self.position.row += 5; // Approximate word boundary
            }
            TextDirection::HorizontalLeftToRight => {
                // Move to next word
                self.position.column += 5; // Approximate word boundary
            }
        }
        self.preferred_column = self.position.column;
    }

    /// Move cursor to specific position
    pub fn move_to(&mut self, position: SpatialPosition) {
        self.position = position;
        self.preferred_column = position.column;
    }

    /// Move cursor by offset
    pub fn move_by(&mut self, row_delta: i32, col_delta: i32) {
        let new_row = (self.position.row as i32 + row_delta).max(0) as usize;
        let new_col = (self.position.column as i32 + col_delta).max(0) as usize;

        self.position = SpatialPosition {
            row: new_row,
            column: new_col,
        };
        self.preferred_column = self.position.column;
    }

    /// Clamp cursor position to buffer bounds
    pub fn clamp_to_bounds(&mut self, max_row: usize, max_col: usize) {
        self.position.row = self.position.row.min(max_row);
        self.position.column = self.position.column.min(max_col);
        self.preferred_column = self.position.column;
    }

    /// Get cursor display character based on style
    pub fn display_char(&self) -> char {
        match self.style {
            TerminalCursorStyle::Block => '█',
            TerminalCursorStyle::Underline => '_',
            TerminalCursorStyle::Bar => '│',
        }
    }

    /// Check if cursor should be displayed (considering blink state)
    pub fn should_display(&self) -> bool {
        self.visible && self.blink_state
    }

    /// Calculate cursor bounds for a given character size
    pub fn bounds(&self, char_width: u16, char_height: u16) -> (u16, u16, u16, u16) {
        let x = self.position.column as u16;
        let y = self.position.row as u16;

        match self.style {
            TerminalCursorStyle::Block => (x, y, char_width, char_height),
            TerminalCursorStyle::Underline => (x, y + char_height - 1, char_width, 1),
            TerminalCursorStyle::Bar => (x, y, 1, char_height),
        }
    }

    /// Save current position as preferred column
    pub fn save_preferred_column(&mut self) {
        self.preferred_column = self.position.column;
    }

    /// Restore preferred column for vertical movement
    pub fn restore_preferred_column(&mut self) {
        self.position.column = self.preferred_column;
    }

    /// Get distance from another position
    pub fn distance_from(&self, other: SpatialPosition) -> (i32, i32) {
        let row_diff = (self.position.row as i32) - (other.row as i32);
        let col_diff = (self.position.column as i32) - (other.column as i32);
        (row_diff, col_diff)
    }

    /// Check if cursor is at specific position
    pub fn is_at(&self, position: SpatialPosition) -> bool {
        self.position == position
    }

    /// Get cursor info for debugging
    pub fn debug_info(&self) -> CursorDebugInfo {
        CursorDebugInfo {
            position: self.position,
            visible: self.visible,
            preferred_column: self.preferred_column,
            blink_state: self.blink_state,
            style: self.style,
        }
    }
}

/// Debug information about cursor state
#[derive(Debug, Clone)]
pub struct CursorDebugInfo {
    pub position: SpatialPosition,
    pub visible: bool,
    pub preferred_column: usize,
    pub blink_state: bool,
    pub style: TerminalCursorStyle,
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_creation() {
        let cursor = TerminalCursor::new();
        assert_eq!(cursor.position(), SpatialPosition::origin());
        assert!(cursor.is_visible());
        assert_eq!(cursor.style(), TerminalCursorStyle::Block);
    }

    #[test]
    fn test_cursor_movement() {
        let mut cursor = TerminalCursor::new();
        let direction = TextDirection::HorizontalLeftToRight;

        cursor.move_right(&direction);
        assert_eq!(cursor.position().column, 1);

        cursor.move_down(&direction);
        assert_eq!(cursor.position().row, 1);
        assert_eq!(cursor.position().column, 1); // Should maintain preferred column

        cursor.move_left(&direction);
        assert_eq!(cursor.position().column, 0);
    }

    #[test]
    fn test_vertical_text_movement() {
        let mut cursor = TerminalCursor::new();
        let direction = TextDirection::VerticalTopToBottom;

        cursor.move_down(&direction);
        assert_eq!(cursor.position().row, 1);
        assert_eq!(cursor.position().column, 0);

        cursor.move_left(&direction); // Should move to next column
        assert_eq!(cursor.position().column, 1);
        assert_eq!(cursor.position().row, 0); // Reset to top of column
    }

    #[test]
    fn test_cursor_style() {
        let mut cursor = TerminalCursor::new();
        cursor.set_style(TerminalCursorStyle::Underline);
        assert_eq!(cursor.style(), TerminalCursorStyle::Underline);
        assert_eq!(cursor.display_char(), '_');
    }

    #[test]
    fn test_cursor_visibility() {
        let mut cursor = TerminalCursor::new();
        assert!(cursor.is_visible());

        cursor.set_visible(false);
        assert!(!cursor.is_visible());

        cursor.set_visible(true);
        cursor.set_blink_state(false);
        assert!(!cursor.is_visible()); // Invisible due to blink state
    }

    #[test]
    fn test_cursor_bounds() {
        let cursor = TerminalCursor::new();
        let (x, y, w, h) = cursor.bounds(1, 1);
        assert_eq!((x, y, w, h), (0, 0, 1, 1));

        let mut cursor = TerminalCursor::new();
        cursor.set_position(SpatialPosition { row: 5, column: 10 });
        let (x, y, w, h) = cursor.bounds(2, 2);
        assert_eq!((x, y, w, h), (10, 5, 2, 2));
    }
}
