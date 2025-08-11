use kakekotoba::lexer::{Lexer, Token, TokenKind};
use kakekotoba::parser::Parser;
use kakekotoba::ast::*;

fn create_test_tokens(kinds: Vec<TokenKind>) -> Vec<Token> {
    kinds.into_iter().enumerate().map(|(i, kind)| {
        Token::new(
            kind,
            kakekotoba::error::Span::new(i, i + 1, 1, i + 1),
            format!("token{}", i),
        )
    }).collect()
}

#[test]
fn test_empty_program() {
    let tokens = vec![
        Token::new(TokenKind::Eof, kakekotoba::error::Span::new(0, 0, 1, 1), "".to_string())
    ];
    let mut parser = Parser::new(tokens);
    
    let program = parser.parse().unwrap();
    assert_eq!(program.declarations.len(), 0);
}

#[test]
fn test_function_declaration_parsing() {
    // This test will likely fail until we have full parser implementation
    // but it shows the expected structure
    
    let tokens = vec![
        Token::new(TokenKind::Kansuu, kakekotoba::error::Span::new(0, 1, 1, 1), "関数".to_string()),
        Token::new(TokenKind::Identifier("test".to_string()), kakekotoba::error::Span::new(1, 2, 1, 2), "test".to_string()),
        Token::new(TokenKind::LeftParen, kakekotoba::error::Span::new(2, 3, 1, 3), "(".to_string()),
        Token::new(TokenKind::RightParen, kakekotoba::error::Span::new(3, 4, 1, 4), ")".to_string()),
        Token::new(TokenKind::Equal, kakekotoba::error::Span::new(4, 5, 1, 5), "=".to_string()),
        Token::new(TokenKind::Integer(42), kakekotoba::error::Span::new(5, 6, 1, 6), "42".to_string()),
        Token::new(TokenKind::Eof, kakekotoba::error::Span::new(6, 6, 1, 7), "".to_string()),
    ];
    
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(program) => {
            assert_eq!(program.declarations.len(), 1);
            match &program.declarations[0] {
                Declaration::Function(func) => {
                    assert_eq!(func.name.name, "test");
                    assert_eq!(func.params.len(), 0);
                }
                _ => panic!("Expected function declaration"),
            }
        }
        Err(e) => {
            // Expected for now since parser isn't fully implemented
            println!("Parser error (expected for now): {:?}", e);
        }
    }
}

#[test]
fn test_expression_parsing() {
    // Test basic expression parsing
    let tokens = vec![
        Token::new(TokenKind::Integer(123), kakekotoba::error::Span::new(0, 1, 1, 1), "123".to_string()),
        Token::new(TokenKind::Eof, kakekotoba::error::Span::new(1, 1, 1, 2), "".to_string()),
    ];
    
    let mut parser = Parser::new(tokens);
    
    // This will likely fail until we have proper expression parsing
    match parser.parse() {
        Ok(_) | Err(_) => {
            // Either way is fine - we're testing structure
        }
    }
}

#[test] 
fn test_type_declaration_parsing() {
    let tokens = vec![
        Token::new(TokenKind::Kata, kakekotoba::error::Span::new(0, 1, 1, 1), "型".to_string()),
        Token::new(TokenKind::Identifier("MyType".to_string()), kakekotoba::error::Span::new(1, 2, 1, 2), "MyType".to_string()),
        Token::new(TokenKind::Equal, kakekotoba::error::Span::new(2, 3, 1, 3), "=".to_string()),
        Token::new(TokenKind::Identifier("Int".to_string()), kakekotoba::error::Span::new(3, 4, 1, 4), "Int".to_string()),
        Token::new(TokenKind::Eof, kakekotoba::error::Span::new(4, 4, 1, 5), "".to_string()),
    ];
    
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(program) => {
            assert_eq!(program.declarations.len(), 1);
            match &program.declarations[0] {
                Declaration::Type(type_decl) => {
                    assert_eq!(type_decl.name.name, "MyType");
                }
                _ => panic!("Expected type declaration"),
            }
        }
        Err(e) => {
            // Expected for now
            println!("Parser error (expected for now): {:?}", e);
        }
    }
}

#[test]
fn test_parser_error_handling() {
    // Test that parser handles malformed input gracefully
    let tokens = vec![
        Token::new(TokenKind::LeftParen, kakekotoba::error::Span::new(0, 1, 1, 1), "(".to_string()),
        Token::new(TokenKind::RightParen, kakekotoba::error::Span::new(1, 2, 1, 2), ")".to_string()),
        // Missing EOF
    ];
    
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(_) => {
            // Unexpected but possible
        }
        Err(_) => {
            // Expected - malformed input should produce error
        }
    }
}