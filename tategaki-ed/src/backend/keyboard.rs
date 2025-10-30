//! Vim-like keyboard handling for vertical text editing
//!
//! This module provides modal editing inspired by Vim, adapted for vertical
//! Japanese text (tategaki) where the navigation semantics are rotated.

use crate::{Result, TategakiError};
use crate::text_engine::TextDirection;
use crate::spatial::SpatialPosition;
use std::collections::HashMap;

/// Editor modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditorMode {
    /// Normal mode - navigation and commands
    Normal,
    /// Insert mode - text input
    Insert,
    /// Visual mode - text selection
    Visual,
    /// Visual line mode - line-wise selection
    VisualLine,
    /// Command mode - ex commands (:w, :q, etc.)
    Command,
}

impl EditorMode {
    /// Get a display string for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual => "VISUAL",
            EditorMode::VisualLine => "V-LINE",
            EditorMode::Command => "COMMAND",
        }
    }

    /// Get the Japanese name for the mode
    pub fn japanese_name(&self) -> &'static str {
        match self {
            EditorMode::Normal => "通常",
            EditorMode::Insert => "挿入",
            EditorMode::Visual => "選択",
            EditorMode::VisualLine => "行選択",
            EditorMode::Command => "命令",
        }
    }
}

/// Key input representation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyInput {
    /// The key character or name
    pub key: String,
    /// Control modifier
    pub ctrl: bool,
    /// Alt/Meta modifier
    pub alt: bool,
    /// Shift modifier (for letters, already in key)
    pub shift: bool,
}

impl KeyInput {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            alt: false,
            shift: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    /// Parse from notcurses key event
    pub fn from_notcurses_key(key_code: u32, ctrl: bool, alt: bool, shift: bool) -> Self {
        // Convert notcurses key code to string
        let key = if key_code < 128 {
            // ASCII character
            (key_code as u8 as char).to_string()
        } else {
            // Special key - map notcurses codes
            match key_code {
                // Arrow keys
                0x103 => "Up".to_string(),
                0x102 => "Down".to_string(),
                0x104 => "Left".to_string(),
                0x105 => "Right".to_string(),
                // Function keys
                0x109 => "Escape".to_string(),
                0x10A => "Enter".to_string(),
                0x107 => "Backspace".to_string(),
                0x14A => "Delete".to_string(),
                // Home/End
                0x106 => "Home".to_string(),
                0x168 => "End".to_string(),
                // Page Up/Down
                0x153 => "PageUp".to_string(),
                0x152 => "PageDown".to_string(),
                // Tab
                0x9 => "Tab".to_string(),
                _ => format!("Unknown({})", key_code),
            }
        };

        Self { key, ctrl, alt, shift }
    }
}

/// Editor command
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorCommand {
    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeAtLineStart,
    EnterInsertModeAtLineEnd,
    EnterInsertModeNewLineBelow,
    EnterInsertModeNewLineAbove,
    EnterNormalMode,
    EnterVisualMode,
    EnterVisualLineMode,
    EnterCommandMode,

    // Navigation (adapted for vertical text)
    MoveUp,         // Up in vertical text (previous character in column)
    MoveDown,       // Down in vertical text (next character in column)
    MoveLeft,       // Left in vertical text (next column, right-to-left)
    MoveRight,      // Right in vertical text (previous column)
    MoveWordForward,
    MoveWordBackward,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToFileStart,
    MoveToFileEnd,

    // Editing
    InsertChar(char),
    InsertText(String),
    DeleteChar,
    DeleteCharBackward,
    DeleteLine,
    DeleteWord,
    DeleteToLineEnd,
    Yank,           // Copy
    YankLine,
    Paste,
    PasteBefore,
    Undo,
    Redo,

    // Visual mode
    ExtendSelection,

    // Command mode
    ExecuteCommand(String),

    // File operations
    Save,
    SaveAndQuit,
    Quit,
    QuitForce,

    // Other
    NoOp,
}

/// Keyboard handler with vim-like bindings for vertical text
pub struct KeyboardHandler {
    /// Current editor mode
    mode: EditorMode,
    /// Text direction (affects navigation mapping)
    direction: TextDirection,
    /// Command buffer (for multi-key commands)
    command_buffer: String,
    /// Count prefix (e.g., "3j" for move down 3 times)
    count_prefix: Option<usize>,
    /// Key bindings for each mode
    bindings: HashMap<EditorMode, HashMap<String, EditorCommand>>,
    /// Command line buffer (for : commands)
    command_line: String,
}

impl KeyboardHandler {
    /// Create a new keyboard handler
    pub fn new(direction: TextDirection) -> Self {
        let mut handler = Self {
            mode: EditorMode::Normal,
            direction,
            command_buffer: String::new(),
            count_prefix: None,
            bindings: HashMap::new(),
            command_line: String::new(),
        };

        handler.setup_default_bindings();
        handler
    }

    /// Get the current mode
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Set the text direction (updates navigation bindings)
    pub fn set_direction(&mut self, direction: TextDirection) {
        self.direction = direction;
        self.setup_default_bindings();
    }

    /// Get the command line buffer (for command mode)
    pub fn command_line(&self) -> &str {
        &self.command_line
    }

    /// Setup default vim-like key bindings
    fn setup_default_bindings(&mut self) {
        self.bindings.clear();

        // === NORMAL MODE ===
        let mut normal_bindings = HashMap::new();

        // Mode changes
        normal_bindings.insert("i".to_string(), EditorCommand::EnterInsertMode);
        normal_bindings.insert("a".to_string(), EditorCommand::EnterInsertModeAfter);
        normal_bindings.insert("I".to_string(), EditorCommand::EnterInsertModeAtLineStart);
        normal_bindings.insert("A".to_string(), EditorCommand::EnterInsertModeAtLineEnd);
        normal_bindings.insert("o".to_string(), EditorCommand::EnterInsertModeNewLineBelow);
        normal_bindings.insert("O".to_string(), EditorCommand::EnterInsertModeNewLineAbove);
        normal_bindings.insert("v".to_string(), EditorCommand::EnterVisualMode);
        normal_bindings.insert("V".to_string(), EditorCommand::EnterVisualLineMode);
        normal_bindings.insert(":".to_string(), EditorCommand::EnterCommandMode);

        // Navigation - adapted for vertical text
        // In vertical Japanese text (tategaki):
        // - h/l move between columns (horizontally on screen)
        // - j/k move within a column (vertically on screen)
        if self.direction == TextDirection::VerticalTopToBottom {
            // Vertical text: j/k = up/down in column, h/l = between columns
            normal_bindings.insert("k".to_string(), EditorCommand::MoveUp);
            normal_bindings.insert("j".to_string(), EditorCommand::MoveDown);
            normal_bindings.insert("l".to_string(), EditorCommand::MoveLeft);  // Next column (visual left, RTL)
            normal_bindings.insert("h".to_string(), EditorCommand::MoveRight); // Prev column (visual right)
        } else {
            // Horizontal text: standard vim navigation
            normal_bindings.insert("k".to_string(), EditorCommand::MoveUp);
            normal_bindings.insert("j".to_string(), EditorCommand::MoveDown);
            normal_bindings.insert("h".to_string(), EditorCommand::MoveLeft);
            normal_bindings.insert("l".to_string(), EditorCommand::MoveRight);
        }

        // Arrow keys (always work in visual direction)
        normal_bindings.insert("Up".to_string(), EditorCommand::MoveUp);
        normal_bindings.insert("Down".to_string(), EditorCommand::MoveDown);
        normal_bindings.insert("Left".to_string(), EditorCommand::MoveLeft);
        normal_bindings.insert("Right".to_string(), EditorCommand::MoveRight);

        // Word movement
        normal_bindings.insert("w".to_string(), EditorCommand::MoveWordForward);
        normal_bindings.insert("b".to_string(), EditorCommand::MoveWordBackward);

        // Line movement
        normal_bindings.insert("0".to_string(), EditorCommand::MoveToLineStart);
        normal_bindings.insert("$".to_string(), EditorCommand::MoveToLineEnd);
        normal_bindings.insert("^".to_string(), EditorCommand::MoveToLineStart);

        // File movement
        normal_bindings.insert("gg".to_string(), EditorCommand::MoveToFileStart);
        normal_bindings.insert("G".to_string(), EditorCommand::MoveToFileEnd);

        // Deletion
        normal_bindings.insert("x".to_string(), EditorCommand::DeleteChar);
        normal_bindings.insert("X".to_string(), EditorCommand::DeleteCharBackward);
        normal_bindings.insert("dd".to_string(), EditorCommand::DeleteLine);
        normal_bindings.insert("dw".to_string(), EditorCommand::DeleteWord);
        normal_bindings.insert("D".to_string(), EditorCommand::DeleteToLineEnd);

        // Yank (copy)
        normal_bindings.insert("yy".to_string(), EditorCommand::YankLine);
        normal_bindings.insert("Y".to_string(), EditorCommand::YankLine);

        // Paste
        normal_bindings.insert("p".to_string(), EditorCommand::Paste);
        normal_bindings.insert("P".to_string(), EditorCommand::PasteBefore);

        // Undo/Redo
        normal_bindings.insert("u".to_string(), EditorCommand::Undo);
        normal_bindings.insert("Ctrl+r".to_string(), EditorCommand::Redo);

        self.bindings.insert(EditorMode::Normal, normal_bindings);

        // === INSERT MODE ===
        let mut insert_bindings = HashMap::new();
        insert_bindings.insert("Escape".to_string(), EditorCommand::EnterNormalMode);
        insert_bindings.insert("Backspace".to_string(), EditorCommand::DeleteCharBackward);
        insert_bindings.insert("Enter".to_string(), EditorCommand::InsertChar('\n'));
        // All other keys insert their character
        self.bindings.insert(EditorMode::Insert, insert_bindings);

        // === VISUAL MODE ===
        let mut visual_bindings = HashMap::new();
        visual_bindings.insert("Escape".to_string(), EditorCommand::EnterNormalMode);

        // Navigation (same as normal mode, but extends selection)
        if self.direction == TextDirection::VerticalTopToBottom {
            visual_bindings.insert("k".to_string(), EditorCommand::MoveUp);
            visual_bindings.insert("j".to_string(), EditorCommand::MoveDown);
            visual_bindings.insert("l".to_string(), EditorCommand::MoveLeft);
            visual_bindings.insert("h".to_string(), EditorCommand::MoveRight);
        } else {
            visual_bindings.insert("k".to_string(), EditorCommand::MoveUp);
            visual_bindings.insert("j".to_string(), EditorCommand::MoveDown);
            visual_bindings.insert("h".to_string(), EditorCommand::MoveLeft);
            visual_bindings.insert("l".to_string(), EditorCommand::MoveRight);
        }

        visual_bindings.insert("y".to_string(), EditorCommand::Yank);
        visual_bindings.insert("d".to_string(), EditorCommand::DeleteChar);
        visual_bindings.insert("x".to_string(), EditorCommand::DeleteChar);

        self.bindings.insert(EditorMode::Visual, visual_bindings.clone());
        self.bindings.insert(EditorMode::VisualLine, visual_bindings);

        // === COMMAND MODE ===
        let mut command_bindings = HashMap::new();
        command_bindings.insert("Escape".to_string(), EditorCommand::EnterNormalMode);
        command_bindings.insert("Enter".to_string(), EditorCommand::ExecuteCommand(String::new()));
        command_bindings.insert("Backspace".to_string(), EditorCommand::DeleteCharBackward);
        self.bindings.insert(EditorMode::Command, command_bindings);
    }

    /// Process a key input and return the corresponding command
    pub fn process_key(&mut self, input: KeyInput) -> Result<EditorCommand> {
        // Handle global shortcuts first (regardless of mode)
        if input.ctrl {
            match input.key.as_str() {
                "c" => return Ok(EditorCommand::EnterNormalMode),
                "q" => return Ok(EditorCommand::Quit),
                "s" => return Ok(EditorCommand::Save),
                _ => {}
            }
        }

        match self.mode {
            EditorMode::Normal => self.process_normal_mode(input),
            EditorMode::Insert => self.process_insert_mode(input),
            EditorMode::Visual | EditorMode::VisualLine => self.process_visual_mode(input),
            EditorMode::Command => self.process_command_mode(input),
        }
    }

    /// Process key in normal mode
    fn process_normal_mode(&mut self, input: KeyInput) -> Result<EditorCommand> {
        // Check for count prefix (e.g., "3j")
        if let Ok(digit) = input.key.parse::<usize>() {
            if digit > 0 || self.count_prefix.is_some() {
                let current = self.count_prefix.unwrap_or(0);
                self.count_prefix = Some(current * 10 + digit);
                return Ok(EditorCommand::NoOp);
            }
        }

        // Add to command buffer for multi-key commands
        self.command_buffer.push_str(&input.key);

        // Try to match command
        if let Some(bindings) = self.bindings.get(&EditorMode::Normal) {
            // First try exact match
            if let Some(command) = bindings.get(&self.command_buffer) {
                let cmd = command.clone();
                self.command_buffer.clear();
                let count = self.count_prefix.take().unwrap_or(1);

                // TODO: Repeat command 'count' times
                return Ok(cmd);
            }

            // Check if buffer could be a prefix of a valid command
            let has_potential_match = bindings.keys()
                .any(|k| k.starts_with(&self.command_buffer));

            if !has_potential_match {
                // No match possible, clear buffer
                self.command_buffer.clear();
                self.count_prefix = None;
            }
        }

        Ok(EditorCommand::NoOp)
    }

    /// Process key in insert mode
    fn process_insert_mode(&mut self, input: KeyInput) -> Result<EditorCommand> {
        if let Some(bindings) = self.bindings.get(&EditorMode::Insert) {
            if let Some(command) = bindings.get(&input.key) {
                return Ok(command.clone());
            }
        }

        // Any other key inserts text
        if input.key.len() == 1 {
            if let Some(ch) = input.key.chars().next() {
                return Ok(EditorCommand::InsertChar(ch));
            }
        }

        Ok(EditorCommand::NoOp)
    }

    /// Process key in visual mode
    fn process_visual_mode(&mut self, input: KeyInput) -> Result<EditorCommand> {
        if let Some(bindings) = self.bindings.get(&self.mode) {
            if let Some(command) = bindings.get(&input.key) {
                return Ok(command.clone());
            }
        }

        Ok(EditorCommand::NoOp)
    }

    /// Process key in command mode
    fn process_command_mode(&mut self, input: KeyInput) -> Result<EditorCommand> {
        match input.key.as_str() {
            "Escape" => {
                self.command_line.clear();
                return Ok(EditorCommand::EnterNormalMode);
            }
            "Enter" => {
                let cmd = self.parse_command_line();
                self.command_line.clear();
                return Ok(cmd);
            }
            "Backspace" => {
                self.command_line.pop();
                return Ok(EditorCommand::NoOp);
            }
            key if key.len() == 1 => {
                self.command_line.push_str(key);
                return Ok(EditorCommand::NoOp);
            }
            _ => return Ok(EditorCommand::NoOp),
        }
    }

    /// Parse command line (ex commands like :w, :q, etc.)
    fn parse_command_line(&self) -> EditorCommand {
        let cmd = self.command_line.trim();

        match cmd {
            "w" | "write" => EditorCommand::Save,
            "q" | "quit" => EditorCommand::Quit,
            "q!" | "quit!" => EditorCommand::QuitForce,
            "wq" | "x" => EditorCommand::SaveAndQuit,
            _ => EditorCommand::ExecuteCommand(cmd.to_string()),
        }
    }

    /// Execute an editor command and update mode if necessary
    pub fn execute_command(&mut self, command: &EditorCommand) -> Result<()> {
        match command {
            EditorCommand::EnterNormalMode => {
                self.mode = EditorMode::Normal;
                self.command_buffer.clear();
            }
            EditorCommand::EnterInsertMode |
            EditorCommand::EnterInsertModeAfter |
            EditorCommand::EnterInsertModeAtLineStart |
            EditorCommand::EnterInsertModeAtLineEnd |
            EditorCommand::EnterInsertModeNewLineBelow |
            EditorCommand::EnterInsertModeNewLineAbove => {
                self.mode = EditorMode::Insert;
            }
            EditorCommand::EnterVisualMode => {
                self.mode = EditorMode::Visual;
            }
            EditorCommand::EnterVisualLineMode => {
                self.mode = EditorMode::VisualLine;
            }
            EditorCommand::EnterCommandMode => {
                self.mode = EditorMode::Command;
                self.command_line.clear();
            }
            _ => {
                // Other commands don't change mode
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display_names() {
        assert_eq!(EditorMode::Normal.display_name(), "NORMAL");
        assert_eq!(EditorMode::Insert.display_name(), "INSERT");
        assert_eq!(EditorMode::Visual.display_name(), "VISUAL");
    }

    #[test]
    fn test_key_input_creation() {
        let key = KeyInput::new("h").with_ctrl();
        assert_eq!(key.key, "h");
        assert!(key.ctrl);
        assert!(!key.alt);
    }

    #[test]
    fn test_normal_mode_navigation() {
        let mut handler = KeyboardHandler::new(TextDirection::HorizontalLeftToRight);

        let cmd = handler.process_key(KeyInput::new("j")).unwrap();
        assert_eq!(cmd, EditorCommand::MoveDown);

        let cmd = handler.process_key(KeyInput::new("k")).unwrap();
        assert_eq!(cmd, EditorCommand::MoveUp);
    }

    #[test]
    fn test_mode_switching() {
        let mut handler = KeyboardHandler::new(TextDirection::VerticalTopToBottom);
        assert_eq!(handler.mode(), EditorMode::Normal);

        let cmd = handler.process_key(KeyInput::new("i")).unwrap();
        assert_eq!(cmd, EditorCommand::EnterInsertMode);
        handler.execute_command(&cmd).unwrap();
        assert_eq!(handler.mode(), EditorMode::Insert);

        let cmd = handler.process_key(KeyInput::new("Escape")).unwrap();
        handler.execute_command(&cmd).unwrap();
        assert_eq!(handler.mode(), EditorMode::Normal);
    }

    #[test]
    fn test_command_line_parsing() {
        let mut handler = KeyboardHandler::new(TextDirection::VerticalTopToBottom);

        // Enter command mode
        let cmd = handler.process_key(KeyInput::new(":")).unwrap();
        handler.execute_command(&cmd).unwrap();
        assert_eq!(handler.mode(), EditorMode::Command);

        // Type "wq"
        handler.process_key(KeyInput::new("w")).unwrap();
        handler.process_key(KeyInput::new("q")).unwrap();

        // Execute
        let cmd = handler.process_key(KeyInput::new("Enter")).unwrap();
        assert_eq!(cmd, EditorCommand::SaveAndQuit);
    }

    #[test]
    fn test_vertical_text_navigation() {
        let mut handler = KeyboardHandler::new(TextDirection::VerticalTopToBottom);

        // In vertical text, h/l switch
        let cmd = handler.process_key(KeyInput::new("l")).unwrap();
        assert_eq!(cmd, EditorCommand::MoveLeft); // Next column (visual left)

        let cmd = handler.process_key(KeyInput::new("h")).unwrap();
        assert_eq!(cmd, EditorCommand::MoveRight); // Prev column (visual right)
    }
}
