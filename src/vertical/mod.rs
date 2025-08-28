//! Core vertical text processing infrastructure for Kakekotoba
//!
//! This module provides the foundational tools for handling vertically-oriented
//! programming languages, including bidirectional text processing and 2D positioning.

use unicode_bidi::{BidiInfo, Level};
use unicode_segmentation::UnicodeSegmentation;
use crate::error::Result;

pub mod direction;
pub mod position;
pub mod tokenizer;

pub use direction::*;
pub use position::*;
pub use tokenizer::*;

/// Represents the writing direction for text processing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritingDirection {
    /// Horizontal, left-to-right (English style)
    HorizontalLtr,
    /// Horizontal, right-to-left (Arabic style)
    HorizontalRtl,
    /// Vertical, top-to-bottom, right-to-left (Traditional Japanese)
    VerticalTbRl,
    /// Vertical, top-to-bottom, left-to-right (Mongolian style)
    VerticalTbLr,
    /// Mixed writing direction within same text
    Mixed,
}

/// Core vertical text processor that handles bidirectional text and 2D positioning
pub struct VerticalProcessor {
    /// Bidirectional text analysis
    bidi_info: Option<BidiInfo>,
    /// Current writing direction
    direction: WritingDirection,
    /// Original text content
    content: String,
}

impl VerticalProcessor {
    /// Create a new vertical processor for the given text
    pub fn new(content: &str) -> Self {
        let bidi_info = BidiInfo::new(content, None);
        let direction = Self::detect_writing_direction(&bidi_info);
        
        Self {
            bidi_info: Some(bidi_info),
            direction,
            content: content.to_string(),
        }
    }

    /// Detect the primary writing direction from bidirectional analysis
    fn detect_writing_direction(bidi_info: &BidiInfo) -> WritingDirection {
        // For now, default to vertical Japanese style
        // TODO: Implement actual detection logic based on character analysis
        WritingDirection::VerticalTbRl
    }

    /// Get the writing direction
    pub fn direction(&self) -> WritingDirection {
        self.direction
    }

    /// Convert linear text positions to 2D coordinates
    pub fn to_2d_position(&self, byte_offset: usize) -> Result<Position2D> {
        // Placeholder implementation
        // TODO: Implement actual 2D positioning based on writing direction
        Ok(Position2D::new(0, 0, byte_offset))
    }

    /// Convert 2D coordinates back to linear positions
    pub fn from_2d_position(&self, pos: Position2D) -> Result<usize> {
        // Placeholder implementation
        Ok(pos.byte_offset)
    }

    /// Get grapheme clusters with their 2D positions
    pub fn grapheme_clusters(&self) -> Vec<(String, Position2D)> {
        let mut clusters = Vec::new();
        let mut byte_offset = 0;
        
        for cluster in self.content.graphemes(true) {
            // TODO: Calculate proper 2D position based on writing direction
            let pos = Position2D::new(0, clusters.len(), byte_offset);
            clusters.push((cluster.to_string(), pos));
            byte_offset += cluster.len();
        }
        
        clusters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_processor_creation() {
        let processor = VerticalProcessor::new("関数 main() {}");
        assert_eq!(processor.direction(), WritingDirection::VerticalTbRl);
    }

    #[test]
    fn test_grapheme_clusters() {
        let processor = VerticalProcessor::new("関数");
        let clusters = processor.grapheme_clusters();
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].0, "関");
        assert_eq!(clusters[1].0, "数");
    }
}