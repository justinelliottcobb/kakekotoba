//! Text selection management for GPUI interface

use crate::spatial::SpatialPosition;
use crate::text_engine::LayoutEngine;
use crate::{Result, TategakiError};
#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
/// Text selection handler for spatial text
pub struct SelectionHandler {
    /// Selection start position
    start: Option<SpatialPosition>,
    /// Selection end position
    end: Option<SpatialPosition>,
    /// Selection style
    style: SelectionStyle,
    /// Selection color
    color: Hsla,
    /// Whether selection is active
    active: bool,
}

#[cfg(feature = "gpui")]
/// Selection rendering style
#[derive(Debug, Clone, Copy)]
pub enum SelectionStyle {
    /// Highlight background
    Highlight,
    /// Outline selection
    Outline,
    /// Underline selection
    Underline,
}

#[cfg(feature = "gpui")]
/// Selection region for rendering
#[derive(Debug, Clone)]
pub struct SelectionRegion {
    /// Bounds of the selection
    pub bounds: Bounds<Pixels>,
    /// Whether this is a partial selection (within character)
    pub partial: bool,
}

#[cfg(feature = "gpui")]
impl SelectionHandler {
    /// Create new selection handler
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
            style: SelectionStyle::Highlight,
            color: gpui::rgba(0.0, 0.5, 1.0, 0.3), // Semi-transparent blue
            active: false,
        }
    }

    /// Start new selection at position
    pub fn start_selection(&mut self, position: SpatialPosition) {
        self.start = Some(position);
        self.end = Some(position);
        self.active = true;
    }

    /// Extend selection to position
    pub fn extend_to_position(&mut self, position: SpatialPosition) {
        if self.start.is_some() {
            self.end = Some(position);
            self.active = true;
        } else {
            self.start_selection(position);
        }
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.active = false;
    }

    /// Check if selection is active
    pub fn is_active(&self) -> bool {
        self.active && self.start.is_some() && self.end.is_some()
    }

    /// Get selection range
    pub fn range(&self) -> Option<(SpatialPosition, SpatialPosition)> {
        if let (Some(start), Some(end)) = (self.start, self.end) {
            // Normalize range (start should be before end)
            if start <= end {
                Some((start, end))
            } else {
                Some((end, start))
            }
        } else {
            None
        }
    }

    /// Get selected text length (approximate)
    pub fn length(&self) -> usize {
        if let Some((start, end)) = self.range() {
            // Simple calculation - would need proper text measurement
            let row_diff = end.row.saturating_sub(start.row);
            let col_diff = if row_diff == 0 {
                end.column.saturating_sub(start.column)
            } else {
                // Multi-row selection approximation
                end.column + 80 * row_diff // Assume 80 chars per row
            };
            col_diff
        } else {
            0
        }
    }

    /// Set selection style
    pub fn set_style(&mut self, style: SelectionStyle) {
        self.style = style;
    }

    /// Set selection color
    pub fn set_color(&mut self, color: Hsla) {
        self.color = color;
    }

    /// Check if position is within selection
    pub fn contains(&self, position: SpatialPosition) -> bool {
        if let Some((start, end)) = self.range() {
            position >= start && position <= end
        } else {
            false
        }
    }

    /// Calculate selection regions for rendering
    pub fn calculate_regions(
        &self,
        layout_engine: &LayoutEngine,
        bounds: Bounds<Pixels>,
    ) -> Result<Vec<SelectionRegion>> {
        let mut regions = Vec::new();

        if let Some((start, end)) = self.range() {
            if start.row == end.row {
                // Single row selection
                if let (Ok((start_x, start_y)), Ok((end_x, end_y))) = (
                    layout_engine.logical_to_visual(start.row, start.column),
                    layout_engine.logical_to_visual(end.row, end.column),
                ) {
                    let region_bounds = Bounds {
                        origin: Point {
                            x: Pixels(start_x.min(end_x)),
                            y: Pixels(start_y.min(end_y)),
                        },
                        size: Size {
                            width: Pixels((end_x - start_x).abs()),
                            height: Pixels(20.0), // Character height
                        },
                    };

                    if region_bounds.intersects(&bounds) {
                        regions.push(SelectionRegion {
                            bounds: region_bounds,
                            partial: false,
                        });
                    }
                }
            } else {
                // Multi-row selection
                for row in start.row..=end.row {
                    let start_col = if row == start.row { start.column } else { 0 };
                    let end_col = if row == end.row { end.column } else { 100 }; // Max columns

                    if let (Ok((start_x, start_y)), Ok((end_x, end_y))) = (
                        layout_engine.logical_to_visual(row, start_col),
                        layout_engine.logical_to_visual(row, end_col),
                    ) {
                        let region_bounds = Bounds {
                            origin: Point {
                                x: Pixels(start_x.min(end_x)),
                                y: Pixels(start_y.min(end_y)),
                            },
                            size: Size {
                                width: Pixels((end_x - start_x).abs()),
                                height: Pixels(20.0),
                            },
                        };

                        if region_bounds.intersects(&bounds) {
                            regions.push(SelectionRegion {
                                bounds: region_bounds,
                                partial: row == start.row || row == end.row,
                            });
                        }
                    }
                }
            }
        }

        Ok(regions)
    }

    /// Render selection
    pub fn render(
        &mut self,
        bounds: Bounds<Pixels>,
        cx: &mut WindowContext,
        layout_engine: &LayoutEngine,
    ) {
        if !self.is_active() {
            return;
        }

        match self.calculate_regions(layout_engine, bounds) {
            Ok(regions) => {
                for region in regions {
                    match self.style {
                        SelectionStyle::Highlight => {
                            self.render_highlight_region(&region, cx);
                        }
                        SelectionStyle::Outline => {
                            self.render_outline_region(&region, cx);
                        }
                        SelectionStyle::Underline => {
                            self.render_underline_region(&region, cx);
                        }
                    }
                }
            }
            Err(_) => {
                // Fallback rendering or error handling
            }
        }
    }

    /// Render highlight-style selection
    fn render_highlight_region(&self, region: &SelectionRegion, cx: &mut WindowContext) {
        cx.paint_quad(Quad {
            bounds: region.bounds,
            corner_radii: Corners::all(Pixels(2.0)),
            background: self.color.into(),
            border_widths: Edges::all(Pixels(0.0)),
            border_color: Hsla::transparent_black(),
        });
    }

    /// Render outline-style selection
    fn render_outline_region(&self, region: &SelectionRegion, cx: &mut WindowContext) {
        cx.paint_quad(Quad {
            bounds: region.bounds,
            corner_radii: Corners::all(Pixels(1.0)),
            background: Hsla::transparent_black().into(),
            border_widths: Edges::all(Pixels(1.0)),
            border_color: self.color,
        });
    }

    /// Render underline-style selection
    fn render_underline_region(&self, region: &SelectionRegion, cx: &mut WindowContext) {
        let underline_bounds = Bounds {
            origin: Point {
                x: region.bounds.left(),
                y: region.bounds.bottom() - Pixels(2.0),
            },
            size: Size {
                width: region.bounds.size.width,
                height: Pixels(2.0),
            },
        };

        cx.paint_quad(Quad {
            bounds: underline_bounds,
            corner_radii: Corners::all(Pixels(0.0)),
            background: self.color.into(),
            border_widths: Edges::all(Pixels(0.0)),
            border_color: Hsla::transparent_black(),
        });
    }

    /// Handle mouse selection drag
    pub fn handle_drag(&mut self, from: SpatialPosition, to: SpatialPosition) {
        self.start = Some(from);
        self.end = Some(to);
        self.active = true;
    }

    /// Copy selected text (placeholder)
    pub fn copy_selection(&self) -> Option<String> {
        if self.is_active() {
            // TODO: Extract actual text from buffer
            Some(format!(
                "Selected text from {:?} to {:?}",
                self.start, self.end
            ))
        } else {
            None
        }
    }

    /// Get selection statistics
    pub fn stats(&self) -> SelectionStats {
        if let Some((start, end)) = self.range() {
            SelectionStats {
                active: true,
                start,
                end,
                length: self.length(),
                multi_row: start.row != end.row,
            }
        } else {
            SelectionStats {
                active: false,
                start: SpatialPosition::origin(),
                end: SpatialPosition::origin(),
                length: 0,
                multi_row: false,
            }
        }
    }
}

#[cfg(feature = "gpui")]
/// Selection statistics
#[derive(Debug, Clone)]
pub struct SelectionStats {
    /// Whether selection is active
    pub active: bool,
    /// Selection start
    pub start: SpatialPosition,
    /// Selection end
    pub end: SpatialPosition,
    /// Selection length
    pub length: usize,
    /// Whether selection spans multiple rows
    pub multi_row: bool,
}

#[cfg(feature = "gpui")]
impl Default for SelectionHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "gpui"))]
/// Placeholder selection handler when GPUI is disabled
pub struct SelectionHandler {
    start: Option<SpatialPosition>,
    end: Option<SpatialPosition>,
}

#[cfg(not(feature = "gpui"))]
impl SelectionHandler {
    pub fn new() -> Self {
        Self {
            start: None,
            end: None,
        }
    }

    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }

    pub fn extend_to_position(&mut self, position: SpatialPosition) {
        self.end = Some(position);
    }

    pub fn render(&mut self, _bounds: (), _cx: (), _layout_engine: &LayoutEngine) {}
}
