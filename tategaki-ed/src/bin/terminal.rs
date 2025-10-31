//! Terminal-based vertical text editor using notcurses backend
//!
//! This binary provides a terminal UI for the tategaki vertical text editor,
//! using the notcurses rendering backend with vim-like keyboard controls.

use tategaki_ed::{
    EditorConfig, TextDirection, VerticalTextBuffer,
    backend::{
        RenderBackend, terminal::TerminalBackend,
        Color, TextStyle, Rect, CursorInfo, CursorStyle,
        EditorMode, EditorCommand, KeyboardHandler, KeyInput,
    },
    SpatialPosition, Result, TategakiError,
};
use std::path::PathBuf;

/// Command-line arguments
#[derive(Debug)]
struct Args {
    /// File to open
    file: Option<PathBuf>,
    /// Force vertical mode
    vertical: bool,
    /// Debug mode
    debug: bool,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut file = None;
        let mut vertical = true; // Default to vertical for this editor
        let mut debug = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--horizontal" | "-H" => vertical = false,
                "--vertical" | "-v" => vertical = true,
                "--debug" | "-d" => debug = true,
                "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    if !arg.starts_with('-') {
                        file = Some(PathBuf::from(arg));
                    }
                }
            }
        }

        Self { file, vertical, debug }
    }
}

fn print_help() {
    println!("Tategaki Terminal Editor - Vertical Text Editor with Vim Keybindings");
    println!();
    println!("USAGE:");
    println!("    tategaki-ed-terminal [OPTIONS] [FILE]");
    println!();
    println!("OPTIONS:");
    println!("    -v, --vertical     Use vertical text mode (default)");
    println!("    -H, --horizontal   Use horizontal text mode");
    println!("    -d, --debug        Enable debug mode");
    println!("    --help             Print this help message");
    println!();
    println!("VIM KEYBINDINGS:");
    println!("  Normal Mode:");
    println!("    h, j, k, l         Navigate (adapted for vertical text)");
    println!("    i                  Enter insert mode");
    println!("    a                  Insert after cursor");
    println!("    o                  Insert new line below");
    println!("    O                  Insert new line above");
    println!("    v                  Enter visual mode");
    println!("    x                  Delete character");
    println!("    dd                 Delete line");
    println!("    yy                 Yank (copy) line");
    println!("    p                  Paste");
    println!("    u                  Undo");
    println!("    Ctrl+R             Redo");
    println!("    :w                 Save");
    println!("    :q                 Quit");
    println!("    :wq                Save and quit");
    println!();
    println!("  Insert Mode:");
    println!("    Escape             Return to normal mode");
    println!("    Ctrl+C             Return to normal mode");
    println!();
    println!("  Global:");
    println!("    Ctrl+Q             Force quit");
    println!("    Ctrl+S             Save");
    println!();
}

/// Terminal editor with vim-like keyboard handling
struct TerminalEditor {
    /// Rendering backend
    backend: TerminalBackend,
    /// Text buffer (stores lines of text)
    lines: Vec<String>,
    /// File path (if loaded from file)
    file_path: Option<PathBuf>,
    /// Editor configuration
    config: EditorConfig,
    /// Cursor position (column, row in logical space)
    cursor: SpatialPosition,
    /// Keyboard handler (vim-like bindings)
    keyboard: KeyboardHandler,
    /// Running state
    running: bool,
    /// Modified flag (unsaved changes)
    modified: bool,
    /// Clipboard (for yank/paste)
    clipboard: Vec<String>,
    /// Message to display in status bar
    message: String,
    /// Debug mode
    debug: bool,
}

impl TerminalEditor {
    fn new(config: EditorConfig, debug: bool) -> Result<Self> {
        let backend = TerminalBackend::new()?;
        let direction = config.text_direction;
        let keyboard = KeyboardHandler::new(direction);
        let cursor = SpatialPosition { column: 0, row: 0, byte_offset: 0 };

        Ok(Self {
            backend,
            lines: vec![String::new()],
            file_path: None,
            config,
            cursor,
            keyboard,
            running: true,
            modified: false,
            clipboard: Vec::new(),
            message: String::new(),
            debug,
        })
    }

    fn load_file(&mut self, path: PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(&path).map_err(|e| {
            TategakiError::Io(e)
        })?;

        self.lines = content.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }

        self.file_path = Some(path);
        self.modified = false;
        self.message = format!("Loaded: {} ({} lines)",
            self.file_path.as_ref().unwrap().display(),
            self.lines.len()
        );

        Ok(())
    }

    fn save_file(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            let content = self.lines.join("\n");
            std::fs::write(path, content).map_err(|e| TategakiError::Io(e))?;
            self.modified = false;
            self.message = format!("Saved: {}", path.display());
            Ok(())
        } else {
            self.message = "No file name specified".to_string();
            Err(TategakiError::Format("No file path".to_string()))
        }
    }

    fn init(&mut self) -> Result<()> {
        self.backend.init()?;
        Ok(())
    }

    fn current_line(&self) -> &str {
        self.lines.get(self.cursor.row).map(|s| s.as_str()).unwrap_or("")
    }

    fn current_line_mut(&mut self) -> &mut String {
        let row = self.cursor.row;
        if row >= self.lines.len() {
            self.lines.resize(row + 1, String::new());
        }
        &mut self.lines[row]
    }

    fn clamp_cursor(&mut self) {
        // Ensure cursor is within bounds
        if self.cursor.row >= self.lines.len() {
            self.cursor.row = self.lines.len().saturating_sub(1);
        }

        let line_len = self.current_line().chars().count();
        if self.cursor.column > line_len {
            self.cursor.column = line_len;
        }
    }

    fn execute_command(&mut self, command: &EditorCommand) -> Result<bool> {
        match command {
            EditorCommand::NoOp => return Ok(false),

            // Mode changes (handled by keyboard handler)
            EditorCommand::EnterNormalMode |
            EditorCommand::EnterInsertMode |
            EditorCommand::EnterInsertModeAfter |
            EditorCommand::EnterInsertModeAtLineStart |
            EditorCommand::EnterInsertModeAtLineEnd |
            EditorCommand::EnterInsertModeNewLineBelow |
            EditorCommand::EnterInsertModeNewLineAbove |
            EditorCommand::EnterVisualMode |
            EditorCommand::EnterVisualLineMode |
            EditorCommand::EnterCommandMode => {
                // Mode change is handled by keyboard handler
                return Ok(true);
            }

            // Navigation
            EditorCommand::MoveUp => {
                self.cursor.row = self.cursor.row.saturating_sub(1);
            }
            EditorCommand::MoveDown => {
                if self.cursor.row < self.lines.len().saturating_sub(1) {
                    self.cursor.row += 1;
                }
            }
            EditorCommand::MoveLeft => {
                self.cursor.column = self.cursor.column.saturating_sub(1);
            }
            EditorCommand::MoveRight => {
                let line_len = self.current_line().chars().count();
                if self.cursor.column < line_len {
                    self.cursor.column += 1;
                }
            }
            EditorCommand::MoveToLineStart => {
                self.cursor.column = 0;
            }
            EditorCommand::MoveToLineEnd => {
                self.cursor.column = self.current_line().chars().count();
            }
            EditorCommand::MoveToFileStart => {
                self.cursor.row = 0;
                self.cursor.column = 0;
            }
            EditorCommand::MoveToFileEnd => {
                self.cursor.row = self.lines.len().saturating_sub(1);
                self.cursor.column = self.current_line().chars().count();
            }

            // Editing
            EditorCommand::InsertChar(ch) => {
                if *ch == '\n' {
                    // Handle newline: split line at cursor
                    let current_line = self.current_line().to_string();
                    let byte_pos: usize = current_line.chars().take(self.cursor.column).map(|c| c.len_utf8()).sum();

                    let (before, after) = current_line.split_at(byte_pos);
                    self.lines[self.cursor.row] = before.to_string();
                    self.lines.insert(self.cursor.row + 1, after.to_string());

                    self.cursor.row += 1;
                    self.cursor.column = 0;
                    self.modified = true;
                } else {
                    // Insert regular character
                    let cursor_col = self.cursor.column;
                    let line = self.current_line_mut();
                    let byte_pos = line.chars().take(cursor_col).map(|c| c.len_utf8()).sum();
                    line.insert(byte_pos, *ch);
                    self.cursor.column += 1;
                    self.modified = true;
                }
            }
            EditorCommand::DeleteChar => {
                let cursor_col = self.cursor.column;
                let line = self.current_line_mut();
                if cursor_col < line.chars().count() {
                    let byte_pos = line.chars().take(cursor_col).map(|c| c.len_utf8()).sum();
                    line.remove(byte_pos);
                    self.modified = true;
                }
            }
            EditorCommand::DeleteCharBackward => {
                if self.cursor.column > 0 {
                    self.cursor.column -= 1;
                    let cursor_col = self.cursor.column;
                    let line = self.current_line_mut();
                    let byte_pos = line.chars().take(cursor_col).map(|c| c.len_utf8()).sum();
                    line.remove(byte_pos);
                    self.modified = true;
                } else if self.cursor.row > 0 {
                    // Join with previous line
                    let current_line = self.lines.remove(self.cursor.row);
                    self.cursor.row -= 1;
                    self.cursor.column = self.current_line().chars().count();
                    self.current_line_mut().push_str(&current_line);
                    self.modified = true;
                }
            }
            EditorCommand::DeleteLine => {
                if self.lines.len() > 1 {
                    self.clipboard = vec![self.lines.remove(self.cursor.row)];
                    if self.cursor.row >= self.lines.len() {
                        self.cursor.row = self.lines.len().saturating_sub(1);
                    }
                    self.modified = true;
                } else {
                    self.clipboard = vec![self.current_line().to_string()];
                    self.lines[0].clear();
                    self.cursor.column = 0;
                    self.modified = true;
                }
            }
            EditorCommand::YankLine => {
                self.clipboard = vec![self.current_line().to_string()];
                self.message = "Yanked line".to_string();
            }
            EditorCommand::Paste => {
                if !self.clipboard.is_empty() {
                    for (i, line) in self.clipboard.iter().enumerate() {
                        self.lines.insert(self.cursor.row + i + 1, line.clone());
                    }
                    self.cursor.row += 1;
                    self.modified = true;
                    self.message = format!("Pasted {} line(s)", self.clipboard.len());
                }
            }

            // File operations
            EditorCommand::Save => {
                self.save_file()?;
            }
            EditorCommand::SaveAndQuit => {
                self.save_file()?;
                self.running = false;
            }
            EditorCommand::Quit => {
                if self.modified {
                    self.message = "Unsaved changes! Use :q! to force quit or :wq to save".to_string();
                } else {
                    self.running = false;
                }
            }
            EditorCommand::QuitForce => {
                self.running = false;
            }

            _ => {
                if self.debug {
                    self.message = format!("Unimplemented: {:?}", command);
                }
            }
        }

        self.clamp_cursor();
        Ok(true)
    }

    fn render(&mut self) -> Result<()> {
        // Clear screen
        let bg_color = Color::from_hex(&self.config.color_scheme.background)?;
        self.backend.clear(bg_color)?;

        // Get viewport size
        let (cols, rows) = self.backend.viewport_size();
        let content_rows = rows.saturating_sub(2); // Reserve space for status and command line

        // Render text content
        let fg_color = Color::from_hex(&self.config.color_scheme.foreground)?;
        let style = TextStyle {
            color: fg_color,
            background: None,
            font_style: tategaki_ed::backend::FontStyle::Normal,
            font_size: self.config.font_config.font_size,
        };

        // Render visible lines
        let start_line = self.cursor.row.saturating_sub(content_rows as usize / 2);
        let end_line = (start_line + content_rows as usize).min(self.lines.len());

        match self.config.text_direction {
            TextDirection::VerticalTopToBottom => {
                // Render columns right-to-left (pass logical column number, backend will convert)
                for (idx, line_idx) in (start_line..end_line).enumerate() {
                    let line = &self.lines[line_idx];
                    // Pass logical column number (idx) and row offset
                    self.backend.render_text(
                        line,
                        (idx as f32, 1.0),  // Backend will position this from the right
                        &style,
                        self.config.text_direction,
                    )?;
                }

                // Render cursor (convert logical to screen position)
                let col_offset = self.cursor.row.saturating_sub(start_line);
                let logical_col = col_offset as f32;  // Logical column from the right
                let logical_row = self.cursor.column as f32;

                // Convert to screen coordinates: columns go right-to-left
                let screen_col = ((cols as f32 - logical_col * 2.0 - 2.0).max(0.0)) as usize;
                let screen_row = (logical_row + 1.0) as usize;

                let cursor_color = Color::from_hex(&self.config.color_scheme.cursor)?;
                let cursor_info = CursorInfo {
                    position: SpatialPosition {
                        column: screen_col,
                        row: screen_row,
                        byte_offset: 0,
                    },
                    color: cursor_color,
                    style: if self.keyboard.mode() == EditorMode::Insert {
                        CursorStyle::Line
                    } else {
                        CursorStyle::Block
                    },
                };
                self.backend.render_cursor(&cursor_info)?;
            }
            _ => {
                // Render lines top-to-bottom (horizontal text)
                for (screen_row, line_idx) in (start_line..end_line).enumerate() {
                    let line = &self.lines[line_idx];
                    self.backend.render_text(
                        line,
                        (1.0, screen_row as f32),
                        &style,
                        self.config.text_direction,
                    )?;
                }

                // Render cursor
                let screen_row = self.cursor.row.saturating_sub(start_line);
                let cursor_color = Color::from_hex(&self.config.color_scheme.cursor)?;
                let cursor_info = CursorInfo {
                    position: SpatialPosition {
                        column: self.cursor.column + 1,
                        row: screen_row,
                        byte_offset: 0,
                    },
                    color: cursor_color,
                    style: if self.keyboard.mode() == EditorMode::Insert {
                        CursorStyle::Line
                    } else {
                        CursorStyle::Block
                    },
                };
                self.backend.render_cursor(&cursor_info)?;
            }
        }

        // Render status bar
        self.render_status_bar()?;

        // Render command line if in command mode
        if self.keyboard.mode() == EditorMode::Command {
            self.render_command_line()?;
        }

        // Present frame
        self.backend.present()?;

        Ok(())
    }

    fn render_status_bar(&mut self) -> Result<()> {
        let (cols, rows) = self.backend.viewport_size();
        let status_row = rows.saturating_sub(2);

        // Status bar background
        let status_bg = Color::new(40, 40, 40, 255);
        let status_rect = Rect::new(0.0, status_row as f32, cols as f32, 1.0);
        self.backend.render_rect(status_rect, status_bg, true)?;

        // Build status text
        let mode_name = self.keyboard.mode().display_name();
        let mode_jp = self.keyboard.mode().japanese_name();
        let modified_indicator = if self.modified { "[+]" } else { "" };
        let file_name = self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");

        let direction_str = match self.config.text_direction {
            TextDirection::VerticalTopToBottom => "縦",
            _ => "横",
        };

        let status_text = format!(
            " {} ({}) | {}{}  | {}:{} / {} | {}",
            mode_name,
            mode_jp,
            file_name,
            modified_indicator,
            self.cursor.row + 1,
            self.cursor.column + 1,
            self.lines.len(),
            direction_str,
        );

        let status_style = TextStyle {
            color: Color::white(),
            background: Some(status_bg),
            font_style: tategaki_ed::backend::FontStyle::Bold,
            font_size: self.config.font_config.font_size,
        };

        self.backend.render_text(
            &status_text,
            (0.0, status_row as f32),
            &status_style,
            TextDirection::HorizontalLeftToRight,
        )?;

        // Render message on second status line
        let msg_row = rows.saturating_sub(1);
        if !self.message.is_empty() {
            self.backend.render_text(
                &self.message,
                (0.0, msg_row as f32),
                &status_style,
                TextDirection::HorizontalLeftToRight,
            )?;
        }

        Ok(())
    }

    fn render_command_line(&mut self) -> Result<()> {
        let (cols, rows) = self.backend.viewport_size();
        let cmd_row = rows.saturating_sub(1);

        let cmd_text = format!(":{}", self.keyboard.command_line());
        let cmd_style = TextStyle {
            color: Color::white(),
            background: Some(Color::new(20, 20, 20, 255)),
            font_style: tategaki_ed::backend::FontStyle::Normal,
            font_size: self.config.font_config.font_size,
        };

        self.backend.render_text(
            &cmd_text,
            (0.0, cmd_row as f32),
            &cmd_style,
            TextDirection::HorizontalLeftToRight,
        )?;

        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        #[cfg(feature = "notcurses")]
        {
            // Get input from notcurses backend
            if let Some((key, ctrl, alt, shift)) = self.backend.get_input() {
                // Convert notcurses input to KeyInput
                let key_input = KeyInput::from_notcurses_key(key, ctrl, alt, shift);

                if self.debug {
                    self.message = format!("Key: {:?} (raw: {})", key_input, key);
                }

                // Process key through keyboard handler
                let command = self.keyboard.process_key(key_input.clone())?;

                if self.debug {
                    self.message = format!("Mode: {:?}, Key: {:?}, Cmd: {:?}",
                        self.keyboard.mode(), key_input, command);
                }

                // Execute mode changes through keyboard handler
                self.keyboard.execute_command(&command)?;

                // Execute the command in the editor
                self.execute_command(&command)?;
            } else {
                // No input available, short sleep to avoid busy loop
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
        }

        #[cfg(not(feature = "notcurses"))]
        {
            std::thread::sleep(std::time::Duration::from_millis(16));
        }

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        self.init()?;

        // Main event loop
        while self.running && self.backend.is_active() {
            self.render()?;
            self.handle_input()?;

            // Clear message after displaying (but keep debug messages)
            if !self.message.is_empty() && !self.debug {
                // Keep message for a few frames
                static mut MESSAGE_COUNTER: u32 = 0;
                unsafe {
                    MESSAGE_COUNTER += 1;
                    if MESSAGE_COUNTER > 60 {
                        self.message.clear();
                        MESSAGE_COUNTER = 0;
                    }
                }
            }
        }

        self.backend.shutdown()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup configuration
    let mut config = EditorConfig::default();
    config.text_direction = if args.vertical {
        TextDirection::VerticalTopToBottom
    } else {
        TextDirection::HorizontalLeftToRight
    };

    // Create editor
    let mut editor = TerminalEditor::new(config, args.debug)?;

    // Load file if specified
    if let Some(file_path) = args.file {
        editor.load_file(file_path)?;
    } else {
        // Load demo text
        let demo_lines = if args.vertical {
            vec![
                "掛".to_string(),
                "詞".to_string(),
                "プ".to_string(),
                "ロ".to_string(),
                "グ".to_string(),
                "ラ".to_string(),
                "ミ".to_string(),
                "ン".to_string(),
                "グ".to_string(),
                "言".to_string(),
                "語".to_string(),
            ]
        } else {
            vec![
                "Kakekotoba Programming Language".to_string(),
                "掛詞プログラミング言語".to_string(),
                "".to_string(),
                "Vim-like vertical text editor".to_string(),
                "".to_string(),
                "Press 'i' to enter insert mode".to_string(),
                "Press ':q' to quit".to_string(),
            ]
        };
        editor.lines = demo_lines;
    }

    if args.debug {
        eprintln!("Debug mode enabled");
        eprintln!("Text direction: {:?}", editor.config.text_direction);
        eprintln!("Lines: {}", editor.lines.len());
    }

    // Run the editor
    editor.run()?;

    println!("Thanks for using Tategaki Editor!");

    Ok(())
}
