# Tategaki-ed - Vertical Text Editor

## Project Overview

**tategaki-ed** is a vertical text editor designed for Japanese and other vertical writing systems, with vim-like keyboard controls and a modern floating command bar interface.

## Current Status (2025-11-30)

### Working Features

#### Core Editor Functionality
- ✅ Vertical text rendering (top-to-bottom, right-to-left)
- ✅ Horizontal text rendering (fallback)
- ✅ Multi-byte UTF-8 character support (Japanese, Chinese, etc.)
- ✅ File loading and saving
- ✅ Cursor positioning and movement in vertical mode
- ✅ Text insertion and deletion with proper UTF-8 handling

#### Terminal Backend (Notcurses)
- ✅ Notcurses-based terminal rendering
- ✅ UTF-8 locale initialization for CJK character support
- ✅ Keyboard input handling with escape sequence support
- ✅ Custom key code mapping for various terminals
- ✅ Banner suppression for clean startup

#### Vim-like Keyboard Bindings

**Normal Mode:**
- `h/j/k/l` - Navigation (adapted for vertical text)
- `i/a/o/O` - Enter insert mode variants
- `x/X` - Delete character forward/backward
- `Delete` - Delete character under cursor
- `Backspace` - Move left
- `dd` - Delete line
- `yy` - Yank (copy) line
- `p/P` - Paste after/before
- `u` - Undo
- `Ctrl+R` - Redo
- `:` - Enter command mode
- Arrow keys - Navigation

**Insert Mode:**
- `Escape/Ctrl+C` - Return to normal mode
- `Backspace` - Delete character backward (with UTF-8 support)
- `Delete` - Delete character forward
- `Enter` - Insert newline
- All printable characters insert text

**Command Mode:**
- `:w` - Save file
- `:q` - Quit
- `:wq` - Save and quit
- `:q!` - Force quit without saving
- `Backspace` - Edit command line

**Global:**
- `Ctrl+Q` - Force quit
- `Ctrl+S` - Save

#### Floating Command Bar
- ✅ Configurable floating command bar overlay
- ✅ Multiple positioning options (Center, TopCenter, BottomCenter, NearCursor, Anchored)
- ✅ Vertical orientation support (text flows top-to-bottom in bar)
- ✅ Visual styling (borders, padding, shadows)
- ✅ Auto-hide functionality

**Floating Bar Controls (z prefix):**
- `zp` - Cycle through preset positions
- `zt` - Toggle floating bar visibility
- `zk/zj/zh/zl` - Move bar up/down/left/right
- `zo` - Toggle vertical/horizontal orientation

### Known Issues & Limitations

#### Terminal Compatibility

**Nushell Compatibility Issue:**
- ⚠️ **Notcurses rendering does not work properly in nushell**
- Symptom: Blank screen, incorrect rendering, or cursor positioning issues
- Workaround: Use bash, zsh, or other standard shells
- Tested working: bash, zsh
- Location: Any terminal running nushell as the shell

**Terminal Emulator Notes:**
- Works best in standard terminal emulators (GNOME Terminal, iTerm2, etc.)
- iPad + Bluetooth keyboard + terminal emulator: Special key code mappings required
- Some terminals send non-standard key codes for Backspace/Delete

#### Key Code Variations

Different terminals send different codes for special keys:
- **Backspace**: Can be `0x107`, `0x11037F` (1115007), `8`, or `127`
- **Delete**: Can be `0x14A`, `0x110380` (1115008)
- **Enter**: Can be `10`, `13`, `0x10A`, or `1115121`

Current implementation handles multiple variations.

#### Rendering Issues (Resolved)
- ✅ Fixed: UTF-8 encoding warning by calling `setlocale()` before notcurses init
- ✅ Fixed: Cursor positioning offset in vertical mode (was 2 cells off)
- ✅ Fixed: Character deletion corrupting UTF-8 strings (now uses `drain()` instead of `remove()`)
- ✅ Fixed: Arrow keys being split into ESC + letter (added 10ms timeout)

### Architecture

```
tategaki-ed/
├── src/
│   ├── lib.rs                   # Core library, config structures
│   ├── backend/
│   │   ├── mod.rs              # Backend trait definitions
│   │   ├── terminal.rs         # Notcurses terminal backend
│   │   ├── keyboard.rs         # Vim-like keyboard handler
│   │   └── ...
│   ├── ui/
│   │   └── floating_bar.rs     # Floating command bar implementation
│   ├── bin/
│   │   └── terminal.rs         # Terminal editor binary
│   └── ...
└── ...
```

### Key Technical Decisions

1. **UTF-8 First:** All text operations use character counts, not byte offsets
2. **Locale Initialization:** Call `setlocale(LC_ALL, "")` before notcurses init for CJK support
3. **Input Timeout:** 10ms timeout for `notcurses_get()` to properly capture escape sequences
4. **Character Deletion:** Use `String::drain()` for UTF-8-safe character removal
5. **Vertical Text Layout:**
   - Logical columns map to visual columns right-to-left
   - Each character column is approximately 2 terminal cells wide
   - Formula: `screen_col = viewport_width - logical_col * 2.0`

### Configuration

Default configuration in `EditorConfig`:
- Text direction: Vertical top-to-bottom
- Floating command bar: Enabled, centered
- Colors: Dark theme (#1e1e1e background, #d4d4d4 foreground)
- Font size: 14pt

### Build & Run

**Requirements:**
- Rust toolchain
- notcurses library (`libnotcurses-dev`)
- GCC headers for bindgen

**Build:**
```bash
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include"
cargo build --no-default-features --features notcurses --bin tategaki-ed-terminal
```

**Run:**
```bash
BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include" \
  cargo run --no-default-features --features notcurses --bin tategaki-ed-terminal [FILE]
```

**Important:** Run from bash/zsh, NOT nushell!

### Testing

```bash
# Library tests
cargo test --lib

# Keyboard handler tests
cargo test --lib keyboard

# All tests
cargo test
```

Current test status: 140 tests passing (82 library + 58 vim keyboard)

### Recent Fixes (Session 2025-11-30)

1. **Nushell incompatibility discovered** - notcurses doesn't render properly
2. **UTF-8 locale initialization** - Added `setlocale()` call for CJK characters
3. **Cursor positioning** - Fixed 2-cell offset in vertical mode
4. **Character deletion** - Fixed UTF-8 corruption using `drain()` instead of `remove()`
5. **Backspace/Delete keys** - Added support for multiple key code variants
6. **Input timing** - Changed from 0ns to 10ms timeout for escape sequence capture
7. **Floating bar vertical orientation** - Added toggle with `zo` command

### Debug Utilities

Several debug logging files created during development:
- `/tmp/tategaki_keys.log` - Raw key code logging
- `/tmp/tategaki_backspace_debug.log` - Backspace command flow
- `/tmp/tategaki_delete_debug.log` - Deletion execution details
- `/tmp/tategaki_render_debug.log` - Character rendering details

These can be enabled by adding debug logging code to the relevant sections.

### Future Work

#### High Priority
- [ ] Undo/Redo implementation (currently stubbed)
- [ ] Visual mode selection (partially implemented)
- [ ] Search and replace
- [ ] Line numbers
- [ ] Syntax highlighting for vertical code

#### Medium Priority
- [ ] Multiple file/buffer support
- [ ] Split windows
- [ ] Command history
- [ ] Configuration file support
- [ ] Theme customization

#### Low Priority
- [ ] Macro recording
- [ ] Plugin system
- [ ] Integration with external tools
- [ ] GPUI backend (currently has compilation issues)

### Documentation

- `README.md` - Project overview and installation
- `FLOATING_COMMAND_BAR_DESIGN.md` - Floating bar design and features
- `TERMINAL_SUSPEND_ISSUE.md` - Ctrl+Z suspension recovery
- `QUICK_BUILD_REFERENCE.md` - Build commands and troubleshooting
- `CLAUDE_CONTEXT.md` - This file

### Contributing

When working on this project:
1. Always test in bash/zsh, not nushell
2. Remember to set `BINDGEN_EXTRA_CLANG_ARGS` for builds
3. Test with both ASCII and Japanese text
4. Verify UTF-8 character handling for any text operations
5. Run tests before committing: `cargo test`

### Performance Notes

- Render loop runs at ~60 FPS (16ms sleep when no input)
- Input polling uses 10ms timeout
- No performance issues observed with files up to several thousand lines
- Vertical text rendering is slightly more expensive than horizontal due to character-by-character positioning

### Dependencies

Key dependencies:
- `libnotcurses-sys` - Terminal rendering
- `unicode-segmentation` - UTF-8 text handling
- `serde` - Configuration serialization

See `Cargo.toml` for full dependency list.

---

**Last Updated:** 2025-11-30
**Version:** 0.1.0
**Status:** Alpha - Core functionality working, some features incomplete
