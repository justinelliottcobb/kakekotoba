pub mod ast;
pub mod codegen;
pub mod error;
pub mod inference;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod pipeline;
pub mod repl;
pub mod sexp_parser;
pub mod types;

// Vertical programming infrastructure modules
pub mod japanese;
pub mod layout;
pub mod spatial_ast;
pub mod vertical;

pub use error::{Error, Result};
pub use pipeline::Compiler;
