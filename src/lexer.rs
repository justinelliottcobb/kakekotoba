use crate::error::{Error, Result, Span};
use unicode_segmentation::UnicodeSegmentation;
use serde::{Deserialize, Serialize};

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
    Arrow, // ->
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
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            current_char: input.chars().next(),
            input,
            position: 0,
            line: 1,
            column: 1,
        };
        lexer.normalize_input();
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
            },
            Some(')') => {
                self.advance();
                Ok(Some(Token::new(
                    TokenKind::RightParen,
                    Span::new(start_pos, self.position, start_line, start_column),
                    ")".to_string(),
                )))
            },
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
        self.input.grapheme_indices(true)
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