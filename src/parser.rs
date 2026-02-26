use crate::ast::*;
use crate::error::{Error, Result, Span};
use crate::layout::CodeLayout;
use crate::lexer::{Token, TokenKind};
use crate::spatial_ast::{SourceInfo, SpatialASTBuilder, SpatialProgram};
use crate::vertical::{Position2D, SpatialToken, WritingDirection};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    // Spatial parsing extensions
    spatial_tokens: Option<Vec<SpatialToken>>,
    source_text: Option<String>,
    writing_direction: WritingDirection,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            spatial_tokens: None,
            source_text: None,
            writing_direction: WritingDirection::VerticalTbRl,
        }
    }

    /// Create a parser with spatial token support
    pub fn with_spatial(
        tokens: Vec<Token>,
        spatial_tokens: Vec<SpatialToken>,
        source_text: String,
        writing_direction: WritingDirection,
    ) -> Self {
        Self {
            tokens,
            current: 0,
            spatial_tokens: Some(spatial_tokens),
            source_text: Some(source_text),
            writing_direction,
        }
    }

    pub fn parse(&mut self) -> Result<Program> {
        let declarations = self.parse_declarations()?;
        Ok(Program { declarations })
    }

    /// Parse with spatial information to create a spatial AST
    pub fn parse_spatial(&mut self) -> Result<SpatialProgram> {
        // First parse normally to get the regular AST
        let program = self.parse()?;

        // If we have spatial tokens, create spatial AST
        if let (Some(spatial_tokens), Some(source_text)) =
            (self.spatial_tokens.as_ref(), self.source_text.as_ref())
        {
            // Analyze layout from spatial tokens
            let layout = CodeLayout::analyze(spatial_tokens)?;

            // Create source info
            let source_info = SourceInfo::new(
                None, // file_path
                source_text.clone(),
                self.writing_direction,
            );

            // Build spatial AST
            let mut builder = SpatialASTBuilder::new().with_layout(layout);
            builder.build_program(program, source_info)
        } else {
            // Fallback: create minimal spatial program without layout info
            let layout = CodeLayout::new(self.writing_direction);
            let source_info = SourceInfo::new(None, self.source_code(), self.writing_direction);
            let mut builder = SpatialASTBuilder::new().with_layout(layout);
            builder.build_program(program, source_info)
        }
    }

    fn parse_declarations(&mut self) -> Result<Vec<Declaration>> {
        let mut declarations = Vec::new();

        while !self.is_at_end() && !self.check(&TokenKind::Eof) {
            match self.parse_declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => return Err(e),
            }
        }

        Ok(declarations)
    }

    fn parse_declaration(&mut self) -> Result<Declaration> {
        if self.match_token(&TokenKind::Kansuu) {
            self.parse_function_declaration()
        } else if self.match_token(&TokenKind::Kata) {
            self.parse_type_declaration()
        } else {
            Err(Error::Parser {
                src: self.source_code(),
                span: self.current_span().into(),
                expected: vec![
                    "function declaration".to_string(),
                    "type declaration".to_string(),
                ],
                found: self.peek().lexeme.clone(),
            })
        }
    }

    fn parse_function_declaration(&mut self) -> Result<Declaration> {
        let start_span = self.previous().span.clone();
        let name = self.parse_identifier()?;

        // Parse type parameters (optional)
        let type_params = if self.match_token(&TokenKind::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parse function parameters
        self.consume(&TokenKind::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_parameters()?;
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse return type (optional)
        let return_type = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse function body
        self.consume(&TokenKind::Equal, "Expected '=' before function body")?;
        let body = self.parse_expression()?;

        let end_span = self.previous().span.clone();
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Declaration::Function(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
        }))
    }

    fn parse_type_declaration(&mut self) -> Result<Declaration> {
        // Placeholder implementation
        let start_span = self.previous().span.clone();
        let name = self.parse_identifier()?;

        let type_params = if self.match_token(&TokenKind::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        self.consume(&TokenKind::Equal, "Expected '=' in type declaration")?;

        // Simple alias for now - extend for sum/product types
        let definition = TypeDefinition::Alias(self.parse_type()?);

        let end_span = self.previous().span.clone();
        let span = Span::new(
            start_span.start,
            end_span.end,
            start_span.line,
            start_span.column,
        );

        Ok(Declaration::Type(TypeDecl {
            name,
            type_params,
            definition,
            span,
        }))
    }

    fn parse_type_parameters(&mut self) -> Result<Vec<TypeParam>> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::Greater) {
            loop {
                let name = self.parse_identifier()?;
                let constraints = Vec::new(); // TODO: Parse constraints
                let span = name.span.clone();

                params.push(TypeParam {
                    name,
                    constraints,
                    span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenKind::Greater, "Expected '>' after type parameters")?;
        Ok(params)
    }

    fn parse_parameters(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RightParen) {
            loop {
                let name = self.parse_identifier()?;
                let param_type = if self.match_token(&TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                let span = name.span.clone();
                params.push(Parameter {
                    name,
                    param_type,
                    span,
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        // Placeholder - implement expression parsing
        if let Some(token) = self.advance().cloned() {
            match &token.kind {
                TokenKind::Integer(n) => Ok(Expression::Literal(Literal::Integer(*n))),
                TokenKind::String(s) => Ok(Expression::Literal(Literal::String(s.clone()))),
                TokenKind::Bool(b) => Ok(Expression::Literal(Literal::Bool(*b))),
                TokenKind::Identifier(name) => Ok(Expression::Identifier(Identifier {
                    name: name.clone(),
                    span: token.span.clone(),
                })),
                _ => Err(Error::Parser {
                    src: self.source_code(),
                    span: token.span.clone().into(),
                    expected: vec!["expression".to_string()],
                    found: token.lexeme.clone(),
                }),
            }
        } else {
            Err(Error::Parser {
                src: self.source_code(),
                span: self.current_span().into(),
                expected: vec!["expression".to_string()],
                found: "EOF".to_string(),
            })
        }
    }

    fn parse_type(&mut self) -> Result<crate::types::Type> {
        // Placeholder - implement type parsing
        if let Some(token) = self.advance().cloned() {
            match &token.kind {
                TokenKind::Identifier(name) => match name.as_str() {
                    "Int" => Ok(crate::types::Type::Int),
                    "String" => Ok(crate::types::Type::String),
                    "Bool" => Ok(crate::types::Type::Bool),
                    _ => Ok(crate::types::Type::Constructor {
                        name: name.clone(),
                        args: Vec::new(),
                    }),
                },
                _ => Err(Error::Parser {
                    src: self.source_code(),
                    span: token.span.clone().into(),
                    expected: vec!["type".to_string()],
                    found: token.lexeme.clone(),
                }),
            }
        } else {
            Err(Error::Parser {
                src: self.source_code(),
                span: self.current_span().into(),
                expected: vec!["type".to_string()],
                found: "EOF".to_string(),
            })
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        if let Some(token) = self.advance().cloned() {
            match &token.kind {
                TokenKind::Identifier(name) => Ok(Identifier {
                    name: name.clone(),
                    span: token.span.clone(),
                }),
                _ => Err(Error::Parser {
                    src: self.source_code(),
                    span: token.span.clone().into(),
                    expected: vec!["identifier".to_string()],
                    found: token.lexeme.clone(),
                }),
            }
        } else {
            Err(Error::Parser {
                src: self.source_code(),
                span: self.current_span().into(),
                expected: vec!["identifier".to_string()],
                found: "EOF".to_string(),
            })
        }
    }

    // Helper methods
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous_option()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().kind, TokenKind::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current.min(self.tokens.len().saturating_sub(1))]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn previous_option(&self) -> Option<&Token> {
        if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        }
    }

    fn consume(&mut self, kind: &TokenKind, _message: &str) -> Result<&Token> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            Err(Error::Parser {
                src: self.source_code(),
                span: self.current_span().into(),
                expected: vec![format!("{:?}", kind)],
                found: self.peek().lexeme.clone(),
            })
        }
    }

    fn source_code(&self) -> String {
        // TODO: Store original source code
        self.tokens
            .iter()
            .map(|t| t.lexeme.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn current_span(&self) -> Span {
        if self.is_at_end() && !self.tokens.is_empty() {
            let last = &self.tokens[self.tokens.len() - 1];
            Span::new(
                last.span.end,
                last.span.end,
                last.span.line,
                last.span.column,
            )
        } else {
            self.peek().span.clone()
        }
    }

    /// Get the spatial token corresponding to the current regular token
    #[allow(dead_code)]
    fn current_spatial_token(&self) -> Option<&SpatialToken> {
        if let Some(spatial_tokens) = &self.spatial_tokens {
            // Simple approach: match by content and approximate position
            // In a production implementation, you'd maintain better correspondence
            let current_token = self.peek();
            spatial_tokens
                .iter()
                .find(|st| st.content == current_token.lexeme)
        } else {
            None
        }
    }

    /// Get 2D position for current parsing location
    #[allow(dead_code)]
    fn current_position_2d(&self) -> Position2D {
        if let Some(spatial_token) = self.current_spatial_token() {
            spatial_token.span.start
        } else {
            // Fallback: convert regular span to 2D position
            let span = self.current_span();
            Position2D::new(
                span.column.saturating_sub(1),
                span.line.saturating_sub(1),
                span.start,
            )
        }
    }

    /// Check if we're parsing in vertical mode
    #[allow(dead_code)]
    fn is_vertical_mode(&self) -> bool {
        matches!(
            self.writing_direction,
            WritingDirection::VerticalTbRl | WritingDirection::VerticalTbLr
        )
    }

    /// Get layout-aware error context for spatial parsing
    #[allow(dead_code)]
    fn spatial_error_context(&self, expected: Vec<String>, found: String) -> Error {
        let _position = self.current_position_2d();

        Error::Parser {
            src: self
                .source_text
                .as_ref()
                .unwrap_or(&self.source_code())
                .clone(),
            span: self.current_span().into(),
            expected,
            found,
        }
    }
}
