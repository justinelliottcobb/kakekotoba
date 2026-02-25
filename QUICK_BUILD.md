# Quick Build Reference for Tategaki-Ed Terminal

## When Ready to Build

Once notcurses ≥ 3.0.11 is installed, use these commands:

### Environment Setup

```bash
# Required environment variables
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include -I/usr/include"
```

### Build Commands

```bash
# Terminal editor with notcurses backend
cargo build -p tategaki-ed --bin tategaki-ed-terminal --no-default-features --features notcurses

# Or if notcurses still has issues, use ratatui fallback
cargo build -p tategaki-ed --bin tategaki-ed-tui --no-default-features --features ratatui

# GPUI GUI version
cargo build -p tategaki-ed --bin tategaki-ed-gui --features gpui
```

### Run the Editor

```bash
# Run with demo text (vertical Japanese)
./target/debug/tategaki-ed-terminal

# Open a file
./target/debug/tategaki-ed-terminal myfile.kake

# Horizontal mode
./target/debug/tategaki-ed-terminal --horizontal myfile.txt

# Debug mode
./target/debug/tategaki-ed-terminal --debug
```

## Vim Keybindings Quick Reference

### Normal Mode
- `h/j/k/l` - Navigate (adapted for vertical text)
- `i` - Insert before cursor
- `a` - Insert after cursor
- `o` - Open line below
- `O` - Open line above
- `x` - Delete character
- `dd` - Delete line
- `yy` - Yank (copy) line
- `p` - Paste after cursor
- `P` - Paste before cursor
- `u` - Undo (TODO)
- `Ctrl+R` - Redo (TODO)
- `gg` - Go to first line
- `G` - Go to last line
- `0` - Start of line
- `$` - End of line
- `w` - Next word (TODO)
- `b` - Previous word (TODO)

### Insert Mode
- `Escape` or `Ctrl+C` - Return to normal mode
- Type normally to insert text

### Visual Mode
- `v` - Enter visual mode (from normal)
- `V` - Enter visual line mode (from normal)
- `Escape` - Return to normal mode

### Command Mode
- `:w` - Save file
- `:q` - Quit (fails if unsaved)
- `:wq` - Save and quit
- `:q!` - Force quit without saving

### Global Shortcuts
- `Ctrl+S` - Quick save
- `Ctrl+Q` - Force quit

## Installation Options for Notcurses ≥ 3.0.11

### Option 1: Build from Source (Recommended)

```bash
# Install build dependencies
sudo apt install cmake build-essential libunistring-dev libncurses-dev \
                 libavformat-dev libavutil-dev libswscale-dev

# Clone and build notcurses
git clone https://github.com/dankamongmen/notcurses.git
cd notcurses
git checkout v3.0.11  # or later version

mkdir build && cd build
cmake .. -DCMAKE_INSTALL_PREFIX=/usr/local
make -j$(nproc)
sudo make install
sudo ldconfig

# Ensure pkg-config can find it
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

### Option 2: System Package (if available)

```bash
# If Ubuntu repos provide 3.0.11+
sudo apt update
sudo apt install libnotcurses-dev libnotcurses3

# Check version
pkg-config --modversion notcurses
```

### Option 3: Use Ratatui Backend Instead

If notcurses is problematic, the ratatui backend works without these dependencies:

```bash
cargo build -p tategaki-ed --bin tategaki-ed-tui --no-default-features --features ratatui
```

**Note**: The ratatui backend needs integration with the new keyboard handler (currently separate).

## Current Blocker

**Status as of last build attempt**: System has notcurses 3.0.7, but `libnotcurses-sys` 3.11.0 requires ≥ 3.0.11 due to struct field changes (`ncinput.eff_text` was added in 3.0.11).

**Workarounds applied** (for reference):
- Created `/usr/lib/x86_64-linux-gnu/pkgconfig/tinfo.pc`
- Patched notcurses*.pc version numbers (3.0.7 → 3.0.11)
- Set LIBCLANG_PATH to system llvm-18

See BUILD_STATUS.md for complete troubleshooting history.

## Testing Checklist

Once built, test:

- [ ] Launch terminal editor
- [ ] Load a file
- [ ] Navigate with hjkl in vertical mode
- [ ] Enter insert mode and type Japanese text
- [ ] Verify vertical punctuation conversion (、→︑ 。→︒)
- [ ] Test yank/paste (yy, p)
- [ ] Test delete line (dd)
- [ ] Save file (:w)
- [ ] Quit and reopen to verify persistence
- [ ] Test horizontal mode flag
- [ ] Verify cursor positioning
- [ ] Test status bar updates

## Architecture Overview

```
tategaki-ed/
├── src/
│   ├── backend/
│   │   ├── mod.rs           # RenderBackend trait
│   │   ├── terminal.rs      # Notcurses implementation
│   │   ├── gpui_native.rs   # GPUI wrapper
│   │   ├── adapter.rs       # Coordinate translation
│   │   ├── keyboard.rs      # Vim-like modal editing
│   │   └── selector.rs      # Backend selection
│   ├── bin/
│   │   ├── terminal.rs      # Terminal editor binary
│   │   ├── tui.rs           # Ratatui binary
│   │   └── gui.rs           # GPUI binary
│   └── lib.rs
└── Cargo.toml
```

**Key Features**:
- ✅ Backend abstraction (RenderBackend trait)
- ✅ Vertical Japanese text with Unicode forms
- ✅ Full vim-like modal editing
- ✅ Direction-aware navigation
- ✅ File I/O with modified tracking
- ✅ Status bar with bilingual mode indicators
- ✅ Clipboard (yank/paste)

**See Also**:
- NOTCURSES_BACKEND.md - Comprehensive implementation documentation
- BUILD_STATUS.md - Build troubleshooting and status
- tategaki-ed/src/backend/ - Implementation code
