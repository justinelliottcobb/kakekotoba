//! 2D code layout analysis for vertical programming languages
//!
//! This module provides tools for analyzing and understanding the spatial
//! structure of vertically-written code, including indentation analysis,
//! block detection, and layout-aware parsing support.

use crate::vertical::{Position2D, Span2D, SpatialToken, WritingDirection};
use crate::error::Result;
use std::collections::HashMap;

pub mod indentation;
pub mod blocks;
pub mod flow;

pub use indentation::*;
pub use blocks::*;
pub use flow::*;

/// Represents the layout structure of a piece of code
#[derive(Debug, Clone)]
pub struct CodeLayout {
    /// Writing direction for this layout
    pub direction: WritingDirection,
    /// Indentation levels mapped by position
    pub indentation_map: HashMap<Position2D, usize>,
    /// Detected code blocks
    pub blocks: Vec<CodeBlock>,
    /// Text flow analysis
    pub flow: TextFlow,
}

impl CodeLayout {
    /// Create a new empty code layout
    pub fn new(direction: WritingDirection) -> Self {
        Self {
            direction,
            indentation_map: HashMap::new(),
            blocks: Vec::new(),
            flow: TextFlow::new(direction),
        }
    }

    /// Analyze layout from spatial tokens
    pub fn analyze(tokens: &[SpatialToken]) -> Result<Self> {
        let direction = tokens.first()
            .map(|t| t.direction)
            .unwrap_or(WritingDirection::VerticalTbRl);
        
        let mut layout = Self::new(direction);
        
        // Analyze indentation
        let indentation_analyzer = IndentationAnalyzer::new(direction);
        layout.indentation_map = indentation_analyzer.analyze_tokens(tokens)?;
        
        // Detect blocks
        let block_detector = BlockDetector::new(direction);
        layout.blocks = block_detector.detect_blocks(tokens, &layout.indentation_map)?;
        
        // Analyze text flow
        layout.flow = TextFlow::analyze(tokens)?;
        
        Ok(layout)
    }

    /// Get indentation level at a specific position
    pub fn indentation_at(&self, pos: Position2D) -> usize {
        self.indentation_map.get(&pos).copied().unwrap_or(0)
    }

    /// Find the block containing a specific position
    pub fn block_at(&self, pos: Position2D) -> Option<&CodeBlock> {
        self.blocks.iter().find(|block| block.contains(pos))
    }

    /// Get all blocks at a specific indentation level
    pub fn blocks_at_level(&self, level: usize) -> Vec<&CodeBlock> {
        self.blocks.iter()
            .filter(|block| block.indentation_level == level)
            .collect()
    }

    /// Check if a position represents the start of a new block
    pub fn is_block_start(&self, pos: Position2D) -> bool {
        self.blocks.iter().any(|block| block.span.start == pos)
    }
}

/// Utility for measuring 2D distances and relationships
pub struct SpatialMeasurer {
    direction: WritingDirection,
}

impl SpatialMeasurer {
    /// Create a new spatial measurer
    pub fn new(direction: WritingDirection) -> Self {
        Self { direction }
    }

    /// Calculate the reading-order distance between two positions
    pub fn reading_distance(&self, from: Position2D, to: Position2D) -> f64 {
        match self.direction {
            WritingDirection::VerticalTbRl => {
                // In vertical text, we read top-to-bottom, right-to-left
                let vertical_weight = 1.0;
                let horizontal_weight = 2.0; // Columns are more significant
                
                let dx = (from.column as isize - to.column as isize) as f64;
                let dy = (to.row as isize - from.row as isize) as f64;
                
                (dx * dx * horizontal_weight + dy * dy * vertical_weight).sqrt()
            }
            WritingDirection::HorizontalLtr => {
                // Standard left-to-right horizontal text
                let horizontal_weight = 2.0;
                let vertical_weight = 1.0;
                
                let dx = (to.column as isize - from.column as isize) as f64;
                let dy = (to.row as isize - from.row as isize) as f64;
                
                (dx * dx * horizontal_weight + dy * dy * vertical_weight).sqrt()
            }
            _ => {
                // Fallback to Euclidean distance
                from.distance_to(&to)
            }
        }
    }

    /// Check if one position comes before another in reading order
    pub fn comes_before(&self, first: Position2D, second: Position2D) -> bool {
        match self.direction {
            WritingDirection::VerticalTbRl => {
                // Right-to-left, then top-to-bottom
                if first.column != second.column {
                    first.column > second.column
                } else {
                    first.row < second.row
                }
            }
            WritingDirection::HorizontalLtr => {
                // Top-to-bottom, then left-to-right
                if first.row != second.row {
                    first.row < second.row
                } else {
                    first.column < second.column
                }
            }
            _ => first < second, // Use default ordering
        }
    }

    /// Find the next position in reading order
    pub fn next_position(&self, current: Position2D, step_size: usize) -> Position2D {
        let mut next = current;
        
        match self.direction {
            WritingDirection::VerticalTbRl => {
                next.move_vertical(step_size as isize);
            }
            WritingDirection::HorizontalLtr => {
                next.move_horizontal(step_size as isize);
            }
            _ => {
                next.move_horizontal(step_size as isize);
            }
        }
        
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::{SpatialTokenKind, SpatialTokenIterator};

    #[test]
    fn test_code_layout_creation() {
        let layout = CodeLayout::new(WritingDirection::VerticalTbRl);
        assert_eq!(layout.direction, WritingDirection::VerticalTbRl);
        assert!(layout.blocks.is_empty());
        assert!(layout.indentation_map.is_empty());
    }

    #[test]
    fn test_spatial_measurer() {
        let measurer = SpatialMeasurer::new(WritingDirection::VerticalTbRl);
        
        let pos1 = Position2D::new(0, 0, 0);
        let pos2 = Position2D::new(0, 1, 5);
        
        assert!(measurer.comes_before(pos1, pos2));
        
        let distance = measurer.reading_distance(pos1, pos2);
        assert!(distance > 0.0);
    }

    #[test]
    fn test_layout_analysis() {
        // Create simple tokens for testing
        let text = "関数\n  本体";
        let iter = SpatialTokenIterator::new(text, WritingDirection::VerticalTbRl).unwrap();
        let tokens: Vec<_> = iter.collect();
        
        let layout = CodeLayout::analyze(&tokens).unwrap();
        assert_eq!(layout.direction, WritingDirection::VerticalTbRl);
    }
}