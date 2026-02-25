//! GPUI-based graphical vertical text editor
//!
//! This binary provides a rich graphical interface for vertical Japanese text editing
//! using GPUI with advanced rendering capabilities and spatial programming features.

use clap::{Arg, Command};
use std::path::PathBuf;
use tategaki_ed::{EditorConfig, GraphicalVerticalEditor, Result, TategakiError, TextDirection};

#[cfg(feature = "gpui")]
use gpui::*;

#[cfg(feature = "gpui")]
struct TategakiApp {
    editor: GraphicalVerticalEditor,
    config: EditorConfig,
}

#[cfg(feature = "gpui")]
impl TategakiApp {
    fn new(config: EditorConfig) -> Self {
        let editor = GraphicalVerticalEditor::new(config.clone());
        Self { editor, config }
    }

    fn load_file(&mut self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| TategakiError::Io(format!("Failed to read file: {}", e)))?;
            self.editor.load_text(&content)?;
            println!("Loaded file: {}", path.display());
        } else {
            println!("Creating new file: {}", path.display());
        }
        Ok(())
    }
}

#[cfg(feature = "gpui")]
impl Render for TategakiApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div().size_full().bg(rgb(0x1e1e1e)).child(
            div()
                .id("editor-container")
                .size_full()
                .child(self.editor.render(cx)),
        )
    }
}

fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("tategaki-gui")
        .version("0.1.0")
        .author("Kakekotoba Project")
        .about("Graphical vertical text editor with spatial programming support")
        .arg(
            Arg::new("file")
                .help("File to open")
                .value_name("FILE")
                .index(1),
        )
        .arg(
            Arg::new("direction")
                .long("direction")
                .short('d')
                .help("Text direction")
                .value_parser(["vertical", "horizontal"])
                .default_value("vertical"),
        )
        .arg(
            Arg::new("font-size")
                .long("font-size")
                .short('s')
                .help("Font size")
                .value_parser(clap::value_parser!(u32))
                .default_value("14"),
        )
        .arg(
            Arg::new("enable-ime")
                .long("enable-ime")
                .help("Enable Japanese IME")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug mode")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Configure editor
    let text_direction = match matches.get_one::<String>("direction").unwrap().as_str() {
        "vertical" => TextDirection::VerticalTopToBottom,
        "horizontal" => TextDirection::HorizontalLeftToRight,
        _ => TextDirection::VerticalTopToBottom,
    };

    let font_size = *matches.get_one::<u32>("font-size").unwrap();
    let enable_ime = matches.get_flag("enable-ime");
    let debug_mode = matches.get_flag("debug");

    let config = EditorConfig {
        text_direction,
        enable_ime,
        font_config: tategaki_ed::FontConfig {
            family: "Noto Sans CJK JP".to_string(),
            size: font_size as f32,
            weight: tategaki_ed::FontWeight::Normal,
        },
        debug_mode,
        ..EditorConfig::default()
    };

    // Get file path if provided
    let file_path = matches.get_one::<String>("file").map(PathBuf::from);

    #[cfg(feature = "gpui")]
    {
        // Initialize GPUI app
        App::new().run(|cx: &mut AppContext| {
            // Create main window
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Tategaki Editor".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    center: true,
                    focus: true,
                    show: true,
                    kind: WindowKind::Normal,
                    is_movable: true,
                    display_id: None,
                },
                |cx| {
                    let mut app = TategakiApp::new(config);

                    // Load file if provided
                    if let Some(path) = file_path {
                        if let Err(e) = app.load_file(&path) {
                            eprintln!("Error loading file: {}", e);
                        }
                    }

                    cx.new_view(|_| app)
                },
            )
            .unwrap();
        });

        Ok(())
    }

    #[cfg(not(feature = "gpui"))]
    {
        eprintln!("Error: GPUI feature not enabled. This binary requires the 'gpui' feature.");
        eprintln!("Build with: cargo build --features gpui --bin gui");
        Err(TategakiError::Configuration(
            "GPUI feature not enabled".to_string(),
        ))
    }
}

#[cfg(feature = "gpui")]
/// Handle application events
impl TategakiApp {
    fn handle_action(&mut self, action: &dyn Action, cx: &mut ViewContext<Self>) {
        // Handle application-specific actions
        // This would be expanded to handle file operations, preferences, etc.
    }
}

#[cfg(feature = "gpui")]
/// Define application actions
actions! {
    tategaki_gui,
    [
        NewFile,
        OpenFile,
        SaveFile,
        SaveFileAs,
        CloseFile,
        Quit,
        ToggleDirection,
        ToggleIME,
        ShowPreferences,
        ShowAbout,
    ]
}

#[cfg(feature = "gpui")]
/// Implement action handlers
impl TategakiApp {
    fn new_file(&mut self, _: &NewFile, cx: &mut ViewContext<Self>) {
        // Clear editor content
        if let Err(e) = self.editor.load_text("") {
            eprintln!("Error creating new file: {}", e);
        }
        cx.notify();
    }

    fn open_file(&mut self, _: &OpenFile, cx: &mut ViewContext<Self>) {
        // This would open a file dialog in a real implementation
        println!("Open file dialog would appear here");
        cx.notify();
    }

    fn save_file(&mut self, _: &SaveFile, cx: &mut ViewContext<Self>) {
        // This would save the current file
        println!("Save file operation");
        cx.notify();
    }

    fn toggle_direction(&mut self, _: &ToggleDirection, cx: &mut ViewContext<Self>) {
        self.config.text_direction = match self.config.text_direction {
            TextDirection::VerticalTopToBottom => TextDirection::HorizontalLeftToRight,
            TextDirection::HorizontalLeftToRight => TextDirection::VerticalTopToBottom,
        };

        // Update editor with new direction
        self.editor = GraphicalVerticalEditor::new(self.config.clone());
        cx.notify();
    }

    fn toggle_ime(&mut self, _: &ToggleIME, cx: &mut ViewContext<Self>) {
        self.config.enable_ime = !self.config.enable_ime;
        self.editor = GraphicalVerticalEditor::new(self.config.clone());
        cx.notify();
    }

    fn quit(&mut self, _: &Quit, cx: &mut ViewContext<Self>) {
        cx.app_context().quit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = EditorConfig {
            text_direction: TextDirection::VerticalTopToBottom,
            enable_ime: true,
            debug_mode: false,
            ..EditorConfig::default()
        };
        assert_eq!(config.text_direction, TextDirection::VerticalTopToBottom);
        assert!(config.enable_ime);
    }

    #[test]
    #[cfg(feature = "gpui")]
    fn test_app_creation() {
        let config = EditorConfig::default();
        let app = TategakiApp::new(config);
        assert_eq!(app.editor.text(), "");
    }
}
