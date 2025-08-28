//! Text rendering engine for terminal interface

#[cfg(feature = "ratatui")]
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{Result, TategakiError};
use crate::text_engine::{VerticalTextBuffer, TextDirection};
use crate::spatial::{SpatialPosition, SpatialRange};
use super::cursor::TerminalCursor;

#[cfg(feature = "ratatui")]
/// Terminal text renderer for vertical text
pub struct TerminalRenderer {
    /// Text direction
    direction: TextDirection,
    /// Normal text style
    text_style: Style,
    /// Selection highlight style
    selection_style: Style,
    /// Cursor style
    cursor_style: Style,
    /// Line number style
    line_number_style: Style,
    /// Syntax highlighting enabled
    syntax_highlighting: bool,
}

#[cfg(feature = "ratatui")]
impl TerminalRenderer {
    /// Create new terminal renderer
    pub fn new(direction: TextDirection) -> Self {
        Self {
            direction,
            text_style: Style::default().fg(Color::White),
            selection_style: Style::default().bg(Color::Blue).fg(Color::White),
            cursor_style: Style::default().bg(Color::Green).fg(Color::Black),
            line_number_style: Style::default().fg(Color::DarkGray),
            syntax_highlighting: false,
        }
    }

    /// Set text direction
    pub fn set_direction(&mut self, direction: TextDirection) {
        self.direction = direction;
    }

    /// Set text styles
    pub fn set_text_style(&mut self, style: Style) {
        self.text_style = style;
    }

    /// Set selection style
    pub fn set_selection_style(&mut self, style: Style) {
        self.selection_style = style;
    }

    /// Set cursor style
    pub fn set_cursor_style(&mut self, style: Style) {
        self.cursor_style = style;
    }

    /// Enable/disable syntax highlighting
    pub fn set_syntax_highlighting(&mut self, enabled: bool) {
        self.syntax_highlighting = enabled;
    }

    /// Render text buffer in the given area
    pub fn render_buffer(
        &self,
        buffer: &VerticalTextBuffer,
        f: &mut Frame,
        area: Rect,
        cursor: &mut TerminalCursor,
        selection: Option<&SpatialRange>,
    ) {
        match self.direction {
            TextDirection::VerticalTopToBottom => {
                self.render_vertical_text(buffer, f, area, cursor, selection);
            }
            TextDirection::HorizontalLeftToRight => {
                self.render_horizontal_text(buffer, f, area, cursor, selection);
            }
        }
    }

    /// Render text in vertical layout (top to bottom, right to left columns)
    fn render_vertical_text(
        &self,
        buffer: &VerticalTextBuffer,
        f: &mut Frame,
        area: Rect,
        cursor: &TerminalCursor,
        selection: Option<&SpatialRange>,
    ) {
        let text_content = buffer.as_text();
        let lines: Vec<&str> = text_content.lines().collect();
        
        // Calculate columns that fit in the area
        let chars_per_column = area.height as usize;
        let max_columns = area.width as usize;
        
        let mut rendered_text = Vec::new();
        let cursor_pos = cursor.position();
        
        // Process text column by column (right to left)
        for col_idx in 0..max_columns.min(lines.len()) {
            let line_idx = lines.len() - 1 - col_idx; // Right to left
            let line = lines.get(line_idx).unwrap_or(&"");
            
            let mut column_chars = Vec::new();
            
            // Process characters in this column (top to bottom)
            for (char_idx, ch) in line.chars().enumerate() {
                if char_idx >= chars_per_column {
                    break; // Column full
                }
                
                let char_pos = SpatialPosition {
                    row: char_idx,
                    column: col_idx,
                };
                
                let style = self.calculate_char_style(char_pos, cursor_pos, selection);
                
                column_chars.push(Span::styled(ch.to_string(), style));
            }
            
            // Pad column to full height
            while column_chars.len() < chars_per_column {
                column_chars.push(Span::styled(" ", self.text_style));
            }
            
            // Add column to rendered text
            if col_idx == 0 {
                rendered_text = column_chars;
            } else {
                // This is simplified - real vertical layout is more complex
                for (i, span) in column_chars.into_iter().enumerate() {
                    if let Some(line) = rendered_text.get_mut(i) {
                        // Combine spans - this is a simplification
                        let combined_text = format!("{} {}", line.content, span.content);
                        *line = Span::styled(combined_text, line.style);
                    }
                }
            }
        }
        
        // Convert spans to lines
        let text_lines: Vec<Line> = rendered_text
            .into_iter()
            .map(|span| Line::from(span))
            .collect();
        
        let paragraph = Paragraph::new(Text::from(text_lines))
            .wrap(Wrap { trim: false });
        
        f.render_widget(paragraph, area);
    }

    /// Render text in horizontal layout (left to right)
    fn render_horizontal_text(
        &self,
        buffer: &VerticalTextBuffer,
        f: &mut Frame,
        area: Rect,
        cursor: &TerminalCursor,
        selection: Option<&SpatialRange>,
    ) {
        let text_content = buffer.as_text();
        let lines: Vec<&str> = text_content.lines().collect();
        let cursor_pos = cursor.position();
        
        let mut rendered_lines = Vec::new();
        
        // Process each line
        for (line_idx, line) in lines.iter().enumerate().take(area.height as usize) {
            let mut line_spans = Vec::new();
            
            // Process each character in the line
            for (char_idx, ch) in line.chars().enumerate().take(area.width as usize) {
                let char_pos = SpatialPosition {
                    row: line_idx,
                    column: char_idx,
                };
                
                let style = self.calculate_char_style(char_pos, cursor_pos, selection);
                line_spans.push(Span::styled(ch.to_string(), style));
            }
            
            rendered_lines.push(Line::from(line_spans));
        }
        
        let paragraph = Paragraph::new(Text::from(rendered_lines))
            .wrap(Wrap { trim: false });
        
        f.render_widget(paragraph, area);
    }

    /// Calculate style for a character at given position
    fn calculate_char_style(
        &self,
        char_pos: SpatialPosition,
        cursor_pos: SpatialPosition,
        selection: Option<&SpatialRange>,
    ) -> Style {
        let mut style = self.text_style;
        
        // Check if character is in selection
        if let Some(sel) = selection {
            if self.is_position_in_selection(char_pos, sel) {
                style = self.selection_style;
            }
        }
        
        // Check if character is at cursor position
        if char_pos == cursor_pos {
            style = self.cursor_style;
        }
        
        // Apply syntax highlighting if enabled
        if self.syntax_highlighting {
            style = self.apply_syntax_highlighting(char_pos, style);
        }
        
        style
    }

    /// Check if position is within selection range
    fn is_position_in_selection(&self, pos: SpatialPosition, selection: &SpatialRange) -> bool {
        let (start, end) = selection.normalized();
        
        if pos.row < start.row || pos.row > end.row {
            return false;
        }
        
        if pos.row == start.row && pos.row == end.row {
            // Single line selection
            pos.column >= start.column && pos.column <= end.column
        } else if pos.row == start.row {
            // Start line of multi-line selection
            pos.column >= start.column
        } else if pos.row == end.row {
            // End line of multi-line selection
            pos.column <= end.column
        } else {
            // Middle line of multi-line selection
            true
        }
    }

    /// Apply syntax highlighting to character style
    fn apply_syntax_highlighting(&self, _pos: SpatialPosition, style: Style) -> Style {
        // TODO: Implement syntax highlighting for spatial programming
        // This would analyze the character/context for keywords, operators, etc.
        
        // For now, just return the original style
        style
    }

    /// Render cursor overlay
    pub fn render_cursor_overlay(
        &self,
        f: &mut Frame,
        area: Rect,
        cursor: &TerminalCursor,
        buffer: &VerticalTextBuffer,
    ) {
        if !cursor.is_visible() {
            return;
        }

        let cursor_pos = cursor.position();
        
        // Calculate screen position for cursor
        if let Some((screen_x, screen_y)) = self.buffer_to_screen_position(cursor_pos, area) {
            // Render cursor as highlighted character or block
            let cursor_area = Rect {
                x: screen_x,
                y: screen_y,
                width: 1,
                height: 1,
            };

            // Get character at cursor position
            let cursor_char = buffer.char_at(cursor_pos).unwrap_or(' ');
            
            let cursor_widget = Paragraph::new(cursor_char.to_string())
                .style(self.cursor_style);
            
            f.render_widget(cursor_widget, cursor_area);
        }
    }

    /// Convert buffer position to screen coordinates
    fn buffer_to_screen_position(&self, pos: SpatialPosition, area: Rect) -> Option<(u16, u16)> {
        match self.direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, row maps to Y, column maps to X (but reversed)
                if pos.row < area.height as usize && pos.column < area.width as usize {
                    let screen_x = area.x + (area.width - 1) - (pos.column as u16);
                    let screen_y = area.y + (pos.row as u16);
                    Some((screen_x, screen_y))
                } else {
                    None
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // Standard horizontal mapping
                if pos.row < area.height as usize && pos.column < area.width as usize {
                    let screen_x = area.x + (pos.column as u16);
                    let screen_y = area.y + (pos.row as u16);
                    Some((screen_x, screen_y))
                } else {
                    None
                }
            }
        }
    }

    /// Render debug information overlay
    pub fn render_debug_overlay(
        &self,
        f: &mut Frame,
        area: Rect,
        cursor: &TerminalCursor,
        selection: Option<&SpatialRange>,
    ) {
        // Render debug info in bottom-right corner
        let debug_area = Rect {
            x: area.x + area.width.saturating_sub(30),
            y: area.y + area.height.saturating_sub(5),
            width: 30,
            height: 5,
        };

        let cursor_pos = cursor.position();
        let debug_info = vec![
            format!("Pos: {}:{}", cursor_pos.column, cursor_pos.row),
            format!("Dir: {:?}", self.direction),
            if let Some(sel) = selection {
                format!("Sel: {}:{} - {}:{}", 
                    sel.start.column, sel.start.row,
                    sel.end.column, sel.end.row)
            } else {
                "No selection".to_string()
            },
            format!("Visible: {}", cursor.is_visible()),
        ];

        let debug_widget = Paragraph::new(debug_info.join("\n"))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Debug")
                .border_style(Style::default().fg(Color::Gray)));

        f.render_widget(debug_widget, debug_area);
    }

    /// Get renderer statistics
    pub fn stats(&self) -> RendererStats {
        RendererStats {
            direction: self.direction,
            syntax_highlighting: self.syntax_highlighting,
            text_color: match self.text_style.fg {
                Some(color) => format!("{:?}", color),
                None => "None".to_string(),
            },
            selection_color: match self.selection_style.bg {
                Some(color) => format!("{:?}", color),
                None => "None".to_string(),
            },
        }
    }
}

#[cfg(feature = "ratatui")]
/// Renderer statistics for debugging
#[derive(Debug, Clone)]
pub struct RendererStats {
    pub direction: TextDirection,
    pub syntax_highlighting: bool,
    pub text_color: String,
    pub selection_color: String,
}

#[cfg(feature = "ratatui")]
impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new(TextDirection::VerticalTopToBottom)
    }
}

#[cfg(not(feature = "ratatui"))]
/// Placeholder renderer when Ratatui is disabled
pub struct TerminalRenderer {
    direction: TextDirection,
}

#[cfg(not(feature = "ratatui"))]
impl TerminalRenderer {
    pub fn new(direction: TextDirection) -> Self {
        Self { direction }
    }

    pub fn set_direction(&mut self, direction: TextDirection) {
        self.direction = direction;
    }

    pub fn render_buffer(
        &self,
        _buffer: &VerticalTextBuffer,
        _f: (),
        _area: (),
        _cursor: &mut TerminalCursor,
        _selection: Option<&SpatialRange>,
    ) -> Result<()> {
        Err(TategakiError::Rendering("Ratatui feature not enabled".to_string()))
    }
}