//! S-expression parser for kakekotoba
//!
//! Parses S-expression syntax into the existing AST types.
//! This is the primary syntax path for kakekotoba's homoiconic design.

use crate::ast::*;
use crate::error::{Error, Result, Span};
use crate::lexer::{Token, TokenKind};
use crate::types::Type;

/// An S-expression: either an atom or a list of S-expressions
#[derive(Debug, Clone)]
enum SExpr {
    Atom(Token),
    List(Vec<SExpr>, Span),
}

impl SExpr {
    fn span(&self) -> Span {
        match self {
            SExpr::Atom(token) => token.span.clone(),
            SExpr::List(_, span) => span.clone(),
        }
    }
}

pub struct SExpParser {
    tokens: Vec<Token>,
    current: usize,
    source: String,
}

impl SExpParser {
    pub fn new(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            current: 0,
            source,
        }
    }

    /// Parse a complete program (sequence of top-level S-expressions)
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut declarations = Vec::new();

        while !self.is_at_end() {
            let sexpr = self.parse_sexpr()?;
            match self.sexpr_to_declaration(&sexpr) {
                Ok(decl) => declarations.push(decl),
                Err(_) => {
                    // If it's not a declaration, treat as a top-level expression
                    // wrapped in a dummy function for evaluation
                    let expr = self.sexpr_to_expression(&sexpr)?;
                    declarations.push(Declaration::Function(FunctionDecl {
                        name: Identifier {
                            name: "_main".to_string(),
                            span: sexpr.span(),
                        },
                        type_params: Vec::new(),
                        params: Vec::new(),
                        return_type: None,
                        body: expr,
                        span: sexpr.span(),
                    }));
                }
            }
        }

        Ok(Program { declarations })
    }

    /// Parse a single S-expression (for REPL use)
    pub fn parse_single(&mut self) -> Result<SExprResult> {
        let sexpr = self.parse_sexpr()?;

        // Try as declaration first
        if let Ok(decl) = self.sexpr_to_declaration(&sexpr) {
            return Ok(SExprResult::Declaration(Box::new(decl)));
        }

        // Otherwise, parse as expression
        let expr = self.sexpr_to_expression(&sexpr)?;
        Ok(SExprResult::Expression(expr))
    }

    // ========================================================================
    // S-expression structure parsing
    // ========================================================================

    fn parse_sexpr(&mut self) -> Result<SExpr> {
        if self.is_at_end() {
            return Err(self.error("Unexpected end of input"));
        }

        if self.check(&TokenKind::LeftParen) {
            self.parse_list()
        } else {
            self.parse_atom()
        }
    }

    fn parse_list(&mut self) -> Result<SExpr> {
        let open = self.consume_paren_open()?;
        let start_span = open.span.clone();
        let mut elements = Vec::new();

        while !self.is_at_end() && !self.check(&TokenKind::RightParen) {
            elements.push(self.parse_sexpr()?);
        }

        let close = self.consume_paren_close()?;
        let span = Span::new(
            start_span.start,
            close.span.end,
            start_span.line,
            start_span.column,
        );

        Ok(SExpr::List(elements, span))
    }

    fn parse_atom(&mut self) -> Result<SExpr> {
        let token = self.advance_token()?;
        Ok(SExpr::Atom(token))
    }

    // ========================================================================
    // S-expression → AST conversion
    // ========================================================================

    fn sexpr_to_declaration(&self, sexpr: &SExpr) -> Result<Declaration> {
        match sexpr {
            SExpr::List(elements, span) if !elements.is_empty() => {
                if let SExpr::Atom(token) = &elements[0] {
                    match self.identifier_name(token) {
                        Some(name) if name == "定義" || name == "define" => {
                            return self.parse_define(elements, span);
                        }
                        _ => {}
                    }
                }
                Err(self.error_at(span, "Expected a declaration (定義)"))
            }
            _ => Err(self.error_at(&sexpr.span(), "Expected a declaration")),
        }
    }

    fn parse_define(&self, elements: &[SExpr], span: &Span) -> Result<Declaration> {
        // (定義 name type-sig body)
        // (定義 name body)   — type inferred
        if elements.len() < 3 {
            return Err(self.error_at(span, "定義 requires at least a name and body"));
        }

        let name = self.sexpr_to_identifier(&elements[1])?;

        let (return_type, body_idx) = if elements.len() >= 4 {
            // Try to parse elements[2] as a type signature
            match self.try_parse_type_sig(&elements[2]) {
                Some(ty) => (Some(ty), 3),
                None => (None, 2),
            }
        } else {
            (None, 2)
        };

        let body = self.sexpr_to_expression(&elements[body_idx])?;

        // Don't create dummy params from the type signature — the interpreter
        // extracts actual parameter names from free variables in the body.
        let params = Vec::new();

        Ok(Declaration::Function(FunctionDecl {
            name,
            type_params: Vec::new(),
            params,
            return_type,
            body,
            span: span.clone(),
        }))
    }

    fn sexpr_to_expression(&self, sexpr: &SExpr) -> Result<Expression> {
        match sexpr {
            SExpr::Atom(token) => self.atom_to_expression(token),
            SExpr::List(elements, span) => {
                if elements.is_empty() {
                    return Ok(Expression::Literal(Literal::Unit));
                }
                self.list_to_expression(elements, span)
            }
        }
    }

    fn atom_to_expression(&self, token: &Token) -> Result<Expression> {
        match &token.kind {
            TokenKind::Integer(n) => Ok(Expression::Literal(Literal::Integer(*n))),
            TokenKind::Float(f) => Ok(Expression::Literal(Literal::Float(*f))),
            TokenKind::String(s) => Ok(Expression::Literal(Literal::String(s.clone()))),
            TokenKind::Bool(b) => Ok(Expression::Literal(Literal::Bool(*b))),
            TokenKind::Identifier(name) => match name.as_str() {
                "無" | "none" => Ok(Expression::Literal(Literal::Unit)),
                "単位" | "unit" => Ok(Expression::Literal(Literal::Unit)),
                _ => Ok(Expression::Identifier(Identifier {
                    name: name.clone(),
                    span: token.span.clone(),
                })),
            },
            // Operators as identifiers in expression context
            TokenKind::Plus => Ok(Expression::Identifier(Identifier {
                name: "+".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::Minus => Ok(Expression::Identifier(Identifier {
                name: "-".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::Star => Ok(Expression::Identifier(Identifier {
                name: "*".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::Slash => Ok(Expression::Identifier(Identifier {
                name: "/".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::EqualEqual => Ok(Expression::Identifier(Identifier {
                name: "==".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::BangEqual => Ok(Expression::Identifier(Identifier {
                name: "!=".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::Less => Ok(Expression::Identifier(Identifier {
                name: "<".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::LessEqual => Ok(Expression::Identifier(Identifier {
                name: "<=".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::Greater => Ok(Expression::Identifier(Identifier {
                name: ">".to_string(),
                span: token.span.clone(),
            })),
            TokenKind::GreaterEqual => Ok(Expression::Identifier(Identifier {
                name: ">=".to_string(),
                span: token.span.clone(),
            })),
            _ => Err(self.error_at(
                &token.span,
                &format!("Unexpected token in expression: {:?}", token.kind),
            )),
        }
    }

    fn list_to_expression(&self, elements: &[SExpr], span: &Span) -> Result<Expression> {
        // Check head for special forms
        if let SExpr::Atom(token) = &elements[0] {
            if let Some(name) = self.identifier_name(token) {
                match name {
                    "もし" | "if" => return self.parse_if(elements, span),
                    "場合" | "match" => return self.parse_match(elements, span),
                    "匿名" | "lambda" | "fn" => return self.parse_lambda(elements, span),
                    "束縛" | "let" => return self.parse_let(elements, span),
                    _ => {}
                }
            }

            // Check for binary operators: (op a b)
            if let Some(op) = self.token_to_binary_op(token) {
                if elements.len() == 3 {
                    let left = self.sexpr_to_expression(&elements[1])?;
                    let right = self.sexpr_to_expression(&elements[2])?;
                    return Ok(Expression::Binary(BinaryExpr {
                        left: Box::new(left),
                        operator: op,
                        right: Box::new(right),
                        span: span.clone(),
                    }));
                }
            }

            // Check for unary operators: (- x) with single arg
            if elements.len() == 2 {
                if let Some(op) = self.token_to_unary_op(token) {
                    let operand = self.sexpr_to_expression(&elements[1])?;
                    return Ok(Expression::Unary(UnaryExpr {
                        operator: op,
                        operand: Box::new(operand),
                        span: span.clone(),
                    }));
                }
            }
        }

        // General case: function application (func arg1 arg2 ...)
        let func = self.sexpr_to_expression(&elements[0])?;
        let args: Result<Vec<Expression>> = elements[1..]
            .iter()
            .map(|e| self.sexpr_to_expression(e))
            .collect();

        Ok(Expression::Application(Application {
            function: Box::new(func),
            arguments: args?,
            span: span.clone(),
        }))
    }

    fn parse_if(&self, elements: &[SExpr], span: &Span) -> Result<Expression> {
        // (もし condition then-expr else-expr)
        if elements.len() < 3 || elements.len() > 4 {
            return Err(self.error_at(
                span,
                "もし requires 2 or 3 arguments: (もし condition then [else])",
            ));
        }

        let condition = self.sexpr_to_expression(&elements[1])?;
        let then_branch = self.sexpr_to_expression(&elements[2])?;
        let else_branch = if elements.len() == 4 {
            Some(Box::new(self.sexpr_to_expression(&elements[3])?))
        } else {
            None
        };

        Ok(Expression::If(IfExpr {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            span: span.clone(),
        }))
    }

    fn parse_match(&self, elements: &[SExpr], span: &Span) -> Result<Expression> {
        // (場合 scrutinee (pattern -> body) (pattern -> body) ...)
        if elements.len() < 3 {
            return Err(self.error_at(span, "場合 requires a scrutinee and at least one arm"));
        }

        let scrutinee = self.sexpr_to_expression(&elements[1])?;

        let mut arms = Vec::new();
        for arm_sexpr in &elements[2..] {
            arms.push(self.parse_match_arm(arm_sexpr)?);
        }

        Ok(Expression::Match(MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            span: span.clone(),
        }))
    }

    fn parse_match_arm(&self, sexpr: &SExpr) -> Result<MatchArm> {
        // (pattern -> body)
        match sexpr {
            SExpr::List(elements, span) => {
                // Find the -> separator
                let arrow_idx = elements.iter().position(|e| {
                    if let SExpr::Atom(token) = e {
                        matches!(token.kind, TokenKind::Arrow)
                    } else {
                        false
                    }
                });

                match arrow_idx {
                    Some(idx) if idx > 0 && idx < elements.len() - 1 => {
                        let pattern = self.sexpr_to_pattern(&elements[0])?;
                        let body = self.sexpr_to_expression(&elements[idx + 1])?;

                        Ok(MatchArm {
                            pattern,
                            guard: None,
                            body,
                            span: span.clone(),
                        })
                    }
                    _ => Err(self.error_at(span, "Match arm must be (pattern -> body)")),
                }
            }
            _ => Err(self.error_at(&sexpr.span(), "Match arm must be a list (pattern -> body)")),
        }
    }

    fn parse_lambda(&self, elements: &[SExpr], span: &Span) -> Result<Expression> {
        // (匿名 (param1 param2 ...) body)
        if elements.len() != 3 {
            return Err(self.error_at(
                span,
                "匿名 requires parameters and body: (匿名 (params) body)",
            ));
        }

        let params = match &elements[1] {
            SExpr::List(param_elems, _) => {
                let mut params = Vec::new();
                for p in param_elems {
                    let id = self.sexpr_to_identifier(p)?;
                    let param_span = id.span.clone();
                    params.push(Parameter {
                        name: id,
                        param_type: None,
                        span: param_span,
                    });
                }
                params
            }
            SExpr::Atom(_) => {
                // Single param without parens
                let id = self.sexpr_to_identifier(&elements[1])?;
                let param_span = id.span.clone();
                vec![Parameter {
                    name: id,
                    param_type: None,
                    span: param_span,
                }]
            }
        };

        let body = self.sexpr_to_expression(&elements[2])?;

        Ok(Expression::Lambda(Lambda {
            params,
            body: Box::new(body),
            span: span.clone(),
        }))
    }

    fn parse_let(&self, elements: &[SExpr], span: &Span) -> Result<Expression> {
        // (束縛 ((name value) (name value) ...) body)
        if elements.len() != 3 {
            return Err(self.error_at(
                span,
                "束縛 requires bindings and body: (束縛 ((name value) ...) body)",
            ));
        }

        let bindings = match &elements[1] {
            SExpr::List(binding_elems, _) => {
                let mut bindings = Vec::new();
                for b in binding_elems {
                    match b {
                        SExpr::List(pair, pair_span) if pair.len() == 2 => {
                            let pattern = self.sexpr_to_pattern(&pair[0])?;
                            let value = self.sexpr_to_expression(&pair[1])?;
                            bindings.push(Binding {
                                pattern,
                                value,
                                span: pair_span.clone(),
                            });
                        }
                        _ => return Err(self.error_at(&b.span(), "Binding must be (name value)")),
                    }
                }
                bindings
            }
            _ => return Err(self.error_at(&elements[1].span(), "Expected binding list")),
        };

        let body = self.sexpr_to_expression(&elements[2])?;

        Ok(Expression::Let(LetExpr {
            bindings,
            body: Box::new(body),
            span: span.clone(),
        }))
    }

    // ========================================================================
    // Pattern parsing
    // ========================================================================

    fn sexpr_to_pattern(&self, sexpr: &SExpr) -> Result<Pattern> {
        match sexpr {
            SExpr::Atom(token) => match &token.kind {
                TokenKind::Integer(n) => Ok(Pattern::Literal(Literal::Integer(*n))),
                TokenKind::Float(f) => Ok(Pattern::Literal(Literal::Float(*f))),
                TokenKind::String(s) => Ok(Pattern::Literal(Literal::String(s.clone()))),
                TokenKind::Bool(b) => Ok(Pattern::Literal(Literal::Bool(*b))),
                TokenKind::Identifier(name) => match name.as_str() {
                    "_" => Ok(Pattern::Wildcard),
                    _ => Ok(Pattern::Identifier(Identifier {
                        name: name.clone(),
                        span: token.span.clone(),
                    })),
                },
                _ => Err(self.error_at(&token.span, "Unexpected token in pattern")),
            },
            SExpr::List(elements, _) if elements.is_empty() => Ok(Pattern::Literal(Literal::Unit)),
            SExpr::List(elements, _) => {
                // Constructor pattern: (Name p1 p2 ...)
                let name = self.sexpr_to_identifier(&elements[0])?;
                let sub_patterns: Result<Vec<Pattern>> = elements[1..]
                    .iter()
                    .map(|e| self.sexpr_to_pattern(e))
                    .collect();
                Ok(Pattern::Constructor {
                    name,
                    patterns: sub_patterns?,
                })
            }
        }
    }

    // ========================================================================
    // Type parsing
    // ========================================================================

    fn try_parse_type_sig(&self, sexpr: &SExpr) -> Option<Type> {
        self.sexpr_to_type(sexpr).ok()
    }

    fn sexpr_to_type(&self, sexpr: &SExpr) -> Result<Type> {
        match sexpr {
            SExpr::Atom(token) => self.atom_to_type(token),
            SExpr::List(elements, span) => {
                // (type -> type) for function types
                // Find -> separator
                let arrow_idx = elements.iter().position(|e| {
                    if let SExpr::Atom(token) = e {
                        matches!(token.kind, TokenKind::Arrow)
                    } else {
                        false
                    }
                });

                if let Some(idx) = arrow_idx {
                    // Everything before -> is param types, everything after is return type
                    let param_types: Result<Vec<Type>> = elements[..idx]
                        .iter()
                        .map(|e| self.sexpr_to_type(e))
                        .collect();

                    // For simplicity, the return type is the single element after ->
                    if idx + 1 < elements.len() {
                        let return_type = self.sexpr_to_type(&elements[idx + 1])?;
                        Ok(Type::Function {
                            params: param_types?,
                            return_type: Box::new(return_type),
                        })
                    } else {
                        Err(self.error_at(span, "Expected return type after ->"))
                    }
                } else {
                    // Type constructor application: (List Int) etc.
                    if elements.is_empty() {
                        Ok(Type::Unit)
                    } else {
                        let name = self.type_name_from_sexpr(&elements[0])?;
                        let args: Result<Vec<Type>> = elements[1..]
                            .iter()
                            .map(|e| self.sexpr_to_type(e))
                            .collect();
                        Ok(Type::Constructor { name, args: args? })
                    }
                }
            }
        }
    }

    fn atom_to_type(&self, token: &Token) -> Result<Type> {
        match &token.kind {
            TokenKind::Identifier(name) => {
                match name.as_str() {
                    // Japanese type names
                    "数" | "整数" | "Int" => Ok(Type::Int),
                    "浮動" | "Float" => Ok(Type::Float),
                    "文字列" | "String" => Ok(Type::String),
                    "真偽" | "Bool" => Ok(Type::Bool),
                    "単位" | "Unit" => Ok(Type::Unit),
                    // Constructor type
                    _ => Ok(Type::Constructor {
                        name: name.clone(),
                        args: Vec::new(),
                    }),
                }
            }
            _ => Err(self.error_at(&token.span, "Expected type name")),
        }
    }

    fn type_name_from_sexpr(&self, sexpr: &SExpr) -> Result<String> {
        match sexpr {
            SExpr::Atom(token) => match &token.kind {
                TokenKind::Identifier(name) => Ok(name.clone()),
                _ => Err(self.error_at(&token.span, "Expected type name")),
            },
            _ => Err(self.error_at(&sexpr.span(), "Expected type name")),
        }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn sexpr_to_identifier(&self, sexpr: &SExpr) -> Result<Identifier> {
        match sexpr {
            SExpr::Atom(token) => match &token.kind {
                TokenKind::Identifier(name) => Ok(Identifier {
                    name: name.clone(),
                    span: token.span.clone(),
                }),
                _ => Err(self.error_at(&token.span, "Expected identifier")),
            },
            _ => Err(self.error_at(&sexpr.span(), "Expected identifier")),
        }
    }

    fn identifier_name<'a>(&self, token: &'a Token) -> Option<&'a str> {
        match &token.kind {
            TokenKind::Identifier(name) => Some(name.as_str()),
            _ => None,
        }
    }

    fn token_to_binary_op(&self, token: &Token) -> Option<BinaryOp> {
        match &token.kind {
            TokenKind::Plus => Some(BinaryOp::Add),
            TokenKind::Minus => Some(BinaryOp::Sub),
            TokenKind::Star => Some(BinaryOp::Mul),
            TokenKind::Slash => Some(BinaryOp::Div),
            TokenKind::EqualEqual => Some(BinaryOp::Eq),
            TokenKind::BangEqual => Some(BinaryOp::Ne),
            TokenKind::Less => Some(BinaryOp::Lt),
            TokenKind::LessEqual => Some(BinaryOp::Le),
            TokenKind::Greater => Some(BinaryOp::Gt),
            TokenKind::GreaterEqual => Some(BinaryOp::Ge),
            TokenKind::Identifier(name) => match name.as_str() {
                "%" | "mod" => Some(BinaryOp::Mod),
                "&&" | "and" => Some(BinaryOp::And),
                "||" | "or" => Some(BinaryOp::Or),
                _ => None,
            },
            _ => None,
        }
    }

    fn token_to_unary_op(&self, token: &Token) -> Option<UnaryOp> {
        match &token.kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Identifier(name) if name == "not" || name == "否定" => Some(UnaryOp::Not),
            _ => None,
        }
    }

    // ========================================================================
    // Token navigation
    // ========================================================================

    fn advance_token(&mut self) -> Result<Token> {
        if self.is_at_end() {
            return Err(self.error("Unexpected end of input"));
        }
        let token = self.tokens[self.current].clone();
        self.current += 1;
        Ok(token)
    }

    fn consume_paren_open(&mut self) -> Result<Token> {
        if self.check(&TokenKind::LeftParen) {
            self.advance_token()
        } else {
            Err(self.error("Expected '(' or '「' or '【'"))
        }
    }

    fn consume_paren_close(&mut self) -> Result<Token> {
        if self.check(&TokenKind::RightParen) {
            self.advance_token()
        } else {
            Err(self.error("Expected ')' or '」' or '】'"))
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.tokens[self.current].kind) == std::mem::discriminant(kind)
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
            || matches!(self.tokens[self.current].kind, TokenKind::Eof)
    }

    fn error(&self, msg: &str) -> Error {
        let span = if self.current < self.tokens.len() {
            self.tokens[self.current].span.clone()
        } else if !self.tokens.is_empty() {
            self.tokens.last().unwrap().span.clone()
        } else {
            Span::new(0, 0, 1, 1)
        };

        Error::Parser {
            src: self.source.clone(),
            span: span.into(),
            expected: vec![msg.to_string()],
            found: if self.current < self.tokens.len() {
                self.tokens[self.current].lexeme.clone()
            } else {
                "EOF".to_string()
            },
        }
    }

    fn error_at(&self, span: &Span, msg: &str) -> Error {
        Error::Parser {
            src: self.source.clone(),
            span: span.clone().into(),
            expected: vec![msg.to_string()],
            found: String::new(),
        }
    }
}

/// Result of parsing a single S-expression (for REPL)
#[derive(Debug)]
pub enum SExprResult {
    Declaration(Box<Declaration>),
    Expression(Expression),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_expr(input: &str) -> Expression {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, input.to_string());
        match parser.parse_single().unwrap() {
            SExprResult::Expression(expr) => expr,
            SExprResult::Declaration(_) => panic!("Expected expression, got declaration"),
        }
    }

    fn parse_decl(input: &str) -> Declaration {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, input.to_string());
        match parser.parse_single().unwrap() {
            SExprResult::Declaration(decl) => *decl,
            SExprResult::Expression(_) => panic!("Expected declaration, got expression"),
        }
    }

    #[test]
    fn test_integer_literal() {
        let expr = parse_expr("42");
        assert!(matches!(expr, Expression::Literal(Literal::Integer(42))));
    }

    #[test]
    fn test_string_literal() {
        let expr = parse_expr(r#""hello""#);
        assert!(matches!(expr, Expression::Literal(Literal::String(ref s)) if s == "hello"));
    }

    #[test]
    fn test_boolean_literals() {
        assert!(matches!(
            parse_expr("真"),
            Expression::Literal(Literal::Bool(true))
        ));
        assert!(matches!(
            parse_expr("偽"),
            Expression::Literal(Literal::Bool(false))
        ));
        assert!(matches!(
            parse_expr("true"),
            Expression::Literal(Literal::Bool(true))
        ));
    }

    #[test]
    fn test_identifier() {
        let expr = parse_expr("foo");
        assert!(matches!(expr, Expression::Identifier(ref id) if id.name == "foo"));
    }

    #[test]
    fn test_japanese_identifier() {
        let expr = parse_expr("二倍");
        assert!(matches!(expr, Expression::Identifier(ref id) if id.name == "二倍"));
    }

    #[test]
    fn test_binary_op() {
        let expr = parse_expr("(+ 1 2)");
        match expr {
            Expression::Binary(bin) => {
                assert!(matches!(bin.operator, BinaryOp::Add));
                assert!(matches!(
                    *bin.left,
                    Expression::Literal(Literal::Integer(1))
                ));
                assert!(matches!(
                    *bin.right,
                    Expression::Literal(Literal::Integer(2))
                ));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_nested_binary() {
        let expr = parse_expr("(* 2 (+ 1 3))");
        match expr {
            Expression::Binary(bin) => {
                assert!(matches!(bin.operator, BinaryOp::Mul));
                assert!(matches!(*bin.right, Expression::Binary(_)));
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_function_application() {
        let expr = parse_expr("(二倍 21)");
        match expr {
            Expression::Application(app) => {
                assert!(
                    matches!(*app.function, Expression::Identifier(ref id) if id.name == "二倍")
                );
                assert_eq!(app.arguments.len(), 1);
            }
            _ => panic!("Expected application"),
        }
    }

    #[test]
    fn test_if_expression() {
        let expr = parse_expr("(もし 真 1 2)");
        assert!(matches!(expr, Expression::If(_)));
    }

    #[test]
    fn test_define() {
        let decl = parse_decl("(定義 二倍 (数 -> 数) (* 2 x))");
        match decl {
            Declaration::Function(f) => {
                assert_eq!(f.name.name, "二倍");
                assert!(f.return_type.is_some());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_define_without_type() {
        let decl = parse_decl("(定義 inc (+ 1 x))");
        match decl {
            Declaration::Function(f) => {
                assert_eq!(f.name.name, "inc");
                assert!(f.return_type.is_none());
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_match_expression() {
        let expr = parse_expr("(場合 x (0 -> 1) (n -> (* n (f (- n 1)))))");
        match expr {
            Expression::Match(m) => {
                assert_eq!(m.arms.len(), 2);
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_lambda() {
        let expr = parse_expr("(匿名 (x y) (+ x y))");
        match expr {
            Expression::Lambda(lam) => {
                assert_eq!(lam.params.len(), 2);
            }
            _ => panic!("Expected lambda"),
        }
    }

    #[test]
    fn test_let_binding() {
        let expr = parse_expr("(束縛 ((x 10) (y 20)) (+ x y))");
        match expr {
            Expression::Let(l) => {
                assert_eq!(l.bindings.len(), 2);
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_japanese_brackets() {
        let expr = parse_expr("「+ 1 2」");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_lenticular_brackets() {
        let expr = parse_expr("【+ 1 2】");
        assert!(matches!(expr, Expression::Binary(_)));
    }

    #[test]
    fn test_unit_literal() {
        let expr = parse_expr("()");
        assert!(matches!(expr, Expression::Literal(Literal::Unit)));
    }

    #[test]
    fn test_program_parse() {
        let input = "(定義 二倍 (数 -> 数) (* 2 x)) (定義 三倍 (数 -> 数) (* 3 x))";
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = SExpParser::new(tokens, input.to_string());
        let program = parser.parse_program().unwrap();
        assert_eq!(program.declarations.len(), 2);
    }
}
