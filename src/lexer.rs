use crate::error::{Error, Result, Span};
use crate::japanese::{JapaneseAnalyzer, KeywordDetector, KeywordType};
use crate::vertical::{
    Position2D, Span2D, SpatialToken, SpatialTokenKind, VerticalProcessor, WritingDirection,
};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Japanese keywords (placeholder - implement actual keywords)
    Kansuu,     // 関数 (function)
    Kata,       // 型 (type)
    Moshi,      // もし (if)
    Sore,       // それ (then/else)
    Kurikaeshi, // 繰り返し (loop/iterate)

    // Literals
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),

    // Identifiers
    Identifier(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Colon,
    Arrow,    // ->
    FatArrow, // =>

    // Type system
    TypeArrow, // ::
    Lambda,    // λ (lambda)

    // Whitespace and comments
    Whitespace,
    Comment(String),

    // End of file
    Eof,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: String,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, lexeme: String) -> Self {
        Self { kind, span, lexeme }
    }
}

#[derive(Debug)]
pub struct Lexer {
    input: String,
    position: usize,
    line: usize,
    column: usize,
    current_char: Option<char>,
    // Vertical programming extensions
    vertical_processor: VerticalProcessor,
    japanese_analyzer: JapaneseAnalyzer,
    writing_direction: WritingDirection,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            current_char: input.chars().next(),
            vertical_processor: VerticalProcessor::new(&input),
            japanese_analyzer: JapaneseAnalyzer::new(),
            writing_direction: WritingDirection::VerticalTbRl, // Default to vertical Japanese
            input,
            position: 0,
            line: 1,
            column: 1,
        };
        lexer.normalize_input();
        lexer
    }

    /// Create a new lexer with specific writing direction
    pub fn with_direction(input: String, direction: WritingDirection) -> Self {
        let mut lexer = Self::new(input);
        lexer.writing_direction = direction;
        lexer.vertical_processor = VerticalProcessor::new(&lexer.input);
        lexer
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            if let Some(token) = self.next_token()? {
                if !matches!(token.kind, TokenKind::Whitespace) {
                    tokens.push(token);
                }
            }
        }

        tokens.push(Token::new(
            TokenKind::Eof,
            self.current_span(),
            String::new(),
        ));

        Ok(tokens)
    }

    /// Tokenize with spatial/2D positioning information
    pub fn tokenize_spatial(&mut self) -> Result<Vec<SpatialToken>> {
        let mut spatial_tokens = Vec::new();
        let mut position_tracker = Position2D::origin();

        // Get grapheme clusters with their 2D positions from vertical processor
        let clusters = self.vertical_processor.grapheme_clusters();

        for (cluster_text, pos_2d) in clusters {
            if cluster_text.trim().is_empty() {
                // Handle whitespace
                let span = Span2D::new(
                    pos_2d,
                    Position2D::new(
                        pos_2d.column + cluster_text.len(),
                        pos_2d.row,
                        pos_2d.byte_offset + cluster_text.len(),
                    ),
                );

                let token_kind = if cluster_text.contains('\n') {
                    SpatialTokenKind::LineBreak
                } else {
                    SpatialTokenKind::Whitespace
                };

                spatial_tokens.push(SpatialToken::new(
                    cluster_text,
                    span,
                    token_kind,
                    self.writing_direction,
                ));
                continue;
            }

            // Classify the token using Japanese analyzer
            let token_kind = self.classify_spatial_token(&cluster_text);

            let end_pos = Position2D::new(
                pos_2d.column + cluster_text.chars().count(),
                pos_2d.row,
                pos_2d.byte_offset + cluster_text.len(),
            );
            let span = Span2D::new(pos_2d, end_pos);

            spatial_tokens.push(SpatialToken::new(
                cluster_text,
                span,
                token_kind,
                self.writing_direction,
            ));
        }

        Ok(spatial_tokens)
    }

    /// Classify a token spatially using Japanese and character analysis
    fn classify_spatial_token(&self, text: &str) -> SpatialTokenKind {
        // Check for Japanese keywords first
        if self.japanese_analyzer.has_keywords(text) {
            return SpatialTokenKind::Japanese;
        }

        // Classify based on character content
        let primary_script = self.japanese_analyzer.primary_script(text);

        match primary_script {
            crate::japanese::JapaneseScript::Kanji
            | crate::japanese::JapaneseScript::Hiragana
            | crate::japanese::JapaneseScript::Katakana => SpatialTokenKind::Japanese,
            crate::japanese::JapaneseScript::Ascii => {
                // Further classify ASCII content
                if text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    SpatialTokenKind::Ascii
                } else {
                    SpatialTokenKind::Punctuation
                }
            }
            _ => SpatialTokenKind::Other,
        }
    }

    /// Convert spatial tokens back to regular tokens (for compatibility)
    pub fn spatial_to_regular(spatial_tokens: Vec<SpatialToken>) -> Vec<Token> {
        spatial_tokens
            .into_iter()
            .map(|spatial_token| {
                let kind = Self::spatial_to_token_kind(&spatial_token);
                let span = Self::spatial_to_regular_span(&spatial_token.span);
                Token::new(kind, span, spatial_token.content)
            })
            .collect()
    }

    /// Convert spatial token kind to regular token kind
    fn spatial_to_token_kind(spatial_token: &SpatialToken) -> TokenKind {
        match spatial_token.kind {
            SpatialTokenKind::Japanese => {
                // Check for specific Japanese keywords
                match spatial_token.content.as_str() {
                    "関数" => TokenKind::Kansuu,
                    "型" => TokenKind::Kata,
                    "もし" => TokenKind::Moshi,
                    "それ" => TokenKind::Sore,
                    "繰り返し" => TokenKind::Kurikaeshi,
                    _ => TokenKind::Identifier(spatial_token.content.clone()),
                }
            }
            SpatialTokenKind::Ascii => {
                // Try to parse as number first
                if let Ok(int_val) = spatial_token.content.parse::<i64>() {
                    TokenKind::Integer(int_val)
                } else if let Ok(float_val) = spatial_token.content.parse::<f64>() {
                    TokenKind::Float(float_val)
                } else {
                    TokenKind::Identifier(spatial_token.content.clone())
                }
            }
            SpatialTokenKind::Punctuation => {
                // Map punctuation to specific token types
                match spatial_token.content.as_str() {
                    "(" => TokenKind::LeftParen,
                    ")" => TokenKind::RightParen,
                    "{" => TokenKind::LeftBrace,
                    "}" => TokenKind::RightBrace,
                    "[" => TokenKind::LeftBracket,
                    "]" => TokenKind::RightBracket,
                    "," => TokenKind::Comma,
                    ";" => TokenKind::Semicolon,
                    ":" => TokenKind::Colon,
                    "+" => TokenKind::Plus,
                    "-" => TokenKind::Minus,
                    "*" => TokenKind::Star,
                    "/" => TokenKind::Slash,
                    "=" => TokenKind::Equal,
                    "==" => TokenKind::EqualEqual,
                    "!=" => TokenKind::BangEqual,
                    "<" => TokenKind::Less,
                    "<=" => TokenKind::LessEqual,
                    ">" => TokenKind::Greater,
                    ">=" => TokenKind::GreaterEqual,
                    "->" => TokenKind::Arrow,
                    "=>" => TokenKind::FatArrow,
                    "::" => TokenKind::TypeArrow,
                    "λ" => TokenKind::Lambda,
                    _ => TokenKind::Identifier(spatial_token.content.clone()),
                }
            }
            SpatialTokenKind::Whitespace => TokenKind::Whitespace,
            SpatialTokenKind::LineBreak => TokenKind::Whitespace,
            SpatialTokenKind::Other => TokenKind::Identifier(spatial_token.content.clone()),
        }
    }

    /// Convert 2D span to regular span
    fn spatial_to_regular_span(span_2d: &Span2D) -> Span {
        Span::new(
            span_2d.start.byte_offset,
            span_2d.end.byte_offset,
            span_2d.start.row + 1,    // Convert 0-based to 1-based line numbers
            span_2d.start.column + 1, // Convert 0-based to 1-based column numbers
        )
    }

    fn normalize_input(&mut self) {
        use unicode_normalization::UnicodeNormalization;
        self.input = self.input.nfc().collect();
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        // Placeholder implementation - extend with actual tokenization logic
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;

        match self.current_char {
            Some('(') => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::LeftParen,
                    Span::new(start_pos, self.position, start_line, start_column),
                    "(".to_string(),
                )))
            }
            Some(')') => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::RightParen,
                    Span::new(start_pos, self.position, start_line, start_column),
                    ")".to_string(),
                )))
            }
            // Add more tokenization rules here
            _ => {
                self.advance();
                Err(Error::Lexer {
                    src: self.input.clone(),
                    position: Span::new(start_pos, self.position, start_line, start_column).into(),
                    message: format!("Unexpected character: {:?}", self.current_char),
                })
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            self.position += ch.len_utf8();

            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            self.current_char = self.input.chars().nth(self.char_position());
        }
    }

    fn char_position(&self) -> usize {
        self.input
            .grapheme_indices(true)
            .take_while(|(byte_idx, _)| *byte_idx < self.position)
            .count()
    }

    fn is_at_end(&self) -> bool {
        self.current_char.is_none()
    }

    fn current_span(&self) -> Span {
        Span::new(self.position, self.position, self.line, self.column)
    }
}
