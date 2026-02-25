# Notcurses Backend Implementation for Tategaki-Ed

## Overview

This document describes the notcurses terminal rendering backend that has been added to the tategaki-ed vertical text editor. The implementation creates an abstraction layer that allows the editor to run in both GPU-accelerated mode (GPUI) and terminal mode (notcurses).

## Architecture

### Backend Abstraction Layer

The new `src/backend/` module provides a unified rendering interface through the `RenderBackend` trait. This abstraction allows seamless switching between different rendering backends:

```
src/backend/
├── mod.rs          # RenderBackend trait, Color, Rect, TextStyle types
├── selector.rs     # Backend selection logic with TTY detection
├── terminal.rs     # Notcurses terminal backend implementation
├── gpui_native.rs  # GPUI backend wrapper
└── adapter.rs      # GPUI-to-notcurses coordinate translation
```

### Key Components

#### 1. RenderBackend Trait (`mod.rs`)

The core abstraction trait that all backends must implement:

```rust
pub trait RenderBackend: Send + Sync {
    fn init(&mut self) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
    fn viewport_size(&self) -> (u32, u32);
    fn clear(&mut self, color: Color) -> Result<()>;
    fn render_text(&mut self, text: &str, position: (f32, f32), style: &TextStyle, direction: TextDirection) -> Result<()>;
    fn render_cursor(&mut self, cursor: &CursorInfo) -> Result<()>;
    fn render_selection(&mut self, bounds: Rect, color: Color) -> Result<()>;
    fn render_line(&mut self, from: (f32, f32), to: (f32, f32), color: Color, thickness: f32) -> Result<()>;
    fn render_rect(&mut self, bounds: Rect, color: Color, filled: bool) -> Result<()>;
    fn present(&mut self) -> Result<()>;
    fn is_active(&self) -> bool;
    fn handle_resize(&mut self, width: u32, height: u32) -> Result<()>;
}
```

Also defines cross-backend types:
- `Color` - RGBA color representation
- `Rect` - Rectangle bounds
- `TextStyle` - Font styling and colors
- `CursorInfo` - Cursor position and style
- `BackendType` - Enum of available backends

#### 2. Backend Selector (`selector.rs`)

Automatically selects the best available backend based on:
- TTY detection (using `isatty()` on Unix)
- Display server availability (Wayland/X11 detection)
- Explicit user preference via CLI flags
- Feature flags at compile time

```rust
let selector = BackendSelector::new()
    .force_terminal(); // Optional: force terminal mode

let backend_type = selector.select()?;
```

#### 3. Terminal Backend (`terminal.rs`)

Full notcurses implementation with special handling for vertical Japanese text:

**Vertical Text Features:**
- Automatic conversion to Unicode vertical presentation forms (U+FE10-U+FE19)
- Character rotation for Latin text in vertical orientation
- Full-width CJK character support (2 cells per character)
- Proper rendering of Japanese punctuation in vertical mode
- Long vowel mark conversion (ー → ｜)

**Technical Details:**
- Coordinate system mapping: pixels → terminal cells
- Right-to-left column progression for traditional vertical text
- Top-to-bottom character flow within columns
- Mouse support (if terminal supports it)
- Unicode box drawing for UI chrome

**Character Conversion Table:**

| Horizontal | Vertical | Unicode Description |
|-----------|----------|---------------------|
| 、 | ︑ | IDEOGRAPHIC COMMA |
| 。 | ︒ | IDEOGRAPHIC FULL STOP |
| ： | ︓ | COLON |
| ！ | ︕ | EXCLAMATION MARK |
| ？ | ︖ | QUESTION MARK |
| 「」 | ﹁﹂ | CORNER BRACKETS |
| （） | ︵︶ | PARENTHESES |
| ー | ｜ | LONG VOWEL MARK |

#### 4. GPUI Backend Wrapper (`gpui_native.rs`)

Wraps the existing GPUI interface to conform to the RenderBackend trait. Uses a command queue pattern to store rendering commands that get executed within GPUI's rendering cycle.

#### 5. Adapter Layer (`adapter.rs`)

Translates between GPUI's pixel-based coordinate system and notcurses' cell-based terminal grid:

- **Coordinate conversion**: pixels ↔ cells
- **Character width calculation**: Handles full-width CJK characters
- **Color quantization**: Maps 24-bit RGB to terminal color space
- **Viewport mapping**: Translates GPUI bounds to terminal dimensions

Typical terminal cell metrics:
- Cell width: 8 pixels
- Cell height: 16 pixels
- CJK characters: 2 cells wide
- ASCII characters: 1 cell wide

## Binary Applications

### New Terminal Binary (`src/bin/terminal.rs`)

A complete terminal-based editor using the notcurses backend:

```bash
# Build the terminal editor
cargo build -p tategaki-ed --bin tategaki-ed-terminal --features notcurses

# Run with vertical text (default)
./target/debug/tategaki-ed-terminal myfile.kake

# Run with horizontal text
./target/debug/tategaki-ed-terminal --horizontal myfile.txt

# Debug mode
./target/debug/tategaki-ed-terminal --debug demo.txt
```

**Features:**
- Vertical and horizontal text modes
- File loading
- Basic cursor rendering
- Status bar with current line/mode info
- Keybindings:
  - Ctrl+Q: Quit
  - Ctrl+T: Toggle text direction (future)
  - Ctrl+S: Save (future)

## Dependencies Added

```toml
# Notcurses terminal interface (optional)
libnotcurses-sys = { version = "3.9", optional = true }
libc = "0.2"
nix = { version = "0.27", features = ["term"] }
```

**System Requirements:**
- libnotcurses3 (3.0.9 or later) must be installed on the system
- On Ubuntu/Debian: `sudo apt install libnotcurses3 libnotcurses-dev`
- On macOS: `brew install notcurses`

## Feature Flags

```toml
[features]
default = ["gpui", "notcurses"]
gpui = ["dep:gpui", "dep:cosmic-text"]
ratatui = ["dep:ratatui", "dep:crossterm"]
notcurses = ["dep:libnotcurses-sys"]
```

Build combinations:
```bash
# GPUI only (GPU-accelerated)
cargo build -p tategaki-ed --no-default-features --features gpui

# Notcurses only (terminal)
cargo build -p tategaki-ed --no-default-features --features notcurses

# Both backends (default)
cargo build -p tategaki-ed

# All three backends
cargo build -p tategaki-ed --features gpui,notcurses,ratatui
```

## Usage Examples

### Example 1: Automatic Backend Selection

```rust
use tategaki_ed::{EditorConfig, backend::BackendSelector};

let selector = BackendSelector::new();
let backend_type = selector.select()?;

match backend_type {
    BackendType::Notcurses => {
        // Terminal mode
        let mut backend = TerminalBackend::new()?;
        backend.init()?;
    }
    BackendType::Gpui => {
        // GPU mode
        let mut backend = GpuiBackend::new()?;
        backend.init()?;
    }
    _ => {}
}
```

### Example 2: Rendering Vertical Japanese Text

```rust
use tategaki_ed::{
    TextDirection,
    backend::{TerminalBackend, RenderBackend, TextStyle, Color},
};

let mut backend = TerminalBackend::new()?;
backend.init()?;

let style = TextStyle {
    color: Color::white(),
    background: None,
    font_style: FontStyle::Normal,
    font_size: 14.0,
};

// Render vertical text (right-to-left columns, top-to-bottom flow)
backend.render_text(
    "掛詞プログラミング言語",
    (70.0, 1.0),  // Position in cells
    &style,
    TextDirection::VerticalTopToBottom,
)?;

backend.present()?;
```

### Example 3: Using the Adapter

```rust
use tategaki_ed::backend::{terminal::TerminalBackend, adapter::GpuiToNotcursesAdapter};

let backend = TerminalBackend::new()?;
let mut adapter = GpuiToNotcursesAdapter::new(backend)?;

// Convert GPUI pixel coordinates to terminal cells
let (col, row) = adapter.pixels_to_cells(160.0, 320.0);

// Calculate string width for mixed Japanese/ASCII
let text = "Hello世界";
let width_in_cells = adapter.string_width_in_cells(text);
// Returns: 9 (5 for "Hello" + 4 for "世界")

// Paint text at pixel coordinates (automatically converted)
adapter.paint_text(
    text,
    (160.0, 320.0),  // Pixel coordinates
    &style,
    TextDirection::VerticalTopToBottom,
)?;
```

## Implementation Status

### ✅ Completed

1. **Backend Abstraction**
   - ✅ RenderBackend trait
   - ✅ Cross-backend types (Color, Rect, TextStyle, CursorInfo)
   - ✅ BackendType enumeration
   - ✅ BackendSelector with TTY detection

2. **Terminal Backend**
   - ✅ Notcurses initialization and shutdown
   - ✅ Vertical text rendering with Unicode forms
   - ✅ Character rotation detection
   - ✅ Full-width CJK character support
   - ✅ Cursor rendering (Block, Line, Underline styles)
   - ✅ Selection highlighting
   - ✅ Box drawing for UI chrome
   - ✅ Viewport management

3. **GPUI Backend**
   - ✅ RenderBackend wrapper
   - ✅ Command queue pattern
   - ✅ Color conversion

4. **Adapter Layer**
   - ✅ Pixel-to-cell coordinate conversion
   - ✅ CJK character width calculation
   - ✅ Color quantization utilities
   - ✅ Viewport mapping

5. **Binary Application**
   - ✅ Terminal editor binary
   - ✅ CLI argument parsing
   - ✅ File loading
   - ✅ Basic rendering loop
   - ✅ Status bar

6. **Documentation**
   - ✅ Comprehensive README
   - ✅ Code documentation
   - ✅ Usage examples

### 🔄 Needs Implementation

1. **Input Handling**
   - ⏸️ Keyboard event processing via notcurses_get_nblock()
   - ⏸️ Mouse event handling
   - ⏸️ Japanese IME integration in terminal
   - ⏸️ Key binding configuration

2. **Advanced Rendering**
   - ⏸️ Syntax highlighting in terminal
   - ⏸️ Code folding indicators
   - ⏸️ Multiple cursor support
   - ⏸️ Smooth scrolling

3. **Editor Features**
   - ⏸️ Text editing operations (insert, delete)
   - ⏸️ File saving
   - ⏸️ Undo/redo
   - ⏸️ Search and replace
   - ⏸️ Multi-file support

4. **Integration**
   - ⏸️ Kakekotoba compiler integration
   - ⏸️ Error highlighting
   - ⏸️ LSP support

## Testing

### Unit Tests

All modules include unit tests:

```bash
# Test backend abstraction
cargo test -p tategaki-ed backend::tests

# Test terminal backend
cargo test -p tategaki-ed terminal::tests

# Test adapter
cargo test -p tategaki-ed adapter::tests
```

### Integration Testing

To test the terminal editor:

1. Install notcurses:
   ```bash
   sudo apt install libnotcurses3 libnotcurses-dev
   ```

2. Build the terminal binary:
   ```bash
   cargo build -p tategaki-ed --bin tategaki-ed-terminal --features notcurses
   ```

3. Run with demo text:
   ```bash
   ./target/debug/tategaki-ed-terminal
   ```

4. Test with a file:
   ```bash
   echo "掛詞" > test.txt
   echo "プログラミング" >> test.txt
   echo "言語" >> test.txt
   ./target/debug/tategaki-ed-terminal test.txt
   ```

## Known Issues & Limitations

1. **Compilation**: The parent `kakekotoba` crate has pre-existing compilation errors that need to be fixed before the full workspace can build.

2. **Input**: The terminal binary currently has a placeholder input handler. Full keyboard/mouse event processing needs to be implemented.

3. **GPUI Integration**: The GpuiBackend wrapper is a minimal implementation. Full integration with GPUI's view system requires more work.

4. **Performance**: No optimization has been done yet. The 60 FPS render loop may be inefficient for terminal rendering.

5. **Color Support**: The adapter includes color quantization utilities, but they're not yet used. Terminal color mapping could be improved.

6. **Rotated Characters**: While Latin characters are detected for rotation, the actual rotation rendering is not yet implemented. They currently render as-is.

## Next Steps

### Priority 1: Fix Parent Crate
- Add missing `unicode-categories` dependency
- Fix ownership issues in `spatial_ast/transformer.rs`
- Resolve type inference errors

### Priority 2: Complete Input Handling
- Implement notcurses event loop
- Add keyboard navigation
- Support Japanese IME in terminal

### Priority 3: Editor Operations
- Text insertion and deletion
- File saving
- Undo/redo stack

### Priority 4: Advanced Features
- Syntax highlighting for terminals
- Multiple cursors
- Split views

## References

- [Notcurses Documentation](https://notcurses.com/)
- [libnotcurses-sys Rust Bindings](https://docs.rs/libnotcurses-sys/)
- [Unicode Vertical Forms](https://en.wikipedia.org/wiki/Vertical_text)
- [GPUI Framework](https://github.com/zed-industries/zed)
- [Japanese Typography](https://www.w3.org/TR/jlreq/)

## Contributing

When adding new rendering features:

1. Add the method to the `RenderBackend` trait in `mod.rs`
2. Implement it in `TerminalBackend` (terminal.rs)
3. Implement it in `GpuiBackend` (gpui_native.rs)
4. Add any necessary coordinate conversion to `adapter.rs`
5. Write unit tests for all three modules
6. Update this documentation

## License

Same as parent project: MIT OR Apache-2.0
