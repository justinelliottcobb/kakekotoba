//! Core terminal editor components

#[cfg(feature = "ratatui")]
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::spatial::{SpatialPosition, SpatialRange};
use crate::text_engine::{TextDirection, VerticalTextBuffer};
use crate::{Result, TategakiError};

#[cfg(feature = "ratatui")]
/// Terminal editor widget for vertical text
pub struct TerminalEditor {
    /// Current scroll offset
    scroll_offset: SpatialPosition,
    /// Whether to show line numbers
    show_line_numbers: bool,
    /// Line number width
    line_number_width: u16,
    /// Editor background color
    background_color: Color,
    /// Text color
    text_color: Color,
    /// Selection color
    selection_color: Color,
    /// Border style
    border_style: Style,
}

#[cfg(feature = "ratatui")]
impl TerminalEditor {
    /// Create new terminal editor
    pub fn new() -> Self {
        Self {
            scroll_offset: SpatialPosition::origin(),
            show_line_numbers: true,
            line_number_width: 4,
            background_color: Color::Black,
            text_color: Color::White,
            selection_color: Color::Blue,
            border_style: Style::default().fg(Color::Gray),
        }
    }

    /// Set scroll offset
    pub fn set_scroll_offset(&mut self, offset: SpatialPosition) {
        self.scroll_offset = offset;
    }

    /// Get scroll offset
    pub fn scroll_offset(&self) -> SpatialPosition {
        self.scroll_offset
    }

    /// Set colors
    pub fn set_colors(&mut self, text: Color, background: Color, selection: Color) {
        self.text_color = text;
        self.background_color = background;
        self.selection_color = selection;
    }

    /// Set line number display
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Calculate visible area accounting for line numbers
    pub fn calculate_text_area(&self, area: Rect) -> Rect {
        if self.show_line_numbers {
            Rect {
                x: area.x + self.line_number_width + 1,
                y: area.y,
                width: area.width.saturating_sub(self.line_number_width + 1),
                height: area.height,
            }
        } else {
            area
        }
    }

    /// Render line numbers
    pub fn render_line_numbers(
        &self,
        f: &mut Frame,
        area: Rect,
        first_line: usize,
        line_count: usize,
    ) {
        if !self.show_line_numbers {
            return;
        }

        let line_number_area = Rect {
            x: area.x,
            y: area.y,
            width: self.line_number_width,
            height: area.height,
        };

        let mut lines = Vec::new();
        for i in 0..(area.height as usize).min(line_count) {
            let line_num = first_line + i + 1;
            lines.push(format!("{:>3}", line_num));
        }

        let line_numbers = Paragraph::new(lines.join("\n"))
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .bg(self.background_color),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(line_numbers, line_number_area);

        // Render separator
        let separator_area = Rect {
            x: area.x + self.line_number_width,
            y: area.y,
            width: 1,
            height: area.height,
        };

        let separator = Paragraph::new("│".repeat(area.height as usize))
            .style(Style::default().fg(Color::DarkGray));

        f.render_widget(separator, separator_area);
    }

    /// Calculate which lines are visible in the viewport
    pub fn calculate_visible_lines(&self, area: Rect, total_lines: usize) -> (usize, usize) {
        let start_line = self.scroll_offset.row;
        let visible_count = (area.height as usize).min(total_lines.saturating_sub(start_line));
        (start_line, visible_count)
    }

    /// Handle scrolling
    pub fn scroll(&mut self, delta_row: i32, delta_col: i32, max_lines: usize, max_cols: usize) {
        // Update row offset
        let new_row = (self.scroll_offset.row as i32 + delta_row)
            .max(0)
            .min(max_lines.saturating_sub(1) as i32) as usize;

        // Update column offset
        let new_col = (self.scroll_offset.column as i32 + delta_col)
            .max(0)
            .min(max_cols.saturating_sub(1) as i32) as usize;

        self.scroll_offset = SpatialPosition {
            row: new_row,
            column: new_col,
        };
    }

    /// Ensure position is visible by scrolling if necessary
    pub fn ensure_visible(&mut self, position: SpatialPosition, viewport_size: (u16, u16)) {
        let (viewport_width, viewport_height) = viewport_size;

        // Scroll vertically if needed
        if position.row < self.scroll_offset.row {
            self.scroll_offset.row = position.row;
        } else if position.row >= self.scroll_offset.row + (viewport_height as usize) {
            self.scroll_offset.row = position.row.saturating_sub((viewport_height as usize) - 1);
        }

        // Scroll horizontally if needed
        if position.column < self.scroll_offset.column {
            self.scroll_offset.column = position.column;
        } else if position.column >= self.scroll_offset.column + (viewport_width as usize) {
            self.scroll_offset.column = position
                .column
                .saturating_sub((viewport_width as usize) - 1);
        }
    }

    /// Convert screen coordinates to buffer position
    pub fn screen_to_buffer_position(
        &self,
        screen_x: u16,
        screen_y: u16,
        text_area: Rect,
        direction: TextDirection,
    ) -> SpatialPosition {
        let relative_x = screen_x.saturating_sub(text_area.x);
        let relative_y = screen_y.saturating_sub(text_area.y);

        match direction {
            TextDirection::VerticalTopToBottom => {
                // In vertical text, columns are visual X, rows are visual Y
                SpatialPosition {
                    row: self.scroll_offset.row + (relative_y as usize),
                    column: self.scroll_offset.column + (relative_x as usize),
                }
            }
            TextDirection::HorizontalLeftToRight => {
                // Standard horizontal layout
                SpatialPosition {
                    row: self.scroll_offset.row + (relative_y as usize),
                    column: self.scroll_offset.column + (relative_x as usize),
                }
            }
        }
    }

    /// Convert buffer position to screen coordinates
    pub fn buffer_to_screen_position(
        &self,
        position: SpatialPosition,
        text_area: Rect,
        direction: TextDirection,
    ) -> Option<(u16, u16)> {
        match direction {
            TextDirection::VerticalTopToBottom => {
                if position.row < self.scroll_offset.row
                    || position.column < self.scroll_offset.column
                {
                    return None; // Position is scrolled off screen
                }

                let relative_row = position.row.saturating_sub(self.scroll_offset.row);
                let relative_col = position.column.saturating_sub(self.scroll_offset.column);

                if relative_row >= text_area.height as usize
                    || relative_col >= text_area.width as usize
                {
                    return None; // Position is outside viewport
                }

                Some((
                    text_area.x + relative_col as u16,
                    text_area.y + relative_row as u16,
                ))
            }
            TextDirection::HorizontalLeftToRight => {
                if position.row < self.scroll_offset.row
                    || position.column < self.scroll_offset.column
                {
                    return None;
                }

                let relative_row = position.row.saturating_sub(self.scroll_offset.row);
                let relative_col = position.column.saturating_sub(self.scroll_offset.column);

                if relative_row >= text_area.height as usize
                    || relative_col >= text_area.width as usize
                {
                    return None;
                }

                Some((
                    text_area.x + relative_col as u16,
                    text_area.y + relative_row as u16,
                ))
            }
        }
    }

    /// Get editor statistics
    pub fn stats(&self) -> EditorStats {
        EditorStats {
            scroll_offset: self.scroll_offset,
            show_line_numbers: self.show_line_numbers,
            line_number_width: self.line_number_width,
        }
    }
}

#[cfg(feature = "ratatui")]
/// Editor statistics for debugging
#[derive(Debug, Clone)]
pub struct EditorStats {
    pub scroll_offset: SpatialPosition,
    pub show_line_numbers: bool,
    pub line_number_width: u16,
}

#[cfg(feature = "ratatui")]
impl Default for TerminalEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "ratatui")]
/// Layout calculator for terminal editor
pub struct TerminalLayout {
    /// Available area
    area: Rect,
    /// Header height
    header_height: u16,
    /// Status line height
    status_height: u16,
    /// Whether to show borders
    show_borders: bool,
}

#[cfg(feature = "ratatui")]
impl TerminalLayout {
    /// Create new layout calculator
    pub fn new(area: Rect) -> Self {
        Self {
            area,
            header_height: 1,
            status_height: 1,
            show_borders: true,
        }
    }

    /// Set header height
    pub fn set_header_height(&mut self, height: u16) {
        self.header_height = height;
    }

    /// Set status height
    pub fn set_status_height(&mut self, height: u16) {
        self.status_height = height;
    }

    /// Set border display
    pub fn set_show_borders(&mut self, show: bool) {
        self.show_borders = show;
    }

    /// Calculate layout areas
    pub fn calculate(&self) -> LayoutAreas {
        let border_offset = if self.show_borders { 1 } else { 0 };
        let available_height = self.area.height.saturating_sub(border_offset * 2);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(self.header_height),
                Constraint::Min(1), // Main editor area
                Constraint::Length(self.status_height),
            ])
            .split(Rect {
                x: self.area.x + border_offset,
                y: self.area.y + border_offset,
                width: self.area.width.saturating_sub(border_offset * 2),
                height: available_height,
            });

        LayoutAreas {
            full: self.area,
            header: chunks.get(0).copied().unwrap_or_default(),
            editor: chunks.get(1).copied().unwrap_or_default(),
            status: chunks.get(2).copied().unwrap_or_default(),
        }
    }
}

#[cfg(feature = "ratatui")]
/// Layout areas for terminal editor
#[derive(Debug, Clone, Copy)]
pub struct LayoutAreas {
    pub full: Rect,
    pub header: Rect,
    pub editor: Rect,
    pub status: Rect,
}

#[cfg(not(feature = "ratatui"))]
/// Placeholder editor when Ratatui is disabled
pub struct TerminalEditor {
    scroll_offset: SpatialPosition,
}

#[cfg(not(feature = "ratatui"))]
impl TerminalEditor {
    pub fn new() -> Self {
        Self {
            scroll_offset: SpatialPosition::origin(),
        }
    }

    pub fn set_scroll_offset(&mut self, offset: SpatialPosition) {
        self.scroll_offset = offset;
    }

    pub fn scroll_offset(&self) -> SpatialPosition {
        self.scroll_offset
    }
}
