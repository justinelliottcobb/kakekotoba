//! GPUI backend wrapper
//!
//! This module wraps the existing GPUI rendering interface to conform to
//! the RenderBackend trait, allowing seamless backend switching.

#[cfg(feature = "gpui")]
use gpui::*;
use crate::{Result, TategakiError};
use crate::text_engine::TextDirection;
use super::{RenderBackend, Color, Rect, TextStyle, CursorInfo, CursorStyle};

/// GPUI backend wrapper
///
/// Note: This is a simplified wrapper. In practice, GPUI rendering happens
/// through the WindowContext in the render() method of views. This wrapper
/// provides a bridge interface but actual rendering would need to be coordinated
/// with GPUI's view system.
pub struct GpuiBackend {
    /// Viewport size in logical pixels
    viewport: (u32, u32),
    /// Whether the backend is active
    active: bool,
    /// Pending render commands (stored until present)
    #[cfg(feature = "gpui")]
    render_commands: Vec<RenderCommand>,
}

#[cfg(feature = "gpui")]
enum RenderCommand {
    Clear(Color),
    Text {
        text: String,
        position: (f32, f32),
        style: TextStyle,
        direction: TextDirection,
    },
    Cursor(CursorInfo),
    Selection { bounds: Rect, color: Color },
    Line { from: (f32, f32), to: (f32, f32), color: Color, thickness: f32 },
    Rect { bounds: Rect, color: Color, filled: bool },
}

impl GpuiBackend {
    /// Create a new GPUI backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            viewport: (800, 600),
            active: false,
            #[cfg(feature = "gpui")]
            render_commands: Vec::new(),
        })
    }

    #[cfg(feature = "gpui")]
    /// Convert our Color to GPUI's color type
    fn to_gpui_color(color: Color) -> Rgba {
        Rgba {
            r: color.r as f32 / 255.0,
            g: color.g as f32 / 255.0,
            b: color.b as f32 / 255.0,
            a: color.a as f32 / 255.0,
        }
    }

    #[cfg(feature = "gpui")]
    /// Convert our Rect to GPUI's Bounds
    fn to_gpui_bounds(rect: Rect) -> Bounds<Pixels> {
        Bounds {
            origin: point(px(rect.x), px(rect.y)),
            size: size(px(rect.width), px(rect.height)),
        }
    }

    #[cfg(feature = "gpui")]
    /// Execute stored render commands in a GPUI context
    ///
    /// This would be called from within a GPUI view's render method
    pub fn execute_commands(&mut self, cx: &mut WindowContext) {
        for command in self.render_commands.drain(..) {
            match command {
                RenderCommand::Clear(color) => {
                    // GPUI clearing is typically done by setting background
                    // This would need to be integrated with the view system
                }
                RenderCommand::Text { text, position, style, direction } => {
                    // GPUI text rendering would use cosmic-text or similar
                    // This is a placeholder - actual implementation would need
                    // to create text runs and render them
                }
                RenderCommand::Cursor(cursor_info) => {
                    // Render cursor as a small rectangle or line
                }
                RenderCommand::Selection { bounds, color } => {
                    // Render selection highlight
                }
                RenderCommand::Line { from, to, color, thickness } => {
                    // GPUI line rendering
                }
                RenderCommand::Rect { bounds, color, filled } => {
                    // GPUI rectangle rendering
                }
            }
        }
    }
}

impl RenderBackend for GpuiBackend {
    fn init(&mut self) -> Result<()> {
        // GPUI initialization is handled by the application setup
        // This is more of a state marker
        self.active = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.active = false;
        #[cfg(feature = "gpui")]
        {
            self.render_commands.clear();
        }
        Ok(())
    }

    fn viewport_size(&self) -> (u32, u32) {
        self.viewport
    }

    fn clear(&mut self, color: Color) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Clear(color));
        }
        Ok(())
    }

    fn render_text(
        &mut self,
        text: &str,
        position: (f32, f32),
        style: &TextStyle,
        direction: TextDirection,
    ) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Text {
                text: text.to_string(),
                position,
                style: style.clone(),
                direction,
            });
        }
        Ok(())
    }

    fn render_cursor(&mut self, cursor: &CursorInfo) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Cursor(cursor.clone()));
        }
        Ok(())
    }

    fn render_selection(&mut self, bounds: Rect, color: Color) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Selection { bounds, color });
        }
        Ok(())
    }

    fn render_line(
        &mut self,
        from: (f32, f32),
        to: (f32, f32),
        color: Color,
        thickness: f32,
    ) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Line {
                from,
                to,
                color,
                thickness,
            });
        }
        Ok(())
    }

    fn render_rect(&mut self, bounds: Rect, color: Color, filled: bool) -> Result<()> {
        #[cfg(feature = "gpui")]
        {
            self.render_commands.push(RenderCommand::Rect {
                bounds,
                color,
                filled,
            });
        }
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        // In GPUI, presenting is handled by the framework
        // Commands are stored and executed in the render cycle
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

impl Default for GpuiBackend {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpui_backend_creation() {
        let backend = GpuiBackend::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_gpui_backend_init() {
        let mut backend = GpuiBackend::new().unwrap();
        assert!(backend.init().is_ok());
        assert!(backend.is_active());
    }

    #[test]
    #[cfg(feature = "gpui")]
    fn test_color_conversion() {
        let color = Color::new(255, 128, 64, 255);
        let gpui_color = GpuiBackend::to_gpui_color(color);

        assert!((gpui_color.r - 1.0).abs() < 0.01);
        assert!((gpui_color.g - 0.5).abs() < 0.01);
        assert!((gpui_color.b - 0.25).abs() < 0.02);
    }
}
