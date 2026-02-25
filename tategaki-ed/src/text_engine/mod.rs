//! Core text engine for vertical text editing
//!
//! This module provides the foundational text storage and manipulation capabilities
//! for vertical Japanese text, with 2D-aware operations and spatial positioning.

use crate::{Result, TategakiError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicode_segmentation::UnicodeSegmentation;

pub mod buffer;
pub mod layout;
pub mod operations;

pub use buffer::*;
pub use layout::*;
pub use operations::*;

/// Text direction enum supporting multiple writing directions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDirection {
    /// Traditional Japanese vertical, top-to-bottom, right-to-left
    VerticalTopToBottom,
    /// Mongolian-style vertical, top-to-bottom, left-to-right
    VerticalTopToBottomLtr,
    /// Horizontal left-to-right (Western style)
    HorizontalLeftToRight,
    /// Horizontal right-to-left (Arabic style)
    HorizontalRightToLeft,
    /// Mixed direction (automatic detection)
    Mixed,
}

impl Default for TextDirection {
    fn default() -> Self {
        Self::VerticalTopToBottom
    }
}

impl TextDirection {
    /// Check if this direction is vertical
    pub fn is_vertical(&self) -> bool {
        matches!(
            self,
            Self::VerticalTopToBottom | Self::VerticalTopToBottomLtr
        )
    }

    /// Check if this direction is horizontal
    pub fn is_horizontal(&self) -> bool {
        matches!(
            self,
            Self::HorizontalLeftToRight | Self::HorizontalRightToLeft
        )
    }

    /// Get the primary reading direction
    pub fn primary_axis(&self) -> ReadingAxis {
        match self {
            Self::VerticalTopToBottom | Self::VerticalTopToBottomLtr => ReadingAxis::Vertical,
            Self::HorizontalLeftToRight | Self::HorizontalRightToLeft => ReadingAxis::Horizontal,
            Self::Mixed => ReadingAxis::Mixed,
        }
    }

    /// Get the secondary reading direction
    pub fn secondary_axis(&self) -> ReadingAxis {
        match self {
            Self::VerticalTopToBottom | Self::VerticalTopToBottomLtr => ReadingAxis::Horizontal,
            Self::HorizontalLeftToRight | Self::HorizontalRightToLeft => ReadingAxis::Vertical,
            Self::Mixed => ReadingAxis::Mixed,
        }
    }

    /// Check if text flows right-to-left in the secondary axis
    pub fn is_rtl_secondary(&self) -> bool {
        matches!(
            self,
            Self::VerticalTopToBottom | Self::HorizontalRightToLeft
        )
    }
}

/// Reading axis enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingAxis {
    Vertical,
    Horizontal,
    Mixed,
}

/// 2D-aware text buffer for vertical programming languages
#[derive(Debug, Clone)]
pub struct VerticalTextBuffer {
    /// Text content stored as grapheme clusters
    content: Vec<String>,
    /// Line break positions for 2D navigation
    line_breaks: Vec<usize>,
    /// Text direction setting
    direction: TextDirection,
    /// Character width cache for layout calculations
    char_widths: BTreeMap<String, f32>,
    /// Spatial metadata for programming language features
    spatial_metadata: SpatialMetadata,
    /// Change tracking for undo/redo
    change_history: ChangeHistory,
}

/// Spatial metadata for programming language integration
#[derive(Debug, Clone, Default)]
pub struct SpatialMetadata {
    /// Indentation levels by line
    pub indentation_levels: BTreeMap<usize, usize>,
    /// Code block boundaries
    pub block_boundaries: Vec<BlockBoundary>,
    /// Syntax highlighting regions
    pub syntax_regions: Vec<SyntaxRegion>,
    /// Error/warning markers
    pub diagnostic_markers: Vec<DiagnosticMarker>,
}

/// Code block boundary information
#[derive(Debug, Clone)]
pub struct BlockBoundary {
    /// Start position of the block
    pub start: crate::spatial::SpatialPosition,
    /// End position of the block
    pub end: crate::spatial::SpatialPosition,
    /// Type of block
    pub block_type: BlockType,
    /// Nesting level
    pub nesting_level: usize,
}

/// Types of code blocks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Function,
    Type,
    Conditional,
    Loop,
    Generic,
    Comment,
}

/// Syntax highlighting region
#[derive(Debug, Clone)]
pub struct SyntaxRegion {
    /// Start position
    pub start: crate::spatial::SpatialPosition,
    /// End position
    pub end: crate::spatial::SpatialPosition,
    /// Syntax element type
    pub syntax_type: SyntaxType,
}

/// Types of syntax elements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxType {
    Keyword,
    String,
    Number,
    Comment,
    Operator,
    Identifier,
    Type,
    Error,
}

/// Diagnostic marker for errors and warnings
#[derive(Debug, Clone)]
pub struct DiagnosticMarker {
    /// Position of the diagnostic
    pub position: crate::spatial::SpatialPosition,
    /// Severity level
    pub severity: DiagnosticSeverity,
    /// Message text
    pub message: String,
    /// Optional fix suggestions
    pub fixes: Vec<String>,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Change tracking for undo/redo functionality
#[derive(Debug, Clone, Default)]
pub struct ChangeHistory {
    /// History of changes
    pub changes: Vec<TextChange>,
    /// Current position in history
    pub current_index: usize,
    /// Maximum history size
    pub max_size: usize,
}

/// A single text change operation
#[derive(Debug, Clone)]
pub struct TextChange {
    /// Position where change occurred
    pub position: crate::spatial::SpatialPosition,
    /// Type of change
    pub change_type: ChangeType,
    /// Text that was added or removed
    pub text: String,
    /// Timestamp of change
    pub timestamp: std::time::SystemTime,
}

/// Types of text changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Insert,
    Delete,
    Replace,
}

impl VerticalTextBuffer {
    /// Create a new empty vertical text buffer
    pub fn new(direction: TextDirection) -> Self {
        Self {
            content: Vec::new(),
            line_breaks: vec![0],
            direction,
            char_widths: BTreeMap::new(),
            spatial_metadata: SpatialMetadata::default(),
            change_history: ChangeHistory {
                changes: Vec::new(),
                current_index: 0,
                max_size: 1000,
            },
        }
    }

    /// Create buffer from text content
    pub fn from_text(text: &str, direction: TextDirection) -> Result<Self> {
        let mut buffer = Self::new(direction);
        buffer.insert_text_at_start(text)?;
        Ok(buffer)
    }

    /// Get the current text direction
    pub fn direction(&self) -> TextDirection {
        self.direction
    }

    /// Set the text direction
    pub fn set_direction(&mut self, direction: TextDirection) {
        self.direction = direction;
        // TODO: Recalculate layout when direction changes
    }

    /// Get total character count
    pub fn char_count(&self) -> usize {
        self.content.len()
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.line_breaks.len().saturating_sub(1)
    }

    /// Get text content as string
    pub fn as_text(&self) -> String {
        self.content.join("")
    }

    /// Insert text at the beginning (placeholder)
    fn insert_text_at_start(&mut self, text: &str) -> Result<()> {
        let graphemes: Vec<String> = text.graphemes(true).map(|s| s.to_string()).collect();

        // Track line breaks
        let mut line_break_offset = 0;
        for (i, grapheme) in graphemes.iter().enumerate() {
            if grapheme == "\n" {
                self.line_breaks.push(i + 1 + line_break_offset);
                line_break_offset = 0;
            } else {
                line_break_offset += 1;
            }
        }

        self.content = graphemes;
        Ok(())
    }

    /// Get character at position (placeholder)
    pub fn char_at(&self, index: usize) -> Option<&str> {
        self.content.get(index).map(|s| s.as_str())
    }

    /// Get spatial metadata
    pub fn spatial_metadata(&self) -> &SpatialMetadata {
        &self.spatial_metadata
    }

    /// Get mutable spatial metadata
    pub fn spatial_metadata_mut(&mut self) -> &mut SpatialMetadata {
        &mut self.spatial_metadata
    }

    /// Update syntax highlighting regions
    pub fn update_syntax_highlighting(&mut self, regions: Vec<SyntaxRegion>) {
        self.spatial_metadata.syntax_regions = regions;
    }

    /// Add diagnostic marker
    pub fn add_diagnostic(&mut self, marker: DiagnosticMarker) {
        self.spatial_metadata.diagnostic_markers.push(marker);
    }

    /// Clear all diagnostics
    pub fn clear_diagnostics(&mut self) {
        self.spatial_metadata.diagnostic_markers.clear();
    }

    /// Get diagnostics at position
    pub fn diagnostics_at(
        &self,
        position: &crate::spatial::SpatialPosition,
    ) -> Vec<&DiagnosticMarker> {
        self.spatial_metadata
            .diagnostic_markers
            .iter()
            .filter(|marker| &marker.position == position)
            .collect()
    }
}

/// Layout engine for converting between logical and visual coordinates
#[derive(Debug)]
pub struct LayoutEngine {
    /// Text direction for calculations
    direction: TextDirection,
    /// Font metrics cache
    font_metrics: FontMetrics,
    /// Viewport information
    viewport: Viewport,
}

/// Font metrics for layout calculations
#[derive(Debug, Clone)]
pub struct FontMetrics {
    /// Character width (for monospace assumption)
    pub char_width: f32,
    /// Line height
    pub line_height: f32,
    /// Baseline offset
    pub baseline: f32,
    /// Japanese character width multiplier
    pub japanese_width_factor: f32,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            char_width: 8.0,
            line_height: 16.0,
            baseline: 12.0,
            japanese_width_factor: 2.0,
        }
    }
}

/// Viewport information for rendering
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Width in pixels
    pub width: f32,
    /// Height in pixels
    pub height: f32,
    /// Scroll offset X
    pub scroll_x: f32,
    /// Scroll offset Y
    pub scroll_y: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        }
    }
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new(direction: TextDirection) -> Self {
        Self {
            direction,
            font_metrics: FontMetrics::default(),
            viewport: Viewport::default(),
        }
    }

    /// Convert logical position to visual coordinates
    pub fn logical_to_visual(
        &self,
        position: &crate::spatial::SpatialPosition,
    ) -> Result<(f32, f32)> {
        match self.direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, columns go right-to-left, rows go top-to-bottom
                let x =
                    self.viewport.width - (position.column as f32 * self.font_metrics.char_width);
                let y = position.row as f32 * self.font_metrics.line_height;
                Ok((x, y))
            }
            TextDirection::HorizontalLeftToRight => {
                // Standard horizontal layout
                let x = position.column as f32 * self.font_metrics.char_width;
                let y = position.row as f32 * self.font_metrics.line_height;
                Ok((x, y))
            }
            _ => {
                // Placeholder for other directions
                Ok((0.0, 0.0))
            }
        }
    }

    /// Convert visual coordinates to logical position
    pub fn visual_to_logical(&self, x: f32, y: f32) -> Result<crate::spatial::SpatialPosition> {
        match self.direction {
            TextDirection::VerticalTopToBottom => {
                let column = ((self.viewport.width - x) / self.font_metrics.char_width) as usize;
                let row = (y / self.font_metrics.line_height) as usize;
                Ok(crate::spatial::SpatialPosition::new(column, row, 0))
            }
            TextDirection::HorizontalLeftToRight => {
                let column = (x / self.font_metrics.char_width) as usize;
                let row = (y / self.font_metrics.line_height) as usize;
                Ok(crate::spatial::SpatialPosition::new(column, row, 0))
            }
            _ => {
                // Placeholder for other directions
                Ok(crate::spatial::SpatialPosition::new(0, 0, 0))
            }
        }
    }

    /// Update viewport settings
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    /// Update font metrics
    pub fn set_font_metrics(&mut self, metrics: FontMetrics) {
        self.font_metrics = metrics;
    }

    /// Get current font metrics
    pub fn font_metrics(&self) -> &FontMetrics {
        &self.font_metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_direction_properties() {
        assert!(TextDirection::VerticalTopToBottom.is_vertical());
        assert!(TextDirection::HorizontalLeftToRight.is_horizontal());
        assert!(TextDirection::VerticalTopToBottom.is_rtl_secondary());
    }

    #[test]
    fn test_vertical_text_buffer_creation() {
        let buffer = VerticalTextBuffer::new(TextDirection::VerticalTopToBottom);
        assert_eq!(buffer.direction(), TextDirection::VerticalTopToBottom);
        assert_eq!(buffer.char_count(), 0);
        assert_eq!(buffer.line_count(), 0);
    }

    #[test]
    fn test_buffer_from_text() {
        let buffer =
            VerticalTextBuffer::from_text("Hello\nWorld", TextDirection::VerticalTopToBottom)
                .unwrap();
        assert_eq!(buffer.char_count(), 11); // Including newline
        assert_eq!(buffer.as_text(), "Hello\nWorld");
    }

    #[test]
    fn test_layout_engine() {
        let engine = LayoutEngine::new(TextDirection::VerticalTopToBottom);
        let pos = crate::spatial::SpatialPosition::new(1, 2, 0);
        let visual = engine.logical_to_visual(&pos).unwrap();

        // Should place text right-to-left for vertical
        assert!(visual.0 < engine.viewport.width);
        assert!(visual.1 > 0.0);
    }
}
