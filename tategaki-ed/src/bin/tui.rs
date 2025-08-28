//! Ratatui-based terminal vertical text editor
//!
//! This binary provides a console-based editor interface using Ratatui with
//! Unicode support for vertical Japanese text editing in terminal environments.

use clap::{Arg, Command};
use std::path::PathBuf;
use tategaki_ed::{EditorConfig, TerminalVerticalEditor, Result, TategakiError, TextDirection};

#[cfg(feature = "ratatui")]
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

#[cfg(feature = "ratatui")]
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(feature = "ratatui")]
use std::io::{self, Stdout};

fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("tategaki-tui")
        .version("0.1.0")
        .author("Kakekotoba Project")
        .about("Terminal-based vertical text editor with spatial programming support")
        .arg(
            Arg::new("file")
                .help("File to open")
                .value_name("FILE")
                .index(1)
        )
        .arg(
            Arg::new("direction")
                .long("direction")
                .short('d')
                .help("Text direction")
                .value_parser(["vertical", "horizontal"])
                .default_value("vertical")
        )
        .arg(
            Arg::new("enable-ime")
                .long("enable-ime")
                .help("Enable Japanese IME")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("vim-mode")
                .long("vim-mode")
                .help("Enable Vim-like keybindings")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug mode")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-mouse")
                .long("no-mouse")
                .help("Disable mouse support")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("theme")
                .long("theme")
                .short('t')
                .help("Color theme")
                .value_parser(["dark", "light", "high-contrast"])
                .default_value("dark")
        )
        .get_matches();

    // Configure editor
    let text_direction = match matches.get_one::<String>("direction").unwrap().as_str() {
        "vertical" => TextDirection::VerticalTopToBottom,
        "horizontal" => TextDirection::HorizontalLeftToRight,
        _ => TextDirection::VerticalTopToBottom,
    };

    let enable_ime = matches.get_flag("enable-ime");
    let vim_mode = matches.get_flag("vim-mode");
    let debug_mode = matches.get_flag("debug");
    let mouse_support = !matches.get_flag("no-mouse");
    let theme = matches.get_one::<String>("theme").unwrap();

    let config = EditorConfig {
        text_direction,
        enable_ime,
        vim_keybindings: vim_mode,
        debug_mode,
        mouse_support,
        theme: parse_theme(theme),
        ..EditorConfig::default()
    };

    // Get file path if provided
    let file_path = matches.get_one::<String>("file").map(PathBuf::from);

    // Print startup message
    if debug_mode {
        eprintln!("Starting Tategaki TUI Editor");
        eprintln!("Text direction: {:?}", text_direction);
        eprintln!("IME enabled: {}", enable_ime);
        eprintln!("Vim mode: {}", vim_mode);
        if let Some(ref path) = file_path {
            eprintln!("File: {}", path.display());
        }
    }

    #[cfg(feature = "ratatui")]
    {
        run_terminal_editor(config, file_path)
    }

    #[cfg(not(feature = "ratatui"))]
    {
        eprintln!("Error: Ratatui feature not enabled. This binary requires the 'ratatui' feature.");
        eprintln!("Build with: cargo build --features ratatui --bin tui");
        Err(TategakiError::Configuration(
            "Ratatui feature not enabled".to_string()
        ))
    }
}

#[cfg(feature = "ratatui")]
fn run_terminal_editor(config: EditorConfig, file_path: Option<PathBuf>) -> Result<()> {
    // Initialize terminal
    let mut terminal = setup_terminal()?;
    
    // Create editor
    let mut editor = TerminalVerticalEditor::new(config);
    
    // Load file if provided
    if let Some(path) = file_path {
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| TategakiError::Io(format!("Failed to read file: {}", e)))?;
            editor.load_text(&content)?;
            if editor.config.debug_mode {
                eprintln!("Loaded {} characters from {}", content.len(), path.display());
            }
        } else {
            if editor.config.debug_mode {
                eprintln!("Creating new file: {}", path.display());
            }
        }
    }

    // Show welcome message for new users
    if file_path.is_none() {
        show_welcome_message();
    }

    // Run editor
    let result = editor.run(CrosstermBackend::new(io::stdout()));
    
    // Cleanup terminal
    cleanup_terminal(&mut terminal)?;
    
    result
}

#[cfg(feature = "ratatui")]
fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().map_err(|e| TategakiError::Terminal(format!("Failed to enable raw mode: {}", e)))?;
    
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| TategakiError::Terminal(format!("Failed to setup terminal: {}", e)))?;
    
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)
        .map_err(|e| TategakiError::Terminal(format!("Failed to create terminal: {}", e)))?;
    
    Ok(terminal)
}

#[cfg(feature = "ratatui")]
fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().map_err(|e| TategakiError::Terminal(format!("Failed to disable raw mode: {}", e)))?;
    
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(|e| TategakiError::Terminal(format!("Failed to cleanup terminal: {}", e)))?;
    
    terminal.show_cursor()
        .map_err(|e| TategakiError::Terminal(format!("Failed to show cursor: {}", e)))?;
    
    Ok(())
}

fn parse_theme(theme_name: &str) -> tategaki_ed::Theme {
    match theme_name {
        "light" => tategaki_ed::Theme::Light,
        "high-contrast" => tategaki_ed::Theme::HighContrast,
        "dark" | _ => tategaki_ed::Theme::Dark,
    }
}

fn show_welcome_message() {
    println!("╭─────────────────────────────────────────────────────────────╮");
    println!("│                    Tategaki Editor                          │");
    println!("│              Terminal Vertical Text Editor                  │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ Quick Start:                                                │");
    println!("│   i     - Enter insert mode                                │");
    println!("│   Esc   - Return to normal mode                            │");
    println!("│   :q    - Quit                                              │");
    println!("│   :w    - Save (when file operations are implemented)      │");
    println!("│   Ctrl+C - Force quit                                       │");
    println!("│   Ctrl+T - Toggle text direction                           │");
    println!("│                                                             │");
    println!("│ Navigation (Normal mode):                                   │");
    println!("│   h/j/k/l - Move left/down/up/right                        │");
    println!("│   0/$     - Start/end of line                               │");
    println!("│   Arrow keys also work                                      │");
    println!("│                                                             │");
    println!("│ Press any key to start editing...                          │");
    println!("╰─────────────────────────────────────────────────────────────╯");
    
    // Wait for key press
    #[cfg(feature = "ratatui")]
    {
        use crossterm::event::{self, Event, KeyCode};
        
        enable_raw_mode().ok();
        while let Ok(Event::Key(key)) = event::read() {
            if matches!(key.code, KeyCode::Char(_) | KeyCode::Enter | KeyCode::Esc) {
                break;
            }
        }
        disable_raw_mode().ok();
        
        // Clear the screen
        print!("\x1B[2J\x1B[1;1H");
    }
}

/// Print usage help and examples
fn print_usage_examples() {
    println!("Tategaki TUI Editor - Usage Examples:");
    println!();
    println!("Basic usage:");
    println!("  tategaki-tui                    # Start with empty document");
    println!("  tategaki-tui file.txt          # Open existing file");
    println!("  tategaki-tui --direction horizontal file.txt");
    println!();
    println!("Advanced usage:");
    println!("  tategaki-tui --enable-ime --vim-mode file.txt");
    println!("  tategaki-tui --theme light --no-mouse file.txt");
    println!("  tategaki-tui --debug file.txt  # Debug mode");
    println!();
    println!("Text directions:");
    println!("  vertical   - Traditional Japanese (top-to-bottom, right-to-left)");
    println!("  horizontal - Standard horizontal (left-to-right, top-to-bottom)");
    println!();
    println!("Themes:");
    println!("  dark         - Dark theme (default)");
    println!("  light        - Light theme");
    println!("  high-contrast - High contrast theme");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_parsing() {
        assert_eq!(parse_theme("dark"), tategaki_ed::Theme::Dark);
        assert_eq!(parse_theme("light"), tategaki_ed::Theme::Light);
        assert_eq!(parse_theme("high-contrast"), tategaki_ed::Theme::HighContrast);
        assert_eq!(parse_theme("invalid"), tategaki_ed::Theme::Dark); // Default fallback
    }

    #[test]
    fn test_config_creation() {
        let config = EditorConfig {
            text_direction: TextDirection::VerticalTopToBottom,
            enable_ime: true,
            vim_keybindings: true,
            debug_mode: false,
            ..EditorConfig::default()
        };
        
        assert_eq!(config.text_direction, TextDirection::VerticalTopToBottom);
        assert!(config.enable_ime);
        assert!(config.vim_keybindings);
    }

    #[test]
    #[cfg(feature = "ratatui")]
    fn test_editor_creation() {
        let config = EditorConfig::default();
        let editor = TerminalVerticalEditor::new(config);
        assert_eq!(editor.text(), "");
    }

    #[test]
    fn test_file_path_handling() {
        // Test that PathBuf conversion works
        let path = PathBuf::from("test.txt");
        assert_eq!(path.to_string_lossy(), "test.txt");
    }
}