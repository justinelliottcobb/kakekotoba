//! Indentation analysis for vertical code layouts

use crate::vertical::{Position2D, SpatialToken, SpatialTokenKind, WritingDirection};
use crate::error::Result;
use std::collections::HashMap;

/// Analyzes indentation patterns in vertically-written code
pub struct IndentationAnalyzer {
    direction: WritingDirection,
}

impl IndentationAnalyzer {
    /// Create a new indentation analyzer
    pub fn new(direction: WritingDirection) -> Self {
        Self { direction }
    }

    /// Analyze indentation levels from spatial tokens
    pub fn analyze_tokens(&self, tokens: &[SpatialToken]) -> Result<HashMap<Position2D, usize>> {
        let mut indentation_map = HashMap::new();
        let mut current_line_start = Position2D::origin();
        let mut current_indent = 0;
        let mut in_indentation = true;

        for token in tokens {
            match token.kind {
                SpatialTokenKind::LineBreak => {
                    // Reset for new line
                    current_line_start = Position2D::new(
                        token.span.end.column,
                        token.span.end.row,
                        token.span.end.byte_offset,
                    );
                    current_indent = 0;
                    in_indentation = true;
                }
                SpatialTokenKind::Whitespace if in_indentation => {
                    // Count indentation
                    current_indent += self.calculate_indent_contribution(&token.content);
                }
                _ => {
                    // Non-whitespace token, record indentation for this line
                    if in_indentation {
                        indentation_map.insert(token.span.start, current_indent);
                        in_indentation = false;
                    }
                }
            }
        }

        Ok(indentation_map)
    }

    /// Calculate how much a whitespace token contributes to indentation
    fn calculate_indent_contribution(&self, content: &str) -> usize {
        let mut contribution = 0;
        
        for ch in content.chars() {
            match ch {
                ' ' => contribution += 1,
                '\t' => contribution += 4, // Tab = 4 spaces by default
                _ => {} // Other whitespace doesn't count toward indentation
            }
        }
        
        contribution
    }

    /// Get the indentation style used in the code (spaces vs tabs)
    pub fn detect_indentation_style(&self, tokens: &[SpatialToken]) -> IndentationStyle {
        let mut space_count = 0;
        let mut tab_count = 0;
        let mut in_indentation = true;

        for token in tokens {
            match token.kind {
                SpatialTokenKind::LineBreak => {
                    in_indentation = true;
                }
                SpatialTokenKind::Whitespace if in_indentation => {
                    for ch in token.content.chars() {
                        match ch {
                            ' ' => space_count += 1,
                            '\t' => tab_count += 1,
                            _ => {}
                        }
                    }
                }
                _ => {
                    in_indentation = false;
                }
            }
        }

        if tab_count > space_count {
            IndentationStyle::Tabs
        } else if space_count > 0 {
            IndentationStyle::Spaces(self.detect_space_width(tokens))
        } else {
            IndentationStyle::Mixed
        }
    }

    /// Detect the number of spaces used per indentation level
    fn detect_space_width(&self, tokens: &[SpatialToken]) -> usize {
        // Simple heuristic: find the most common non-zero indentation difference
        let indents = self.analyze_tokens(tokens).unwrap_or_default();
        let mut levels: Vec<usize> = indents.values().copied().collect();
        levels.sort_unstable();
        levels.dedup();

        if levels.len() < 2 {
            return 4; // Default to 4 spaces
        }

        // Find the most common difference between consecutive levels
        let mut differences = Vec::new();
        for window in levels.windows(2) {
            if let [a, b] = window {
                if *b > *a {
                    differences.push(b - a);
                }
            }
        }

        differences.sort_unstable();
        
        // Return the most common difference, or 4 as default
        differences.first().copied().unwrap_or(4)
    }
}

/// Represents different indentation styles
#[derive(Debug, Clone, PartialEq)]
pub enum IndentationStyle {
    /// Using tab characters
    Tabs,
    /// Using spaces (with specified width per level)
    Spaces(usize),
    /// Mixed or inconsistent indentation
    Mixed,
}

/// Represents an indentation level change
#[derive(Debug, Clone)]
pub struct IndentationChange {
    /// Position where the change occurs
    pub position: Position2D,
    /// Previous indentation level
    pub from_level: usize,
    /// New indentation level
    pub to_level: usize,
    /// Type of change
    pub change_type: IndentationChangeType,
}

/// Types of indentation changes
#[derive(Debug, Clone, PartialEq)]
pub enum IndentationChangeType {
    /// Indentation increased (block started)
    Increase,
    /// Indentation decreased (block ended)
    Decrease,
    /// Indentation stayed the same
    NoChange,
}

impl IndentationChange {
    /// Create a new indentation change
    pub fn new(position: Position2D, from_level: usize, to_level: usize) -> Self {
        let change_type = if to_level > from_level {
            IndentationChangeType::Increase
        } else if to_level < from_level {
            IndentationChangeType::Decrease
        } else {
            IndentationChangeType::NoChange
        };

        Self {
            position,
            from_level,
            to_level,
            change_type,
        }
    }

    /// Get the magnitude of the change
    pub fn magnitude(&self) -> usize {
        if self.to_level > self.from_level {
            self.to_level - self.from_level
        } else {
            self.from_level - self.to_level
        }
    }
}

/// Tracks indentation changes throughout a document
pub struct IndentationTracker {
    changes: Vec<IndentationChange>,
    current_level: usize,
}

impl IndentationTracker {
    /// Create a new indentation tracker
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            current_level: 0,
        }
    }

    /// Record a new indentation level at a position
    pub fn record_level(&mut self, position: Position2D, level: usize) {
        if level != self.current_level {
            let change = IndentationChange::new(position, self.current_level, level);
            self.changes.push(change);
            self.current_level = level;
        }
    }

    /// Get all recorded indentation changes
    pub fn changes(&self) -> &[IndentationChange] {
        &self.changes
    }

    /// Get the current indentation level
    pub fn current_level(&self) -> usize {
        self.current_level
    }

    /// Find all block starts (indentation increases)
    pub fn block_starts(&self) -> Vec<&IndentationChange> {
        self.changes.iter()
            .filter(|change| change.change_type == IndentationChangeType::Increase)
            .collect()
    }

    /// Find all block ends (indentation decreases)
    pub fn block_ends(&self) -> Vec<&IndentationChange> {
        self.changes.iter()
            .filter(|change| change.change_type == IndentationChangeType::Decrease)
            .collect()
    }
}

impl Default for IndentationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vertical::{Span2D, SpatialToken};

    #[test]
    fn test_indentation_analyzer() {
        let analyzer = IndentationAnalyzer::new(WritingDirection::VerticalTbRl);
        
        // Create mock tokens with indentation
        let tokens = vec![
            SpatialToken::new(
                "  ".to_string(),
                Span2D::new(Position2D::new(0, 0, 0), Position2D::new(2, 0, 2)),
                SpatialTokenKind::Whitespace,
                WritingDirection::VerticalTbRl,
            ),
            SpatialToken::new(
                "code".to_string(),
                Span2D::new(Position2D::new(2, 0, 2), Position2D::new(6, 0, 6)),
                SpatialTokenKind::Ascii,
                WritingDirection::VerticalTbRl,
            ),
        ];

        let indents = analyzer.analyze_tokens(&tokens).unwrap();
        assert_eq!(indents.get(&Position2D::new(2, 0, 2)), Some(&2));
    }

    #[test]
    fn test_indentation_change() {
        let change = IndentationChange::new(Position2D::new(0, 0, 0), 0, 4);
        assert_eq!(change.change_type, IndentationChangeType::Increase);
        assert_eq!(change.magnitude(), 4);
    }

    #[test]
    fn test_indentation_tracker() {
        let mut tracker = IndentationTracker::new();
        
        tracker.record_level(Position2D::new(0, 0, 0), 0);
        tracker.record_level(Position2D::new(0, 1, 10), 4);
        tracker.record_level(Position2D::new(0, 2, 20), 0);
        
        assert_eq!(tracker.current_level(), 0);
        assert_eq!(tracker.changes().len(), 2);
        
        let block_starts = tracker.block_starts();
        assert_eq!(block_starts.len(), 1);
        assert_eq!(block_starts[0].to_level, 4);
    }

    #[test]
    fn test_indentation_style_detection() {
        let analyzer = IndentationAnalyzer::new(WritingDirection::VerticalTbRl);
        
        // Test with tab indentation
        let tab_tokens = vec![
            SpatialToken::new(
                "\t".to_string(),
                Span2D::new(Position2D::new(0, 0, 0), Position2D::new(1, 0, 1)),
                SpatialTokenKind::Whitespace,
                WritingDirection::VerticalTbRl,
            ),
        ];
        
        let style = analyzer.detect_indentation_style(&tab_tokens);
        assert_eq!(style, IndentationStyle::Tabs);
    }
}