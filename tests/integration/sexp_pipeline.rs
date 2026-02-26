//! End-to-end integration tests for the S-expression pipeline
//!
//! Tests the full flow: source text → lex → sexp parse → interpret → result

use kakekotoba::interpreter::{Interpreter, Value};
use kakekotoba::lexer::Lexer;
use kakekotoba::pipeline::Compiler;
use kakekotoba::sexp_parser::SExpParser;

fn run(input: &str) -> Value {
    let compiler = Compiler::new();
    compiler.run_source(input.to_string()).unwrap()
}

fn run_with_output(input: &str) -> (Value, Vec<String>) {
    let mut lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = SExpParser::new(tokens, input.to_string());
    let program = parser.parse_program().unwrap();
    let mut interpreter = Interpreter::new();
    interpreter.capture_output();
    let result = interpreter.eval_program(&program).unwrap();
    let output = interpreter.get_output().unwrap().to_vec();
    (result, output)
}

// ============================================================
// Basic arithmetic
// ============================================================

#[test]
fn test_addition() {
    assert_eq!(run("(+ 1 2)"), Value::Integer(3));
}

#[test]
fn test_subtraction() {
    assert_eq!(run("(- 10 3)"), Value::Integer(7));
}

#[test]
fn test_multiplication() {
    assert_eq!(run("(* 6 7)"), Value::Integer(42));
}

#[test]
fn test_division() {
    assert_eq!(run("(/ 100 4)"), Value::Integer(25));
}

#[test]
fn test_modulo() {
    assert_eq!(run("(% 10 3)"), Value::Integer(1));
}

#[test]
fn test_nested_arithmetic() {
    assert_eq!(run("(+ (* 2 3) (* 4 5))"), Value::Integer(26));
}

#[test]
fn test_deeply_nested() {
    assert_eq!(run("(* (+ 1 2) (- 10 (/ 8 2)))"), Value::Integer(18));
}

// ============================================================
// Comparisons and booleans
// ============================================================

#[test]
fn test_equality() {
    assert_eq!(run("(== 42 42)"), Value::Bool(true));
    assert_eq!(run("(== 1 2)"), Value::Bool(false));
}

#[test]
fn test_inequality() {
    assert_eq!(run("(!= 1 2)"), Value::Bool(true));
}

#[test]
fn test_ordering() {
    assert_eq!(run("(< 1 2)"), Value::Bool(true));
    assert_eq!(run("(> 1 2)"), Value::Bool(false));
    assert_eq!(run("(<= 5 5)"), Value::Bool(true));
    assert_eq!(run("(>= 5 4)"), Value::Bool(true));
}

// ============================================================
// Conditionals
// ============================================================

#[test]
fn test_if_true() {
    assert_eq!(run("(もし 真 42 0)"), Value::Integer(42));
}

#[test]
fn test_if_false() {
    assert_eq!(run("(もし 偽 42 0)"), Value::Integer(0));
}

#[test]
fn test_if_with_comparison() {
    assert_eq!(run("(もし (> 10 5) 1 0)"), Value::Integer(1));
}

#[test]
fn test_if_english() {
    assert_eq!(run("(if true 1 0)"), Value::Integer(1));
}

// ============================================================
// Function definitions and calls
// ============================================================

#[test]
fn test_function_definition_and_call() {
    assert_eq!(
        run("(定義 二倍 (数 -> 数) (* 2 x)) (二倍 21)"),
        Value::Integer(42)
    );
}

#[test]
fn test_function_without_type_annotation() {
    assert_eq!(run("(定義 inc (+ 1 x)) (inc 41)"), Value::Integer(42));
}

#[test]
fn test_multiple_functions() {
    let input = r#"
        (定義 二倍 (数 -> 数) (* 2 x))
        (定義 三倍 (数 -> 数) (* 3 x))
        (+ (二倍 10) (三倍 10))
    "#;
    assert_eq!(run(input), Value::Integer(50));
}

#[test]
fn test_function_composition_manual() {
    let input = r#"
        (定義 二倍 (数 -> 数) (* 2 x))
        (二倍 (二倍 5))
    "#;
    assert_eq!(run(input), Value::Integer(20));
}

// ============================================================
// Recursive functions
// ============================================================

#[test]
fn test_factorial() {
    let input = r#"
        (定義 階乗 (数 -> 数)
            (場合 n
                (0 -> 1)
                (n -> (* n (階乗 (- n 1))))))
        (階乗 5)
    "#;
    assert_eq!(run(input), Value::Integer(120));
}

#[test]
fn test_factorial_10() {
    let input = r#"
        (定義 階乗 (数 -> 数)
            (場合 n
                (0 -> 1)
                (n -> (* n (階乗 (- n 1))))))
        (階乗 10)
    "#;
    assert_eq!(run(input), Value::Integer(3628800));
}

#[test]
fn test_fibonacci() {
    let input = r#"
        (定義 fib (数 -> 数)
            (場合 n
                (0 -> 0)
                (1 -> 1)
                (n -> (+ (fib (- n 1)) (fib (- n 2))))))
        (fib 10)
    "#;
    assert_eq!(run(input), Value::Integer(55));
}

// ============================================================
// Lambda expressions
// ============================================================

#[test]
fn test_lambda_immediate() {
    assert_eq!(run("((匿名 (x) (* x x)) 5)"), Value::Integer(25));
}

#[test]
fn test_lambda_multi_param() {
    assert_eq!(run("((匿名 (x y) (+ x y)) 10 20)"), Value::Integer(30));
}

// ============================================================
// Let bindings
// ============================================================

#[test]
fn test_let_single() {
    assert_eq!(run("(束縛 ((x 42)) x)"), Value::Integer(42));
}

#[test]
fn test_let_multiple() {
    assert_eq!(run("(束縛 ((x 10) (y 20)) (+ x y))"), Value::Integer(30));
}

#[test]
fn test_let_nested() {
    assert_eq!(
        run("(束縛 ((x 10)) (束縛 ((y (* x 2))) (+ x y)))"),
        Value::Integer(30)
    );
}

// ============================================================
// Pattern matching
// ============================================================

#[test]
fn test_match_literal() {
    assert_eq!(run("(場合 0 (0 -> 1) (n -> 0))"), Value::Integer(1));
}

#[test]
fn test_match_variable() {
    assert_eq!(run("(場合 42 (0 -> 0) (n -> (* n 2)))"), Value::Integer(84));
}

#[test]
fn test_match_wildcard() {
    assert_eq!(run("(場合 99 (0 -> 0) (_ -> 1))"), Value::Integer(1));
}

// ============================================================
// Strings
// ============================================================

#[test]
fn test_string_concatenation() {
    assert_eq!(
        run(r#"(+ "hello" " world")"#),
        Value::String("hello world".to_string())
    );
}

// ============================================================
// Print / output
// ============================================================

#[test]
fn test_print_integer() {
    let (_, output) = run_with_output("(表示 42)");
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_print_string() {
    let (_, output) = run_with_output(r#"(表示 "こんにちは世界")"#);
    assert_eq!(output, vec!["こんにちは世界"]);
}

#[test]
fn test_print_multiple() {
    let (_, output) = run_with_output(r#"(表示 1) (表示 2) (表示 3)"#);
    assert_eq!(output, vec!["1", "2", "3"]);
}

// ============================================================
// Japanese bracket variants
// ============================================================

#[test]
fn test_corner_brackets() {
    assert_eq!(run("「+ 1 2」"), Value::Integer(3));
}

#[test]
fn test_lenticular_brackets() {
    assert_eq!(run("【* 3 4】"), Value::Integer(12));
}

#[test]
fn test_mixed_bracket_nesting() {
    assert_eq!(run("「+ 1 【* 2 3】」"), Value::Integer(7));
}

#[test]
fn test_full_program_corner_brackets() {
    let input = "「定義 二倍 「数 -> 数」 「* 2 x」」 「二倍 21」";
    assert_eq!(run(input), Value::Integer(42));
}

// ============================================================
// Error handling
// ============================================================

#[test]
fn test_division_by_zero_error() {
    let compiler = Compiler::new();
    assert!(compiler.run_source("(/ 1 0)".to_string()).is_err());
}

#[test]
fn test_undefined_variable_error() {
    let compiler = Compiler::new();
    assert!(compiler.run_source("(+ x 1)".to_string()).is_err());
}

#[test]
fn test_non_exhaustive_match_error() {
    let compiler = Compiler::new();
    assert!(compiler
        .run_source("(場合 5 (0 -> 1) (1 -> 2))".to_string())
        .is_err());
}

// ============================================================
// Combined: the milestone test
// ============================================================

#[test]
fn test_milestone_double_function() {
    // The "surface reading works" milestone from the roadmap
    let input = "(定義 二倍 (数 -> 数) (* 2 x)) (二倍 21)";
    assert_eq!(run(input), Value::Integer(42));
}

#[test]
fn test_milestone_hello_world() {
    let (_, output) = run_with_output(r#"(表示 "掛詞の世界へようこそ")"#);
    assert_eq!(output, vec!["掛詞の世界へようこそ"]);
}
