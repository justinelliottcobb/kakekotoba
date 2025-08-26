//! Direction-aware text processing utilities

use unicode_bidi::{BidiInfo, Level};
use crate::error::Result;

/// Direction information for text segments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectionInfo {
    /// Whether this segment is right-to-left
    pub is_rtl: bool,
    /// Bidirectional embedding level
    pub level: u8,
    /// Primary writing direction
    pub writing_mode: super::WritingDirection,
}

impl DirectionInfo {
    /// Create new direction info
    pub fn new(is_rtl: bool, level: u8, writing_mode: super::WritingDirection) -> Self {
        Self {
            is_rtl,
            level,
            writing_mode,
        }
    }

    /// Create direction info from Unicode bidirectional level
    pub fn from_bidi_level(level: Level, writing_mode: super::WritingDirection) -> Self {
        Self {
            is_rtl: level.is_rtl(),
            level: level.number(),
            writing_mode,
        }
    }
}

/// Analyzes text direction for mixed content
pub struct DirectionAnalyzer {
    bidi_info: BidiInfo,
}

impl DirectionAnalyzer {
    /// Create a new direction analyzer
    pub fn new(text: &str) -> Self {
        Self {
            bidi_info: BidiInfo::new(text, None),
        }
    }

    /// Get direction info for a specific byte range
    pub fn direction_at_range(&self, start: usize, end: usize) -> Result<DirectionInfo> {
        // Get the bidirectional level at the start position
        let levels = self.bidi_info.levels();
        
        if let Some(level) = levels.get(start) {
            Ok(DirectionInfo::from_bidi_level(
                *level,
                super::WritingDirection::VerticalTbRl, // Default for Japanese
            ))
        } else {
            Ok(DirectionInfo::new(
                false,
                0,
                super::WritingDirection::VerticalTbRl,
            ))
        }
    }

    /// Check if the text contains mixed writing directions
    pub fn has_mixed_directions(&self) -> bool {
        let levels = self.bidi_info.levels();
        if levels.is_empty() {
            return false;
        }
        
        let first_level = levels[0];
        levels.iter().any(|&level| level != first_level)
    }

    /// Get all direction changes in the text
    pub fn direction_changes(&self) -> Vec<(usize, DirectionInfo)> {
        let mut changes = Vec::new();
        let levels = self.bidi_info.levels();
        
        if levels.is_empty() {
            return changes;
        }
        
        let mut current_level = levels[0];
        changes.push((0, DirectionInfo::from_bidi_level(
            current_level,
            super::WritingDirection::VerticalTbRl,
        )));
        
        for (i, &level) in levels.iter().enumerate().skip(1) {
            if level != current_level {
                current_level = level;
                changes.push((i, DirectionInfo::from_bidi_level(
                    level,
                    super::WritingDirection::VerticalTbRl,
                )));
            }
        }
        
        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_info_creation() {
        let info = DirectionInfo::new(false, 0, super::WritingDirection::VerticalTbRl);
        assert!(!info.is_rtl);
        assert_eq!(info.level, 0);
    }

    #[test]
    fn test_direction_analyzer() {
        let analyzer = DirectionAnalyzer::new("Hello 世界");
        assert!(!analyzer.has_mixed_directions());
        
        let changes = analyzer.direction_changes();
        assert!(!changes.is_empty());
    }
}