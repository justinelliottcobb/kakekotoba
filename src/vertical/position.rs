//! 2D positioning system for vertical programming languages

use std::cmp::Ordering;
use serde::{Deserialize, Serialize};

/// Represents a 2D position in vertically-oriented source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position2D {
    /// Vertical position (row/line in traditional terms)
    pub column: usize,
    /// Horizontal position (column in traditional terms, but reversed for vertical)
    pub row: usize,
    /// Byte offset from start of file
    pub byte_offset: usize,
}

impl Position2D {
    /// Create a new 2D position
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

    /// Move position by delta in vertical direction
    pub fn move_vertical(&mut self, delta: isize) {
        if delta >= 0 {
            self.row = self.row.saturating_add(delta as usize);
        } else {
            self.row = self.row.saturating_sub((-delta) as usize);
        }
    }

    /// Move position by delta in horizontal direction
    pub fn move_horizontal(&mut self, delta: isize) {
        if delta >= 0 {
            self.column = self.column.saturating_add(delta as usize);
        } else {
            self.column = self.column.saturating_sub((-delta) as usize);
        }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position2D) -> f64 {
        let dx = (self.column as isize - other.column as isize) as f64;
        let dy = (self.row as isize - other.row as isize) as f64;
        (dx * dx + dy * dy).sqrt()
    }
}

impl PartialOrd for Position2D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position2D {
    /// Compare positions in vertical reading order (top-to-bottom, right-to-left)
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare columns (right-to-left, so higher column comes first)
        match other.column.cmp(&self.column) {
            Ordering::Equal => {
                // Then compare rows (top-to-bottom)
                self.row.cmp(&other.row)
            }
            ordering => ordering,
        }
    }
}

/// Represents a span between two 2D positions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span2D {
    /// Start position (inclusive)
    pub start: Position2D,
    /// End position (exclusive)
    pub end: Position2D,
}

impl Span2D {
    /// Create a new 2D span
    pub fn new(start: Position2D, end: Position2D) -> Self {
        Self { start, end }
    }

    /// Create a single-character span at the given position
    pub fn at_position(pos: Position2D) -> Self {
        let mut end = pos;
        end.byte_offset += 1;
        Self::new(pos, end)
    }

    /// Check if this span contains a position
    pub fn contains(&self, pos: Position2D) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Check if this span overlaps with another span
    pub fn overlaps(&self, other: &Span2D) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Get the length of this span in bytes
    pub fn byte_length(&self) -> usize {
        self.end.byte_offset.saturating_sub(self.start.byte_offset)
    }

    /// Get the width of this span (horizontal extent)
    pub fn width(&self) -> usize {
        if self.start.row == self.end.row {
            self.end.column.saturating_sub(self.start.column)
        } else {
            // Multi-row span, calculate maximum width
            // For now, return 0 as placeholder
            0
        }
    }

    /// Get the height of this span (vertical extent)
    pub fn height(&self) -> usize {
        self.end.row.saturating_sub(self.start.row) + 1
    }
}

/// Utility for converting between linear and 2D positions
pub struct PositionMapper {
    /// Line breaks in the source text (byte positions)
    line_breaks: Vec<usize>,
    /// Writing direction for position calculations
    writing_direction: super::WritingDirection,
}

impl PositionMapper {
    /// Create a new position mapper for the given text
    pub fn new(text: &str, writing_direction: super::WritingDirection) -> Self {
        let mut line_breaks = vec![0]; // Start of file is position 0
        
        for (byte_pos, _) in text.match_indices('\n') {
            line_breaks.push(byte_pos + 1); // Position after the newline
        }
        
        Self {
            line_breaks,
            writing_direction,
        }
    }

    /// Convert byte offset to 2D position
    pub fn byte_to_2d(&self, byte_offset: usize) -> Position2D {
        // Find which line this byte offset is on
        let row = self.line_breaks
            .binary_search(&byte_offset)
            .unwrap_or_else(|i| i.saturating_sub(1));
        
        let line_start = self.line_breaks.get(row).copied().unwrap_or(0);
        let column = byte_offset.saturating_sub(line_start);
        
        Position2D::new(column, row, byte_offset)
    }

    /// Convert 2D position to byte offset (best effort)
    pub fn to_byte_offset(&self, pos: Position2D) -> usize {
        let line_start = self.line_breaks.get(pos.row).copied().unwrap_or(0);
        line_start + pos.column
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_2d_creation() {
        let pos = Position2D::new(5, 10, 100);
        assert_eq!(pos.column, 5);
        assert_eq!(pos.row, 10);
        assert_eq!(pos.byte_offset, 100);
    }

    #[test]
    fn test_position_ordering() {
        let pos1 = Position2D::new(0, 0, 0);
        let pos2 = Position2D::new(1, 0, 5);
        let pos3 = Position2D::new(0, 1, 10);
        
        // In vertical reading order: higher column comes first
        assert!(pos2 < pos1);
        assert!(pos1 < pos3);
    }

    #[test]
    fn test_span_2d() {
        let start = Position2D::new(0, 0, 0);
        let end = Position2D::new(5, 0, 5);
        let span = Span2D::new(start, end);
        
        assert_eq!(span.byte_length(), 5);
        assert_eq!(span.width(), 5);
        assert_eq!(span.height(), 1);
        
        let pos = Position2D::new(2, 0, 2);
        assert!(span.contains(pos));
    }

    #[test]
    fn test_position_mapper() {
        let text = "line1\nline2\nline3";
        let mapper = PositionMapper::new(text, super::WritingDirection::VerticalTbRl);
        
        let pos = mapper.byte_to_2d(7); // 'i' in "line2"
        assert_eq!(pos.row, 1);
        assert_eq!(pos.column, 1);
        
        let byte_offset = mapper.to_byte_offset(Position2D::new(1, 1, 0));
        assert_eq!(byte_offset, 7);
    }
}