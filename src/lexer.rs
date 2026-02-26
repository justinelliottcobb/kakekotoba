use crate::error::{Error, Result, Span};
use crate::japanese::JapaneseAnalyzer;
use crate::vertical::{
    Position2D, Span2D, SpatialToken, SpatialTokenKind, VerticalProcessor, WritingDirection,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Japanese keywords (kept for backwards compatibility with Haskell-style parser)
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

    // Identifiers (used for Japanese keywords, ASCII names, and operators in S-expr context)
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
    LeftParen,  // ( or 「 or 【
    RightParen, // ) or 」 or 】
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

/// Tracks which bracket style opened a paren group (for matching validation)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketStyle {
    Round,      // ( )
    Corner,     // 「 」
    Lenticular, // 【 】
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
    /// Characters pre-extracted for O(1) indexed access
    chars: Vec<char>,
    /// Current index into chars vec
    char_idx: usize,
    /// Current byte position in input
    position: usize,
    line: usize,
    column: usize,
    /// Stack of open bracket styles for matching validation
    bracket_stack: Vec<BracketStyle>,
    // Vertical programming extensions
    vertical_processor: VerticalProcessor,
    japanese_analyzer: JapaneseAnalyzer,
    writing_direction: WritingDirection,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            chars: input.chars().collect(),
            char_idx: 0,
            vertical_processor: VerticalProcessor::new(&input),
            japanese_analyzer: JapaneseAnalyzer::new(),
            writing_direction: WritingDirection::VerticalTbRl,
            input,
            position: 0,
            line: 1,
            column: 1,
            bracket_stack: Vec::new(),
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
                if !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment(_)) {
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

        // Get grapheme clusters with their 2D positions from vertical processor
        let clusters = self.vertical_processor.grapheme_clusters();

        for (cluster_text, pos_2d) in clusters {
            if cluster_text.trim().is_empty() {
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

    fn classify_spatial_token(&self, text: &str) -> SpatialTokenKind {
        if self.japanese_analyzer.has_keywords(text) {
            return SpatialTokenKind::Japanese;
        }

        let primary_script = self.japanese_analyzer.primary_script(text);

        match primary_script {
            crate::japanese::JapaneseScript::Kanji
            | crate::japanese::JapaneseScript::Hiragana
            | crate::japanese::JapaneseScript::Katakana => SpatialTokenKind::Japanese,
            crate::japanese::JapaneseScript::Ascii => {
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

    fn spatial_to_token_kind(spatial_token: &SpatialToken) -> TokenKind {
        match spatial_token.kind {
            SpatialTokenKind::Japanese => match spatial_token.content.as_str() {
                "関数" => TokenKind::Kansuu,
                "型" => TokenKind::Kata,
                "もし" => TokenKind::Moshi,
                "それ" => TokenKind::Sore,
                "繰り返し" => TokenKind::Kurikaeshi,
                _ => TokenKind::Identifier(spatial_token.content.clone()),
            },
            SpatialTokenKind::Ascii => {
                if let Ok(int_val) = spatial_token.content.parse::<i64>() {
                    TokenKind::Integer(int_val)
                } else if let Ok(float_val) = spatial_token.content.parse::<f64>() {
                    TokenKind::Float(float_val)
                } else {
                    TokenKind::Identifier(spatial_token.content.clone())
                }
            }
            SpatialTokenKind::Punctuation => match spatial_token.content.as_str() {
                "(" | "「" | "【" => TokenKind::LeftParen,
                ")" | "」" | "】" => TokenKind::RightParen,
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
            },
            SpatialTokenKind::Whitespace => TokenKind::Whitespace,
            SpatialTokenKind::LineBreak => TokenKind::Whitespace,
            SpatialTokenKind::Other => TokenKind::Identifier(spatial_token.content.clone()),
        }
    }

    fn spatial_to_regular_span(span_2d: &Span2D) -> Span {
        Span::new(
            span_2d.start.byte_offset,
            span_2d.end.byte_offset,
            span_2d.start.row + 1,
            span_2d.start.column + 1,
        )
    }

    fn normalize_input(&mut self) {
        use unicode_normalization::UnicodeNormalization;
        self.input = self.input.nfc().collect();
        self.chars = self.input.chars().collect();
    }

    // ========================================================================
    // Core tokenizer
    // ========================================================================

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.char_idx).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.char_idx + 1).copied()
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(None);
        }

        let start_pos = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let ch = self.current_char().unwrap();

        let token = match ch {
            // Comments: -- to end of line
            '-' if self.peek_char() == Some('-') => {
                self.advance();
                self.advance();
                let mut comment = String::new();
                while let Some(c) = self.current_char() {
                    if c == '\n' {
                        break;
                    }
                    comment.push(c);
                    self.advance();
                }
                Token::new(
                    TokenKind::Comment(comment.clone()),
                    Span::new(start_pos, self.position, start_line, start_col),
                    format!("--{}", comment),
                )
            }

            // Parentheses and Japanese bracket equivalents
            '(' => {
                self.bracket_stack.push(BracketStyle::Round);
                self.advance();
                Token::new(
                    TokenKind::LeftParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "(".to_string(),
                )
            }
            ')' => {
                self.validate_close_bracket(BracketStyle::Round, start_pos, start_line, start_col)?;
                self.advance();
                Token::new(
                    TokenKind::RightParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    ")".to_string(),
                )
            }
            '「' => {
                self.bracket_stack.push(BracketStyle::Corner);
                self.advance();
                Token::new(
                    TokenKind::LeftParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "「".to_string(),
                )
            }
            '」' => {
                self.validate_close_bracket(
                    BracketStyle::Corner,
                    start_pos,
                    start_line,
                    start_col,
                )?;
                self.advance();
                Token::new(
                    TokenKind::RightParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "」".to_string(),
                )
            }
            '【' => {
                self.bracket_stack.push(BracketStyle::Lenticular);
                self.advance();
                Token::new(
                    TokenKind::LeftParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "【".to_string(),
                )
            }
            '】' => {
                self.validate_close_bracket(
                    BracketStyle::Lenticular,
                    start_pos,
                    start_line,
                    start_col,
                )?;
                self.advance();
                Token::new(
                    TokenKind::RightParen,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "】".to_string(),
                )
            }

            // Other delimiters
            '{' => {
                self.advance();
                Token::new(
                    TokenKind::LeftBrace,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "{".to_string(),
                )
            }
            '}' => {
                self.advance();
                Token::new(
                    TokenKind::RightBrace,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "}".to_string(),
                )
            }
            '[' => {
                self.advance();
                Token::new(
                    TokenKind::LeftBracket,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "[".to_string(),
                )
            }
            ']' => {
                self.advance();
                Token::new(
                    TokenKind::RightBracket,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "]".to_string(),
                )
            }
            ',' => {
                self.advance();
                Token::new(
                    TokenKind::Comma,
                    Span::new(start_pos, self.position, start_line, start_col),
                    ",".to_string(),
                )
            }
            ';' => {
                self.advance();
                Token::new(
                    TokenKind::Semicolon,
                    Span::new(start_pos, self.position, start_line, start_col),
                    ";".to_string(),
                )
            }

            // Multi-character operators and single-char operators
            '-' => {
                self.advance();
                if self.current_char() == Some('>') {
                    self.advance();
                    Token::new(
                        TokenKind::Arrow,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "->".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Minus,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "-".to_string(),
                    )
                }
            }
            '=' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::new(
                        TokenKind::EqualEqual,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "==".to_string(),
                    )
                } else if self.current_char() == Some('>') {
                    self.advance();
                    Token::new(
                        TokenKind::FatArrow,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "=>".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Equal,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "=".to_string(),
                    )
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::new(
                        TokenKind::BangEqual,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "!=".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Identifier("!".to_string()),
                        Span::new(start_pos, self.position, start_line, start_col),
                        "!".to_string(),
                    )
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::new(
                        TokenKind::LessEqual,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "<=".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Less,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "<".to_string(),
                    )
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == Some('=') {
                    self.advance();
                    Token::new(
                        TokenKind::GreaterEqual,
                        Span::new(start_pos, self.position, start_line, start_col),
                        ">=".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Greater,
                        Span::new(start_pos, self.position, start_line, start_col),
                        ">".to_string(),
                    )
                }
            }
            ':' => {
                self.advance();
                if self.current_char() == Some(':') {
                    self.advance();
                    Token::new(
                        TokenKind::TypeArrow,
                        Span::new(start_pos, self.position, start_line, start_col),
                        "::".to_string(),
                    )
                } else {
                    Token::new(
                        TokenKind::Colon,
                        Span::new(start_pos, self.position, start_line, start_col),
                        ":".to_string(),
                    )
                }
            }
            '+' => {
                self.advance();
                Token::new(
                    TokenKind::Plus,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "+".to_string(),
                )
            }
            '*' => {
                self.advance();
                Token::new(
                    TokenKind::Star,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "*".to_string(),
                )
            }
            '/' => {
                self.advance();
                Token::new(
                    TokenKind::Slash,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "/".to_string(),
                )
            }
            '%' => {
                self.advance();
                Token::new(
                    TokenKind::Identifier("%".to_string()),
                    Span::new(start_pos, self.position, start_line, start_col),
                    "%".to_string(),
                )
            }
            'λ' => {
                self.advance();
                Token::new(
                    TokenKind::Lambda,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "λ".to_string(),
                )
            }

            // String literals
            '"' => self.read_string(start_pos, start_line, start_col)?,

            // Numbers
            c if c.is_ascii_digit() => self.read_number(start_pos, start_line, start_col)?,

            // ASCII identifiers
            c if c.is_ascii_alphabetic() || c == '_' => {
                self.read_ascii_identifier(start_pos, start_line, start_col)?
            }

            // Japanese identifiers (kanji, hiragana, katakana)
            c if is_japanese_char(c) => {
                self.read_japanese_identifier(start_pos, start_line, start_col)?
            }

            // Japanese arrow →
            '→' => {
                self.advance();
                Token::new(
                    TokenKind::Arrow,
                    Span::new(start_pos, self.position, start_line, start_col),
                    "→".to_string(),
                )
            }

            _ => {
                let c = ch;
                self.advance();
                return Err(Error::Lexer {
                    src: self.input.clone(),
                    position: Span::new(start_pos, self.position, start_line, start_col).into(),
                    message: format!("Unexpected character: '{}'", c),
                });
            }
        };

        Ok(Some(token))
    }

    fn read_string(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token> {
        self.advance(); // consume opening "
        let mut s = String::new();

        loop {
            match self.current_char() {
                None => {
                    return Err(Error::Lexer {
                        src: self.input.clone(),
                        position: Span::new(start_pos, self.position, start_line, start_col).into(),
                        message: "Unterminated string literal".to_string(),
                    });
                }
                Some('"') => {
                    self.advance(); // consume closing "
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char() {
                        Some('n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            s.push('\t');
                            self.advance();
                        }
                        Some('\\') => {
                            s.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            s.push('"');
                            self.advance();
                        }
                        Some(c) => {
                            s.push('\\');
                            s.push(c);
                            self.advance();
                        }
                        None => {
                            return Err(Error::Lexer {
                                src: self.input.clone(),
                                position: Span::new(
                                    start_pos,
                                    self.position,
                                    start_line,
                                    start_col,
                                )
                                .into(),
                                message: "Unterminated escape sequence".to_string(),
                            });
                        }
                    }
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }

        let lexeme = format!("\"{}\"", s);
        Ok(Token::new(
            TokenKind::String(s),
            Span::new(start_pos, self.position, start_line, start_col),
            lexeme,
        ))
    }

    fn read_number(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token> {
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                // Check if next char is a digit (to distinguish from method calls etc.)
                if let Some(next) = self.peek_char() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let span = Span::new(start_pos, self.position, start_line, start_col);

        if is_float {
            let val: f64 = num_str.parse().map_err(|_| Error::Lexer {
                src: self.input.clone(),
                position: span.clone().into(),
                message: format!("Invalid float literal: {}", num_str),
            })?;
            Ok(Token::new(TokenKind::Float(val), span, num_str))
        } else {
            let val: i64 = num_str.parse().map_err(|_| Error::Lexer {
                src: self.input.clone(),
                position: span.clone().into(),
                message: format!("Invalid integer literal: {}", num_str),
            })?;
            Ok(Token::new(TokenKind::Integer(val), span, num_str))
        }
    }

    fn read_ascii_identifier(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token> {
        let mut name = String::new();

        while let Some(c) = self.current_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let span = Span::new(start_pos, self.position, start_line, start_col);

        // Check for boolean literals
        let kind = match name.as_str() {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Identifier(name.clone()),
        };

        Ok(Token::new(kind, span, name))
    }

    fn read_japanese_identifier(
        &mut self,
        start_pos: usize,
        start_line: usize,
        start_col: usize,
    ) -> Result<Token> {
        let mut name = String::new();

        while let Some(c) = self.current_char() {
            if is_japanese_char(c) {
                name.push(c);
                self.advance();
            } else {
                break;
            }
        }

        let span = Span::new(start_pos, self.position, start_line, start_col);

        // Check for Japanese boolean/unit literals
        let kind = match name.as_str() {
            "真" => TokenKind::Bool(true),
            "偽" => TokenKind::Bool(false),
            _ => TokenKind::Identifier(name.clone()),
        };

        Ok(Token::new(kind, span, name))
    }

    fn validate_close_bracket(
        &mut self,
        expected: BracketStyle,
        pos: usize,
        line: usize,
        col: usize,
    ) -> Result<()> {
        match self.bracket_stack.pop() {
            Some(open) if open == expected => Ok(()),
            Some(open) => Err(Error::Lexer {
                src: self.input.clone(),
                position: Span::new(pos, pos + 1, line, col).into(),
                message: format!(
                    "Mismatched brackets: opened with {} but closed with {}",
                    bracket_style_open_char(open),
                    bracket_style_close_char(expected),
                ),
            }),
            None => Err(Error::Lexer {
                src: self.input.clone(),
                position: Span::new(pos, pos + 1, line, col).into(),
                message: format!(
                    "Unexpected closing bracket '{}'",
                    bracket_style_close_char(expected),
                ),
            }),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.chars.get(self.char_idx) {
            self.position += ch.len_utf8();

            if *ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            self.char_idx += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.char_idx >= self.chars.len()
    }

    fn current_span(&self) -> Span {
        Span::new(self.position, self.position, self.line, self.column)
    }
}

/// Check if a character is a Japanese character (kanji, hiragana, katakana)
fn is_japanese_char(c: char) -> bool {
    // Hiragana
    ('\u{3040}'..='\u{309F}').contains(&c) ||
    // Katakana
    ('\u{30A0}'..='\u{30FF}').contains(&c) ||
    // Katakana half-width
    ('\u{FF65}'..='\u{FF9F}').contains(&c) ||
    // CJK Unified Ideographs (main block)
    ('\u{4E00}'..='\u{9FAF}').contains(&c) ||
    // CJK Extension A
    ('\u{3400}'..='\u{4DBF}').contains(&c) ||
    // CJK Extension B
    ('\u{20000}'..='\u{2A6DF}').contains(&c) ||
    // CJK Compatibility Ideographs
    ('\u{F900}'..='\u{FAFF}').contains(&c) ||
    // Prolonged sound mark (ー)
    c == '\u{30FC}' ||
    // Middle dot (・)
    c == '\u{30FB}'
}

fn bracket_style_open_char(style: BracketStyle) -> char {
    match style {
        BracketStyle::Round => '(',
        BracketStyle::Corner => '「',
        BracketStyle::Lenticular => '【',
    }
}

fn bracket_style_close_char(style: BracketStyle) -> char {
    match style {
        BracketStyle::Round => ')',
        BracketStyle::Corner => '」',
        BracketStyle::Lenticular => '】',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parens() {
        let mut lexer = Lexer::new("()".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::RightParen);
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn test_japanese_brackets() {
        let mut lexer = Lexer::new("「」【】".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::RightParen);
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_mismatched_brackets() {
        let mut lexer = Lexer::new("「)".to_string());
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Float(3.14));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello world""#.to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("hello world".to_string()));
    }

    #[test]
    fn test_ascii_identifier() {
        let mut lexer = Lexer::new("foo bar_baz".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("bar_baz".to_string()));
    }

    #[test]
    fn test_japanese_identifier() {
        let mut lexer = Lexer::new("定義 二倍 数".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Identifier("定義".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Identifier("二倍".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("数".to_string()));
    }

    #[test]
    fn test_boolean_literals() {
        let mut lexer = Lexer::new("true false 真 偽".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Bool(true));
        assert_eq!(tokens[1].kind, TokenKind::Bool(false));
        assert_eq!(tokens[2].kind, TokenKind::Bool(true));
        assert_eq!(tokens[3].kind, TokenKind::Bool(false));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != < <= > >= ->".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::EqualEqual);
        assert_eq!(tokens[5].kind, TokenKind::BangEqual);
        assert_eq!(tokens[6].kind, TokenKind::Less);
        assert_eq!(tokens[7].kind, TokenKind::LessEqual);
        assert_eq!(tokens[8].kind, TokenKind::Greater);
        assert_eq!(tokens[9].kind, TokenKind::GreaterEqual);
        assert_eq!(tokens[10].kind, TokenKind::Arrow);
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("42 -- this is a comment\n43".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Integer(42));
        assert_eq!(tokens[1].kind, TokenKind::Integer(43));
    }

    #[test]
    fn test_sexp_tokenization() {
        let mut lexer = Lexer::new("(定義 二倍 (数 -> 数) (* 2 数))".to_string());
        let tokens = lexer.tokenize().unwrap();
        // ( 定義 二倍 ( 数 -> 数 ) ( * 2 数 ) ) EOF
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("定義".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Identifier("二倍".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::LeftParen);
        assert_eq!(tokens[4].kind, TokenKind::Identifier("数".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::Arrow);
        assert_eq!(tokens[6].kind, TokenKind::Identifier("数".to_string()));
        assert_eq!(tokens[7].kind, TokenKind::RightParen);
        assert_eq!(tokens[8].kind, TokenKind::LeftParen);
        assert_eq!(tokens[9].kind, TokenKind::Star);
        assert_eq!(tokens[10].kind, TokenKind::Integer(2));
        assert_eq!(tokens[11].kind, TokenKind::Identifier("数".to_string()));
        assert_eq!(tokens[12].kind, TokenKind::RightParen);
        assert_eq!(tokens[13].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_japanese_bracket_sexp() {
        let mut lexer = Lexer::new("「+ 1 2」".to_string());
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[2].kind, TokenKind::Integer(1));
        assert_eq!(tokens[3].kind, TokenKind::Integer(2));
        assert_eq!(tokens[4].kind, TokenKind::RightParen);
    }

    #[test]
    fn test_mixed_bracket_styles() {
        let mut lexer = Lexer::new("(定義 f 【数 -> 数】 「+ 1 数」)".to_string());
        let tokens = lexer.tokenize().unwrap();
        // Should tokenize without error — all bracket types are interchangeable
        // but must match: ( with ), 【 with 】, 「 with 」
        assert_eq!(tokens[0].kind, TokenKind::LeftParen);
        assert!(tokens.last().unwrap().kind == TokenKind::Eof);
    }
}
