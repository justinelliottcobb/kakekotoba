//! Cursor management for GPUI interface

use crate::spatial::SpatialPosition;
use crate::text_engine::LayoutEngine;
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
/// Spatial cursor for vertical text editing
pub struct SpatialCursor {
    /// Current logical position
    position: SpatialPosition,
    /// Visual position (screen coordinates)
    visual_position: Point<Pixels>,
    /// Cursor visibility (for blinking)
    visible: bool,
    /// Cursor style
    style: CursorStyle,
    /// Cursor color
    color: Hsla,
    /// Cursor width
    width: Pixels,
    /// Cursor height
    height: Pixels,
}

#[cfg(feature = "gpui")]
/// Cursor style options
#[derive(Debug, Clone, Copy)]
pub enum CursorStyle {
    /// Vertical line cursor (traditional)
    Line,
    /// Block cursor (covers character)
    Block,
    /// Underline cursor
    Underline,
}

#[cfg(feature = "gpui")]
impl SpatialCursor {
    /// Create new cursor
    pub fn new() -> Self {
        Self {
            position: SpatialPosition::origin(),
            visual_position: Point::default(),
            visible: true,
            style: CursorStyle::Line,
            color: gpui::rgb(0x00ff00), // Green cursor
            width: Pixels(2.0),
            height: Pixels(20.0),
        }
    }

    /// Set cursor position
    pub fn set_position(&mut self, position: SpatialPosition) {
        self.position = position;
    }

    /// Get current position
    pub fn position(&self) -> SpatialPosition {
        self.position
    }

    /// Set cursor style
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }

    /// Set cursor color
    pub fn set_color(&mut self, color: Hsla) {
        self.color = color;
    }

    /// Move cursor up
    pub fn move_up(&mut self) {
        if self.position.row > 0 {
            self.position.row -= 1;
        }
    }

    /// Move cursor down
    pub fn move_down(&mut self) {
        self.position.row += 1;
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        self.position.column += 1;
    }

    /// Move cursor backward in text flow
    pub fn move_backward(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        } else if self.position.row > 0 {
            self.position.row -= 1;
            // Would need to get line length here
            self.position.column = 0; // Placeholder
        }
    }

    /// Move cursor forward in text flow
    pub fn advance(&mut self) {
        self.position.column += 1;
        // Line wrapping logic would go here
    }

    /// Move to next column (for vertical text)
    pub fn move_to_next_column(&mut self) {
        self.position.column += 1;
        self.position.row = 0; // Reset to top of column
    }

    /// Move to previous column (for vertical text)
    pub fn move_to_prev_column(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
            self.position.row = 0; // Reset to top of column
        }
    }

    /// Reset cursor to origin
    pub fn reset(&mut self) {
        self.position = SpatialPosition::origin();
    }

    /// Toggle cursor visibility (for blinking)
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Set cursor visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Update visual position based on layout
    pub fn update_visual_position(&mut self, layout_engine: &LayoutEngine) -> Result<()> {
        match layout_engine.logical_to_visual(self.position.row, self.position.column) {
            Ok((x, y)) => {
                self.visual_position = Point {
                    x: Pixels(x),
                    y: Pixels(y),
                };
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Render cursor
    pub fn render(
        &mut self,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
        layout_engine: &LayoutEngine,
    ) {
        if !self.visible {
            return;
        }

        // Update visual position
        if let Err(_) = self.update_visual_position(layout_engine) {
            return; // Can't render without valid position
        }

        // Ensure cursor is within bounds
        if self.visual_position.x < bounds.left()
            || self.visual_position.x > bounds.right()
            || self.visual_position.y < bounds.top()
            || self.visual_position.y > bounds.bottom()
        {
            return;
        }

        match self.style {
            CursorStyle::Line => self.render_line_cursor(cx),
            CursorStyle::Block => self.render_block_cursor(cx),
            CursorStyle::Underline => self.render_underline_cursor(cx),
        }
    }

    /// Render line-style cursor
    fn render_line_cursor(&self, cx: &mut WindowContext) {
        let cursor_bounds = Bounds {
            origin: self.visual_position,
            size: Size {
                width: self.width,
                height: self.height,
            },
        };

        cx.paint_quad(Quad {
            bounds: cursor_bounds,
            corner_radii: Corners::all(Pixels(0.0)),
            background: self.color.into(),
            border_widths: Edges::all(Pixels(0.0)),
            border_color: Hsla::transparent_black(),
        });
    }

    /// Render block-style cursor
    fn render_block_cursor(&self, cx: &mut WindowContext) {
        // Block cursor covers the entire character cell
        let cursor_bounds = Bounds {
            origin: self.visual_position,
            size: Size {
                width: self.height, // Character width (for vertical text)
                height: self.height,
            },
        };

        // Draw background
        cx.paint_quad(Quad {
            bounds: cursor_bounds,
            corner_radii: Corners::all(Pixels(1.0)),
            background: self.color.into(),
            border_widths: Edges::all(Pixels(1.0)),
            border_color: self.color,
        });
    }

    /// Render underline-style cursor
    fn render_underline_cursor(&self, cx: &mut WindowContext) {
        let cursor_bounds = Bounds {
            origin: Point {
                x: self.visual_position.x,
                y: self.visual_position.y + self.height - Pixels(2.0),
            },
            size: Size {
                width: self.height, // Character width
                height: Pixels(2.0),
            },
        };

        cx.paint_quad(Quad {
            bounds: cursor_bounds,
            corner_radii: Corners::all(Pixels(0.0)),
            background: self.color.into(),
            border_widths: Edges::all(Pixels(0.0)),
            border_color: Hsla::transparent_black(),
        });
    }

    /// Get cursor bounds for collision detection
    pub fn bounds(&self) -> Bounds<Pixels> {
        Bounds {
            origin: self.visual_position,
            size: Size {
                width: self.width,
                height: self.height,
            },
        }
    }

    /// Check if cursor is at position
    pub fn is_at(&self, position: SpatialPosition) -> bool {
        self.position == position
    }
}

#[cfg(feature = "gpui")]
impl Default for SpatialCursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder cursor when GPUI is disabled
pub struct SpatialCursor {
    position: SpatialPosition,
}

#[cfg(not(feature = "gpui"))]
impl SpatialCursor {
    pub fn new() -> Self {
        Self {
            position: SpatialPosition::origin(),
        }
    }

    pub fn set_position(&mut self, position: SpatialPosition) {
        self.position = position;
    }

    pub fn position(&self) -> SpatialPosition {
        self.position
    }

    pub fn move_up(&mut self) {
        if self.position.row > 0 {
            self.position.row -= 1;
        }
    }

    pub fn move_down(&mut self) {
        self.position.row += 1;
    }

    pub fn move_backward(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        }
    }

    pub fn advance(&mut self) {
        self.position.column += 1;
    }

    pub fn move_to_next_column(&mut self) {
        self.position.column += 1;
    }

    pub fn move_to_prev_column(&mut self) {
        if self.position.column > 0 {
            self.position.column -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.position = SpatialPosition::origin();
    }

    pub fn render(&mut self, _bounds: (), _cx: (), _layout_engine: &LayoutEngine) {}
}
