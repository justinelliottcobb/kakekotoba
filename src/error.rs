use miette::{Diagnostic, SourceSpan};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(unused_assignments)]
#[derive(Error, Debug, Diagnostic)]
pub enum Error {
    #[error("Lexical error: {message}")]
    #[diagnostic(code(kakekotoba::lexer))]
    Lexer {
        #[source_code]
        src: String,
        #[label("here")]
        position: SourceSpan,
        message: String,
    },

    #[error("Parse error")]
    #[diagnostic(code(kakekotoba::parser))]
    Parser {
        #[source_code]
        src: String,
        #[label("unexpected token")]
        span: SourceSpan,
        expected: Vec<String>,
        found: String,
    },

    #[error("Type error")]
    #[diagnostic(code(kakekotoba::types))]
    Type {
        #[source_code]
        src: String,
        #[label("type mismatch")]
        span: SourceSpan,
        expected: String,
        found: String,
    },

    #[error("Inference error")]
    #[diagnostic(code(kakekotoba::inference))]
    Inference {
        #[source_code]
        src: String,
        #[label("cannot infer type")]
        span: SourceSpan,
        message: String,
    },

    #[error("Runtime error: {message}")]
    #[diagnostic(code(kakekotoba::runtime))]
    Runtime {
        #[source_code]
        src: String,
        #[label("here")]
        span: SourceSpan,
        message: String,
    },

    #[error("Code generation error: {message}")]
    #[diagnostic(code(kakekotoba::codegen))]
    Codegen { message: String },

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 encoding error")]
    Utf8(#[from] std::str::Utf8Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl From<Span> for SourceSpan {
    fn from(span: Span) -> Self {
        SourceSpan::new(span.start.into(), span.len().into())
    }
}
