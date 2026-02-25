//! Text flow analysis for vertical programming languages

use crate::error::Result;
use crate::vertical::{Position2D, SpatialToken, WritingDirection};

/// Analyzes how text flows in 2D space for vertical programming languages
#[derive(Debug, Clone)]
pub struct TextFlow {
    /// Primary writing direction
    pub direction: WritingDirection,
    /// Flow segments representing different reading paths
    pub segments: Vec<FlowSegment>,
    /// Line breaks and flow interruptions
    pub breaks: Vec<FlowBreak>,
}

/// Represents a segment of text that flows in a consistent direction
#[derive(Debug, Clone)]
pub struct FlowSegment {
    /// Start position of the segment
    pub start: Position2D,
    /// End position of the segment
    pub end: Position2D,
    /// Flow direction for this segment
    pub direction: FlowDirection,
    /// Tokens included in this segment
    pub token_count: usize,
}

/// Specific flow directions within text
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowDirection {
    /// Top to bottom flow
    TopToBottom,
    /// Right to left flow  
    RightToLeft,
    /// Left to right flow
    LeftToRight,
    /// Bottom to top flow (rare)
    BottomToTop,
}

/// Represents interruptions in text flow
#[derive(Debug, Clone)]
pub struct FlowBreak {
    /// Position where the break occurs
    pub position: Position2D,
    /// Type of break
    pub break_type: FlowBreakType,
}

/// Types of flow breaks
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowBreakType {
    /// Line break (new line)
    LineBreak,
    /// Column break (new column in vertical text)
    ColumnBreak,
    /// Direction change
    DirectionChange,
    /// Block boundary
    BlockBoundary,
}

impl TextFlow {
    /// Create a new text flow analyzer
    pub fn new(direction: WritingDirection) -> Self {
        Self {
            direction,
            segments: Vec::new(),
            breaks: Vec::new(),
        }
    }

    /// Analyze text flow from spatial tokens
    pub fn analyze(tokens: &[SpatialToken]) -> Result<Self> {
        if tokens.is_empty() {
            return Ok(Self::new(WritingDirection::VerticalTbRl));
        }

        let direction = tokens[0].direction;
        let mut flow = Self::new(direction);

        let mut current_segment: Option<FlowSegmentBuilder> = None;
        let mut last_position = Position2D::origin();

        for token in tokens {
            // Skip pure whitespace for flow analysis
            if token.is_whitespace() {
                // But record line breaks
                if token.content.contains('\n') {
                    flow.breaks.push(FlowBreak {
                        position: token.span.start,
                        break_type: FlowBreakType::LineBreak,
                    });

                    // End current segment if it exists
                    if let Some(builder) = current_segment.take() {
                        flow.segments.push(builder.build());
                    }
                }
                continue;
            }

            let flow_direction =
                Self::determine_flow_direction(last_position, token.span.start, direction);

            // Check if we need to start a new segment
            if let Some(ref mut builder) = current_segment {
                if builder.direction != flow_direction
                    || Self::should_break_segment(last_position, token.span.start)
                {
                    // End current segment and start new one
                    flow.segments.push(builder.build());
                    current_segment =
                        Some(FlowSegmentBuilder::new(token.span.start, flow_direction));
                }
            } else {
                // Start first segment
                current_segment = Some(FlowSegmentBuilder::new(token.span.start, flow_direction));
            }

            // Add token to current segment
            if let Some(ref mut builder) = current_segment {
                builder.add_token_position(token.span.end);
            }

            last_position = token.span.end;
        }

        // Finish any remaining segment
        if let Some(builder) = current_segment {
            flow.segments.push(builder.build());
        }

        Ok(flow)
    }

    /// Determine flow direction between two positions
    fn determine_flow_direction(
        from: Position2D,
        to: Position2D,
        writing_direction: WritingDirection,
    ) -> FlowDirection {
        match writing_direction {
            WritingDirection::VerticalTbRl => {
                if to.row > from.row {
                    FlowDirection::TopToBottom
                } else if to.column < from.column {
                    FlowDirection::RightToLeft
                } else {
                    FlowDirection::TopToBottom // Default
                }
            }
            WritingDirection::HorizontalLtr => {
                if to.column > from.column {
                    FlowDirection::LeftToRight
                } else if to.row > from.row {
                    FlowDirection::TopToBottom
                } else {
                    FlowDirection::LeftToRight // Default
                }
            }
            _ => FlowDirection::TopToBottom, // Fallback
        }
    }

    /// Check if we should break the current segment
    fn should_break_segment(from: Position2D, to: Position2D) -> bool {
        // Break if there's a significant gap
        let row_gap = to.row.saturating_sub(from.row);
        let col_gap = if to.column > from.column {
            to.column - from.column
        } else {
            from.column - to.column
        };

        row_gap > 1 || col_gap > 10 // Heuristic thresholds
    }

    /// Get the primary flow direction for the entire text
    pub fn primary_flow_direction(&self) -> FlowDirection {
        if self.segments.is_empty() {
            return FlowDirection::TopToBottom;
        }

        // Count occurrences of each direction
        let mut counts = [0; 4]; // TopToBottom, RightToLeft, LeftToRight, BottomToTop

        for segment in &self.segments {
            match segment.direction {
                FlowDirection::TopToBottom => counts[0] += segment.token_count,
                FlowDirection::RightToLeft => counts[1] += segment.token_count,
                FlowDirection::LeftToRight => counts[2] += segment.token_count,
                FlowDirection::BottomToTop => counts[3] += segment.token_count,
            }
        }

        // Return the most common direction
        let max_idx = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        match max_idx {
            0 => FlowDirection::TopToBottom,
            1 => FlowDirection::RightToLeft,
            2 => FlowDirection::LeftToRight,
            3 => FlowDirection::BottomToTop,
            _ => FlowDirection::TopToBottom,
        }
    }

    /// Get segments that flow in a specific direction
    pub fn segments_with_direction(&self, direction: FlowDirection) -> Vec<&FlowSegment> {
        self.segments
            .iter()
            .filter(|segment| segment.direction == direction)
            .collect()
    }

    /// Count the total number of flow breaks
    pub fn break_count(&self) -> usize {
        self.breaks.len()
    }

    /// Get breaks of a specific type
    pub fn breaks_of_type(&self, break_type: FlowBreakType) -> Vec<&FlowBreak> {
        self.breaks
            .iter()
            .filter(|break_| break_.break_type == break_type)
            .collect()
    }

    /// Check if the text has mixed flow directions
    pub fn has_mixed_flow(&self) -> bool {
        if self.segments.len() <= 1 {
            return false;
        }

        let first_direction = self.segments[0].direction;
        self.segments
            .iter()
            .any(|seg| seg.direction != first_direction)
    }
}

/// Helper for building flow segments incrementally
struct FlowSegmentBuilder {
    start: Position2D,
    end: Position2D,
    direction: FlowDirection,
    token_count: usize,
}

impl FlowSegmentBuilder {
    fn new(start: Position2D, direction: FlowDirection) -> Self {
        Self {
            start,
            end: start,
            direction,
            token_count: 0,
        }
    }

    fn add_token_position(&mut self, position: Position2D) {
        self.end = position;
        self.token_count += 1;
    }

    fn build(self) -> FlowSegment {
        FlowSegment {
            start: self.start,
            end: self.end,
            direction: self.direction,
            token_count: self.token_count,
        }
    }
}

/// Flow analysis utilities
pub struct FlowAnalyzer;

impl FlowAnalyzer {
    /// Calculate reading complexity based on flow changes
    pub fn calculate_complexity(flow: &TextFlow) -> f64 {
        let base_complexity = flow.segments.len() as f64;
        let break_penalty = flow.breaks.len() as f64 * 0.5;
        let mixed_flow_penalty = if flow.has_mixed_flow() { 2.0 } else { 0.0 };

        base_complexity + break_penalty + mixed_flow_penalty
    }

    /// Get reading order positions for tokens
    pub fn reading_order_positions(flow: &TextFlow) -> Vec<Position2D> {
        let mut positions = Vec::new();

        for segment in &flow.segments {
            // Add segment positions in reading order
            positions.push(segment.start);
        }

        // Sort by reading order based on primary flow direction
        match flow.primary_flow_direction() {
            FlowDirection::TopToBottom => {
                positions.sort_by_key(|pos| (pos.column, pos.row));
            }
            FlowDirection::RightToLeft => {
                positions.sort_by_key(|pos| (pos.row, std::cmp::Reverse(pos.column)));
            }
            FlowDirection::LeftToRight => {
                positions.sort_by_key(|pos| (pos.row, pos.column));
            }
            FlowDirection::BottomToTop => {
                positions.sort_by_key(|pos| (pos.column, std::cmp::Reverse(pos.row)));
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::{Span2D, SpatialTokenKind};

    #[test]
    fn test_flow_creation() {
        let flow = TextFlow::new(WritingDirection::VerticalTbRl);
        assert_eq!(flow.direction, WritingDirection::VerticalTbRl);
        assert!(flow.segments.is_empty());
        assert!(flow.breaks.is_empty());
    }

    #[test]
    fn test_flow_direction_determination() {
        let from = Position2D::new(5, 0, 0);
        let to = Position2D::new(5, 1, 5);

        let direction =
            TextFlow::determine_flow_direction(from, to, WritingDirection::VerticalTbRl);

        assert_eq!(direction, FlowDirection::TopToBottom);
    }

    #[test]
    fn test_flow_segment_builder() {
        let mut builder =
            FlowSegmentBuilder::new(Position2D::new(0, 0, 0), FlowDirection::TopToBottom);

        builder.add_token_position(Position2D::new(0, 1, 5));
        builder.add_token_position(Position2D::new(0, 2, 10));

        let segment = builder.build();
        assert_eq!(segment.token_count, 2);
        assert_eq!(segment.direction, FlowDirection::TopToBottom);
        assert_eq!(segment.end, Position2D::new(0, 2, 10));
    }

    #[test]
    fn test_flow_analysis() {
        let tokens = vec![
            SpatialToken::new(
                "関".to_string(),
                Span2D::new(Position2D::new(0, 0, 0), Position2D::new(1, 0, 3)),
                SpatialTokenKind::Japanese,
                WritingDirection::VerticalTbRl,
            ),
            SpatialToken::new(
                "\n".to_string(),
                Span2D::new(Position2D::new(1, 0, 3), Position2D::new(0, 1, 4)),
                SpatialTokenKind::LineBreak,
                WritingDirection::VerticalTbRl,
            ),
            SpatialToken::new(
                "数".to_string(),
                Span2D::new(Position2D::new(0, 1, 4), Position2D::new(1, 1, 7)),
                SpatialTokenKind::Japanese,
                WritingDirection::VerticalTbRl,
            ),
        ];

        let flow = TextFlow::analyze(&tokens).unwrap();
        assert_eq!(flow.direction, WritingDirection::VerticalTbRl);
        assert_eq!(flow.break_count(), 1);
        assert_eq!(flow.breaks[0].break_type, FlowBreakType::LineBreak);
    }

    #[test]
    fn test_flow_analyzer() {
        let flow = TextFlow::new(WritingDirection::VerticalTbRl);
        let complexity = FlowAnalyzer::calculate_complexity(&flow);
        assert_eq!(complexity, 0.0); // Empty flow has zero complexity
    }
}
