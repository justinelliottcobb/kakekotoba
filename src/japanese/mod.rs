//! Japanese-specific language features for Kakekotoba
//!
//! This module provides specialized handling for Japanese text in programming contexts,
//! including character classification, keyword detection, and linguistic analysis.

use crate::error::Result;
use unicode_normalization::is_nfc;

pub mod characters;
pub mod keywords;
pub mod normalization;

pub use characters::*;
pub use keywords::*;
pub use normalization::*;

/// Core Japanese text analyzer for programming language constructs
#[derive(Debug)]
pub struct JapaneseAnalyzer {
    /// Keyword detector
    keyword_detector: KeywordDetector,
    /// Character classifier
    character_classifier: CharacterClassifier,
    /// Text normalizer
    normalizer: TextNormalizer,
}

impl JapaneseAnalyzer {
    /// Create a new Japanese analyzer
    pub fn new() -> Self {
        Self {
            keyword_detector: KeywordDetector::new(),
            character_classifier: CharacterClassifier::new(),
            normalizer: TextNormalizer::new(),
        }
    }

    /// Analyze a piece of Japanese text for programming language features
    pub fn analyze(&self, text: &str) -> Result<JapaneseTextAnalysis> {
        // First, normalize the text
        let normalized = self.normalizer.normalize(text)?;

        // Classify characters
        let char_analysis = self.character_classifier.classify_text(&normalized)?;

        // Detect keywords
        let keywords = self.keyword_detector.detect_keywords(&normalized)?;

        Ok(JapaneseTextAnalysis {
            original_text: text.to_string(),
            normalized_text: normalized,
            character_analysis: char_analysis,
            keywords,
            is_normalized: is_nfc(text),
        })
    }

    /// Quick check if text contains Japanese programming keywords
    pub fn has_keywords(&self, text: &str) -> bool {
        self.keyword_detector.has_any_keyword(text)
    }

    /// Get the primary script used in the text
    pub fn primary_script(&self, text: &str) -> JapaneseScript {
        self.character_classifier.primary_script(text)
    }
}

impl Default for JapaneseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of analyzing Japanese text
#[derive(Debug, Clone)]
pub struct JapaneseTextAnalysis {
    /// Original input text
    pub original_text: String,
    /// Normalized text (NFC form)
    pub normalized_text: String,
    /// Character-level analysis
    pub character_analysis: CharacterAnalysis,
    /// Detected keywords
    pub keywords: Vec<DetectedKeyword>,
    /// Whether the original text was already normalized
    pub is_normalized: bool,
}

impl JapaneseTextAnalysis {
    /// Get the ratio of Japanese characters to total characters
    pub fn japanese_ratio(&self) -> f64 {
        if self.character_analysis.total_chars == 0 {
            0.0
        } else {
            (self.character_analysis.japanese_chars as f64)
                / (self.character_analysis.total_chars as f64)
        }
    }

    /// Check if this text is primarily Japanese
    pub fn is_primarily_japanese(&self) -> bool {
        self.japanese_ratio() > 0.5
    }

    /// Get all unique keyword types found
    pub fn keyword_types(&self) -> Vec<KeywordType> {
        let mut types: Vec<_> = self.keywords.iter().map(|kw| kw.keyword_type).collect();
        types.sort();
        types.dedup();
        types
    }

    /// Check if a specific keyword type was found
    pub fn has_keyword_type(&self, keyword_type: KeywordType) -> bool {
        self.keywords
            .iter()
            .any(|kw| kw.keyword_type == keyword_type)
    }
}

/// Utility functions for Japanese text processing
pub struct JapaneseUtils;

impl JapaneseUtils {
    /// Convert hiragana to katakana
    pub fn hiragana_to_katakana(text: &str) -> String {
        text.chars()
            .map(|c| {
                if ('あ'..='ん').contains(&c) {
                    // Convert hiragana to katakana
                    char::from_u32(c as u32 + 0x60).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }

    /// Convert katakana to hiragana
    pub fn katakana_to_hiragana(text: &str) -> String {
        text.chars()
            .map(|c| {
                if ('ア'..='ン').contains(&c) {
                    // Convert katakana to hiragana
                    char::from_u32(c as u32 - 0x60).unwrap_or(c)
                } else {
                    c
                }
            })
            .collect()
    }

    /// Check if text contains mixed scripts (dangerous for parsing)
    pub fn has_mixed_scripts(text: &str) -> bool {
        let mut has_hiragana = false;
        let mut has_katakana = false;
        let mut has_kanji = false;
        let mut has_ascii = false;

        for c in text.chars() {
            match c {
                'あ'..='ん' => has_hiragana = true,
                'ア'..='ン' => has_katakana = true,
                '\u{4E00}'..='\u{9FAF}' => has_kanji = true,
                'A'..='Z' | 'a'..='z' | '0'..='9' => has_ascii = true,
                _ => {}
            }
        }

        // Count how many script types we found
        let script_count = [has_hiragana, has_katakana, has_kanji, has_ascii]
            .iter()
            .filter(|&&x| x)
            .count();

        script_count > 2 // Mixed if more than 2 different scripts
    }

    /// Estimate reading difficulty based on character composition
    pub fn reading_difficulty(text: &str) -> ReadingDifficulty {
        let mut kanji_count = 0;
        let mut _hiragana_count = 0;
        let mut _katakana_count = 0;
        let mut total_chars = 0;

        for c in text.chars() {
            if !c.is_whitespace() {
                total_chars += 1;
                match c {
                    'あ'..='ん' => _hiragana_count += 1,
                    'ア'..='ン' => _katakana_count += 1,
                    '\u{4E00}'..='\u{9FAF}' => kanji_count += 1,
                    _ => {}
                }
            }
        }

        if total_chars == 0 {
            return ReadingDifficulty::Easy;
        }

        let kanji_ratio = kanji_count as f64 / total_chars as f64;

        if kanji_ratio < 0.2 {
            ReadingDifficulty::Easy
        } else if kanji_ratio < 0.5 {
            ReadingDifficulty::Medium
        } else {
            ReadingDifficulty::Hard
        }
    }
}

/// Reading difficulty levels for Japanese text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingDifficulty {
    /// Mostly hiragana/katakana, easy to read
    Easy,
    /// Balanced mix of scripts
    Medium,
    /// Heavy use of kanji, difficult to read
    Hard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_analyzer_creation() {
        let analyzer = JapaneseAnalyzer::new();
        assert!(!analyzer.has_keywords("hello world"));
        assert_eq!(analyzer.primary_script("hello"), JapaneseScript::Other);
    }

    #[test]
    fn test_japanese_utils() {
        assert_eq!(JapaneseUtils::hiragana_to_katakana("あいう"), "アイウ");
        assert_eq!(JapaneseUtils::katakana_to_hiragana("アイウ"), "あいう");

        assert!(JapaneseUtils::has_mixed_scripts("helloあいう漢字"));
        assert!(!JapaneseUtils::has_mixed_scripts("あいう"));
    }

    #[test]
    fn test_reading_difficulty() {
        assert_eq!(
            JapaneseUtils::reading_difficulty("あいうえお"),
            ReadingDifficulty::Easy
        );
        assert_eq!(
            JapaneseUtils::reading_difficulty("漢字漢字漢字"),
            ReadingDifficulty::Hard
        );
    }

    #[test]
    fn test_japanese_text_analysis() {
        let analyzer = JapaneseAnalyzer::new();
        let analysis = analyzer.analyze("関数").unwrap();

        assert!(analysis.is_primarily_japanese());
        assert!(analysis.japanese_ratio() > 0.9);
    }
}
