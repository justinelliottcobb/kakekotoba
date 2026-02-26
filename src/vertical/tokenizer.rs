//! Direction-aware tokenization for vertical text

use super::{Span2D, WritingDirection};
use crate::error::Result;
use unicode_segmentation::UnicodeSegmentation;

/// A token with 2D positional information
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialToken {
    /// Token content
    pub content: String,
    /// 2D position span
    pub span: Span2D,
    /// Token type (to be defined by lexer)
    pub kind: SpatialTokenKind,
    /// Writing direction for this token
    pub direction: WritingDirection,
}

/// Basic token kinds for spatial tokenization
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialTokenKind {
    /// Whitespace (spaces, tabs, newlines)
    Whitespace,
    /// Line break / paragraph separator
    LineBreak,
    /// Japanese character (kanji, hiragana, katakana)
    Japanese,
    /// ASCII alphanumeric
    Ascii,
    /// Punctuation or operator
    Punctuation,
    /// Unknown/other character
    Other,
}

impl SpatialToken {
    /// Create a new spatial token
    pub fn new(
        content: String,
        span: Span2D,
        kind: SpatialTokenKind,
        direction: WritingDirection,
    ) -> Self {
        Self {
            content,
            span,
            kind,
            direction,
        }
    }

    /// Get the length of this token in bytes
    pub fn byte_length(&self) -> usize {
        self.span.byte_length()
    }

    /// Check if this token represents a line break
    pub fn is_line_break(&self) -> bool {
        matches!(self.kind, SpatialTokenKind::LineBreak)
    }

    /// Check if this token is whitespace
    pub fn is_whitespace(&self) -> bool {
        matches!(
            self.kind,
            SpatialTokenKind::Whitespace | SpatialTokenKind::LineBreak
        )
    }
}

/// Direction-aware tokenizer for vertical text
pub struct VerticalTokenizer {
    /// Input text
    text: String,
    /// Current byte position
    position: usize,
    /// Position mapper for 2D coordinates
    position_mapper: super::PositionMapper,
    /// Writing direction
    direction: WritingDirection,
}

impl VerticalTokenizer {
    /// Create a new vertical tokenizer
    pub fn new(text: &str, direction: WritingDirection) -> Self {
        let position_mapper = super::PositionMapper::new(text, direction);

        Self {
            text: text.to_string(),
            position: 0,
            position_mapper,
            direction,
        }
    }

    /// Tokenize the entire input into spatial tokens
    pub fn tokenize(&mut self) -> Result<Vec<SpatialToken>> {
        let mut tokens = Vec::new();

        for cluster in self.text.graphemes(true) {
            let start_pos = self.position_mapper.byte_to_2d(self.position);
            let end_pos = self
                .position_mapper
                .byte_to_2d(self.position + cluster.len());

            let span = Span2D::new(start_pos, end_pos);
            let kind = self.classify_grapheme(cluster);

            let token = SpatialToken::new(cluster.to_string(), span, kind, self.direction);

            tokens.push(token);
            self.position += cluster.len();
        }

        Ok(tokens)
    }

    /// Classify a grapheme cluster into a token kind
    fn classify_grapheme(&self, cluster: &str) -> SpatialTokenKind {
        let first_char = cluster.chars().next().unwrap_or('\0');

        match first_char {
            '\n' | '\r' => SpatialTokenKind::LineBreak,
            c if c.is_whitespace() => SpatialTokenKind::Whitespace,
            c if Self::is_japanese_char(c) => SpatialTokenKind::Japanese,
            c if c.is_ascii_alphanumeric() => SpatialTokenKind::Ascii,
            c if c.is_ascii_punctuation() => SpatialTokenKind::Punctuation,
            _ => SpatialTokenKind::Other,
        }
    }

    /// Check if a character is a Japanese character
    fn is_japanese_char(c: char) -> bool {
        matches!(c as u32,
            // Hiragana
            0x3040..=0x309F |
            // Katakana
            0x30A0..=0x30FF |
            // CJK Unified Ideographs (Kanji)
            0x4E00..=0x9FAF |
            // CJK Extension A
            0x3400..=0x4DBF |
            // Halfwidth Katakana
            0xFF65..=0xFF9F
        )
    }

    /// Get the current position
    pub fn current_position(&self) -> usize {
        self.position
    }

    /// Check if we've reached the end of input
    pub fn is_at_end(&self) -> bool {
        self.position >= self.text.len()
    }

    /// Reset the tokenizer to the beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
}

/// Iterator implementation for streaming tokenization
pub struct SpatialTokenIterator {
    _tokenizer: VerticalTokenizer,
    tokens: Vec<SpatialToken>,
    current_index: usize,
}

impl SpatialTokenIterator {
    /// Create a new token iterator
    pub fn new(text: &str, direction: WritingDirection) -> Result<Self> {
        let mut tokenizer = VerticalTokenizer::new(text, direction);
        let tokens = tokenizer.tokenize()?;

        Ok(Self {
            _tokenizer: tokenizer,
            tokens,
            current_index: 0,
        })
    }
}

impl Iterator for SpatialTokenIterator {
    type Item = SpatialToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.tokens.len() {
            let token = self.tokens[self.current_index].clone();
            self.current_index += 1;
            Some(token)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Position2D;
    use super::*;

    #[test]
    fn test_spatial_token_creation() {
        let start = Position2D::new(0, 0, 0);
        let end = Position2D::new(1, 0, 3);
        let span = Span2D::new(start, end);

        let token = SpatialToken::new(
            "関".to_string(),
            span,
            SpatialTokenKind::Japanese,
            WritingDirection::VerticalTbRl,
        );

        assert_eq!(token.content, "関");
        assert_eq!(token.kind, SpatialTokenKind::Japanese);
        assert!(!token.is_whitespace());
    }

    #[test]
    fn test_vertical_tokenizer() {
        let mut tokenizer = VerticalTokenizer::new("関数", WritingDirection::VerticalTbRl);
        let tokens = tokenizer.tokenize().unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].content, "関");
        assert_eq!(tokens[1].content, "数");
        assert!(matches!(tokens[0].kind, SpatialTokenKind::Japanese));
        assert!(matches!(tokens[1].kind, SpatialTokenKind::Japanese));
    }

    #[test]
    fn test_japanese_char_classification() {
        assert!(VerticalTokenizer::is_japanese_char('関'));
        assert!(VerticalTokenizer::is_japanese_char('あ'));
        assert!(VerticalTokenizer::is_japanese_char('ア'));
        assert!(!VerticalTokenizer::is_japanese_char('a'));
        assert!(!VerticalTokenizer::is_japanese_char('1'));
    }

    #[test]
    fn test_token_iterator() {
        let iter = SpatialTokenIterator::new("a関", WritingDirection::VerticalTbRl).unwrap();
        let tokens: Vec<_> = iter.collect();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].content, "a");
        assert_eq!(tokens[1].content, "関");
        assert!(matches!(tokens[0].kind, SpatialTokenKind::Ascii));
        assert!(matches!(tokens[1].kind, SpatialTokenKind::Japanese));
    }
}
