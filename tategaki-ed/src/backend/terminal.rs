//! Notcurses terminal backend for vertical text rendering
//!
//! This module provides a terminal-based rendering backend using notcurses,
//! with special support for vertical Japanese text using Unicode vertical
//! presentation forms and proper character rotation.

#[cfg(feature = "notcurses")]
use libnotcurses_sys::*;
use crate::{Result, TategakiError};
use crate::text_engine::TextDirection;
use crate::spatial::SpatialPosition;
use super::{RenderBackend, Color, Rect, TextStyle, CursorInfo, CursorStyle};
use std::ffi::CString;
use std::ptr;

/// Terminal backend using notcurses
pub struct TerminalBackend {
    /// Notcurses context
    nc: *mut Notcurses,
    /// Standard plane (main rendering surface)
    std_plane: *mut NcPlane,
    /// Current viewport dimensions (columns, rows)
    viewport: (u32, u32),
    /// Whether the backend is active
    active: bool,
    /// Background color
    bg_color: Color,
}

impl TerminalBackend {
    /// Create a new terminal backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            nc: ptr::null_mut(),
            std_plane: ptr::null_mut(),
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

    /// Render a single character at the given position
    unsafe fn render_char(
        &mut self,
        ch: char,
        col: u32,
        row: u32,
        style: &TextStyle,
    ) -> Result<()> {
        if self.std_plane.is_null() {
            return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
        }

        // Set colors
        let fg_channel = self.color_to_channel(style.color);
        let bg_channel = if let Some(bg) = style.background {
            self.color_to_channel(bg)
        } else {
            self.color_to_channel(self.bg_color)
        };

        // Create cell with proper styling
        let mut cell: NcCell = std::mem::zeroed();
        nccell_set_fg_rgb8(&mut cell, style.color.r, style.color.g, style.color.b);
        if let Some(bg) = style.background {
            nccell_set_bg_rgb8(&mut cell, bg.r, bg.g, bg.b);
        }

        // Move cursor to position
        ncplane_cursor_move_yx(self.std_plane, row as i32, col as i32);

        // Put the character
        let ch_str = CString::new(ch.to_string()).map_err(|e| {
            TategakiError::Rendering(format!("Failed to convert character to CString: {}", e))
        })?;

        ncplane_putstr_yx(
            self.std_plane,
            row as i32,
            col as i32,
            ch_str.as_ptr()
        );

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
}

impl RenderBackend for TerminalBackend {
    fn init(&mut self) -> Result<()> {
        unsafe {
            // Initialize notcurses with options for vertical text support
            let mut opts: NotcursesOptions = std::mem::zeroed();
            opts.flags = NCOPTION_SUPPRESS_BANNERS | NCOPTION_NO_ALTERNATE_SCREEN;
            opts.loglevel = NCLOGLEVEL_ERROR;

            self.nc = notcurses_init(&opts, ptr::null_mut());
            if self.nc.is_null() {
                return Err(TategakiError::Rendering(
                    "Failed to initialize notcurses".to_string()
                ));
            }

            // Get the standard plane
            self.std_plane = notcurses_stdplane(self.nc);
            if self.std_plane.is_null() {
                notcurses_stop(self.nc);
                return Err(TategakiError::Rendering(
                    "Failed to get standard plane".to_string()
                ));
            }

            // Get viewport dimensions
            let mut rows: u32 = 0;
            let mut cols: u32 = 0;
            ncplane_dim_yx(self.std_plane, &mut rows, &mut cols);
            self.viewport = (cols, rows);

            // Enable mouse support if available
            notcurses_mouse_enable(self.nc, NCMICE_ALL_EVENTS);

            self.active = true;
            Ok(())
        }
    }

    fn shutdown(&mut self) -> Result<()> {
        unsafe {
            if !self.nc.is_null() {
                notcurses_stop(self.nc);
                self.nc = ptr::null_mut();
                self.std_plane = ptr::null_mut();
            }
            self.active = false;
            Ok(())
        }
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.viewport
    }

    fn clear(&mut self, color: Color) -> Result<()> {
        unsafe {
            if self.std_plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            self.bg_color = color;

            // Set background color and clear
            let channel = self.color_to_channel(color);
            ncplane_set_bg_rgb8(self.std_plane, color.r, color.g, color.b);
            ncplane_erase(self.std_plane);

            Ok(())
        }
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

    fn render_cursor(&mut self, cursor: &CursorInfo) -> Result<()> {
        unsafe {
            if self.std_plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            // Calculate cursor position
            let col = cursor.position.column as i32;
            let row = cursor.position.row as i32;

            // Render cursor based on style
            match cursor.style {
                CursorStyle::Block => {
                    // Draw a filled block
                    ncplane_cursor_move_yx(self.std_plane, row, col);
                    let block_char = CString::new("█").unwrap();
                    ncplane_putstr(self.std_plane, block_char.as_ptr());
                }
                CursorStyle::Line => {
                    // Draw a vertical line
                    ncplane_cursor_move_yx(self.std_plane, row, col);
                    let line_char = CString::new("│").unwrap();
                    ncplane_putstr(self.std_plane, line_char.as_ptr());
                }
                CursorStyle::Underline => {
                    // Draw an underline
                    ncplane_cursor_move_yx(self.std_plane, row + 1, col);
                    let underline_char = CString::new("_").unwrap();
                    ncplane_putstr(self.std_plane, underline_char.as_ptr());
                }
            }

            Ok(())
        }
    }

    fn render_selection(&mut self, bounds: Rect, color: Color) -> Result<()> {
        unsafe {
            if self.std_plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            // Fill the selection rectangle with the specified color
            for y in (bounds.y as u32)..((bounds.y + bounds.height) as u32) {
                for x in (bounds.x as u32)..((bounds.x + bounds.width) as u32) {
                    if x < self.viewport.0 && y < self.viewport.1 {
                        ncplane_cursor_move_yx(self.std_plane, y as i32, x as i32);
                        ncplane_set_bg_rgb8(self.std_plane, color.r, color.g, color.b);
                        let space = CString::new(" ").unwrap();
                        ncplane_putstr(self.std_plane, space.as_ptr());
                    }
                }
            }

            Ok(())
        }
    }

    fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        color: Color,
        _thickness: f32,
    ) -> Result<()> {
        unsafe {
            if self.std_plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            ncplane_set_fg_rgb8(self.std_plane, color.r, color.g, color.b);

            // Simple line drawing using Unicode box drawing characters
            let from_col = from.0 as i32;
            let from_row = from.1 as i32;
            let to_col = to.0 as i32;
            let to_row = to.1 as i32;

            if from_row == to_row {
                // Horizontal line
                let line_char = CString::new("─").unwrap();
                for col in from_col..=to_col {
                    ncplane_cursor_move_yx(self.std_plane, from_row, col);
                    ncplane_putstr(self.std_plane, line_char.as_ptr());
                }
            } else if from_col == to_col {
                // Vertical line
                let line_char = CString::new("│").unwrap();
                for row in from_row..=to_row {
                    ncplane_cursor_move_yx(self.std_plane, row, from_col);
                    ncplane_putstr(self.std_plane, line_char.as_ptr());
                }
            }
            // For diagonal lines, we'd need more complex logic

            Ok(())
        }
    }

    fn render_rect(&mut self, bounds: Rect, color: Color, filled: bool) -> Result<()> {
        unsafe {
            if self.std_plane.is_null() {
                return Err(TategakiError::Rendering("Standard plane not initialized".to_string()));
            }

            ncplane_set_fg_rgb8(self.std_plane, color.r, color.g, color.b);

            let x = bounds.x as i32;
            let y = bounds.y as i32;
            let w = bounds.width as i32;
            let h = bounds.height as i32;

            if filled {
                // Fill the rectangle
                let fill_char = CString::new(" ").unwrap();
                ncplane_set_bg_rgb8(self.std_plane, color.r, color.g, color.b);
                for row in y..(y + h) {
                    for col in x..(x + w) {
                        ncplane_cursor_move_yx(self.std_plane, row, col);
                        ncplane_putstr(self.std_plane, fill_char.as_ptr());
                    }
                }
            } else {
                // Draw rectangle outline using box drawing characters
                let corner_tl = CString::new("┌").unwrap();
                let corner_tr = CString::new("┐").unwrap();
                let corner_bl = CString::new("└").unwrap();
                let corner_br = CString::new("┘").unwrap();
                let horiz = CString::new("─").unwrap();
                let vert = CString::new("│").unwrap();

                // Corners
                ncplane_putstr_yx(self.std_plane, y, x, corner_tl.as_ptr());
                ncplane_putstr_yx(self.std_plane, y, x + w - 1, corner_tr.as_ptr());
                ncplane_putstr_yx(self.std_plane, y + h - 1, x, corner_bl.as_ptr());
                ncplane_putstr_yx(self.std_plane, y + h - 1, x + w - 1, corner_br.as_ptr());

                // Horizontal edges
                for col in (x + 1)..(x + w - 1) {
                    ncplane_putstr_yx(self.std_plane, y, col, horiz.as_ptr());
                    ncplane_putstr_yx(self.std_plane, y + h - 1, col, horiz.as_ptr());
                }

                // Vertical edges
                for row in (y + 1)..(y + h - 1) {
                    ncplane_putstr_yx(self.std_plane, row, x, vert.as_ptr());
                    ncplane_putstr_yx(self.std_plane, row, x + w - 1, vert.as_ptr());
                }
            }

            Ok(())
        }
    }

    fn present(&mut self) -> Result<()> {
        unsafe {
            if self.nc.is_null() {
                return Err(TategakiError::Rendering("Notcurses not initialized".to_string()));
            }

            notcurses_render(self.nc);
            Ok(())
        }
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
