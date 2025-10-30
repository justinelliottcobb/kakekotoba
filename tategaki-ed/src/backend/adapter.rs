//! Adapter layer for translating GPUI API calls to notcurses
//!
//! This module provides translation utilities to map GPUI's pixel-based
//! coordinate system and rendering primitives to notcurses' cell-based
//! terminal rendering.

use crate::{Result, TategakiError};
use crate::text_engine::TextDirection;
use super::{Color, Rect, TextStyle, CursorInfo, RenderBackend};
use super::terminal::TerminalBackend;

/// Adapter for translating GPUI coordinates to terminal cells
pub struct GpuiToNotcursesAdapter {
    /// Reference to the terminal backend
    backend: TerminalBackend,
    /// Scale factor for pixel-to-cell conversion
    cell_width: f32,
    cell_height: f32,
    /// Character width cache for accurate positioning
    avg_char_width: f32,
}

impl GpuiToNotcursesAdapter {
    /// Create a new adapter
    pub fn new(backend: TerminalBackend) -> Result<Self> {
        // Typical monospace terminal cell dimensions
        // These can be adjusted based on actual terminal font metrics
        let cell_width = 8.0;  // pixels per cell width
        let cell_height = 16.0; // pixels per cell height
        let avg_char_width = 1.0; // Average character width in cells

        Ok(Self {
            backend,
            cell_width,
            cell_height,
            avg_char_width,
        })
    }

    /// Get mutable reference to the backend
    pub fn backend_mut(&mut self) -> &mut TerminalBackend {
        &mut self.backend
    }

    /// Get immutable reference to the backend
    pub fn backend(&self) -> &TerminalBackend {
        &self.backend
    }

    /// Consume the adapter and return the backend
    pub fn into_backend(self) -> TerminalBackend {
        self.backend
    }

    /// Convert GPUI pixel coordinates to terminal cell coordinates
    pub fn pixels_to_cells(&self, x: f32, y: f32) -> (u32, u32) {
        let col = (x / self.cell_width).round() as u32;
        let row = (y / self.cell_height).round() as u32;
        (col, row)
    }

    /// Convert terminal cell coordinates to GPUI pixel coordinates
    pub fn cells_to_pixels(&self, col: u32, row: u32) -> (f32, f32) {
        let x = col as f32 * self.cell_width;
        let y = row as f32 * self.cell_height;
        (x, y)
    }

    /// Convert GPUI pixel bounds to terminal cell bounds
    pub fn pixels_bounds_to_cells(&self, bounds: Rect) -> Rect {
        let (x, y) = self.pixels_to_cells(bounds.x, bounds.y);
        let (w, h) = self.pixels_to_cells(bounds.width, bounds.height);

        Rect::new(x as f32, y as f32, w as f32, h as f32)
    }

    /// Calculate how many cells are needed for a string
    ///
    /// This accounts for full-width CJK characters which take 2 cells
    pub fn string_width_in_cells(&self, text: &str) -> u32 {
        let mut width = 0;
        for ch in text.chars() {
            width += self.char_width_in_cells(ch);
        }
        width
    }

    /// Get the width of a character in terminal cells
    ///
    /// CJK characters, fullwidth forms, and emoji typically use 2 cells
    pub fn char_width_in_cells(&self, ch: char) -> u32 {
        use unicode_segmentation::UnicodeSegmentation;

        // Check for full-width characters
        match ch {
            // CJK Unified Ideographs
            '\u{4E00}'..='\u{9FFF}' |
            // CJK Extension A
            '\u{3400}'..='\u{4DBF}' |
            // Hiragana
            '\u{3040}'..='\u{309F}' |
            // Katakana
            '\u{30A0}'..='\u{30FF}' |
            // Fullwidth ASCII variants
            '\u{FF01}'..='\u{FF5E}' |
            // Halfwidth Katakana already handled
            // CJK Compatibility Ideographs
            '\u{F900}'..='\u{FAFF}' => 2,

            // Regular ASCII and most other characters
            _ => 1,
        }
    }

    /// Adjust text style for terminal rendering
    ///
    /// Terminals have limited color support, so we map to closest available colors
    pub fn adapt_text_style(&self, style: &TextStyle) -> TextStyle {
        // For now, pass through as-is
        // In a real implementation, we'd map 24-bit colors to terminal palette
        style.clone()
    }

    /// Convert GPUI paint operations to terminal rendering
    ///
    /// This translates high-level GPUI paint operations (which might include
    /// bezier curves, gradients, etc.) to simple terminal cell operations
    pub fn paint_text(
        &mut self,
        text: &str,
        pixel_position: (f32, f32),
        style: &TextStyle,
        direction: TextDirection,
    ) -> Result<()> {
        // Convert pixel position to cell position
        let cell_position = self.pixels_to_cells(pixel_position.0, pixel_position.1);

        // Adapt the style for terminal
        let terminal_style = self.adapt_text_style(style);

        // Render using the backend
        self.backend.render_text(
            text,
            (cell_position.0 as f32, cell_position.1 as f32),
            &terminal_style,
            direction,
        )
    }

    /// Paint a GPUI cursor as a terminal cursor
    pub fn paint_cursor(
        &mut self,
        cursor: &CursorInfo,
    ) -> Result<()> {
        self.backend.render_cursor(cursor)
    }

    /// Paint a GPUI selection as terminal background highlighting
    pub fn paint_selection(
        &mut self,
        pixel_bounds: Rect,
        color: Color,
    ) -> Result<()> {
        let cell_bounds = self.pixels_bounds_to_cells(pixel_bounds);
        self.backend.render_selection(cell_bounds, color)
    }

    /// Paint a GPUI rectangle in the terminal
    pub fn paint_rect(
        &mut self,
        pixel_bounds: Rect,
        color: Color,
        filled: bool,
    ) -> Result<()> {
        let cell_bounds = self.pixels_bounds_to_cells(pixel_bounds);
        self.backend.render_rect(cell_bounds, color, filled)
    }

    /// Paint a GPUI line in the terminal
    pub fn paint_line(
        &mut self,
        from_pixels: (f32, f32),
        to_pixels: (f32, f32),
        color: Color,
        thickness: f32,
    ) -> Result<()> {
        let from_cells = self.pixels_to_cells(from_pixels.0, from_pixels.1);
        let to_cells = self.pixels_to_cells(to_pixels.0, to_pixels.1);

        self.backend.render_line(
            (from_cells.0 as f32, from_cells.1 as f32),
            (to_cells.0 as f32, to_cells.1 as f32),
            color,
            thickness,
        )
    }

    /// Handle GPUI bounds to terminal viewport mapping
    ///
    /// GPUI uses floating-point pixel bounds; terminals use integer cell grids
    pub fn map_viewport(&self, gpui_width: f32, gpui_height: f32) -> (u32, u32) {
        self.pixels_to_cells(gpui_width, gpui_height)
    }

    /// Calculate optimal font size for terminal rendering
    ///
    /// Since terminals have fixed cell sizes, this is mainly for reference
    pub fn terminal_font_size(&self) -> f32 {
        self.cell_height
    }

    /// Estimate how many lines of text fit in a pixel height
    pub fn lines_in_pixel_height(&self, pixel_height: f32) -> u32 {
        (pixel_height / self.cell_height).floor() as u32
    }

    /// Estimate how many columns fit in a pixel width
    pub fn columns_in_pixel_width(&self, pixel_width: f32) -> u32 {
        (pixel_width / self.cell_width).floor() as u32
    }
}

/// Helper functions for common GPUI-to-terminal conversions

/// Check if a color is "close enough" in terminal color space
///
/// Terminals typically have 256 colors or 16 million colors, but
/// rendering may quantize differently than GPUI
pub fn colors_similar(c1: Color, c2: Color, threshold: u8) -> bool {
    let dr = (c1.r as i16 - c2.r as i16).abs();
    let dg = (c1.g as i16 - c2.g as i16).abs();
    let db = (c1.b as i16 - c2.b as i16).abs();

    dr <= threshold as i16 && dg <= threshold as i16 && db <= threshold as i16
}

/// Quantize a 24-bit RGB color to a 256-color terminal palette
///
/// This uses the standard xterm 256-color palette
pub fn quantize_to_256_colors(color: Color) -> Color {
    // Simplified quantization - in practice you'd want to map to
    // the actual xterm 256 color palette
    let r = (color.r / 43) * 43; // Quantize to 6 levels (0, 43, 85, 128, 170, 213, 255)
    let g = (color.g / 43) * 43;
    let b = (color.b / 43) * 43;

    Color::new(r, g, b, color.a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixels_to_cells() {
        let backend = TerminalBackend::new().unwrap();
        let adapter = GpuiToNotcursesAdapter::new(backend).unwrap();

        let (col, row) = adapter.pixels_to_cells(80.0, 160.0);
        assert_eq!(col, 10); // 80 / 8
        assert_eq!(row, 10); // 160 / 16
    }

    #[test]
    fn test_cells_to_pixels() {
        let backend = TerminalBackend::new().unwrap();
        let adapter = GpuiToNotcursesAdapter::new(backend).unwrap();

        let (x, y) = adapter.cells_to_pixels(10, 10);
        assert_eq!(x, 80.0); // 10 * 8
        assert_eq!(y, 160.0); // 10 * 16
    }

    #[test]
    fn test_char_width_in_cells() {
        let backend = TerminalBackend::new().unwrap();
        let adapter = GpuiToNotcursesAdapter::new(backend).unwrap();

        // ASCII character
        assert_eq!(adapter.char_width_in_cells('A'), 1);

        // Full-width Japanese characters
        assert_eq!(adapter.char_width_in_cells('あ'), 2);
        assert_eq!(adapter.char_width_in_cells('漢'), 2);
        assert_eq!(adapter.char_width_in_cells('カ'), 2);
    }

    #[test]
    fn test_string_width_in_cells() {
        let backend = TerminalBackend::new().unwrap();
        let adapter = GpuiToNotcursesAdapter::new(backend).unwrap();

        // Mixed ASCII and Japanese
        let text = "Hello世界";
        let width = adapter.string_width_in_cells(text);
        assert_eq!(width, 9); // 5 (Hello) + 4 (世界 = 2*2)
    }

    #[test]
    fn test_colors_similar() {
        let c1 = Color::new(100, 150, 200, 255);
        let c2 = Color::new(105, 145, 205, 255);
        let c3 = Color::new(150, 150, 200, 255);

        assert!(colors_similar(c1, c2, 10));
        assert!(!colors_similar(c1, c3, 10));
    }

    #[test]
    fn test_quantize_to_256_colors() {
        let color = Color::new(127, 200, 50, 255);
        let quantized = quantize_to_256_colors(color);

        // Values should be quantized to nearest step
        assert!(quantized.r <= 128);
        assert!(quantized.g >= 170);
        assert!(quantized.b <= 43);
    }
}
