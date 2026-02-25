//! Programming language integration for tategaki editor
//!
//! This module provides features specific to programming in vertical text,
//! including syntax awareness and compiler integration.

use crate::{Result, TategakiError};

/// Programming language support
#[derive(Debug)]
pub struct ProgrammingSupport {
    /// Language name
    pub language: String,
}

impl ProgrammingSupport {
    /// Create new programming support
    pub fn new(language: String) -> Self {
        Self { language }
    }
}
