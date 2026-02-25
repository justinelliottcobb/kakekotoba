//! Keyboard input handling for terminal interface

#[cfg(feature = "ratatui")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::spatial::SpatialPosition;
use crate::text_engine::TextDirection;
use crate::{Result, TategakiError};

#[cfg(feature = "ratatui")]
/// Keyboard input handler for terminal editor
pub struct KeyboardHandler {
    /// Current key bindings
    bindings: KeyBindings,
    /// Key sequence buffer for multi-key commands
    key_sequence: Vec<KeyCode>,
    /// Maximum sequence length
    max_sequence_length: usize,
    /// Timeout for sequence completion (in milliseconds)
    sequence_timeout: u64,
}

#[cfg(feature = "ratatui")]
/// Key binding configuration
#[derive(Debug, Clone)]
pub struct KeyBindings {
    /// Movement keys
    pub movement: MovementKeys,
    /// Editing keys
    pub editing: EditingKeys,
    /// Mode switching keys
    pub mode_switch: ModeSwitchKeys,
    /// Special function keys
    pub functions: FunctionKeys,
}

#[cfg(feature = "ratatui")]
/// Movement key bindings
#[derive(Debug, Clone)]
pub struct MovementKeys {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub word_left: KeyCode,
    pub word_right: KeyCode,
    pub line_start: KeyCode,
    pub line_end: KeyCode,
    pub page_up: KeyCode,
    pub page_down: KeyCode,
    pub document_start: KeyCode,
    pub document_end: KeyCode,
}

#[cfg(feature = "ratatui")]
/// Editing key bindings
#[derive(Debug, Clone)]
pub struct EditingKeys {
    pub insert_char: Option<KeyCode>, // None means any character
    pub backspace: KeyCode,
    pub delete: KeyCode,
    pub new_line: KeyCode,
    pub tab: KeyCode,
    pub undo: KeyCode,
    pub redo: KeyCode,
    pub cut: KeyCode,
    pub copy: KeyCode,
    pub paste: KeyCode,
}

#[cfg(feature = "ratatui")]
/// Mode switching key bindings
#[derive(Debug, Clone)]
pub struct ModeSwitchKeys {
    pub to_normal: KeyCode,
    pub to_insert: KeyCode,
    pub to_visual: KeyCode,
    pub to_command: KeyCode,
    pub append: KeyCode,
    pub open_below: KeyCode,
    pub open_above: KeyCode,
}

#[cfg(feature = "ratatui")]
/// Function key bindings
#[derive(Debug, Clone)]
pub struct FunctionKeys {
    pub save: KeyCode,
    pub quit: KeyCode,
    pub force_quit: KeyCode,
    pub find: KeyCode,
    pub replace: KeyCode,
    pub toggle_direction: KeyCode,
    pub toggle_ime: KeyCode,
    pub help: KeyCode,
}

#[cfg(feature = "ratatui")]
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            movement: MovementKeys {
                up: KeyCode::Char('k'),
                down: KeyCode::Char('j'),
                left: KeyCode::Char('h'),
                right: KeyCode::Char('l'),
                word_left: KeyCode::Char('b'),
                word_right: KeyCode::Char('w'),
                line_start: KeyCode::Char('0'),
                line_end: KeyCode::Char('$'),
                page_up: KeyCode::PageUp,
                page_down: KeyCode::PageDown,
                document_start: KeyCode::Char('g'),
                document_end: KeyCode::Char('G'),
            },
            editing: EditingKeys {
                insert_char: None,
                backspace: KeyCode::Backspace,
                delete: KeyCode::Delete,
                new_line: KeyCode::Enter,
                tab: KeyCode::Tab,
                undo: KeyCode::Char('u'),
                redo: KeyCode::Char('r'),
                cut: KeyCode::Char('x'),
                copy: KeyCode::Char('y'),
                paste: KeyCode::Char('p'),
            },
            mode_switch: ModeSwitchKeys {
                to_normal: KeyCode::Esc,
                to_insert: KeyCode::Char('i'),
                to_visual: KeyCode::Char('v'),
                to_command: KeyCode::Char(':'),
                append: KeyCode::Char('a'),
                open_below: KeyCode::Char('o'),
                open_above: KeyCode::Char('O'),
            },
            functions: FunctionKeys {
                save: KeyCode::Char('s'),
                quit: KeyCode::Char('q'),
                force_quit: KeyCode::Char('Q'),
                find: KeyCode::Char('/'),
                replace: KeyCode::Char('?'),
                toggle_direction: KeyCode::Char('t'),
                toggle_ime: KeyCode::Char('I'),
                help: KeyCode::F(1),
            },
        }
    }
}

#[cfg(feature = "ratatui")]
/// Keyboard input action
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    // Movement actions
    MoveCursor(CursorMovement),
    ScrollView(ScrollDirection),

    // Text editing actions
    InsertChar(char),
    InsertText(String),
    DeleteChar,
    DeleteBackward,
    InsertNewline,
    InsertTab,

    // Selection actions
    StartSelection,
    ExtendSelection(CursorMovement),
    ClearSelection,

    // Mode changes
    SwitchMode(EditingMode),

    // Clipboard operations
    Cut,
    Copy,
    Paste,

    // File operations
    Save,
    Quit,
    ForceQuit,

    // Search operations
    Find(String),
    Replace(String, String),

    // Settings toggles
    ToggleTextDirection,
    ToggleIME,
    ShowHelp,

    // Special actions
    Undo,
    Redo,
    NoOp,

    // Command mode
    ExecuteCommand(String),
}

#[cfg(feature = "ratatui")]
/// Cursor movement directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorMovement {
    Up,
    Down,
    Left,
    Right,
    WordLeft,
    WordRight,
    LineStart,
    LineEnd,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
}

#[cfg(feature = "ratatui")]
/// Scroll directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
}

#[cfg(feature = "ratatui")]
/// Editor modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditingMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[cfg(feature = "ratatui")]
impl KeyboardHandler {
    /// Create new keyboard handler
    pub fn new() -> Self {
        Self {
            bindings: KeyBindings::default(),
            key_sequence: Vec::new(),
            max_sequence_length: 3,
            sequence_timeout: 1000, // 1 second
        }
    }

    /// Set key bindings
    pub fn set_bindings(&mut self, bindings: KeyBindings) {
        self.bindings = bindings;
    }

    /// Process key event and return action
    pub fn process_key_event(
        &mut self,
        event: KeyEvent,
        current_mode: EditingMode,
        text_direction: TextDirection,
    ) -> Result<KeyAction> {
        // Handle global key combinations first
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            return self.handle_ctrl_key(event.code);
        }

        if event.modifiers.contains(KeyModifiers::ALT) {
            return self.handle_alt_key(event.code);
        }

        // Handle mode-specific keys
        match current_mode {
            EditingMode::Normal => self.handle_normal_mode_key(event.code, text_direction),
            EditingMode::Insert => self.handle_insert_mode_key(event.code),
            EditingMode::Visual => self.handle_visual_mode_key(event.code, text_direction),
            EditingMode::Command => self.handle_command_mode_key(event.code),
        }
    }

    /// Handle Control key combinations
    fn handle_ctrl_key(&self, key: KeyCode) -> Result<KeyAction> {
        match key {
            KeyCode::Char('c') => Ok(KeyAction::ForceQuit),
            KeyCode::Char('s') => Ok(KeyAction::Save),
            KeyCode::Char('q') => Ok(KeyAction::Quit),
            KeyCode::Char('f') => Ok(KeyAction::Find(String::new())),
            KeyCode::Char('t') => Ok(KeyAction::ToggleTextDirection),
            KeyCode::Char('i') => Ok(KeyAction::ToggleIME),
            KeyCode::Char('z') => Ok(KeyAction::Undo),
            KeyCode::Char('y') => Ok(KeyAction::Redo),
            KeyCode::Char('x') => Ok(KeyAction::Cut),
            KeyCode::Char('c') => Ok(KeyAction::Copy),
            KeyCode::Char('v') => Ok(KeyAction::Paste),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle Alt key combinations
    fn handle_alt_key(&self, key: KeyCode) -> Result<KeyAction> {
        match key {
            KeyCode::Left => Ok(KeyAction::MoveCursor(CursorMovement::WordLeft)),
            KeyCode::Right => Ok(KeyAction::MoveCursor(CursorMovement::WordRight)),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode_key(&self, key: KeyCode, direction: TextDirection) -> Result<KeyAction> {
        match key {
            // Movement
            k if k == self.bindings.movement.up => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Up, direction),
            )),
            k if k == self.bindings.movement.down => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Down, direction),
            )),
            k if k == self.bindings.movement.left => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Left, direction),
            )),
            k if k == self.bindings.movement.right => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Right, direction),
            )),
            k if k == self.bindings.movement.word_left => {
                Ok(KeyAction::MoveCursor(CursorMovement::WordLeft))
            }
            k if k == self.bindings.movement.word_right => {
                Ok(KeyAction::MoveCursor(CursorMovement::WordRight))
            }
            k if k == self.bindings.movement.line_start => {
                Ok(KeyAction::MoveCursor(CursorMovement::LineStart))
            }
            k if k == self.bindings.movement.line_end => {
                Ok(KeyAction::MoveCursor(CursorMovement::LineEnd))
            }
            k if k == self.bindings.movement.page_up => {
                Ok(KeyAction::MoveCursor(CursorMovement::PageUp))
            }
            k if k == self.bindings.movement.page_down => {
                Ok(KeyAction::MoveCursor(CursorMovement::PageDown))
            }

            // Mode switches
            k if k == self.bindings.mode_switch.to_insert => {
                Ok(KeyAction::SwitchMode(EditingMode::Insert))
            }
            k if k == self.bindings.mode_switch.to_visual => {
                Ok(KeyAction::SwitchMode(EditingMode::Visual))
            }
            k if k == self.bindings.mode_switch.to_command => {
                Ok(KeyAction::SwitchMode(EditingMode::Command))
            }

            // Functions
            k if k == self.bindings.functions.quit => Ok(KeyAction::Quit),

            KeyCode::Arrow(arrow) => self.handle_arrow_key(arrow, direction),

            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle keys in insert mode
    fn handle_insert_mode_key(&self, key: KeyCode) -> Result<KeyAction> {
        match key {
            k if k == self.bindings.mode_switch.to_normal => {
                Ok(KeyAction::SwitchMode(EditingMode::Normal))
            }
            KeyCode::Char(ch) => Ok(KeyAction::InsertChar(ch)),
            k if k == self.bindings.editing.backspace => Ok(KeyAction::DeleteBackward),
            k if k == self.bindings.editing.delete => Ok(KeyAction::DeleteChar),
            k if k == self.bindings.editing.new_line => Ok(KeyAction::InsertNewline),
            k if k == self.bindings.editing.tab => Ok(KeyAction::InsertTab),
            KeyCode::Arrow(arrow) => self.handle_arrow_key_insert(arrow),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle keys in visual mode
    fn handle_visual_mode_key(&self, key: KeyCode, direction: TextDirection) -> Result<KeyAction> {
        match key {
            k if k == self.bindings.mode_switch.to_normal => {
                Ok(KeyAction::SwitchMode(EditingMode::Normal))
            }
            // Movement keys extend selection
            k if k == self.bindings.movement.up => Ok(KeyAction::ExtendSelection(
                self.map_movement_to_direction(CursorMovement::Up, direction),
            )),
            k if k == self.bindings.movement.down => Ok(KeyAction::ExtendSelection(
                self.map_movement_to_direction(CursorMovement::Down, direction),
            )),
            k if k == self.bindings.movement.left => Ok(KeyAction::ExtendSelection(
                self.map_movement_to_direction(CursorMovement::Left, direction),
            )),
            k if k == self.bindings.movement.right => Ok(KeyAction::ExtendSelection(
                self.map_movement_to_direction(CursorMovement::Right, direction),
            )),
            // Copy/cut selected text
            k if k == self.bindings.editing.copy => Ok(KeyAction::Copy),
            k if k == self.bindings.editing.cut => Ok(KeyAction::Cut),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle keys in command mode
    fn handle_command_mode_key(&self, key: KeyCode) -> Result<KeyAction> {
        match key {
            k if k == self.bindings.mode_switch.to_normal => {
                Ok(KeyAction::SwitchMode(EditingMode::Normal))
            }
            KeyCode::Char(ch) => {
                // Build command string - this is simplified
                Ok(KeyAction::ExecuteCommand(ch.to_string()))
            }
            KeyCode::Enter => {
                // Execute current command - simplified
                Ok(KeyAction::SwitchMode(EditingMode::Normal))
            }
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle arrow keys
    fn handle_arrow_key(
        &self,
        arrow: crossterm::event::KeyCode,
        direction: TextDirection,
    ) -> Result<KeyAction> {
        match arrow {
            KeyCode::Up => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Up, direction),
            )),
            KeyCode::Down => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Down, direction),
            )),
            KeyCode::Left => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Left, direction),
            )),
            KeyCode::Right => Ok(KeyAction::MoveCursor(
                self.map_movement_to_direction(CursorMovement::Right, direction),
            )),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Handle arrow keys in insert mode
    fn handle_arrow_key_insert(&self, arrow: crossterm::event::KeyCode) -> Result<KeyAction> {
        // In insert mode, arrow keys just move cursor without mapping
        match arrow {
            KeyCode::Up => Ok(KeyAction::MoveCursor(CursorMovement::Up)),
            KeyCode::Down => Ok(KeyAction::MoveCursor(CursorMovement::Down)),
            KeyCode::Left => Ok(KeyAction::MoveCursor(CursorMovement::Left)),
            KeyCode::Right => Ok(KeyAction::MoveCursor(CursorMovement::Right)),
            _ => Ok(KeyAction::NoOp),
        }
    }

    /// Map movement direction based on text direction
    fn map_movement_to_direction(
        &self,
        movement: CursorMovement,
        text_direction: TextDirection,
    ) -> CursorMovement {
        match text_direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, up/down stay the same, left/right are swapped for columns
                match movement {
                    CursorMovement::Left => CursorMovement::Right, // Move to next column (visually left)
                    CursorMovement::Right => CursorMovement::Left, // Move to prev column (visually right)
                    other => other,                                // Up/Down unchanged
                }
            }
            TextDirection::HorizontalLeftToRight => movement, // No mapping needed
        }
    }

    /// Check if key sequence is complete
    pub fn is_sequence_complete(&self) -> bool {
        self.key_sequence.len() >= self.max_sequence_length
    }

    /// Clear key sequence buffer
    pub fn clear_sequence(&mut self) {
        self.key_sequence.clear();
    }

    /// Get current key sequence
    pub fn current_sequence(&self) -> &[KeyCode] {
        &self.key_sequence
    }

    /// Get key binding description for help
    pub fn get_key_help(&self, mode: EditingMode) -> Vec<(String, String)> {
        let mut help = Vec::new();

        match mode {
            EditingMode::Normal => {
                help.push(("h/j/k/l".to_string(), "Move cursor".to_string()));
                help.push(("i".to_string(), "Enter insert mode".to_string()));
                help.push(("v".to_string(), "Enter visual mode".to_string()));
                help.push((":".to_string(), "Enter command mode".to_string()));
                help.push(("q".to_string(), "Quit".to_string()));
            }
            EditingMode::Insert => {
                help.push(("Esc".to_string(), "Return to normal mode".to_string()));
                help.push(("Backspace".to_string(), "Delete character".to_string()));
                help.push(("Enter".to_string(), "New line".to_string()));
            }
            EditingMode::Visual => {
                help.push(("Esc".to_string(), "Return to normal mode".to_string()));
                help.push(("h/j/k/l".to_string(), "Extend selection".to_string()));
                help.push(("y".to_string(), "Copy selection".to_string()));
                help.push(("x".to_string(), "Cut selection".to_string()));
            }
            EditingMode::Command => {
                help.push(("Esc".to_string(), "Cancel command".to_string()));
                help.push(("q".to_string(), "Quit".to_string()));
                help.push(("w".to_string(), "Save".to_string()));
            }
        }

        help
    }
}

#[cfg(feature = "ratatui")]
impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "ratatui"))]
/// Placeholder keyboard handler when Ratatui is disabled
pub struct KeyboardHandler {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "ratatui"))]
impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}
