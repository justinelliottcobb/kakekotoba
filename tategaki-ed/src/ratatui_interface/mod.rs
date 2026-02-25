//! Ratatui terminal interface for vertical text editing
//!
//! This module provides a console-based editor interface using Ratatui with
//! Unicode support for vertical Japanese text editing in terminal environments.

#[cfg(feature = "ratatui")]
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

#[cfg(feature = "ratatui")]
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::japanese::{InputResult, JapaneseInputMethod};
use crate::spatial::{SpatialPosition, SpatialRange};
use crate::text_engine::{LayoutEngine, TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};

#[cfg(feature = "ratatui")]
pub mod cursor;
#[cfg(feature = "ratatui")]
pub mod editor;
#[cfg(feature = "ratatui")]
pub mod keyboard;
#[cfg(feature = "ratatui")]
pub mod renderer;

#[cfg(feature = "ratatui")]
pub use cursor::*;
#[cfg(feature = "ratatui")]
pub use editor::*;
#[cfg(feature = "ratatui")]
pub use keyboard::*;
#[cfg(feature = "ratatui")]
pub use renderer::*;

#[cfg(feature = "ratatui")]
/// Terminal-based vertical text editor
pub struct TerminalVerticalEditor {
    /// Text buffer
    buffer: VerticalTextBuffer,
    /// Cursor position
    cursor: TerminalCursor,
    /// Japanese input method
    ime: JapaneseInputMethod,
    /// Text renderer
    renderer: TerminalRenderer,
    /// Keyboard handler
    keyboard_handler: KeyboardHandler,
    /// Current selection range
    selection: Option<SpatialRange>,
    /// Viewport offset for scrolling
    viewport_offset: SpatialPosition,
    /// Terminal size
    terminal_size: (u16, u16),
    /// Editor configuration
    config: crate::EditorConfig,
    /// Current editing mode
    mode: EditingMode,
    /// Status message
    status_message: String,
    /// Show IME candidate window
    show_candidates: bool,
    /// IME candidates list state
    candidates_state: ListState,
}

#[cfg(feature = "ratatui")]
/// Editing modes for terminal interface
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditingMode {
    /// Normal navigation mode (like Vim normal mode)
    Normal,
    /// Insert text mode
    Insert,
    /// Visual selection mode
    Visual,
    /// Command mode
    Command,
}

impl Default for EditingMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(feature = "ratatui")]
impl TerminalVerticalEditor {
    /// Create a new terminal vertical editor
    pub fn new(config: crate::EditorConfig) -> Self {
        let direction = config.text_direction;
        let buffer = VerticalTextBuffer::new(direction);
        let cursor = TerminalCursor::new();
        let ime = JapaneseInputMethod::new();
        let renderer = TerminalRenderer::new(direction);
        let keyboard_handler = KeyboardHandler::new();

        Self {
            buffer,
            cursor,
            ime,
            renderer,
            keyboard_handler,
            selection: None,
            viewport_offset: SpatialPosition::origin(),
            terminal_size: (80, 24),
            config,
            mode: EditingMode::default(),
            status_message: "Tategaki Editor - Vertical Text Editor".to_string(),
            show_candidates: false,
            candidates_state: ListState::default(),
        }
    }

    /// Load text from string
    pub fn load_text(&mut self, text: &str) -> Result<()> {
        self.buffer = VerticalTextBuffer::from_text(text, self.config.text_direction)?;
        self.cursor.reset();
        self.selection = None;
        self.update_status("Text loaded");
        Ok(())
    }

    /// Get current text content
    pub fn text(&self) -> String {
        self.buffer.as_text()
    }

    /// Run the editor main loop
    pub fn run<B: Backend>(&mut self, backend: B) -> Result<()> {
        let mut terminal = ratatui::Terminal::new(backend)?;

        // Setup terminal
        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        terminal.clear()?;

        // Main event loop
        let result = self.event_loop(&mut terminal);

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    /// Main event loop
    fn event_loop<B: Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> Result<()> {
        loop {
            // Update terminal size
            self.terminal_size = terminal.size()?.into();

            // Render
            terminal.draw(|f| self.render(f))?;

            // Handle events
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.handle_key_event(key) {
                        Ok(true) => continue, // Continue editing
                        Ok(false) => break,   // Quit requested
                        Err(e) => {
                            self.update_status(&format!("Error: {}", e));
                            continue;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle keyboard events
    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        // Handle global commands first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') => return Ok(false), // Quit
                KeyCode::Char('s') => {
                    self.update_status("Save not implemented yet");
                    return Ok(true);
                }
                KeyCode::Char('t') => {
                    // Toggle text direction
                    self.toggle_text_direction();
                    return Ok(true);
                }
                _ => {}
            }
        }

        match self.mode {
            EditingMode::Normal => self.handle_normal_mode_key(key),
            EditingMode::Insert => self.handle_insert_mode_key(key),
            EditingMode::Visual => self.handle_visual_mode_key(key),
            EditingMode::Command => self.handle_command_mode_key(key),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            // Movement keys (Vim-like)
            KeyCode::Char('h') => {
                self.cursor.move_left(&self.config.text_direction);
                Ok(true)
            }
            KeyCode::Char('j') => {
                self.cursor.move_down(&self.config.text_direction);
                Ok(true)
            }
            KeyCode::Char('k') => {
                self.cursor.move_up(&self.config.text_direction);
                Ok(true)
            }
            KeyCode::Char('l') => {
                self.cursor.move_right(&self.config.text_direction);
                Ok(true)
            }
            KeyCode::Char('0') => {
                self.cursor.move_to_line_start();
                Ok(true)
            }
            KeyCode::Char('$') => {
                self.cursor.move_to_line_end();
                Ok(true)
            }

            // Mode switches
            KeyCode::Char('i') => {
                self.mode = EditingMode::Insert;
                self.update_status("-- INSERT --");
                Ok(true)
            }
            KeyCode::Char('v') => {
                self.mode = EditingMode::Visual;
                self.selection = Some(SpatialRange::at_position(self.cursor.position()));
                self.update_status("-- VISUAL --");
                Ok(true)
            }
            KeyCode::Char(':') => {
                self.mode = EditingMode::Command;
                self.update_status(":");
                Ok(true)
            }

            // Quit
            KeyCode::Char('q') => Ok(false),

            _ => Ok(true),
        }
    }

    /// Handle keys in insert mode
    fn handle_insert_mode_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = EditingMode::Normal;
                self.show_candidates = false;
                self.update_status("");
                Ok(true)
            }
            KeyCode::Char(ch) => {
                // Handle Japanese IME if enabled
                if self.config.enable_ime {
                    match self.ime.process_key_input(&ch.to_string())? {
                        InputResult::Commit(text) => {
                            self.insert_text_at_cursor(&text)?;
                            self.show_candidates = false;
                        }
                        InputResult::Compose(text) => {
                            self.update_status(&format!("Composing: {}", text));
                        }
                        InputResult::ShowCandidates(candidates) => {
                            self.show_candidates = true;
                            if !candidates.is_empty() {
                                self.candidates_state.select(Some(0));
                            }
                        }
                        InputResult::Cancel => {
                            self.show_candidates = false;
                            self.update_status("");
                        }
                        InputResult::NoOp => {
                            // Fall through to direct insertion
                            self.insert_text_at_cursor(&ch.to_string())?;
                        }
                    }
                } else {
                    // Direct character insertion
                    self.insert_text_at_cursor(&ch.to_string())?;
                }
                Ok(true)
            }
            KeyCode::Backspace => {
                self.handle_backspace()?;
                Ok(true)
            }
            KeyCode::Enter => {
                self.insert_text_at_cursor("\n")?;
                Ok(true)
            }
            KeyCode::Tab => {
                if self.show_candidates {
                    // Navigate candidates
                    self.select_next_candidate();
                } else {
                    self.insert_text_at_cursor("    ")?; // Insert 4 spaces
                }
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    /// Handle keys in visual mode
    fn handle_visual_mode_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = EditingMode::Normal;
                self.selection = None;
                self.update_status("");
                Ok(true)
            }
            // Movement keys extend selection
            KeyCode::Char('h') => {
                self.cursor.move_left(&self.config.text_direction);
                self.extend_selection();
                Ok(true)
            }
            KeyCode::Char('j') => {
                self.cursor.move_down(&self.config.text_direction);
                self.extend_selection();
                Ok(true)
            }
            KeyCode::Char('k') => {
                self.cursor.move_up(&self.config.text_direction);
                self.extend_selection();
                Ok(true)
            }
            KeyCode::Char('l') => {
                self.cursor.move_right(&self.config.text_direction);
                self.extend_selection();
                Ok(true)
            }
            _ => Ok(true),
        }
    }

    /// Handle keys in command mode
    fn handle_command_mode_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = EditingMode::Normal;
                self.update_status("");
                Ok(true)
            }
            KeyCode::Char('q') => Ok(false), // :q command
            _ => Ok(true),
        }
    }

    /// Insert text at cursor position
    fn insert_text_at_cursor(&mut self, text: &str) -> Result<()> {
        // TODO: Implement actual text insertion
        // For now, just advance cursor
        for _ in text.chars() {
            self.cursor.advance();
        }
        Ok(())
    }

    /// Handle backspace
    fn handle_backspace(&mut self) -> Result<()> {
        // TODO: Implement backspace
        self.cursor.move_backward();
        Ok(())
    }

    /// Extend visual selection to current cursor position
    fn extend_selection(&mut self) {
        if let Some(ref mut selection) = self.selection {
            selection.end = self.cursor.position();
        }
    }

    /// Select next IME candidate
    fn select_next_candidate(&mut self) {
        let candidates = self.ime.candidates();
        if !candidates.is_empty() {
            let current = self.candidates_state.selected().unwrap_or(0);
            let next = (current + 1) % candidates.len();
            self.candidates_state.select(Some(next));
        }
    }

    /// Toggle text direction
    fn toggle_text_direction(&mut self) {
        self.config.text_direction = match self.config.text_direction {
            TextDirection::VerticalTopToBottom => TextDirection::HorizontalLeftToRight,
            TextDirection::HorizontalLeftToRight => TextDirection::VerticalTopToBottom,
            _ => TextDirection::VerticalTopToBottom,
        };

        // Update buffer direction
        self.buffer.set_direction(self.config.text_direction);

        // Update renderer direction
        self.renderer.set_direction(self.config.text_direction);

        self.update_status(&format!("Direction: {:?}", self.config.text_direction));
    }

    /// Update status message
    fn update_status(&mut self, message: &str) {
        self.status_message = message.to_string();
    }

    /// Render the editor interface
    fn render(&mut self, f: &mut Frame) {
        let size = f.size();

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header
                Constraint::Min(1),    // Main editor area
                Constraint::Length(1), // Status line
            ])
            .split(size);

        // Render header
        self.render_header(f, chunks[0]);

        // Render main editor area
        if self.show_candidates {
            self.render_editor_with_candidates(f, chunks[1]);
        } else {
            self.render_editor(f, chunks[1]);
        }

        // Render status line
        self.render_status_line(f, chunks[2]);
    }

    /// Render header
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            "Tategaki Editor - {} Mode",
            match self.mode {
                EditingMode::Normal => "NORMAL",
                EditingMode::Insert => "INSERT",
                EditingMode::Visual => "VISUAL",
                EditingMode::Command => "COMMAND",
            }
        );

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::White).bg(Color::Blue))
            .block(Block::default());

        f.render_widget(header, area);
    }

    /// Render main editor area
    fn render_editor(&mut self, f: &mut Frame, area: Rect) {
        self.renderer.render_buffer(
            &self.buffer,
            f,
            area,
            &mut self.cursor,
            self.selection.as_ref(),
        );
    }

    /// Render editor with IME candidate window
    fn render_editor_with_candidates(&mut self, f: &mut Frame, area: Rect) {
        // Split area for editor and candidates
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(70), // Editor
                Constraint::Percentage(30), // Candidates
            ])
            .split(area);

        // Render editor
        self.renderer.render_buffer(
            &self.buffer,
            f,
            chunks[0],
            &mut self.cursor,
            self.selection.as_ref(),
        );

        // Render candidates
        self.render_candidates(f, chunks[1]);
    }

    /// Render IME candidates window
    fn render_candidates(&mut self, f: &mut Frame, area: Rect) {
        let candidates = self.ime.candidates();
        let items: Vec<ListItem> = candidates
            .iter()
            .enumerate()
            .map(|(i, candidate)| {
                let content = if let Some(ref reading) = candidate.reading {
                    format!("{}. {} ({})", i + 1, candidate.text, reading)
                } else {
                    format!("{}. {}", i + 1, candidate.text)
                };
                ListItem::new(content)
            })
            .collect();

        let candidates_list = List::new(items)
            .block(Block::default().title("Candidates").borders(Borders::ALL))
            .highlight_style(Style::default().bg(Color::DarkGray));

        f.render_stateful_widget(candidates_list, area, &mut self.candidates_state);
    }

    /// Render status line
    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        let pos = self.cursor.position();
        let status = format!(
            "{} | Pos: {}:{} | Dir: {:?}",
            self.status_message, pos.column, pos.row, self.config.text_direction
        );

        let status_line =
            Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::Black));

        f.render_widget(status_line, area);
    }
}

#[cfg(not(feature = "ratatui"))]
/// Placeholder when Ratatui feature is disabled
pub struct TerminalVerticalEditor {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "ratatui"))]
impl TerminalVerticalEditor {
    pub fn new(_config: crate::EditorConfig) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn load_text(&mut self, _text: &str) -> Result<()> {
        Err(TategakiError::Rendering(
            "Ratatui feature not enabled".to_string(),
        ))
    }

    pub fn run<B>(&mut self, _backend: B) -> Result<()> {
        Err(TategakiError::Rendering(
            "Ratatui feature not enabled".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "ratatui")]
    fn test_terminal_editor_creation() {
        let config = crate::EditorConfig::default();
        let editor = TerminalVerticalEditor::new(config);
        assert_eq!(editor.text(), "");
        assert_eq!(editor.mode, EditingMode::Normal);
    }

    #[test]
    #[cfg(not(feature = "ratatui"))]
    fn test_terminal_editor_disabled() {
        let config = crate::EditorConfig::default();
        let mut editor = TerminalVerticalEditor::new(config);
        assert!(editor.load_text("test").is_err());
    }
}
