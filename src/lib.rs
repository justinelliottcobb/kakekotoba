pub mod error;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod types;
pub mod inference;
pub mod codegen;
pub mod pipeline;

pub use error::{Error, Result};
pub use pipeline::Compiler;