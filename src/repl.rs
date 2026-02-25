//! Interactive REPL for kakekotoba
//!
//! A simple read-eval-print loop using the S-expression parser and interpreter.

use crate::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::sexp_parser::{SExpParser, SExprResult};
use std::io::{self, BufRead, Write};

pub struct Repl {
    interpreter: Interpreter,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run(&mut self) {
        println!("掛詞 v0.1.0 — S式インタプリタ");
        println!(":quit / :終了 で終了");
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            print!("掛詞 > ");
            stdout.flush().unwrap();

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Handle REPL commands
            if trimmed.starts_with(':') {
                if self.handle_command(trimmed) {
                    break;
                }
                continue;
            }

            self.eval_line(trimmed);
        }

        println!();
        println!("さようなら");
    }

    fn handle_command(&mut self, cmd: &str) -> bool {
        match cmd {
            ":quit" | ":q" | ":終了" => return true,
            ":help" | ":h" | ":助け" => {
                println!("Commands:");
                println!("  :quit / :q / :終了     — Exit the REPL");
                println!("  :type <expr> / :型     — Show type (not yet implemented)");
                println!("  :help / :h / :助け     — Show this help");
                println!();
                println!("Examples:");
                println!("  (+ 1 2)");
                println!("  (定義 二倍 (数 -> 数) (* 2 x))");
                println!("  (二倍 21)");
                println!("  「+ 1 2」");
                println!("  【* 3 4】");
            }
            cmd if cmd.starts_with(":type ") || cmd.starts_with(":型 ") => {
                println!("Type inspection not yet implemented");
            }
            _ => {
                eprintln!("Unknown command: {}", cmd);
                eprintln!("Type :help for available commands");
            }
        }
        false
    }

    fn eval_line(&mut self, input: &str) {
        // Lex
        let mut lexer = Lexer::new(input.to_string());
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Lexer error: {}", e);
                return;
            }
        };

        // Parse
        let mut parser = SExpParser::new(tokens, input.to_string());
        let result = match parser.parse_single() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                return;
            }
        };

        // Evaluate
        match result {
            SExprResult::Declaration(decl) => match self.interpreter.eval_declaration(&decl) {
                Ok(Value::Function { ref name, .. }) => {
                    if let Some(name) = name {
                        println!("{} : <defined>", name);
                    }
                }
                Ok(_) => {}
                Err(e) => eprintln!("Error: {}", e),
            },
            SExprResult::Expression(expr) => {
                let env = self.interpreter.env.clone();
                match self.interpreter.eval_expression(&expr, &env) {
                    Ok(value) => println!("{}", value),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
    }
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}
