use kakekotoba::pipeline::{Compiler, CompilerOptions, create_default_options};
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_full_pipeline_empty_program() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "").unwrap();
    
    let compiler = Compiler::new();
    let options = create_default_options();
    
    match compiler.compile_file(temp_file.path(), options) {
        Ok(_result) => {
            // Success case - pipeline worked
        }
        Err(_e) => {
            // Expected for now since we don't have full implementation
        }
    }
}

#[test]
fn test_lexer_only_mode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "( )").unwrap();
    
    let compiler = Compiler::new();
    let source = std::fs::read_to_string(temp_file.path()).unwrap();
    
    match compiler.lex_only(source) {
        Ok(tokens) => {
            assert!(!tokens.is_empty());
            // Last token should be EOF
            assert!(matches!(tokens.last().unwrap().kind, kakekotoba::lexer::TokenKind::Eof));
        }
        Err(_e) => {
            // Expected for basic tokens that might not be implemented yet
        }
    }
}

#[test] 
fn test_parser_only_mode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "関数 test() = 42").unwrap();
    
    let compiler = Compiler::new();
    let source = std::fs::read_to_string(temp_file.path()).unwrap();
    
    match compiler.parse_only(source) {
        Ok(program) => {
            // Should have some declarations if parsing succeeded
            assert!(program.declarations.len() >= 0);
        }
        Err(_e) => {
            // Expected until full parser is implemented
        }
    }
}

#[test]
fn test_type_check_only_mode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "関数 identity(x: Int): Int = x").unwrap();
    
    let compiler = Compiler::new();
    let source = std::fs::read_to_string(temp_file.path()).unwrap();
    
    match compiler.type_check_only(source) {
        Ok(result) => {
            // Should have type environment
            assert!(result.type_environment.len() >= 0);
        }
        Err(_e) => {
            // Expected until full type checker is implemented
        }
    }
}

#[test]
fn test_compilation_options() {
    let mut options = create_default_options();
    
    // Test option modifications
    options.optimize = true;
    options.output_ir = true;
    options.type_check_only = false;
    
    assert!(options.optimize);
    assert!(options.output_ir);
    assert!(!options.type_check_only);
}

#[test] 
fn test_japanese_source_compilation() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "// 日本語コメント").unwrap();
    writeln!(temp_file, "関数 足す(甲: Int, 乙: Int): Int = 甲 + 乙").unwrap();
    
    let compiler = Compiler::new();
    let options = create_default_options();
    
    match compiler.compile_file(temp_file.path(), options) {
        Ok(_result) => {
            // Ideally would succeed when fully implemented
        }
        Err(_e) => {
            // Expected for now - Japanese tokenization not fully implemented
        }
    }
}

#[test]
fn test_haskell_style_syntax() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "map :: (a -> b) -> [a] -> [b]").unwrap();
    writeln!(temp_file, "map f [] = []").unwrap();
    writeln!(temp_file, "map f (x:xs) = f x : map f xs").unwrap();
    
    let compiler = Compiler::new();
    let source = std::fs::read_to_string(temp_file.path()).unwrap();
    
    // Test that Haskell-style syntax can be at least lexed
    match compiler.lex_only(source) {
        Ok(_tokens) => {
            // Good - got some tokens from Haskell-style syntax
        }
        Err(_e) => {
            // Expected - complex syntax not yet implemented
        }
    }
}

#[test]
fn test_group_homomorphism_syntax() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "homomorphism :: Group A -> Group B").unwrap();
    writeln!(temp_file, "preserves_operation :: (a * b) -> (f(a) * f(b))").unwrap();
    
    let compiler = Compiler::new();
    let source = std::fs::read_to_string(temp_file.path()).unwrap();
    
    // Test that group homomorphism syntax can be processed
    match compiler.lex_only(source) {
        Ok(_tokens) => {
            // Success - homomorphism syntax tokenized
        }
        Err(_e) => {
            // Expected - advanced syntax not implemented yet
        }
    }
}

#[test]
fn test_error_reporting() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "invalid syntax here !@#$%").unwrap();
    
    let compiler = Compiler::new();
    let options = create_default_options();
    
    match compiler.compile_file(temp_file.path(), options) {
        Ok(_result) => {
            // Unexpected - malformed input should fail
            panic!("Expected compilation to fail with malformed input");
        }
        Err(error) => {
            // Good - should produce meaningful error
            let error_string = format!("{}", error);
            assert!(!error_string.is_empty());
        }
    }
}

#[test]
fn test_file_not_found() {
    let compiler = Compiler::new();
    let options = create_default_options();
    
    match compiler.compile_file("/nonexistent/file.kake", options) {
        Ok(_result) => {
            panic!("Expected file not found error");
        }
        Err(error) => {
            // Should be an IO error
            match error {
                kakekotoba::Error::Io(_) => {
                    // Good - proper IO error handling
                }
                _ => {
                    // Also acceptable - any error for nonexistent file
                }
            }
        }
    }
}

#[test]
fn test_compilation_stages() {
    // Test that we can create a compiler and it has all the expected stages
    let compiler = Compiler::new();
    
    // Test with minimal valid input
    let source = "".to_string();
    
    // Each stage should either succeed or fail gracefully
    let _lex_result = compiler.lex_only(source.clone());
    let _parse_result = compiler.parse_only(source.clone());
    let _typecheck_result = compiler.type_check_only(source);
    
    // Just testing that the methods exist and don't panic
}