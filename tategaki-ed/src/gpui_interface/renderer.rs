//! Text rendering engine for GPUI interface

use crate::spatial::SpatialPosition;
use crate::text_engine::{LayoutEngine, TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
/// Vertical text renderer for GPUI
pub struct VerticalTextRenderer {
    /// Text direction
    direction: TextDirection,
    /// Font family
    font_family: String,
    /// Font size
    font_size: f32,
    /// Line height multiplier
    line_height: f32,
    /// Text color
    text_color: Hsla,
    /// Background color
    background_color: Hsla,
}

#[cfg(feature = "gpui")]
impl VerticalTextRenderer {
    /// Create new renderer
    pub fn new(direction: TextDirection) -> Self {
        Self {
            direction,
            font_family: "Noto Sans CJK JP".to_string(),
            font_size: 14.0,
            line_height: 1.4,
            text_color: gpui::rgb(0xffffff),
            background_color: gpui::rgb(0x1e1e1e),
        }
    }

    /// Set font configuration
    pub fn set_font(&mut self, family: String, size: f32) {
        self.font_family = family;
        self.font_size = size;
    }

    /// Set colors
    pub fn set_colors(&mut self, text: Hsla, background: Hsla) {
        self.text_color = text;
        self.background_color = background;
    }

    /// Render text buffer
    pub fn render_buffer(
        &self,
        buffer: &VerticalTextBuffer,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
        layout_engine: &LayoutEngine,
    ) {
        let text_content = buffer.as_text();

        match self.direction {
            TextDirection::VerticalTopToBottom => {
                self.render_vertical_text(&text_content, bounds, cx, layout_engine);
            }
            TextDirection::HorizontalLeftToRight => {
                self.render_horizontal_text(&text_content, bounds, cx);
            }
        }
    }

    /// Render text vertically (top to bottom, right to left columns)
    fn render_vertical_text(
        &self,
        text: &str,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
        layout_engine: &LayoutEngine,
    ) {
        let lines: Vec<&str> = text.lines().collect();
        let char_width = self.font_size;
        let char_height = self.font_size * self.line_height;

        // Start from right side of bounds
        let mut column_x = bounds.right() - char_width;

        for line in lines {
            let mut char_y = bounds.top();

            for ch in line.chars() {
                if column_x < bounds.left() {
                    break; // Out of bounds
                }

                // Render character at position
                let char_bounds = Bounds {
                    origin: Point {
                        x: column_x,
                        y: char_y,
                    },
                    size: Size {
                        width: char_width,
                        height: char_height,
                    },
                };

                self.render_character(ch, char_bounds, cx);
                char_y += char_height;

                if char_y >= bounds.bottom() {
                    break; // Out of bounds vertically
                }
            }

            column_x -= char_width * 1.2; // Move to next column with spacing
        }
    }

    /// Render text horizontally (left to right)
    fn render_horizontal_text(&self, text: &str, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        let lines: Vec<&str> = text.lines().collect();
        let line_height = self.font_size * self.line_height;

        let mut line_y = bounds.top();

        for line in lines {
            let line_bounds = Bounds {
                origin: Point {
                    x: bounds.left(),
                    y: line_y,
                },
                size: Size {
                    width: bounds.size.width,
                    height: line_height,
                },
            };

            self.render_line(line, line_bounds, cx);
            line_y += line_height;

            if line_y >= bounds.bottom() {
                break; // Out of bounds
            }
        }
    }

    /// Render a single line of text
    fn render_line(&self, line: &str, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        // For now, use simple text rendering
        // In a real implementation, you'd want proper text shaping for CJK
        cx.paint_text(
            TextRun {
                text: line.into(),
                font: Font {
                    family: self.font_family.clone().into(),
                    features: FontFeatures::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                },
                color: self.text_color,
                background_color: Some(self.background_color),
                decoration: TextDecoration::default(),
            },
            bounds.origin,
        );
    }

    /// Render a single character (for vertical layout)
    fn render_character(&self, ch: char, bounds: Bounds<Pixels>, cx: &mut WindowContext) {
        let text = ch.to_string();

        cx.paint_text(
            TextRun {
                text: text.into(),
                font: Font {
                    family: self.font_family.clone().into(),
                    features: FontFeatures::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                },
                color: self.text_color,
                background_color: Some(self.background_color),
                decoration: TextDecoration::default(),
            },
            bounds.origin,
        );
    }

    /// Calculate text metrics
    pub fn measure_text(&self, text: &str, cx: &WindowContext) -> Size<Pixels> {
        // Simple measurement - in real implementation you'd use proper text metrics
        let char_count = text.chars().count();
        let line_count = text.lines().count().max(1);

        match self.direction {
            TextDirection::VerticalTopToBottom => Size {
                width: Pixels(self.font_size * line_count as f32 * 1.2),
                height: Pixels(self.font_size * self.line_height * char_count as f32),
            },
            TextDirection::HorizontalLeftToRight => {
                Size {
                    width: Pixels(self.font_size * 0.6 * char_count as f32), // Approximate
                    height: Pixels(self.font_size * self.line_height * line_count as f32),
                }
            }
        }
    }

    /// Render syntax highlighting overlay
    pub fn render_syntax_highlighting(
        &self,
        buffer: &VerticalTextBuffer,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
    ) {
        // TODO: Implement syntax highlighting for spatial programming
        // This would analyze the buffer for keywords, operators, etc.
        // and render colored overlays
    }

    /// Render debugging information
    pub fn render_debug_info(
        &self,
        position: SpatialPosition,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
    ) {
        // Render position indicator for debugging
        let debug_text = format!("({}, {})", position.row, position.column);

        cx.paint_text(
            TextRun {
                text: debug_text.into(),
                font: Font {
                    family: "monospace".into(),
                    features: FontFeatures::default(),
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Normal,
                },
                color: gpui::rgb(0x888888),
                background_color: None,
                decoration: TextDecoration::default(),
            },
            Point {
                x: bounds.left(),
                y: bounds.bottom() - Pixels(20.0),
            },
        );
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder renderer when GPUI is disabled
pub struct VerticalTextRenderer {
    direction: TextDirection,
}

#[cfg(not(feature = "gpui"))]
impl VerticalTextRenderer {
    pub fn new(direction: TextDirection) -> Self {
        Self { direction }
    }

    pub fn render_buffer(
        &self,
        _buffer: &VerticalTextBuffer,
        _bounds: (),
        _cx: (),
        _layout_engine: &LayoutEngine,
    ) -> Result<()> {
        Err(TategakiError::Rendering(
            "GPUI feature not enabled".to_string(),
        ))
    }
}
