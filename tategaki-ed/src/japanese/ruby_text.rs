//! Ruby text (furigana) support
use crate::{Result, TategakiError};

/// Ruby text annotation for Japanese characters
#[derive(Debug, Clone)]
pub struct RubyText {
    /// Base text (漢字)
    pub base: String,
    /// Ruby annotation (ふりがな)
    pub ruby: String,
}

impl RubyText {
    /// Create new ruby text annotation
    pub fn new(base: String, ruby: String) -> Self {
        Self { base, ruby }
    }
}
