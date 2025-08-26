//! Text normalization for Japanese programming languages

use unicode_normalization::{UnicodeNormalization, is_nfc, is_nfd, is_nfkc, is_nfkd};
use crate::error::Result;

/// Normalizes Japanese text for consistent processing in programming contexts
pub struct TextNormalizer {
    /// Normalization form to use
    form: NormalizationForm,
    /// Whether to perform additional Japanese-specific normalizations
    japanese_specific: bool,
}

/// Unicode normalization forms
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NormalizationForm {
    /// Canonical Decomposition, followed by Canonical Composition (NFC)
    NFC,
    /// Canonical Decomposition (NFD)
    NFD,
    /// Compatibility Decomposition, followed by Canonical Composition (NFKC)
    NFKC,
    /// Compatibility Decomposition (NFKD)
    NFKD,
}

impl TextNormalizer {
    /// Create a new text normalizer with default settings (NFC, Japanese-specific enabled)
    pub fn new() -> Self {
        Self {
            form: NormalizationForm::NFC,
            japanese_specific: true,
        }
    }

    /// Create a text normalizer with specific settings
    pub fn with_form(form: NormalizationForm, japanese_specific: bool) -> Self {
        Self {
            form,
            japanese_specific,
        }
    }

    /// Normalize text according to the configured form
    pub fn normalize(&self, text: &str) -> Result<String> {
        // First apply Unicode normalization
        let unicode_normalized = match self.form {
            NormalizationForm::NFC => text.nfc().collect::<String>(),
            NormalizationForm::NFD => text.nfd().collect::<String>(),
            NormalizationForm::NFKC => text.nfkc().collect::<String>(),
            NormalizationForm::NFKD => text.nfkd().collect::<String>(),
        };

        // Then apply Japanese-specific normalizations if enabled
        if self.japanese_specific {
            Ok(self.normalize_japanese_specific(&unicode_normalized))
        } else {
            Ok(unicode_normalized)
        }
    }

    /// Apply Japanese-specific normalizations
    fn normalize_japanese_specific(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        
        for c in text.chars() {
            let normalized_char = self.normalize_japanese_char(c);
            result.push(normalized_char);
        }
        
        result
    }

    /// Normalize a single Japanese character
    fn normalize_japanese_char(&self, c: char) -> char {
        match c {
            // Normalize full-width ASCII to half-width
            'Ａ'..='Ｚ' => {
                // Convert full-width A-Z to half-width
                char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
            }
            'ａ'..='ｚ' => {
                // Convert full-width a-z to half-width
                char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
            }
            '０'..='９' => {
                // Convert full-width digits to half-width
                char::from_u32(c as u32 - 0xFEE0).unwrap_or(c)
            }
            
            // Normalize certain punctuation marks
            '．' => '.', // Full-width period to half-width
            '，' => ',', // Full-width comma to half-width
            '！' => '!', // Full-width exclamation to half-width
            '？' => '?', // Full-width question to half-width
            '：' => ':', // Full-width colon to half-width
            '；' => ';', // Full-width semicolon to half-width
            
            // Normalize parentheses and brackets
            '（' => '(',
            '）' => ')',
            '［' => '[',
            '］' => ']',
            '｛' => '{',
            '｝' => '}',
            
            // Leave other characters as-is
            _ => c,
        }
    }

    /// Check if text is already normalized in the target form
    pub fn is_normalized(&self, text: &str) -> bool {
        match self.form {
            NormalizationForm::NFC => is_nfc(text),
            NormalizationForm::NFD => is_nfd(text),
            NormalizationForm::NFKC => is_nfkc(text),
            NormalizationForm::NFKD => is_nfkd(text),
        }
    }

    /// Get normalization statistics for text
    pub fn analyze_normalization(&self, text: &str) -> NormalizationAnalysis {
        let normalized = self.normalize(text).unwrap_or_else(|_| text.to_string());
        
        NormalizationAnalysis {
            original_length: text.len(),
            normalized_length: normalized.len(),
            was_already_normalized: text == normalized,
            form: self.form,
            has_full_width_chars: text.chars().any(|c| self.is_full_width_char(c)),
            has_japanese_punctuation: text.chars().any(|c| self.is_japanese_punctuation(c)),
        }
    }

    /// Check if a character is full-width
    fn is_full_width_char(&self, c: char) -> bool {
        matches!(c,
            'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９' |
            '　' | '！' | '？' | '：' | '；' | '．' | '，' |
            '（' | '）' | '［' | '］' | '｛' | '｝'
        )
    }

    /// Check if a character is Japanese punctuation
    fn is_japanese_punctuation(&self, c: char) -> bool {
        matches!(c,
            '、' | '。' | '「' | '」' | '『' | '』' |
            '・' | '〜' | '〈' | '〉' | '《' | '》' |
            '〔' | '〕' | '【' | '】'
        )
    }

    /// Normalize text specifically for programming identifiers
    pub fn normalize_identifier(&self, text: &str) -> Result<String> {
        let normalized = self.normalize(text)?;
        
        // Additional identifier-specific normalizations
        let mut result = String::with_capacity(normalized.len());
        
        for c in normalized.chars() {
            // For identifiers, we might want to be more restrictive
            match c {
                // Allow Japanese characters
                c if self.is_japanese_identifier_char(c) => result.push(c),
                // Allow ASCII alphanumeric
                'A'..='Z' | 'a'..='z' | '0'..='9' => result.push(c),
                // Convert some symbols to underscore
                '-' | '‐' | '−' => result.push('_'),
                // Skip other characters (they would make invalid identifiers)
                _ => {}
            }
        }
        
        Ok(result)
    }

    /// Check if a character is valid in Japanese identifiers
    fn is_japanese_identifier_char(&self, c: char) -> bool {
        matches!(c,
            // Hiragana
            '\u{3040}'..='\u{309F}' |
            // Katakana
            '\u{30A0}'..='\u{30FF}' |
            // CJK Unified Ideographs
            '\u{4E00}'..='\u{9FAF}' |
            // CJK Extension A
            '\u{3400}'..='\u{4DBF}'
        )
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analysis of text normalization
#[derive(Debug, Clone)]
pub struct NormalizationAnalysis {
    /// Length of original text in bytes
    pub original_length: usize,
    /// Length of normalized text in bytes
    pub normalized_length: usize,
    /// Whether the text was already in normalized form
    pub was_already_normalized: bool,
    /// Normalization form used
    pub form: NormalizationForm,
    /// Whether the text contains full-width characters
    pub has_full_width_chars: bool,
    /// Whether the text contains Japanese punctuation
    pub has_japanese_punctuation: bool,
}

impl NormalizationAnalysis {
    /// Calculate the size change ratio
    pub fn size_change_ratio(&self) -> f64 {
        if self.original_length == 0 {
            1.0
        } else {
            self.normalized_length as f64 / self.original_length as f64
        }
    }

    /// Check if normalization changed the text significantly
    pub fn has_significant_changes(&self) -> bool {
        !self.was_already_normalized && 
        (self.has_full_width_chars || self.has_japanese_punctuation)
    }
}

/// Batch normalizer for processing multiple texts efficiently
pub struct BatchNormalizer {
    normalizer: TextNormalizer,
}

impl BatchNormalizer {
    /// Create a new batch normalizer
    pub fn new(form: NormalizationForm, japanese_specific: bool) -> Self {
        Self {
            normalizer: TextNormalizer::with_form(form, japanese_specific),
        }
    }

    /// Normalize multiple texts in batch
    pub fn normalize_batch(&self, texts: &[&str]) -> Result<Vec<String>> {
        texts.iter()
            .map(|text| self.normalizer.normalize(text))
            .collect()
    }

    /// Get normalization statistics for multiple texts
    pub fn analyze_batch(&self, texts: &[&str]) -> Vec<NormalizationAnalysis> {
        texts.iter()
            .map(|text| self.normalizer.analyze_normalization(text))
            .collect()
    }
}

/// Utilities for working with normalization forms
pub struct NormalizationUtils;

impl NormalizationUtils {
    /// Detect the best normalization form for a given text
    pub fn detect_best_form(text: &str) -> NormalizationForm {
        // Check each form to see which one the text is already in
        if is_nfc(text) {
            NormalizationForm::NFC
        } else if is_nfd(text) {
            NormalizationForm::NFD
        } else if is_nfkc(text) {
            NormalizationForm::NFKC
        } else if is_nfkd(text) {
            NormalizationForm::NFKD
        } else {
            // Default to NFC for programming languages
            NormalizationForm::NFC
        }
    }

    /// Check if two texts are equivalent after normalization
    pub fn are_equivalent(text1: &str, text2: &str, form: NormalizationForm) -> bool {
        let normalizer = TextNormalizer::with_form(form, true);
        let norm1 = normalizer.normalize(text1).unwrap_or_else(|_| text1.to_string());
        let norm2 = normalizer.normalize(text2).unwrap_or_else(|_| text2.to_string());
        norm1 == norm2
    }

    /// Convert between normalization forms
    pub fn convert_form(text: &str, from: NormalizationForm, to: NormalizationForm) -> String {
        if from == to {
            return text.to_string();
        }

        let from_normalizer = TextNormalizer::with_form(from, false);
        let to_normalizer = TextNormalizer::with_form(to, false);
        
        let intermediate = from_normalizer.normalize(text).unwrap_or_else(|_| text.to_string());
        to_normalizer.normalize(&intermediate).unwrap_or(intermediate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_normalizer_creation() {
        let normalizer = TextNormalizer::new();
        assert_eq!(normalizer.form, NormalizationForm::NFC);
        assert!(normalizer.japanese_specific);
    }

    #[test]
    fn test_basic_normalization() {
        let normalizer = TextNormalizer::new();
        
        // Test full-width to half-width conversion
        let result = normalizer.normalize("Ａｂｃ１２３").unwrap();
        assert_eq!(result, "Abc123");
        
        // Test punctuation normalization
        let result = normalizer.normalize("！？：；").unwrap();
        assert_eq!(result, "!?:;");
    }

    #[test]
    fn test_unicode_normalization() {
        let normalizer = TextNormalizer::with_form(NormalizationForm::NFC, false);
        
        // Test with composed vs decomposed characters
        let composed = "が"; // Composed hiragana GA
        let decomposed = "が"; // This might be decomposed depending on source
        
        let result = normalizer.normalize(composed).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_japanese_specific_normalization() {
        let normalizer = TextNormalizer::new();
        let result = normalizer.normalize("（テスト）").unwrap();
        assert_eq!(result, "(テスト)");
    }

    #[test]
    fn test_normalization_analysis() {
        let normalizer = TextNormalizer::new();
        let analysis = normalizer.analyze_normalization("Ａｂｃ！");
        
        assert!(analysis.has_full_width_chars);
        assert!(!analysis.was_already_normalized);
        assert_eq!(analysis.form, NormalizationForm::NFC);
    }

    #[test]
    fn test_identifier_normalization() {
        let normalizer = TextNormalizer::new();
        
        let result = normalizer.normalize_identifier("関数ーＮａｍｅ").unwrap();
        // Should convert full-width to half-width and handle dashes
        assert!(result.contains("関数"));
        assert!(result.contains("Name"));
    }

    #[test]
    fn test_is_normalized() {
        let normalizer = TextNormalizer::with_form(NormalizationForm::NFC, false);
        
        // ASCII text should already be normalized
        assert!(normalizer.is_normalized("hello world"));
    }

    #[test]
    fn test_batch_normalizer() {
        let batch = BatchNormalizer::new(NormalizationForm::NFC, true);
        let texts = vec!["Ａ", "Ｂ", "Ｃ"];
        
        let results = batch.normalize_batch(&texts).unwrap();
        assert_eq!(results, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_normalization_utils() {
        let form = NormalizationUtils::detect_best_form("hello world");
        assert_eq!(form, NormalizationForm::NFC);
        
        assert!(NormalizationUtils::are_equivalent(
            "Ａｂｃ",
            "Abc",
            NormalizationForm::NFC
        ));
    }

    #[test]
    fn test_character_classification() {
        let normalizer = TextNormalizer::new();
        
        assert!(normalizer.is_full_width_char('Ａ'));
        assert!(normalizer.is_full_width_char('１'));
        assert!(!normalizer.is_full_width_char('A'));
        
        assert!(normalizer.is_japanese_punctuation('。'));
        assert!(normalizer.is_japanese_punctuation('、'));
        assert!(!normalizer.is_japanese_punctuation('.'));
    }
}