pub mod error;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod types;
pub mod inference;
pub mod codegen;
pub mod pipeline;

// Vertical programming infrastructure modules
pub mod vertical;
pub mod layout;
pub mod japanese;
pub mod spatial_ast;

pub use error::{Error, Result};
pub use pipeline::Compiler;