//! Japanese language support for vertical text editing
//!
//! This module provides comprehensive Japanese text input, processing, and display
//! capabilities including IME integration, character handling, and text normalization.

use crate::{Result, TategakiError};
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub mod character_handler;
pub mod input_method;
pub mod normalization;
pub mod ruby_text;

pub use character_handler::*;
pub use input_method::*;
pub use normalization::*;
pub use ruby_text::*;

/// Japanese input method engine for text editing
#[derive(Debug)]
pub struct JapaneseInputMethod {
    /// Current input state
    state: InputState,
    /// Composition buffer for IME input
    composition_buffer: String,
    /// Candidate list for conversion
    candidates: Vec<Candidate>,
    /// Currently selected candidate index
    selected_candidate: Option<usize>,
    /// Input mode (hiragana, katakana, etc.)
    input_mode: InputMode,
    /// Character handler for processing
    character_handler: CharacterHandler,
}

/// Input state for IME processing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputState {
    /// Not composing, normal input
    Direct,
    /// Composing text, showing raw input
    Composing,
    /// Converting text, showing candidates
    Converting,
    /// Finished composing, ready to commit
    Committed,
}

/// Input mode for Japanese text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMode {
    /// Direct ASCII input
    Direct,
    /// Hiragana input
    Hiragana,
    /// Katakana input
    Katakana,
    /// Full-width ASCII
    FullWidthAscii,
    /// Half-width Katakana
    HalfWidthKatakana,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Hiragana
    }
}

/// Conversion candidate for IME
#[derive(Debug, Clone)]
pub struct Candidate {
    /// Display text for the candidate
    pub text: String,
    /// Reading (furigana) for the candidate
    pub reading: Option<String>,
    /// Part of speech information
    pub part_of_speech: Option<String>,
    /// Frequency/priority score
    pub score: f32,
}

impl JapaneseInputMethod {
    /// Create a new Japanese input method engine
    pub fn new() -> Self {
        Self {
            state: InputState::Direct,
            composition_buffer: String::new(),
            candidates: Vec::new(),
            selected_candidate: None,
            input_mode: InputMode::default(),
            character_handler: CharacterHandler::new(),
        }
    }

    /// Process a key input event
    pub fn process_key_input(&mut self, key: &str) -> Result<InputResult> {
        match &self.state {
            InputState::Direct => self.handle_direct_input(key),
            InputState::Composing => self.handle_composition_input(key),
            InputState::Converting => self.handle_conversion_input(key),
            InputState::Committed => {
                // Reset to direct after commit
                self.state = InputState::Direct;
                self.handle_direct_input(key)
            }
        }
    }

    /// Handle direct input (no composition)
    fn handle_direct_input(&mut self, key: &str) -> Result<InputResult> {
        match self.input_mode {
            InputMode::Direct => {
                // Pass through ASCII directly
                Ok(InputResult::Commit(key.to_string()))
            }
            InputMode::Hiragana => {
                // Start composition for Japanese input
                self.composition_buffer = key.to_string();
                self.state = InputState::Composing;
                Ok(InputResult::Compose(self.composition_buffer.clone()))
            }
            _ => {
                // Other input modes - placeholder
                Ok(InputResult::Commit(key.to_string()))
            }
        }
    }

    /// Handle composition input (building up text)
    fn handle_composition_input(&mut self, key: &str) -> Result<InputResult> {
        match key {
            " " => {
                // Space triggers conversion
                self.generate_candidates()?;
                if !self.candidates.is_empty() {
                    self.state = InputState::Converting;
                    self.selected_candidate = Some(0);
                    Ok(InputResult::ShowCandidates(self.candidates.clone()))
                } else {
                    // No candidates, commit as-is
                    let text = self.composition_buffer.clone();
                    self.reset_composition();
                    Ok(InputResult::Commit(text))
                }
            }
            "Enter" => {
                // Enter commits current composition
                let text = self.composition_buffer.clone();
                self.reset_composition();
                Ok(InputResult::Commit(text))
            }
            "Escape" => {
                // Escape cancels composition
                self.reset_composition();
                Ok(InputResult::Cancel)
            }
            "Backspace" => {
                // Remove last character from composition
                if !self.composition_buffer.is_empty() {
                    let mut chars: Vec<char> = self.composition_buffer.chars().collect();
                    chars.pop();
                    self.composition_buffer = chars.into_iter().collect();

                    if self.composition_buffer.is_empty() {
                        self.state = InputState::Direct;
                        Ok(InputResult::Cancel)
                    } else {
                        Ok(InputResult::Compose(self.composition_buffer.clone()))
                    }
                } else {
                    self.state = InputState::Direct;
                    Ok(InputResult::Cancel)
                }
            }
            _ => {
                // Add character to composition
                self.composition_buffer.push_str(key);
                Ok(InputResult::Compose(self.composition_buffer.clone()))
            }
        }
    }

    /// Handle conversion input (selecting candidates)
    fn handle_conversion_input(&mut self, key: &str) -> Result<InputResult> {
        match key {
            "Enter" => {
                // Commit selected candidate
                if let Some(index) = self.selected_candidate {
                    if let Some(candidate) = self.candidates.get(index) {
                        let text = candidate.text.clone();
                        self.reset_composition();
                        return Ok(InputResult::Commit(text));
                    }
                }
                // Fallback: commit composition buffer
                let text = self.composition_buffer.clone();
                self.reset_composition();
                Ok(InputResult::Commit(text))
            }
            "Escape" => {
                // Cancel conversion, back to composition
                self.state = InputState::Composing;
                self.selected_candidate = None;
                Ok(InputResult::Compose(self.composition_buffer.clone()))
            }
            "ArrowDown" | "Tab" => {
                // Next candidate
                self.select_next_candidate();
                Ok(InputResult::ShowCandidates(self.candidates.clone()))
            }
            "ArrowUp" | "Shift+Tab" => {
                // Previous candidate
                self.select_previous_candidate();
                Ok(InputResult::ShowCandidates(self.candidates.clone()))
            }
            _ if key.len() == 1 && key.chars().next().unwrap().is_ascii_digit() => {
                // Direct candidate selection by number
                if let Ok(digit) = key.parse::<usize>() {
                    if digit > 0 && digit <= self.candidates.len() {
                        let candidate = &self.candidates[digit - 1];
                        let text = candidate.text.clone();
                        self.reset_composition();
                        return Ok(InputResult::Commit(text));
                    }
                }
                Ok(InputResult::ShowCandidates(self.candidates.clone()))
            }
            _ => {
                // Other keys during conversion - add to composition and continue
                self.composition_buffer.push_str(key);
                self.generate_candidates()?;
                Ok(InputResult::ShowCandidates(self.candidates.clone()))
            }
        }
    }

    /// Generate conversion candidates for current composition
    fn generate_candidates(&mut self) -> Result<()> {
        // Placeholder candidate generation
        // In a real implementation, this would use a dictionary and ML models
        self.candidates.clear();

        let input = &self.composition_buffer;

        // Simple hiragana to kanji conversion examples
        match input.as_str() {
            "かんすう" => {
                self.candidates.push(Candidate {
                    text: "関数".to_string(),
                    reading: Some("かんすう".to_string()),
                    part_of_speech: Some("名詞".to_string()),
                    score: 1.0,
                });
                self.candidates.push(Candidate {
                    text: "函数".to_string(),
                    reading: Some("かんすう".to_string()),
                    part_of_speech: Some("名詞".to_string()),
                    score: 0.5,
                });
            }
            "かた" => {
                self.candidates.push(Candidate {
                    text: "型".to_string(),
                    reading: Some("かた".to_string()),
                    part_of_speech: Some("名詞".to_string()),
                    score: 1.0,
                });
                self.candidates.push(Candidate {
                    text: "形".to_string(),
                    reading: Some("かた".to_string()),
                    part_of_speech: Some("名詞".to_string()),
                    score: 0.7,
                });
            }
            "もし" => {
                self.candidates.push(Candidate {
                    text: "もし".to_string(), // Keep as hiragana for conditional
                    reading: Some("もし".to_string()),
                    part_of_speech: Some("副詞".to_string()),
                    score: 1.0,
                });
            }
            _ => {
                // Default: keep as hiragana
                self.candidates.push(Candidate {
                    text: input.to_string(),
                    reading: Some(input.to_string()),
                    part_of_speech: None,
                    score: 0.5,
                });
            }
        }

        Ok(())
    }

    /// Select next candidate
    fn select_next_candidate(&mut self) {
        if let Some(current) = self.selected_candidate {
            let next = (current + 1) % self.candidates.len();
            self.selected_candidate = Some(next);
        }
    }

    /// Select previous candidate
    fn select_previous_candidate(&mut self) {
        if let Some(current) = self.selected_candidate {
            let prev = if current == 0 {
                self.candidates.len().saturating_sub(1)
            } else {
                current - 1
            };
            self.selected_candidate = Some(prev);
        }
    }

    /// Reset composition state
    fn reset_composition(&mut self) {
        self.state = InputState::Direct;
        self.composition_buffer.clear();
        self.candidates.clear();
        self.selected_candidate = None;
    }

    /// Get current input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Set input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        // Reset composition when changing modes
        self.reset_composition();
    }

    /// Get current composition text
    pub fn composition_text(&self) -> &str {
        &self.composition_buffer
    }

    /// Get current candidates
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// Get selected candidate index
    pub fn selected_candidate(&self) -> Option<usize> {
        self.selected_candidate
    }

    /// Check if currently composing
    pub fn is_composing(&self) -> bool {
        !matches!(self.state, InputState::Direct)
    }
}

impl Default for JapaneseInputMethod {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of input processing
#[derive(Debug, Clone)]
pub enum InputResult {
    /// Commit final text to buffer
    Commit(String),
    /// Show composition text (not yet committed)
    Compose(String),
    /// Show candidate list for selection
    ShowCandidates(Vec<Candidate>),
    /// Cancel current input
    Cancel,
    /// No action needed
    NoOp,
}

/// Character handler for Japanese text processing
#[derive(Debug)]
pub struct CharacterHandler {
    /// Text normalizer
    normalizer: TextNormalizer,
}

impl CharacterHandler {
    /// Create a new character handler
    pub fn new() -> Self {
        Self {
            normalizer: TextNormalizer::new(),
        }
    }

    /// Classify a character
    pub fn classify_character(&self, ch: char) -> CharacterClass {
        match ch {
            // Hiragana
            '\u{3040}'..='\u{309F}' => CharacterClass::Hiragana,
            // Katakana
            '\u{30A0}'..='\u{30FF}' => CharacterClass::Katakana,
            // CJK Unified Ideographs (Kanji)
            '\u{4E00}'..='\u{9FAF}' => CharacterClass::Kanji,
            // Half-width Katakana
            '\u{FF65}'..='\u{FF9F}' => CharacterClass::HalfWidthKatakana,
            // Full-width ASCII
            '\u{FF01}'..='\u{FF5E}' => CharacterClass::FullWidthAscii,
            // ASCII
            '\u{0020}'..='\u{007E}' => CharacterClass::Ascii,
            // Whitespace
            c if c.is_whitespace() => CharacterClass::Whitespace,
            // Other
            _ => CharacterClass::Other,
        }
    }

    /// Calculate character width for layout
    pub fn character_width(&self, ch: char) -> f32 {
        match self.classify_character(ch) {
            CharacterClass::Ascii | CharacterClass::HalfWidthKatakana => 1.0,
            CharacterClass::Hiragana
            | CharacterClass::Katakana
            | CharacterClass::Kanji
            | CharacterClass::FullWidthAscii => 2.0,
            CharacterClass::Whitespace => 1.0,
            CharacterClass::Other => 1.0,
        }
    }

    /// Normalize text for consistent processing
    pub fn normalize_text(&self, text: &str) -> String {
        self.normalizer
            .normalize(text)
            .unwrap_or_else(|_| text.to_string())
    }

    /// Convert between character types
    pub fn convert_character(&self, ch: char, target: ConversionTarget) -> char {
        match target {
            ConversionTarget::Hiragana => self.to_hiragana(ch),
            ConversionTarget::Katakana => self.to_katakana(ch),
            ConversionTarget::HalfWidth => self.to_half_width(ch),
            ConversionTarget::FullWidth => self.to_full_width(ch),
        }
    }

    /// Convert character to hiragana
    fn to_hiragana(&self, ch: char) -> char {
        match ch {
            // Katakana to Hiragana
            'ア'..='ン' => char::from_u32(ch as u32 - 0x60).unwrap_or(ch),
            _ => ch,
        }
    }

    /// Convert character to katakana
    fn to_katakana(&self, ch: char) -> char {
        match ch {
            // Hiragana to Katakana
            'あ'..='ん' => char::from_u32(ch as u32 + 0x60).unwrap_or(ch),
            _ => ch,
        }
    }

    /// Convert to half-width
    fn to_half_width(&self, ch: char) -> char {
        match ch {
            // Full-width ASCII to half-width
            '！'..='～' => char::from_u32(ch as u32 - 0xFEE0).unwrap_or(ch),
            _ => ch,
        }
    }

    /// Convert to full-width
    fn to_full_width(&self, ch: char) -> char {
        match ch {
            // Half-width ASCII to full-width
            '!'..='~' => char::from_u32(ch as u32 + 0xFEE0).unwrap_or(ch),
            _ => ch,
        }
    }
}

impl Default for CharacterHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Character classification for Japanese text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Hiragana,
    Katakana,
    Kanji,
    HalfWidthKatakana,
    FullWidthAscii,
    Ascii,
    Whitespace,
    Other,
}

/// Character conversion targets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionTarget {
    Hiragana,
    Katakana,
    HalfWidth,
    FullWidth,
}

/// Text normalizer for Japanese text
#[derive(Debug)]
pub struct TextNormalizer {
    /// Normalization form to use
    form: NormalizationForm,
}

/// Unicode normalization forms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizationForm {
    NFC,  // Canonical Decomposition, followed by Canonical Composition
    NFD,  // Canonical Decomposition
    NFKC, // Compatibility Decomposition, followed by Canonical Composition
    NFKD, // Compatibility Decomposition
}

impl TextNormalizer {
    /// Create a new text normalizer
    pub fn new() -> Self {
        Self {
            form: NormalizationForm::NFC,
        }
    }

    /// Normalize text according to the configured form
    pub fn normalize(&self, text: &str) -> Result<String> {
        let normalized = match self.form {
            NormalizationForm::NFC => text.nfc().collect::<String>(),
            NormalizationForm::NFD => text.nfd().collect::<String>(),
            NormalizationForm::NFKC => text.nfkc().collect::<String>(),
            NormalizationForm::NFKD => text.nfkd().collect::<String>(),
        };
        Ok(normalized)
    }

    /// Set normalization form
    pub fn set_form(&mut self, form: NormalizationForm) {
        self.form = form;
    }
}

impl Default for TextNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_input_method_creation() {
        let ime = JapaneseInputMethod::new();
        assert_eq!(ime.input_mode(), InputMode::Hiragana);
        assert!(!ime.is_composing());
    }

    #[test]
    fn test_character_classification() {
        let handler = CharacterHandler::new();

        assert_eq!(handler.classify_character('あ'), CharacterClass::Hiragana);
        assert_eq!(handler.classify_character('ア'), CharacterClass::Katakana);
        assert_eq!(handler.classify_character('漢'), CharacterClass::Kanji);
        assert_eq!(handler.classify_character('a'), CharacterClass::Ascii);
        assert_eq!(
            handler.classify_character('ａ'),
            CharacterClass::FullWidthAscii
        );
    }

    #[test]
    fn test_character_width_calculation() {
        let handler = CharacterHandler::new();

        assert_eq!(handler.character_width('a'), 1.0); // Half-width
        assert_eq!(handler.character_width('あ'), 2.0); // Full-width
        assert_eq!(handler.character_width('漢'), 2.0); // Full-width
    }

    #[test]
    fn test_character_conversion() {
        let handler = CharacterHandler::new();

        assert_eq!(
            handler.convert_character('あ', ConversionTarget::Katakana),
            'ア'
        );
        assert_eq!(
            handler.convert_character('ア', ConversionTarget::Hiragana),
            'あ'
        );
        assert_eq!(
            handler.convert_character('a', ConversionTarget::FullWidth),
            'ａ'
        );
        assert_eq!(
            handler.convert_character('ａ', ConversionTarget::HalfWidth),
            'a'
        );
    }

    #[test]
    fn test_text_normalization() {
        let normalizer = TextNormalizer::new();
        let result = normalizer.normalize("test").unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_candidate_generation() {
        let mut ime = JapaneseInputMethod::new();
        ime.composition_buffer = "かんすう".to_string();
        ime.generate_candidates().unwrap();

        let candidates = ime.candidates();
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].text, "関数");
    }
}
