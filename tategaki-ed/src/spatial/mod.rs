//! Spatial positioning system for vertical text editing
//!
//! This module provides the coordinate system and spatial operations needed
//! for vertical text editing with semantic spatial relationships.

use crate::{Result, TategakiError};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub mod coordinates;
pub mod navigation;
pub mod selection;

pub use coordinates::*;
pub use navigation::*;
pub use selection::*;

/// 2D spatial position in vertical text context
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpatialPosition {
    /// Column position (in vertical text, this is the vertical column)
    pub column: usize,
    /// Row position (in vertical text, this is the horizontal position within a column)
    pub row: usize,
    /// Byte offset from start of document
    pub byte_offset: usize,
}

impl SpatialPosition {
    /// Create a new spatial position
    pub fn new(column: usize, row: usize, byte_offset: usize) -> Self {
        Self {
            column,
            row,
            byte_offset,
        }
    }

    /// Create position at origin
    pub fn origin() -> Self {
        Self::new(0, 0, 0)
    }

    /// Move position vertically (within column)
    pub fn move_vertical(&mut self, delta: isize) {
        if delta >= 0 {
            self.row = self.row.saturating_add(delta as usize);
        } else {
            self.row = self.row.saturating_sub((-delta) as usize);
        }
    }

    /// Move position horizontally (between columns)
    pub fn move_horizontal(&mut self, delta: isize) {
        if delta >= 0 {
            self.column = self.column.saturating_add(delta as usize);
        } else {
            self.column = self.column.saturating_sub((-delta) as usize);
        }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &SpatialPosition) -> f64 {
        let dx = (self.column as isize - other.column as isize) as f64;
        let dy = (self.row as isize - other.row as isize) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if this position is before another in reading order
    pub fn is_before(&self, other: &SpatialPosition, direction: crate::text_engine::TextDirection) -> bool {
        match direction {
            crate::text_engine::TextDirection::VerticalTopToBottom => {
                // In vertical Japanese: right-to-left columns, top-to-bottom within columns
                if self.column != other.column {
                    self.column > other.column // Higher column number = further right = comes first
                } else {
                    self.row < other.row // Lower row = higher up = comes first
                }
            }
            crate::text_engine::TextDirection::HorizontalLeftToRight => {
                // Standard left-to-right: top-to-bottom rows, left-to-right within rows
                if self.row != other.row {
                    self.row < other.row
                } else {
                    self.column < other.column
                }
            }
            _ => self < other, // Fallback to standard comparison
        }
    }

    /// Get the next position in reading order
    pub fn next_reading_position(&self, direction: crate::text_engine::TextDirection) -> SpatialPosition {
        match direction {
            crate::text_engine::TextDirection::VerticalTopToBottom => {
                // Move down within column, then to next column (left)
                SpatialPosition::new(self.column, self.row + 1, self.byte_offset + 1)
            }
            crate::text_engine::TextDirection::HorizontalLeftToRight => {
                // Move right within row, then to next row
                SpatialPosition::new(self.column + 1, self.row, self.byte_offset + 1)
            }
            _ => SpatialPosition::new(self.column, self.row, self.byte_offset + 1),
        }
    }

    /// Get the previous position in reading order
    pub fn prev_reading_position(&self, direction: crate::text_engine::TextDirection) -> SpatialPosition {
        match direction {
            crate::text_engine::TextDirection::VerticalTopToBottom => {
                // Move up within column, or to previous column (right) at bottom
                if self.row > 0 {
                    SpatialPosition::new(self.column, self.row - 1, self.byte_offset.saturating_sub(1))
                } else {
                    SpatialPosition::new(self.column + 1, 0, self.byte_offset.saturating_sub(1))
                }
            }
            crate::text_engine::TextDirection::HorizontalLeftToRight => {
                // Move left within row, or to previous row at end
                if self.column > 0 {
                    SpatialPosition::new(self.column - 1, self.row, self.byte_offset.saturating_sub(1))
                } else {
                    SpatialPosition::new(0, self.row.saturating_sub(1), self.byte_offset.saturating_sub(1))
                }
            }
            _ => SpatialPosition::new(
                self.column,
                self.row,
                self.byte_offset.saturating_sub(1)
            ),
        }
    }
}

impl PartialOrd for SpatialPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpatialPosition {
    /// Default ordering: by byte offset
    fn cmp(&self, other: &Self) -> Ordering {
        self.byte_offset.cmp(&other.byte_offset)
    }
}

/// Coordinate system for different text layouts
#[derive(Debug, Clone)]
pub struct CoordinateSystem {
    /// Primary text direction
    pub direction: crate::text_engine::TextDirection,
    /// Origin point for coordinates
    pub origin: SpatialPosition,
    /// Scale factors for different axes
    pub scale: CoordinateScale,
}

/// Scale factors for coordinate calculations
#[derive(Debug, Clone)]
pub struct CoordinateScale {
    /// Horizontal scale
    pub x_scale: f32,
    /// Vertical scale
    pub y_scale: f32,
    /// Character width
    pub char_width: f32,
    /// Line height
    pub line_height: f32,
}

impl Default for CoordinateScale {
    fn default() -> Self {
        Self {
            x_scale: 1.0,
            y_scale: 1.0,
            char_width: 8.0,
            line_height: 16.0,
        }
    }
}

impl CoordinateSystem {
    /// Create a new coordinate system
    pub fn new(direction: crate::text_engine::TextDirection) -> Self {
        Self {
            direction,
            origin: SpatialPosition::origin(),
            scale: CoordinateScale::default(),
        }
    }

    /// Convert spatial position to screen coordinates
    pub fn spatial_to_screen(&self, pos: &SpatialPosition) -> (f32, f32) {
        match self.direction {
            crate::text_engine::TextDirection::VerticalTopToBottom => {
                // Vertical text: columns go right-to-left, rows top-to-bottom
                let x = (pos.column as f32) * self.scale.char_width * -1.0; // Negative for RTL
                let y = (pos.row as f32) * self.scale.line_height;
                (x, y)
            }
            crate::text_engine::TextDirection::HorizontalLeftToRight => {
                // Horizontal text: standard mapping
                let x = (pos.column as f32) * self.scale.char_width;
                let y = (pos.row as f32) * self.scale.line_height;
                (x, y)
            }
            _ => ((pos.column as f32) * self.scale.char_width, (pos.row as f32) * self.scale.line_height),
        }
    }

    /// Convert screen coordinates to spatial position
    pub fn screen_to_spatial(&self, x: f32, y: f32) -> SpatialPosition {
        match self.direction {
            crate::text_engine::TextDirection::VerticalTopToBottom => {
                // Reverse the vertical mapping
                let column = (x / (self.scale.char_width * -1.0)) as usize;
                let row = (y / self.scale.line_height) as usize;
                SpatialPosition::new(column, row, 0) // Byte offset would need buffer lookup
            }
            crate::text_engine::TextDirection::HorizontalLeftToRight => {
                let column = (x / self.scale.char_width) as usize;
                let row = (y / self.scale.line_height) as usize;
                SpatialPosition::new(column, row, 0)
            }
            _ => {
                let column = (x / self.scale.char_width) as usize;
                let row = (y / self.scale.line_height) as usize;
                SpatialPosition::new(column, row, 0)
            }
        }
    }

    /// Update coordinate scale
    pub fn set_scale(&mut self, scale: CoordinateScale) {
        self.scale = scale;
    }

    /// Set origin position
    pub fn set_origin(&mut self, origin: SpatialPosition) {
        self.origin = origin;
    }
}

/// Spatial range representing a selection or span of text
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialRange {
    /// Start position (inclusive)
    pub start: SpatialPosition,
    /// End position (exclusive)
    pub end: SpatialPosition,
}

impl SpatialRange {
    /// Create a new spatial range
    pub fn new(start: SpatialPosition, end: SpatialPosition) -> Self {
        Self { start, end }
    }

    /// Create a range from a single position (zero-width)
    pub fn at_position(pos: SpatialPosition) -> Self {
        Self::new(pos, pos)
    }

    /// Check if this range contains a position
    pub fn contains(&self, pos: &SpatialPosition) -> bool {
        pos >= &self.start && pos < &self.end
    }

    /// Check if this range overlaps with another range
    pub fn overlaps(&self, other: &SpatialRange) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get the length of this range in characters
    pub fn char_length(&self) -> usize {
        self.end.byte_offset.saturating_sub(self.start.byte_offset)
    }

    /// Check if this is an empty range
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Expand range to include another position
    pub fn expand_to_include(&mut self, pos: SpatialPosition) {
        if pos < self.start {
            self.start = pos;
        } else if pos >= self.end {
            self.end = pos.next_reading_position(crate::text_engine::TextDirection::VerticalTopToBottom);
        }
    }

    /// Get all positions within this range
    pub fn positions(&self, direction: crate::text_engine::TextDirection) -> Vec<SpatialPosition> {
        let mut positions = Vec::new();
        let mut current = self.start;
        
        while current < self.end {
            positions.push(current);
            current = current.next_reading_position(direction);
        }
        
        positions
    }
}

/// Spatial transformation utilities
pub struct SpatialTransform;

impl SpatialTransform {
    /// Transform position from one coordinate system to another
    pub fn transform_position(
        pos: &SpatialPosition,
        from_system: &CoordinateSystem,
        to_system: &CoordinateSystem,
    ) -> SpatialPosition {
        // Convert to screen coordinates in source system
        let (x, y) = from_system.spatial_to_screen(pos);
        
        // Convert from screen coordinates in target system
        to_system.screen_to_spatial(x, y)
    }

    /// Calculate bounding box for a range of positions
    pub fn bounding_box(
        positions: &[SpatialPosition],
        coord_system: &CoordinateSystem,
    ) -> ((f32, f32), (f32, f32)) {
        if positions.is_empty() {
            return ((0.0, 0.0), (0.0, 0.0));
        }

        let screen_coords: Vec<(f32, f32)> = positions
            .iter()
            .map(|pos| coord_system.spatial_to_screen(pos))
            .collect();

        let min_x = screen_coords.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min);
        let max_x = screen_coords.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = screen_coords.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min);
        let max_y = screen_coords.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max);

        ((min_x, min_y), (max_x, max_y))
    }

    /// Snap position to character grid
    pub fn snap_to_grid(
        pos: &SpatialPosition,
        coord_system: &CoordinateSystem,
    ) -> SpatialPosition {
        let (x, y) = coord_system.spatial_to_screen(pos);
        
        // Round to nearest character position
        let snapped_x = (x / coord_system.scale.char_width).round() * coord_system.scale.char_width;
        let snapped_y = (y / coord_system.scale.line_height).round() * coord_system.scale.line_height;
        
        coord_system.screen_to_spatial(snapped_x, snapped_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_position_creation() {
        let pos = SpatialPosition::new(5, 10, 100);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.row, 10);
        assert_eq!(pos.byte_offset, 100);
    }

    #[test]
    fn test_position_movement() {
        let mut pos = SpatialPosition::new(5, 10, 100);
        pos.move_vertical(3);
        assert_eq!(pos.row, 13);
        
        pos.move_horizontal(-2);
        assert_eq!(pos.column, 3);
    }

    #[test]
    fn test_reading_order_vertical() {
        let pos1 = SpatialPosition::new(1, 0, 0);  // Column 1, Row 0
        let pos2 = SpatialPosition::new(0, 0, 0);  // Column 0, Row 0
        
        // In vertical text, higher column (further right) comes first
        assert!(pos1.is_before(&pos2, crate::text_engine::TextDirection::VerticalTopToBottom));
    }

    #[test]
    fn test_spatial_range() {
        let start = SpatialPosition::new(0, 0, 0);
        let end = SpatialPosition::new(5, 5, 25);
        let range = SpatialRange::new(start, end);
        
        assert_eq!(range.char_length(), 25);
        assert!(!range.is_empty());
        
        let test_pos = SpatialPosition::new(2, 2, 12);
        assert!(range.contains(&test_pos));
    }

    #[test]
    fn test_coordinate_system() {
        let coord_system = CoordinateSystem::new(crate::text_engine::TextDirection::VerticalTopToBottom);
        let pos = SpatialPosition::new(1, 2, 0);
        
        let (x, y) = coord_system.spatial_to_screen(&pos);
        let back = coord_system.screen_to_spatial(x, y);
        
        assert_eq!(pos.column, back.column);
        assert_eq!(pos.row, back.row);
    }

    #[test]
    fn test_distance_calculation() {
        let pos1 = SpatialPosition::new(0, 0, 0);
        let pos2 = SpatialPosition::new(3, 4, 0);
        
        let distance = pos1.distance_to(&pos2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }
}