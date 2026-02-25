//! Scrolling and viewport management for GPUI interface

use crate::spatial::SpatialPosition;
use crate::text_engine::{LayoutEngine, TextDirection};
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
/// Scroll manager for vertical text viewport
pub struct ScrollManager {
    /// Current scroll offset (logical coordinates)
    offset: SpatialPosition,
    /// Viewport size in pixels
    viewport_size: Size<Pixels>,
    /// Content size in logical units
    content_size: SpatialPosition,
    /// Scroll speed multiplier
    scroll_speed: f32,
    /// Smooth scrolling state
    smooth_scroll: SmoothScrollState,
    /// Text direction for scroll behavior
    text_direction: TextDirection,
}

#[cfg(feature = "gpui")]
/// Smooth scrolling animation state
#[derive(Debug, Clone)]
pub struct SmoothScrollState {
    /// Target scroll position
    target: SpatialPosition,
    /// Current animated position
    current: SpatialPosition,
    /// Animation speed (0.0 to 1.0)
    speed: f32,
    /// Whether smooth scrolling is enabled
    enabled: bool,
}

#[cfg(feature = "gpui")]
/// Scroll direction for user input
#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
}

#[cfg(feature = "gpui")]
impl ScrollManager {
    /// Create new scroll manager
    pub fn new() -> Self {
        Self {
            offset: SpatialPosition::origin(),
            viewport_size: Size::default(),
            content_size: SpatialPosition {
                row: 100,
                column: 80,
            }, // Default size
            scroll_speed: 1.0,
            smooth_scroll: SmoothScrollState {
                target: SpatialPosition::origin(),
                current: SpatialPosition::origin(),
                speed: 0.15,
                enabled: true,
            },
            text_direction: TextDirection::VerticalTopToBottom,
        }
    }

    /// Set text direction
    pub fn set_text_direction(&mut self, direction: TextDirection) {
        self.text_direction = direction;
    }

    /// Update viewport size
    pub fn set_viewport_size(&mut self, size: Size<Pixels>) {
        self.viewport_size = size;
    }

    /// Update content size
    pub fn set_content_size(&mut self, size: SpatialPosition) {
        self.content_size = size;
        self.clamp_scroll_offset();
    }

    /// Set scroll speed
    pub fn set_scroll_speed(&mut self, speed: f32) {
        self.scroll_speed = speed.max(0.1);
    }

    /// Enable/disable smooth scrolling
    pub fn set_smooth_scrolling(&mut self, enabled: bool) {
        self.smooth_scroll.enabled = enabled;
    }

    /// Get current scroll offset
    pub fn offset(&self) -> SpatialPosition {
        if self.smooth_scroll.enabled {
            self.smooth_scroll.current
        } else {
            self.offset
        }
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, delta: SpatialPosition) {
        let new_offset = SpatialPosition {
            row: (self.offset.row as i32 + delta.row as i32).max(0) as usize,
            column: (self.offset.column as i32 + delta.column as i32).max(0) as usize,
        };

        self.set_scroll_offset(new_offset);
    }

    /// Set scroll offset directly
    pub fn set_scroll_offset(&mut self, offset: SpatialPosition) {
        let clamped_offset = self.clamp_offset(offset);

        if self.smooth_scroll.enabled {
            self.smooth_scroll.target = clamped_offset;
        } else {
            self.offset = clamped_offset;
            self.smooth_scroll.current = clamped_offset;
            self.smooth_scroll.target = clamped_offset;
        }
    }

    /// Handle scroll direction input
    pub fn handle_scroll_direction(&mut self, direction: ScrollDirection) -> Result<()> {
        let delta = match (direction, self.text_direction) {
            (ScrollDirection::Up, TextDirection::VerticalTopToBottom) => {
                SpatialPosition { row: 1, column: 0 } // Move up in current column
            }
            (ScrollDirection::Down, TextDirection::VerticalTopToBottom) => SpatialPosition {
                row: (1.0 * self.scroll_speed) as usize,
                column: 0,
            },
            (ScrollDirection::Left, TextDirection::VerticalTopToBottom) => {
                SpatialPosition { row: 0, column: 1 } // Next column (left in vertical)
            }
            (ScrollDirection::Right, TextDirection::VerticalTopToBottom) => SpatialPosition {
                row: 0,
                column: (1.0 * self.scroll_speed) as usize,
            },
            (ScrollDirection::PageUp, _) => {
                let page_size = self.calculate_page_size();
                SpatialPosition {
                    row: page_size.row,
                    column: 0,
                }
            }
            (ScrollDirection::PageDown, _) => {
                let page_size = self.calculate_page_size();
                SpatialPosition {
                    row: (page_size.row as i32 * -1).max(-(self.offset.row as i32)) as usize,
                    column: 0,
                }
            }
            (ScrollDirection::Home, _) => {
                return Ok(self.scroll_to_home());
            }
            (ScrollDirection::End, _) => {
                return Ok(self.scroll_to_end());
            }
            // Horizontal text directions
            (ScrollDirection::Up, TextDirection::HorizontalLeftToRight) => {
                SpatialPosition { row: 1, column: 0 }
            }
            (ScrollDirection::Down, TextDirection::HorizontalLeftToRight) => SpatialPosition {
                row: (1.0 * self.scroll_speed) as usize,
                column: 0,
            },
            (ScrollDirection::Left, TextDirection::HorizontalLeftToRight) => {
                SpatialPosition { row: 0, column: 1 }
            }
            (ScrollDirection::Right, TextDirection::HorizontalLeftToRight) => SpatialPosition {
                row: 0,
                column: (1.0 * self.scroll_speed) as usize,
            },
        };

        self.scroll_by(delta);
        Ok(())
    }

    /// Handle mouse wheel scrolling
    pub fn handle_wheel_scroll(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        modifiers: &Modifiers,
    ) -> Result<()> {
        let speed_multiplier = if modifiers.shift { 3.0 } else { 1.0 };

        let delta = match self.text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, wheel up/down scrolls within column
                // wheel left/right or shift+wheel moves between columns
                SpatialPosition {
                    row: (delta_y * speed_multiplier * self.scroll_speed) as usize,
                    column: (delta_x * speed_multiplier * self.scroll_speed) as usize,
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // Standard horizontal scrolling
                SpatialPosition {
                    row: (delta_y * speed_multiplier * self.scroll_speed) as usize,
                    column: (delta_x * speed_multiplier * self.scroll_speed) as usize,
                }
            }
        };

        self.scroll_by(delta);
        Ok(())
    }

    /// Scroll to make position visible
    pub fn ensure_visible(
        &mut self,
        position: SpatialPosition,
        layout_engine: &LayoutEngine,
    ) -> Result<()> {
        let current_offset = self.offset();
        let mut new_offset = current_offset;

        // Calculate viewport bounds in logical coordinates
        let viewport_bounds = self.calculate_viewport_bounds();

        // Check if position is outside viewport
        if position.row < current_offset.row {
            // Scroll up to show position
            new_offset.row = position.row;
        } else if position.row >= current_offset.row + viewport_bounds.row {
            // Scroll down to show position
            new_offset.row = position.row.saturating_sub(viewport_bounds.row - 1);
        }

        if position.column < current_offset.column {
            // Scroll left to show position
            new_offset.column = position.column;
        } else if position.column >= current_offset.column + viewport_bounds.column {
            // Scroll right to show position
            new_offset.column = position.column.saturating_sub(viewport_bounds.column - 1);
        }

        if new_offset != current_offset {
            self.set_scroll_offset(new_offset);
        }

        Ok(())
    }

    /// Update smooth scrolling animation
    pub fn update_smooth_scroll(&mut self) {
        if !self.smooth_scroll.enabled {
            return;
        }

        let current = &mut self.smooth_scroll.current;
        let target = self.smooth_scroll.target;
        let speed = self.smooth_scroll.speed;

        // Interpolate towards target
        if current.row != target.row {
            let diff = (target.row as i32) - (current.row as i32);
            let step = ((diff as f32) * speed).round() as i32;
            if step.abs() >= 1 {
                current.row = ((current.row as i32) + step).max(0) as usize;
            } else {
                current.row = target.row;
            }
        }

        if current.column != target.column {
            let diff = (target.column as i32) - (current.column as i32);
            let step = ((diff as f32) * speed).round() as i32;
            if step.abs() >= 1 {
                current.column = ((current.column as i32) + step).max(0) as usize;
            } else {
                current.column = target.column;
            }
        }

        // Update actual offset for non-smooth operations
        self.offset = current.clone();
    }

    /// Calculate page size for page up/down
    fn calculate_page_size(&self) -> SpatialPosition {
        // Estimate based on viewport size
        SpatialPosition {
            row: 20,   // Approximate lines per page
            column: 5, // Approximate columns per page
        }
    }

    /// Calculate viewport bounds in logical coordinates
    fn calculate_viewport_bounds(&self) -> SpatialPosition {
        // Convert pixel viewport to logical coordinates
        // This is an approximation - real implementation would use layout engine
        SpatialPosition {
            row: (self.viewport_size.height.0 / 20.0) as usize, // Assume 20px per line
            column: (self.viewport_size.width.0 / 20.0) as usize, // Assume 20px per char
        }
    }

    /// Clamp scroll offset to valid bounds
    fn clamp_offset(&self, offset: SpatialPosition) -> SpatialPosition {
        let viewport_bounds = self.calculate_viewport_bounds();

        SpatialPosition {
            row: offset
                .row
                .min(self.content_size.row.saturating_sub(viewport_bounds.row)),
            column: offset.column.min(
                self.content_size
                    .column
                    .saturating_sub(viewport_bounds.column),
            ),
        }
    }

    /// Clamp current scroll offset
    fn clamp_scroll_offset(&mut self) {
        let clamped = self.clamp_offset(self.offset);
        self.offset = clamped;
        self.smooth_scroll.current = clamped;
        self.smooth_scroll.target = clamped;
    }

    /// Scroll to beginning of document
    fn scroll_to_home(&mut self) {
        self.set_scroll_offset(SpatialPosition::origin());
    }

    /// Scroll to end of document
    fn scroll_to_end(&mut self) {
        let viewport_bounds = self.calculate_viewport_bounds();
        let end_offset = SpatialPosition {
            row: self.content_size.row.saturating_sub(viewport_bounds.row),
            column: self
                .content_size
                .column
                .saturating_sub(viewport_bounds.column),
        };
        self.set_scroll_offset(end_offset);
    }

    /// Check if scrolling is needed for given content size
    pub fn needs_scrollbar(&self) -> (bool, bool) {
        let viewport_bounds = self.calculate_viewport_bounds();
        (
            self.content_size.row > viewport_bounds.row, // Vertical scrollbar
            self.content_size.column > viewport_bounds.column, // Horizontal scrollbar
        )
    }

    /// Get scroll percentage (0.0 to 1.0)
    pub fn scroll_percentage(&self) -> (f32, f32) {
        let viewport_bounds = self.calculate_viewport_bounds();
        let max_row_offset = self.content_size.row.saturating_sub(viewport_bounds.row);
        let max_col_offset = self
            .content_size
            .column
            .saturating_sub(viewport_bounds.column);

        let row_pct = if max_row_offset > 0 {
            (self.offset.row as f32) / (max_row_offset as f32)
        } else {
            0.0
        };

        let col_pct = if max_col_offset > 0 {
            (self.offset.column as f32) / (max_col_offset as f32)
        } else {
            0.0
        };

        (row_pct.clamp(0.0, 1.0), col_pct.clamp(0.0, 1.0))
    }
}

#[cfg(feature = "gpui")]
impl Default for ScrollManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder scroll manager when GPUI is disabled
pub struct ScrollManager {
    offset: SpatialPosition,
}

#[cfg(not(feature = "gpui"))]
impl ScrollManager {
    pub fn new() -> Self {
        Self {
            offset: SpatialPosition::origin(),
        }
    }
}
