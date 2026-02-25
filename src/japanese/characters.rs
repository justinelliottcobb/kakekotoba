//! Japanese character classification and analysis

use crate::error::Result;
use unicode_categories::UnicodeCategories;

/// Classifies Japanese characters for programming language analysis
pub struct CharacterClassifier {
    // Could add configuration or caching here in the future
}

impl CharacterClassifier {
    /// Create a new character classifier
    pub fn new() -> Self {
        Self {}
    }

    /// Classify all characters in a text string
    pub fn classify_text(&self, text: &str) -> Result<CharacterAnalysis> {
        let mut analysis = CharacterAnalysis::default();

        for c in text.chars() {
            analysis.total_chars += 1;

            let classification = self.classify_char(c);
            match classification {
                CharacterClass::Kanji => {
                    analysis.kanji_count += 1;
                    analysis.japanese_chars += 1;
                }
                CharacterClass::Hiragana => {
                    analysis.hiragana_count += 1;
                    analysis.japanese_chars += 1;
                }
                CharacterClass::Katakana => {
                    analysis.katakana_count += 1;
                    analysis.japanese_chars += 1;
                }
                CharacterClass::JapanesePunctuation => {
                    analysis.punctuation_count += 1;
                    analysis.japanese_chars += 1;
                }
                CharacterClass::Ascii => {
                    analysis.ascii_count += 1;
                }
                CharacterClass::AsciiPunctuation => {
                    analysis.punctuation_count += 1;
                }
                CharacterClass::Whitespace => {
                    analysis.whitespace_count += 1;
                }
                CharacterClass::Other => {
                    analysis.other_count += 1;
                }
            }
        }

        Ok(analysis)
    }

    /// Classify a single character
    pub fn classify_char(&self, c: char) -> CharacterClass {
        match c {
            // Hiragana block
            '\u{3040}'..='\u{309F}' => CharacterClass::Hiragana,

            // Katakana block
            '\u{30A0}'..='\u{30FF}' => CharacterClass::Katakana,

            // Halfwidth Katakana
            '\u{FF65}'..='\u{FF9F}' => CharacterClass::Katakana,

            // CJK Unified Ideographs (main Kanji block)
            '\u{4E00}'..='\u{9FAF}' => CharacterClass::Kanji,

            // CJK Extension A
            '\u{3400}'..='\u{4DBF}' => CharacterClass::Kanji,

            // CJK Extension B, C, D (less common)
            '\u{20000}'..='\u{2A6DF}' | '\u{2A700}'..='\u{2B73F}' | '\u{2B740}'..='\u{2B81F}' => {
                CharacterClass::Kanji
            }

            // Japanese punctuation
            '、' | '。' | '「' | '」' | '『' | '』' | '（' | '）' | '［' | '］' | '｛' | '｝'
            | '〈' | '〉' | '《' | '》' | '〔' | '〕' | '〖' | '〗' | '〘' | '〙' | '〚' | '〛'
            | '・' | '：' | '；' | '？' | '！' => CharacterClass::JapanesePunctuation,

            // ASCII alphanumeric
            'A'..='Z' | 'a'..='z' | '0'..='9' => CharacterClass::Ascii,

            // ASCII punctuation
            '!' | '"' | '#' | '$' | '%' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | '-' | '.'
            | '/' | ':' | ';' | '<' | '=' | '>' | '?' | '@' | '[' | '\\' | ']' | '^' | '_'
            | '`' | '{' | '|' | '}' | '~' => CharacterClass::AsciiPunctuation,

            // Whitespace
            c if c.is_whitespace() => CharacterClass::Whitespace,

            // Everything else
            _ => CharacterClass::Other,
        }
    }

    /// Determine the primary script used in text
    pub fn primary_script(&self, text: &str) -> JapaneseScript {
        let analysis = self.classify_text(text).unwrap_or_default();

        if analysis.total_chars == 0 {
            return JapaneseScript::Other;
        }

        // Find the most common script
        let kanji_ratio = analysis.kanji_count as f64 / analysis.total_chars as f64;
        let hiragana_ratio = analysis.hiragana_count as f64 / analysis.total_chars as f64;
        let katakana_ratio = analysis.katakana_count as f64 / analysis.total_chars as f64;
        let ascii_ratio = analysis.ascii_count as f64 / analysis.total_chars as f64;

        if kanji_ratio > 0.4 {
            JapaneseScript::Kanji
        } else if hiragana_ratio > 0.4 {
            JapaneseScript::Hiragana
        } else if katakana_ratio > 0.4 {
            JapaneseScript::Katakana
        } else if ascii_ratio > 0.4 {
            JapaneseScript::Ascii
        } else {
            JapaneseScript::Mixed
        }
    }

    /// Check if a character can start an identifier in Kakekotoba
    pub fn can_start_identifier(&self, c: char) -> bool {
        matches!(
            self.classify_char(c),
            CharacterClass::Kanji
                | CharacterClass::Hiragana
                | CharacterClass::Katakana
                | CharacterClass::Ascii
        ) && c != '_' // Underscore might be reserved
    }

    /// Check if a character can continue an identifier in Kakekotoba
    pub fn can_continue_identifier(&self, c: char) -> bool {
        self.can_start_identifier(c) || c.is_ascii_digit()
    }

    /// Check if a character is a valid operator character
    pub fn is_operator_char(&self, c: char) -> bool {
        matches!(
            c,
            '+' | '-' | '*' | '/' | '%' |
            '=' | '<' | '>' | '!' |
            '&' | '|' | '^' | '~' |
            '.' | ',' | ';' | ':' |
            '→' | '←' | '↑' | '↓' |  // Japanese arrows
            '∧' | '∨' | '¬' |       // Logic symbols
            '∈' | '∉' | '⊂' | '⊃' // Set theory symbols
        )
    }

    /// Get detailed information about a character
    pub fn char_info(&self, c: char) -> CharacterInfo {
        CharacterInfo {
            character: c,
            classification: self.classify_char(c),
            unicode_category: c.general_category_group(),
            can_start_identifier: self.can_start_identifier(c),
            can_continue_identifier: self.can_continue_identifier(c),
            is_operator: self.is_operator_char(c),
            byte_length: c.len_utf8(),
        }
    }
}

impl Default for CharacterClassifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of characters for parsing purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    /// Kanji characters (Chinese-derived ideographs)
    Kanji,
    /// Hiragana syllabary
    Hiragana,
    /// Katakana syllabary
    Katakana,
    /// Japanese-specific punctuation
    JapanesePunctuation,
    /// ASCII letters and numbers
    Ascii,
    /// ASCII punctuation and operators
    AsciiPunctuation,
    /// Whitespace characters
    Whitespace,
    /// Other characters
    Other,
}

/// Primary script classifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JapaneseScript {
    /// Primarily Kanji
    Kanji,
    /// Primarily Hiragana
    Hiragana,
    /// Primarily Katakana
    Katakana,
    /// Primarily ASCII
    Ascii,
    /// Mixed scripts
    Mixed,
    /// Other/unknown script
    Other,
}

/// Analysis result for character classification
#[derive(Debug, Clone, Default)]
pub struct CharacterAnalysis {
    pub total_chars: usize,
    pub japanese_chars: usize,
    pub kanji_count: usize,
    pub hiragana_count: usize,
    pub katakana_count: usize,
    pub ascii_count: usize,
    pub punctuation_count: usize,
    pub whitespace_count: usize,
    pub other_count: usize,
}

impl CharacterAnalysis {
    /// Get the ratio of Japanese to total characters
    pub fn japanese_ratio(&self) -> f64 {
        if self.total_chars == 0 {
            0.0
        } else {
            self.japanese_chars as f64 / self.total_chars as f64
        }
    }

    /// Get the most common character class
    pub fn primary_class(&self) -> CharacterClass {
        let counts = [
            (self.kanji_count, CharacterClass::Kanji),
            (self.hiragana_count, CharacterClass::Hiragana),
            (self.katakana_count, CharacterClass::Katakana),
            (self.ascii_count, CharacterClass::Ascii),
            (self.punctuation_count, CharacterClass::AsciiPunctuation),
            (self.whitespace_count, CharacterClass::Whitespace),
            (self.other_count, CharacterClass::Other),
        ];

        counts
            .iter()
            .max_by_key(|(count, _)| *count)
            .map(|(_, class)| *class)
            .unwrap_or(CharacterClass::Other)
    }
}

/// Detailed information about a specific character
#[derive(Debug, Clone)]
pub struct CharacterInfo {
    pub character: char,
    pub classification: CharacterClass,
    pub unicode_category: unicode_categories::GeneralCategoryGroup,
    pub can_start_identifier: bool,
    pub can_continue_identifier: bool,
    pub is_operator: bool,
    pub byte_length: usize,
}

/// Utilities for working with Japanese character ranges
pub struct CharacterRanges;

impl CharacterRanges {
    /// All hiragana characters
    pub const HIRAGANA: std::ops::RangeInclusive<char> = '\u{3040}'..='\u{309F}';

    /// All katakana characters
    pub const KATAKANA: std::ops::RangeInclusive<char> = '\u{30A0}'..='\u{30FF}';

    /// Main kanji block
    pub const KANJI_MAIN: std::ops::RangeInclusive<char> = '\u{4E00}'..='\u{9FAF}';

    /// Kanji extension A
    pub const KANJI_EXT_A: std::ops::RangeInclusive<char> = '\u{3400}'..='\u{4DBF}';

    /// Check if character is in any kanji range
    pub fn is_kanji(c: char) -> bool {
        Self::KANJI_MAIN.contains(&c) || Self::KANJI_EXT_A.contains(&c)
    }

    /// Check if character is hiragana
    pub fn is_hiragana(c: char) -> bool {
        Self::HIRAGANA.contains(&c)
    }

    /// Check if character is katakana
    pub fn is_katakana(c: char) -> bool {
        Self::KATAKANA.contains(&c)
    }

    /// Check if character is any Japanese script
    pub fn is_japanese(c: char) -> bool {
        Self::is_kanji(c) || Self::is_hiragana(c) || Self::is_katakana(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_classifier() {
        let classifier = CharacterClassifier::new();

        assert_eq!(classifier.classify_char('関'), CharacterClass::Kanji);
        assert_eq!(classifier.classify_char('あ'), CharacterClass::Hiragana);
        assert_eq!(classifier.classify_char('ア'), CharacterClass::Katakana);
        assert_eq!(classifier.classify_char('a'), CharacterClass::Ascii);
        assert_eq!(classifier.classify_char('1'), CharacterClass::Ascii);
        assert_eq!(classifier.classify_char(' '), CharacterClass::Whitespace);
        assert_eq!(
            classifier.classify_char('、'),
            CharacterClass::JapanesePunctuation
        );
    }

    #[test]
    fn test_text_analysis() {
        let classifier = CharacterClassifier::new();
        let analysis = classifier.classify_text("関数あa").unwrap();

        assert_eq!(analysis.total_chars, 3);
        assert_eq!(analysis.kanji_count, 1);
        assert_eq!(analysis.hiragana_count, 1);
        assert_eq!(analysis.ascii_count, 1);
        assert_eq!(analysis.japanese_chars, 2);
        assert_eq!(analysis.japanese_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_primary_script() {
        let classifier = CharacterClassifier::new();

        assert_eq!(classifier.primary_script("関数漢字"), JapaneseScript::Kanji);
        assert_eq!(
            classifier.primary_script("あいうえお"),
            JapaneseScript::Hiragana
        );
        assert_eq!(
            classifier.primary_script("アイウエオ"),
            JapaneseScript::Katakana
        );
        assert_eq!(
            classifier.primary_script("hello world"),
            JapaneseScript::Ascii
        );
        assert_eq!(classifier.primary_script("関数a"), JapaneseScript::Mixed);
    }

    #[test]
    fn test_identifier_rules() {
        let classifier = CharacterClassifier::new();

        assert!(classifier.can_start_identifier('関'));
        assert!(classifier.can_start_identifier('あ'));
        assert!(classifier.can_start_identifier('ア'));
        assert!(classifier.can_start_identifier('a'));
        assert!(!classifier.can_start_identifier('1'));
        assert!(!classifier.can_start_identifier(' '));

        assert!(classifier.can_continue_identifier('1'));
        assert!(classifier.can_continue_identifier('関'));
    }

    #[test]
    fn test_operator_chars() {
        let classifier = CharacterClassifier::new();

        assert!(classifier.is_operator_char('+'));
        assert!(classifier.is_operator_char('='));
        assert!(classifier.is_operator_char('→'));
        assert!(!classifier.is_operator_char('関'));
        assert!(!classifier.is_operator_char('a'));
    }

    #[test]
    fn test_char_info() {
        let classifier = CharacterClassifier::new();
        let info = classifier.char_info('関');

        assert_eq!(info.character, '関');
        assert_eq!(info.classification, CharacterClass::Kanji);
        assert!(info.can_start_identifier);
        assert!(info.can_continue_identifier);
        assert!(!info.is_operator);
        assert_eq!(info.byte_length, 3); // Kanji are 3 bytes in UTF-8
    }

    #[test]
    fn test_character_ranges() {
        assert!(CharacterRanges::is_kanji('関'));
        assert!(!CharacterRanges::is_kanji('あ'));

        assert!(CharacterRanges::is_hiragana('あ'));
        assert!(!CharacterRanges::is_hiragana('ア'));

        assert!(CharacterRanges::is_katakana('ア'));
        assert!(!CharacterRanges::is_katakana('関'));

        assert!(CharacterRanges::is_japanese('関'));
        assert!(CharacterRanges::is_japanese('あ'));
        assert!(CharacterRanges::is_japanese('ア'));
        assert!(!CharacterRanges::is_japanese('a'));
    }

    #[test]
    fn test_primary_class() {
        let mut analysis = CharacterAnalysis::default();
        analysis.kanji_count = 5;
        analysis.hiragana_count = 2;
        analysis.ascii_count = 1;

        assert_eq!(analysis.primary_class(), CharacterClass::Kanji);
    }
}
