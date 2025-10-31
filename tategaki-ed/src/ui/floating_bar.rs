//! Floating command bar implementation
//!
//! Provides a floating overlay bar for commands that doesn't interfere with vertical text flow.

use crate::{Result, TategakiError};
use crate::{FloatingBarConfig, FloatingPosition, FloatingBarStyle, BorderStyle};
use crate::{HorizontalAnchor, VerticalAnchor};
use crate::backend::{Color, Rect};
use crate::spatial::SpatialPosition;

/// Floating command bar state
pub struct FloatingCommandBar {
    /// Configuration
    config: FloatingBarConfig,
    /// Current visibility
    visible: bool,
    /// Current content
    content: String,
    /// Current mode
    mode: CommandBarMode,
    /// Command history
    history: Vec<String>,
    /// History navigation index
    history_index: Option<usize>,
    /// Suggestions list
    suggestions: Vec<String>,
    /// Selected suggestion index
    selected_suggestion: Option<usize>,
}

/// Command bar display modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBarMode {
    /// Hidden
    Hidden,
    /// Command input (: commands)
    CommandInput,
    /// Search input (/ search)
    Search,
    /// Quick help
    QuickHelp,
}

impl FloatingCommandBar {
    /// Create a new floating command bar
    pub fn new(config: FloatingBarConfig) -> Self {
        Self {
            config,
            visible: false,
            content: String::new(),
            mode: CommandBarMode::Hidden,
            history: Vec::new(),
            history_index: None,
            suggestions: Vec::new(),
            selected_suggestion: None,
        }
    }

    /// Show the command bar in a specific mode
    pub fn show(&mut self, mode: CommandBarMode) {
        self.visible = true;
        self.mode = mode;
        self.content.clear();
        self.history_index = None;
        self.update_suggestions();
    }

    /// Hide the command bar
    pub fn hide(&mut self) {
        self.visible = false;
        self.mode = CommandBarMode::Hidden;
        self.content.clear();
        self.suggestions.clear();
    }

    /// Check if the command bar is visible
    pub fn is_visible(&self) -> bool {
        self.visible && self.config.enabled
    }

    /// Get the current content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the content
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.update_suggestions();
    }

    /// Add a character to the content
    pub fn push_char(&mut self, ch: char) {
        self.content.push(ch);
        self.update_suggestions();
    }

    /// Remove the last character
    pub fn pop_char(&mut self) {
        self.content.pop();
        self.update_suggestions();
    }

    /// Get the current mode
    pub fn mode(&self) -> CommandBarMode {
        self.mode
    }

    /// Navigate history up
    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        if let Some(idx) = self.history_index {
            if idx > 0 {
                self.history_index = Some(idx - 1);
                self.content = self.history[idx - 1].clone();
            }
        } else {
            // Start from the end
            self.history_index = Some(self.history.len() - 1);
            self.content = self.history[self.history.len() - 1].clone();
        }
    }

    /// Navigate history down
    pub fn history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx < self.history.len() - 1 {
                self.history_index = Some(idx + 1);
                self.content = self.history[idx + 1].clone();
            } else {
                // Clear to empty
                self.history_index = None;
                self.content.clear();
            }
        }
    }

    /// Add command to history
    pub fn add_to_history(&mut self, command: String) {
        if !command.is_empty() && (self.history.is_empty() || self.history.last() != Some(&command)) {
            self.history.push(command);
            // Keep history limited to 100 entries
            if self.history.len() > 100 {
                self.history.remove(0);
            }
        }
    }

    /// Update suggestions based on current content
    fn update_suggestions(&mut self) {
        if !self.config.show_suggestions {
            self.suggestions.clear();
            return;
        }

        // Simple suggestion system - in a real implementation, this would be more sophisticated
        self.suggestions.clear();

        if self.mode == CommandBarMode::CommandInput {
            let input = self.content.to_lowercase();

            // Common vim commands
            let common_commands = vec![
                "w", "write", "q", "quit", "wq", "x",
                "e", "edit", "sp", "split", "vs", "vsplit",
                "tabnew", "tabclose", "set", "help",
            ];

            for cmd in common_commands {
                if cmd.starts_with(&input) {
                    self.suggestions.push(cmd.to_string());
                }
            }
        }
    }

    /// Calculate the position and size of the floating bar
    pub fn calculate_bounds(&self, viewport_width: u32, viewport_height: u32, cursor_pos: &SpatialPosition) -> (usize, usize, usize, usize) {
        let content_width = self.calculate_content_width();
        let width = content_width.max(self.config.style.min_width);
        let width = if let Some(max) = self.config.style.max_width {
            width.min(max)
        } else {
            width
        };

        let height = self.calculate_content_height();

        let (x, y) = match &self.config.position {
            FloatingPosition::Center => {
                let x = (viewport_width as usize).saturating_sub(width) / 2;
                let y = (viewport_height as usize).saturating_sub(height) / 2;
                (x, y)
            }
            FloatingPosition::TopCenter { offset_y } => {
                let x = (viewport_width as usize).saturating_sub(width) / 2;
                (x, *offset_y)
            }
            FloatingPosition::BottomCenter { offset_y } => {
                let x = (viewport_width as usize).saturating_sub(width) / 2;
                let y = (viewport_height as usize).saturating_sub(height + offset_y);
                (x, y)
            }
            FloatingPosition::Absolute { x, y } => (*x, *y),
            FloatingPosition::NearCursor { offset_x, offset_y } => {
                let x = (cursor_pos.column as isize + offset_x).max(0) as usize;
                let y = (cursor_pos.row as isize + offset_y).max(0) as usize;
                (x, y)
            }
            FloatingPosition::Anchored { horizontal, vertical, offset_x, offset_y } => {
                let x = match horizontal {
                    HorizontalAnchor::Left => *offset_x,
                    HorizontalAnchor::Center => (viewport_width as isize - width as isize) / 2 + offset_x,
                    HorizontalAnchor::Right => viewport_width as isize - width as isize + offset_x,
                }.max(0) as usize;

                let y = match vertical {
                    VerticalAnchor::Top => *offset_y,
                    VerticalAnchor::Middle => (viewport_height as isize - height as isize) / 2 + offset_y,
                    VerticalAnchor::Bottom => viewport_height as isize - height as isize + offset_y,
                }.max(0) as usize;

                (x, y)
            }
        };

        (x, y, width, height)
    }

    /// Calculate content width based on current content and styling
    fn calculate_content_width(&self) -> usize {
        let (left_pad, right_pad, _, _) = self.config.style.padding;
        let border_width = if self.config.style.border == BorderStyle::None { 0 } else { 2 };

        let content_width = self.content.len();
        let suggestion_width = self.suggestions.iter()
            .map(|s| s.len())
            .max()
            .unwrap_or(0);

        content_width.max(suggestion_width) + left_pad + right_pad + border_width
    }

    /// Calculate content height based on visible elements
    fn calculate_content_height(&self) -> usize {
        let (_, _, top_pad, bottom_pad) = self.config.style.padding;
        let border_height = if self.config.style.border == BorderStyle::None { 0 } else { 2 };

        let mut height = 1; // Input line

        if self.config.show_suggestions && !self.suggestions.is_empty() {
            height += 1; // Separator
            height += self.suggestions.len().min(5); // Up to 5 suggestions
        }

        height + top_pad + bottom_pad + border_height
    }

    /// Get border characters for the current style
    pub fn get_border_chars(&self) -> Option<[char; 8]> {
        match self.config.style.border {
            BorderStyle::None => None,
            BorderStyle::Single => Some(['┌', '─', '┐', '│', '┘', '─', '└', '│']),
            BorderStyle::Double => Some(['╔', '═', '╗', '║', '╝', '═', '╚', '║']),
            BorderStyle::Rounded => Some(['╭', '─', '╮', '│', '╯', '─', '╰', '│']),
            BorderStyle::Thick => Some(['┏', '━', '┓', '┃', '┛', '━', '┗', '┃']),
        }
    }

    /// Get the formatted content lines for rendering
    pub fn get_content_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Input line with mode indicator
        let mode_char = match self.mode {
            CommandBarMode::CommandInput => ':',
            CommandBarMode::Search => '/',
            CommandBarMode::QuickHelp => '?',
            CommandBarMode::Hidden => ' ',
        };

        lines.push(format!("{} {}", mode_char, self.content));

        // Suggestions
        if self.config.show_suggestions && !self.suggestions.is_empty() {
            lines.push("─────────────".to_string());
            for (i, suggestion) in self.suggestions.iter().enumerate().take(5) {
                let prefix = if Some(i) == self.selected_suggestion { ">" } else { " " };
                lines.push(format!("{} :{}", prefix, suggestion));
            }
        }

        lines
    }

    /// Get background color with alpha
    pub fn get_background_color(&self) -> Result<Color> {
        let base_color = Color::from_hex(&self.config.style.background)?;
        Ok(Color::new(
            base_color.r,
            base_color.g,
            base_color.b,
            self.config.style.background_alpha,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_bar_creation() {
        let config = FloatingBarConfig::default();
        let bar = FloatingCommandBar::new(config);
        assert!(!bar.is_visible());
        assert_eq!(bar.mode(), CommandBarMode::Hidden);
    }

    #[test]
    fn test_show_hide() {
        let config = FloatingBarConfig::default();
        let mut bar = FloatingCommandBar::new(config);

        bar.show(CommandBarMode::CommandInput);
        assert!(bar.is_visible());
        assert_eq!(bar.mode(), CommandBarMode::CommandInput);

        bar.hide();
        assert!(!bar.is_visible());
        assert_eq!(bar.mode(), CommandBarMode::Hidden);
    }

    #[test]
    fn test_content_manipulation() {
        let config = FloatingBarConfig::default();
        let mut bar = FloatingCommandBar::new(config);

        bar.show(CommandBarMode::CommandInput);
        bar.push_char('w');
        assert_eq!(bar.content(), "w");

        bar.push_char('q');
        assert_eq!(bar.content(), "wq");

        bar.pop_char();
        assert_eq!(bar.content(), "w");
    }

    #[test]
    fn test_history() {
        let config = FloatingBarConfig::default();
        let mut bar = FloatingCommandBar::new(config);

        bar.add_to_history("write".to_string());
        bar.add_to_history("quit".to_string());

        bar.show(CommandBarMode::CommandInput);
        bar.history_up();
        assert_eq!(bar.content(), "quit");

        bar.history_up();
        assert_eq!(bar.content(), "write");

        bar.history_down();
        assert_eq!(bar.content(), "quit");
    }

    #[test]
    fn test_border_chars() {
        let mut config = FloatingBarConfig::default();
        let bar = FloatingCommandBar::new(config.clone());

        // Rounded borders
        let chars = bar.get_border_chars().unwrap();
        assert_eq!(chars[0], '╭'); // Top-left

        // Single borders
        config.style.border = BorderStyle::Single;
        let bar = FloatingCommandBar::new(config);
        let chars = bar.get_border_chars().unwrap();
        assert_eq!(chars[0], '┌'); // Top-left
    }
}
