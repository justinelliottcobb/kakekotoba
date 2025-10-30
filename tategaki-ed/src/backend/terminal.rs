//! Notcurses terminal backend for vertical text rendering
//!
//! This module provides a terminal-based rendering backend using notcurses,
//! with special support for vertical Japanese text using Unicode vertical
//! presentation forms and proper character rotation.

#[cfg(feature = "notcurses")]
use libnotcurses_sys::c_api::*;
#[cfg(feature = "notcurses")]
use std::ptr;
use crate::{Result, TategakiError};
use crate::text_engine::TextDirection;
use crate::spatial::SpatialPosition;
use super::{RenderBackend, Color, Rect, TextStyle, CursorInfo, CursorStyle};

/// Terminal backend using notcurses
pub struct TerminalBackend {
    /// Notcurses context (raw pointer)
    #[cfg(feature = "notcurses")]
    nc: *mut notcurses,
    #[cfg(not(feature = "notcurses"))]
    nc: (),
    /// Current viewport dimensions (columns, rows)
    viewport: (u32, u32),
    /// Whether the backend is active
    active: bool,
    /// Background color
    bg_color: Color,
}

// SAFETY: TerminalBackend is designed for single-threaded use.
// The notcurses library is not thread-safe, but we never actually
// share the backend across threads in practice.
#[cfg(feature = "notcurses")]
unsafe impl Send for TerminalBackend {}
#[cfg(feature = "notcurses")]
unsafe impl Sync for TerminalBackend {}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "notcurses")]
            nc: ptr::null_mut(),
            #[cfg(not(feature = "notcurses"))]
            nc: (),
            viewport: (80, 24), // Default terminal size
            active: false,
            bg_color: Color::black(),
        })
    }

    /// Convert our Color to notcurses channel
    fn color_to_channel(&self, color: Color) -> u32 {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
    }

    /// Convert character to vertical form if available
    ///
    /// This handles conversion of punctuation and special characters to their
    /// vertical presentation forms (Unicode U+FE10-U+FE19 range).
    fn to_vertical_form(&self, ch: char) -> char {
        match ch {
            // Punctuation marks
            '、' => '︑', // PRESENTATION FORM FOR VERTICAL IDEOGRAPHIC COMMA
            '。' => '︒', // PRESENTATION FORM FOR VERTICAL IDEOGRAPHIC FULL STOP
            '，' => '︐', // PRESENTATION FORM FOR VERTICAL COMMA
            '．' => '・', // Already vertical
            '：' => '︓', // PRESENTATION FORM FOR VERTICAL COLON
            '；' => '︔', // PRESENTATION FORM FOR VERTICAL SEMICOLON
            '！' => '︕', // PRESENTATION FORM FOR VERTICAL EXCLAMATION MARK
            '？' => '︖', // PRESENTATION FORM FOR VERTICAL QUESTION MARK

            // Brackets - rotate these
            '「' => '﹁', // VERTICAL LEFT CORNER BRACKET
            '」' => '﹂', // VERTICAL RIGHT CORNER BRACKET
            '『' => '﹃', // VERTICAL LEFT DOUBLE ANGLE BRACKET
            '』' => '﹄', // VERTICAL RIGHT DOUBLE ANGLE BRACKET
            '（' => '︵', // PRESENTATION FORM FOR VERTICAL LEFT PARENTHESIS
            '）' => '︶', // PRESENTATION FORM FOR VERTICAL RIGHT PARENTHESIS
            '〔' => '︗', // PRESENTATION FORM FOR VERTICAL LEFT SQUARE BRACKET
            '〕' => '︘', // PRESENTATION FORM FOR VERTICAL RIGHT SQUARE BRACKET

            // Long vowel mark (very important for Japanese vertical text)
            'ー' => '｜', // Convert horizontal long vowel to vertical bar

            // Latin characters and numbers stay the same (will be rotated)
            // CJK characters stay the same (naturally vertical)
            _ => ch,
        }
    }

    /// Check if character should be rotated in vertical text
    fn should_rotate_char(&self, ch: char) -> bool {
        // Rotate Latin alphabet, numbers, and some ASCII punctuation
        matches!(ch,
            'A'..='Z' | 'a'..='z' | '0'..='9' |
            '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' |
            ',' | '-' | '.' | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' |
            '[' | '\\' | ']' | '^' | '_' | '`' | '{' | '|' | '}' | '~'
        )
    }

    /// Get standard plane
    #[cfg(feature = "notcurses")]
    unsafe fn stdplane(&mut self) -> *mut ncplane {
        if self.nc.is_null() {
            ptr::null_mut()
        } else {
            notcurses_stdplane(self.nc)
        }
    }

    /// Render a single character at the given position
    #[cfg(feature = "notcurses")]
    unsafe fn render_char(
        &mut self,
        ch: char,
        col: u32,
        row: u32,
        style: &TextStyle,
    ) -> Result<()> {
        let plane = self.stdplane();
        if plane.is_null() {
            return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
        }

        // Set colors (cast u8 to unsigned/u32)
        ncplane_set_fg_rgb8(plane, style.color.r as u32, style.color.g as u32, style.color.b as u32);
        if let Some(bg) = style.background {
            ncplane_set_bg_rgb8(plane, bg.r as u32, bg.g as u32, bg.b as u32);
        }

        // Put character at position
        let ch_str = ch.to_string();
        ncplane_putegc_yx(&mut *plane, Some(row), Some(col), &ch_str, None);

        Ok(())
    }

    #[cfg(not(feature = "notcurses"))]
    fn render_char(&mut self, _ch: char, _col: u32, _row: u32, _style: &TextStyle) -> Result<()> {
        Ok(())
    }

    /// Calculate position for vertical text rendering
    ///
    /// In vertical Japanese text (tategaki):
    /// - Text flows top-to-bottom in columns
    /// - Columns progress right-to-left
    /// - Each character occupies approximately 2 cells width (for full-width chars)
    fn calc_vertical_position(&self, logical_x: f32, logical_y: f32, char_index: usize) -> (u32, u32) {
        // For vertical text:
        // - logical_x represents the column number (right to left)
        // - logical_y represents the row within that column (top to bottom)

        let cols = self.viewport.0;
        let rows = self.viewport.1;

        // Start from right side for traditional vertical text
        let col = (cols as f32 - logical_x * 2.0) as u32;
        let row = logical_y as u32;

        (col.min(cols.saturating_sub(1)), row.min(rows.saturating_sub(1)))
    }

    /// Calculate position for horizontal text rendering
    fn calc_horizontal_position(&self, logical_x: f32, logical_y: f32) -> (u32, u32) {
        (logical_x as u32, logical_y as u32)
    }

    /// Get input event from notcurses (non-blocking)
    ///
    /// Returns None if no input is available, or Some((keycode, ctrl, alt, shift))
    #[cfg(feature = "notcurses")]
    pub fn get_input(&mut self) -> Option<(u32, bool, bool, bool)> {
        unsafe {
            if self.nc.is_null() {
                return None;
            }

            let mut input: ncinput = std::mem::zeroed();
            let ts = ffi::timespec { tv_sec: 0, tv_nsec: 0 };
            let result = notcurses_get(self.nc, &ts, &mut input);

            if result == 0 {
                return None;
            }

            // Extract modifier flags from the ncinput struct
            let ctrl = input.ctrl;
            let alt = input.alt;
            let shift = input.shift;

            Some((input.id, ctrl, alt, shift))
        }
    }

    #[cfg(not(feature = "notcurses"))]
    pub fn get_input(&mut self) -> Option<(u32, bool, bool, bool)> {
        None
    }
}

impl RenderBackend for TerminalBackend {
    #[cfg(feature = "notcurses")]
    fn init(&mut self) -> Result<()> {
        unsafe {
            // Initialize notcurses with default options
            let mut opts: notcurses_options = std::mem::zeroed();
            opts.flags = 0; // Default flags

            self.nc = notcurses_init(&opts, ptr::null_mut());
            if self.nc.is_null() {
                return Err(TategakiError::Rendering("Failed to initialize notcurses".to_string()));
            }

            // Get viewport dimensions
            let stdplane = notcurses_stdplane(self.nc);
            if !stdplane.is_null() {
                let mut rows: u32 = 0;
                let mut cols: u32 = 0;
                ncplane_dim_yx(stdplane, &mut rows, &mut cols);
                self.viewport = (cols, rows);
            }

            self.active = true;
            Ok(())
        }
    }

    #[cfg(not(feature = "notcurses"))]
    fn init(&mut self) -> Result<()> {
        Err(TategakiError::Rendering("Notcurses feature not enabled".to_string()))
    }

    #[cfg(feature = "notcurses")]
    fn shutdown(&mut self) -> Result<()> {
        unsafe {
            if !self.nc.is_null() {
                notcurses_stop(self.nc);
                self.nc = ptr::null_mut();
            }
        }
        self.active = false;
        Ok(())
    }

    #[cfg(not(feature = "notcurses"))]
    fn shutdown(&mut self) -> Result<()> {
        self.active = false;
        Ok(())
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.viewport
    }

    #[cfg(feature = "notcurses")]
    fn clear(&mut self, color: Color) -> Result<()> {
        unsafe {
            self.bg_color = color;
            let plane = self.stdplane();
            if plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            // Set background color using RGB values (cast u8 to u32)
            ncplane_set_bg_rgb8(plane, color.r as u32, color.g as u32, color.b as u32);
            ncplane_erase(plane);

            Ok(())
        }
    }

    #[cfg(not(feature = "notcurses"))]
    fn clear(&mut self, color: Color) -> Result<()> {
        self.bg_color = color;
        Ok(())
    }

    fn render_text(
        &mut self,
        text: &str,
        position: (f32, f32),
        style: &TextStyle,
        direction: TextDirection,
    ) -> Result<()> {
        match direction {
            TextDirection::VerticalTopToBottom => {
                // Render vertical text top-to-bottom, right-to-left
                for (i, ch) in text.chars().enumerate() {
                    let vertical_ch = self.to_vertical_form(ch);
                    let (col, row) = self.calc_vertical_position(
                        position.0,
                        position.1 + i as f32,
                        i
                    );

                    unsafe {
                        self.render_char(vertical_ch, col, row, style)?;
                    }
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // Render horizontal text left-to-right
                for (i, ch) in text.chars().enumerate() {
                    let (col, row) = self.calc_horizontal_position(
                        position.0 + i as f32,
                        position.1
                    );

                    unsafe {
                        self.render_char(ch, col, row, style)?;
                    }
                }
            }
            _ => {
                // For other directions, fall back to horizontal
                return self.render_text(text, position, style, TextDirection::HorizontalLeftToRight);
            }
        }

        Ok(())
    }

    #[cfg(feature = "notcurses")]
    fn render_cursor(&mut self, cursor: &CursorInfo) -> Result<()> {
        unsafe {
            let plane = self.stdplane();
            if plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            let col = cursor.position.column as i32;
            let row = cursor.position.row as i32;

            // Set cursor color (cast u8 to u32)
            ncplane_set_fg_rgb8(plane, cursor.color.r as u32, cursor.color.g as u32, cursor.color.b as u32);

            // Render cursor based on style
            let cursor_char = match cursor.style {
                CursorStyle::Block => "█",
                CursorStyle::Line => "│",
                CursorStyle::Underline => "_",
            };

            ncplane_putegc_yx(&mut *plane, Some(row as u32), Some(col as u32), &cursor_char, None);

            Ok(())
        }
    }

    #[cfg(not(feature = "notcurses"))]
    fn render_cursor(&mut self, _cursor: &CursorInfo) -> Result<()> {
        Ok(())
    }

    fn render_selection(&mut self, bounds: Rect, color: Color) -> Result<()> {
        #[cfg(feature = "notcurses")]
        unsafe {
            let plane = self.stdplane();
            if plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            ncplane_set_bg_rgb8(plane, color.r as u32, color.g as u32, color.b as u32);

            // Fill selection with spaces
            for y in (bounds.y as i32)..((bounds.y + bounds.height) as i32) {
                for x in (bounds.x as i32)..((bounds.x + bounds.width) as i32) {
                    ncplane_putegc_yx(&mut *plane, Some(y as u32), Some(x as u32), " ", None);
                }
            }
        }
        Ok(())
    }

    fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        color: Color,
        _thickness: f32,
    ) -> Result<()> {
        #[cfg(feature = "notcurses")]
        unsafe {
            let plane = self.stdplane();
            if plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            ncplane_set_fg_rgb8(plane, color.r as u32, color.g as u32, color.b as u32);

            let from_col = from.0 as i32;
            let from_row = from.1 as i32;
            let to_col = to.0 as i32;
            let to_row = to.1 as i32;

            if from_row == to_row {
                // Horizontal line
                for col in from_col..=to_col {
                    ncplane_putegc_yx(&mut *plane, Some(from_row as u32), Some(col as u32), "─", None);
                }
            } else if from_col == to_col {
                // Vertical line
                for row in from_row..=to_row {
                    ncplane_putegc_yx(&mut *plane, Some(row as u32), Some(from_col as u32), "│", None);
                }
            }
        }
        Ok(())
    }

    fn render_rect(&mut self, bounds: Rect, color: Color, filled: bool) -> Result<()> {
        #[cfg(feature = "notcurses")]
        unsafe {
            let plane = self.stdplane();
            if plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            ncplane_set_fg_rgb8(plane, color.r as u32, color.g as u32, color.b as u32);

            let x = bounds.x as i32;
            let y = bounds.y as i32;
            let w = bounds.width as i32;
            let h = bounds.height as i32;

            if filled {
                ncplane_set_bg_rgb8(plane, color.r as u32, color.g as u32, color.b as u32);
                for row in y..(y + h) {
                    for col in x..(x + w) {
                        ncplane_putegc_yx(&mut *plane, Some(row as u32), Some(col as u32), " ", None);
                    }
                }
            } else {
                // Draw outline
                ncplane_putegc_yx(&mut *plane, Some(y), Some(x), "┌", None);
                ncplane_putegc_yx(&mut *plane, Some(y), Some(x + w - 1), "┐", None);
                ncplane_putegc_yx(&mut *plane, Some(y + h - 1), Some(x), "└", None);
                ncplane_putegc_yx(&mut *plane, Some(y + h - 1), Some(x + w - 1), "┘", None);

                for col in (x + 1)..(x + w - 1) {
                    ncplane_putegc_yx(&mut *plane, Some(y), Some(col), "─", None);
                    ncplane_putegc_yx(&mut *plane, Some(y + h - 1), Some(col), "─", None);
                }

                for row in (y + 1)..(y + h - 1) {
                    ncplane_putegc_yx(&mut *plane, Some(row), Some(x), "│", None);
                    ncplane_putegc_yx(&mut *plane, Some(row), Some(x + w - 1), "│", None);
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "notcurses")]
    fn present(&mut self) -> Result<()> {
        unsafe {
            if self.nc.is_null() {
                return Err(TategakiError::Rendering("Notcurses not initialized".to_string()));
            }

            let stdplane = notcurses_stdplane(self.nc);
            if !stdplane.is_null() {
                ncpile_render(stdplane);
                ncpile_rasterize(stdplane);
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "notcurses"))]
    fn present(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn handle_resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.viewport = (width, height);
        Ok(())
    }
}

impl Drop for TerminalBackend {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_form_conversion() {
        let backend = TerminalBackend::new().unwrap();

        assert_eq!(backend.to_vertical_form('、'), '︑');
        assert_eq!(backend.to_vertical_form('。'), '︒');
        assert_eq!(backend.to_vertical_form('ー'), '｜');
    }

    #[test]
    fn test_should_rotate_char() {
        let backend = TerminalBackend::new().unwrap();

        assert!(backend.should_rotate_char('A'));
        assert!(backend.should_rotate_char('5'));
        assert!(!backend.should_rotate_char('あ'));
        assert!(!backend.should_rotate_char('漢'));
    }

    #[test]
    fn test_color_to_channel() {
        let backend = TerminalBackend::new().unwrap();
        let color = Color::new(255, 128, 64, 255);
        let channel = backend.color_to_channel(color);

        assert_eq!((channel >> 16) & 0xFF, 255);
        assert_eq!((channel >> 8) & 0xFF, 128);
        assert_eq!(channel & 0xFF, 64);
    }
}
