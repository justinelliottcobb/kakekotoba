//! Japanese keyword detection and classification

use crate::error::Result;
use std::collections::HashMap;

/// Detects and classifies Japanese keywords in programming contexts
#[derive(Debug)]
pub struct KeywordDetector {
    /// Map from keyword text to keyword type
    keyword_map: HashMap<String, KeywordType>,
}

impl KeywordDetector {
    /// Create a new keyword detector with standard Kakekotoba keywords
    pub fn new() -> Self {
        let mut keyword_map = HashMap::new();

        // Control flow keywords
        keyword_map.insert("関数".to_string(), KeywordType::Function);
        keyword_map.insert("型".to_string(), KeywordType::Type);
        keyword_map.insert("もし".to_string(), KeywordType::If);
        keyword_map.insert("それ".to_string(), KeywordType::Then);
        keyword_map.insert("さもなければ".to_string(), KeywordType::Else);
        keyword_map.insert("繰り返し".to_string(), KeywordType::Loop);
        keyword_map.insert("返す".to_string(), KeywordType::Return);
        keyword_map.insert("停止".to_string(), KeywordType::Break);
        keyword_map.insert("続ける".to_string(), KeywordType::Continue);

        // Data structure keywords
        keyword_map.insert("リスト".to_string(), KeywordType::List);
        keyword_map.insert("辞書".to_string(), KeywordType::Dictionary);
        keyword_map.insert("組".to_string(), KeywordType::Tuple);
        keyword_map.insert("選択".to_string(), KeywordType::Match);
        keyword_map.insert("場合".to_string(), KeywordType::Case);

        // Functional programming keywords
        keyword_map.insert("写像".to_string(), KeywordType::Map);
        keyword_map.insert("畳み込み".to_string(), KeywordType::Fold);
        keyword_map.insert("濾過".to_string(), KeywordType::Filter);
        keyword_map.insert("匿名".to_string(), KeywordType::Lambda);

        // Type system keywords
        keyword_map.insert("特性".to_string(), KeywordType::Trait);
        keyword_map.insert("実装".to_string(), KeywordType::Implementation);
        keyword_map.insert("導出".to_string(), KeywordType::Derive);
        keyword_map.insert("制約".to_string(), KeywordType::Constraint);

        // Module system keywords
        keyword_map.insert("模組".to_string(), KeywordType::Module);
        keyword_map.insert("輸入".to_string(), KeywordType::Import);
        keyword_map.insert("輸出".to_string(), KeywordType::Export);
        keyword_map.insert("公開".to_string(), KeywordType::Public);
        keyword_map.insert("非公開".to_string(), KeywordType::Private);

        // Variable keywords
        keyword_map.insert("甲".to_string(), KeywordType::ParameterX);
        keyword_map.insert("乙".to_string(), KeywordType::ParameterY);
        keyword_map.insert("定数".to_string(), KeywordType::Constant);
        keyword_map.insert("変数".to_string(), KeywordType::Variable);

        // Meta-programming (group homomorphisms)
        keyword_map.insert("群".to_string(), KeywordType::Group);
        keyword_map.insert("準同型".to_string(), KeywordType::Homomorphism);
        keyword_map.insert("合成".to_string(), KeywordType::Composition);
        keyword_map.insert("恒等".to_string(), KeywordType::Identity);

        // Literals and primitives
        keyword_map.insert("真".to_string(), KeywordType::True);
        keyword_map.insert("偽".to_string(), KeywordType::False);
        keyword_map.insert("無".to_string(), KeywordType::None);
        keyword_map.insert("単位".to_string(), KeywordType::Unit);

        Self { keyword_map }
    }

    /// Detect all keywords in the given text
    pub fn detect_keywords(&self, text: &str) -> Result<Vec<DetectedKeyword>> {
        let mut keywords = Vec::new();
        let mut byte_offset = 0;

        // Simple word boundary detection for Japanese
        let words = self.extract_words(text);

        for (word, start_offset) in words {
            if let Some(&keyword_type) = self.keyword_map.get(&word) {
                let byte_length = word.len();
                keywords.push(DetectedKeyword {
                    text: word,
                    keyword_type,
                    byte_offset: start_offset,
                    byte_length,
                });
            }
        }

        Ok(keywords)
    }

    /// Check if text contains any keywords
    pub fn has_any_keyword(&self, text: &str) -> bool {
        let words = self.extract_words(text);
        words
            .iter()
            .any(|(word, _)| self.keyword_map.contains_key(word))
    }

    /// Get the keyword type for a specific word, if it is a keyword
    pub fn keyword_type(&self, word: &str) -> Option<KeywordType> {
        self.keyword_map.get(word).copied()
    }

    /// Extract words from Japanese text (simple approach)
    fn extract_words(&self, text: &str) -> Vec<(String, usize)> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut word_start = 0;
        let mut byte_offset = 0;

        for c in text.chars() {
            let char_bytes = c.len_utf8();

            if self.is_word_separator(c) {
                // End current word if it exists
                if !current_word.is_empty() {
                    words.push((current_word.clone(), word_start));
                    current_word.clear();
                }
                word_start = byte_offset + char_bytes;
            } else {
                // Continue building current word
                if current_word.is_empty() {
                    word_start = byte_offset;
                }
                current_word.push(c);
            }

            byte_offset += char_bytes;
        }

        // Don't forget the last word
        if !current_word.is_empty() {
            words.push((current_word, word_start));
        }

        words
    }

    /// Check if a character separates words in Japanese text
    fn is_word_separator(&self, c: char) -> bool {
        matches!(
            c,
            ' ' | '\t' | '\n' | '\r' |  // Whitespace
            '(' | ')' | '[' | ']' | '{' | '}' |  // Brackets
            '、' | '。' | '，' | '．' |  // Japanese punctuation
            '!' | '?' | ':' | ';' |  // ASCII punctuation
            '！' | '？' | '：' | '；' // Full-width punctuation
        )
    }

    /// Add a custom keyword to the detector
    pub fn add_keyword(&mut self, text: String, keyword_type: KeywordType) {
        self.keyword_map.insert(text, keyword_type);
    }

    /// Remove a keyword from the detector
    pub fn remove_keyword(&mut self, text: &str) {
        self.keyword_map.remove(text);
    }

    /// Get all registered keywords
    pub fn all_keywords(&self) -> &HashMap<String, KeywordType> {
        &self.keyword_map
    }
}

impl Default for KeywordDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// A detected keyword in text
#[derive(Debug, Clone, PartialEq)]
pub struct DetectedKeyword {
    /// The keyword text
    pub text: String,
    /// Type of keyword
    pub keyword_type: KeywordType,
    /// Byte offset in the source text
    pub byte_offset: usize,
    /// Length in bytes
    pub byte_length: usize,
}

impl DetectedKeyword {
    /// Get the end byte offset of this keyword
    pub fn end_offset(&self) -> usize {
        self.byte_offset + self.byte_length
    }

    /// Check if this keyword is a control flow keyword
    pub fn is_control_flow(&self) -> bool {
        matches!(
            self.keyword_type,
            KeywordType::If
                | KeywordType::Then
                | KeywordType::Else
                | KeywordType::Loop
                | KeywordType::Return
                | KeywordType::Break
                | KeywordType::Continue
                | KeywordType::Match
                | KeywordType::Case
        )
    }

    /// Check if this keyword is a type-related keyword
    pub fn is_type_related(&self) -> bool {
        matches!(
            self.keyword_type,
            KeywordType::Type
                | KeywordType::Trait
                | KeywordType::Implementation
                | KeywordType::Derive
                | KeywordType::Constraint
        )
    }

    /// Check if this keyword is a functional programming keyword
    pub fn is_functional(&self) -> bool {
        matches!(
            self.keyword_type,
            KeywordType::Map
                | KeywordType::Fold
                | KeywordType::Filter
                | KeywordType::Lambda
                | KeywordType::Composition
        )
    }
}

/// Types of keywords in the Kakekotoba language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeywordType {
    // Control flow
    Function,
    If,
    Then,
    Else,
    Loop,
    Return,
    Break,
    Continue,
    Match,
    Case,

    // Data structures
    Type,
    List,
    Dictionary,
    Tuple,

    // Functional programming
    Map,
    Fold,
    Filter,
    Lambda,
    Composition,

    // Type system
    Trait,
    Implementation,
    Derive,
    Constraint,

    // Module system
    Module,
    Import,
    Export,
    Public,
    Private,

    // Variables
    ParameterX,
    ParameterY,
    Constant,
    Variable,

    // Meta-programming
    Group,
    Homomorphism,
    Identity,

    // Literals
    True,
    False,
    None,
    Unit,
}

impl KeywordType {
    /// Get a human-readable description of this keyword type
    pub fn description(&self) -> &'static str {
        match self {
            KeywordType::Function => "Function definition",
            KeywordType::If => "Conditional expression",
            KeywordType::Then => "Then branch",
            KeywordType::Else => "Else branch",
            KeywordType::Loop => "Loop construct",
            KeywordType::Return => "Return statement",
            KeywordType::Break => "Break from loop",
            KeywordType::Continue => "Continue loop",
            KeywordType::Match => "Pattern matching",
            KeywordType::Case => "Match case",
            KeywordType::Type => "Type definition",
            KeywordType::List => "List data structure",
            KeywordType::Dictionary => "Dictionary/map structure",
            KeywordType::Tuple => "Tuple structure",
            KeywordType::Map => "Map function",
            KeywordType::Fold => "Fold/reduce function",
            KeywordType::Filter => "Filter function",
            KeywordType::Lambda => "Anonymous function",
            KeywordType::Composition => "Function composition",
            KeywordType::Trait => "Trait definition",
            KeywordType::Implementation => "Trait implementation",
            KeywordType::Derive => "Derive macro",
            KeywordType::Constraint => "Type constraint",
            KeywordType::Module => "Module definition",
            KeywordType::Import => "Import statement",
            KeywordType::Export => "Export statement",
            KeywordType::Public => "Public visibility",
            KeywordType::Private => "Private visibility",
            KeywordType::ParameterX => "X parameter",
            KeywordType::ParameterY => "Y parameter",
            KeywordType::Constant => "Constant value",
            KeywordType::Variable => "Variable binding",
            KeywordType::Group => "Group structure",
            KeywordType::Homomorphism => "Group homomorphism",
            KeywordType::Identity => "Identity element",
            KeywordType::True => "Boolean true",
            KeywordType::False => "Boolean false",
            KeywordType::None => "None/null value",
            KeywordType::Unit => "Unit type",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_detector_creation() {
        let detector = KeywordDetector::new();
        assert!(detector.keyword_map.contains_key("関数"));
        assert!(detector.keyword_map.contains_key("型"));
        assert!(detector.keyword_map.contains_key("もし"));
    }

    #[test]
    fn test_keyword_detection() {
        let detector = KeywordDetector::new();
        let keywords = detector
            .detect_keywords("関数 main() { 返す 42; }")
            .unwrap();

        assert_eq!(keywords.len(), 2);
        assert_eq!(keywords[0].text, "関数");
        assert_eq!(keywords[0].keyword_type, KeywordType::Function);
        assert_eq!(keywords[1].text, "返す");
        assert_eq!(keywords[1].keyword_type, KeywordType::Return);
    }

    #[test]
    fn test_has_any_keyword() {
        let detector = KeywordDetector::new();
        assert!(detector.has_any_keyword("これは 関数 です"));
        assert!(!detector.has_any_keyword("これは テスト です"));
    }

    #[test]
    fn test_keyword_type_lookup() {
        let detector = KeywordDetector::new();
        assert_eq!(detector.keyword_type("関数"), Some(KeywordType::Function));
        assert_eq!(detector.keyword_type("unknown"), None);
    }

    #[test]
    fn test_word_extraction() {
        let detector = KeywordDetector::new();
        let words = detector.extract_words("関数 main(甲、乙)");

        assert!(words.len() >= 3);
        assert!(words.iter().any(|(word, _)| word == "関数"));
        assert!(words.iter().any(|(word, _)| word == "甲"));
        assert!(words.iter().any(|(word, _)| word == "乙"));
    }

    #[test]
    fn test_detected_keyword_properties() {
        let keyword = DetectedKeyword {
            text: "もし".to_string(),
            keyword_type: KeywordType::If,
            byte_offset: 10,
            byte_length: 6, // "もし" is 6 bytes in UTF-8
        };

        assert_eq!(keyword.end_offset(), 16);
        assert!(keyword.is_control_flow());
        assert!(!keyword.is_type_related());
        assert!(!keyword.is_functional());
    }

    #[test]
    fn test_custom_keywords() {
        let mut detector = KeywordDetector::new();
        detector.add_keyword("テスト".to_string(), KeywordType::Function);

        assert_eq!(detector.keyword_type("テスト"), Some(KeywordType::Function));
        assert!(detector.has_any_keyword("これは テスト です"));

        detector.remove_keyword("テスト");
        assert_eq!(detector.keyword_type("テスト"), None);
    }

    #[test]
    fn test_keyword_type_descriptions() {
        assert_eq!(KeywordType::Function.description(), "Function definition");
        assert_eq!(KeywordType::If.description(), "Conditional expression");
        assert_eq!(KeywordType::Map.description(), "Map function");
    }
}
