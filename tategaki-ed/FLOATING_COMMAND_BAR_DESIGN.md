# Floating Command Bar Design

## Overview
A floating command bar that overlays vertical text content, providing a flexible command interface that doesn't interfere with the natural flow of vertical text.

## Key Features

### 1. Floating Positioning
The command bar is not tied to viewport edges but can float anywhere in the terminal:

```rust
pub enum FloatingPosition {
    /// Centered in viewport
    Center,
    /// Top-center, offset by Y pixels/rows
    TopCenter { offset_y: usize },
    /// Bottom-center, offset by Y pixels/rows
    BottomCenter { offset_y: usize },
    /// Custom absolute position (x, y)
    Absolute { x: usize, y: usize },
    /// Relative to cursor position
    NearCursor { offset_x: isize, offset_y: isize },
    /// Custom anchoring with offsets
    Anchored {
        horizontal: HorizontalAnchor,
        vertical: VerticalAnchor,
        offset_x: isize,
        offset_y: isize,
    },
}

pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

pub enum VerticalAnchor {
    Top,
    Middle,
    Bottom,
}
```

### 2. Visual Styling

```rust
pub struct FloatingBarStyle {
    /// Background color with alpha channel
    pub background: Color,
    /// Border style
    pub border: BorderStyle,
    /// Padding (left, right, top, bottom)
    pub padding: (usize, usize, usize, usize),
    /// Minimum width
    pub min_width: usize,
    /// Maximum width (None = no limit)
    pub max_width: Option<usize>,
    /// Shadow effect
    pub shadow: bool,
}

pub enum BorderStyle {
    None,
    Single,
    Double,
    Rounded,
    Custom { chars: [char; 8] }, // TL, T, TR, R, BR, B, BL, L
}
```

### 3. Command Bar Modes

```rust
pub enum CommandBarMode {
    /// Hidden (not rendered)
    Hidden,
    /// Command input mode (: commands)
    CommandInput,
    /// Command palette (fuzzy searchable commands)
    CommandPalette,
    /// Quick help overlay
    QuickHelp,
    /// Search mode (/)
    Search,
    /// Custom content
    Custom(String),
}
```

### 4. Content Layout

```
┌─────────────────────────────────────┐
│  Command: :write file.txt           │  ← Header/title
├─────────────────────────────────────┤
│  > :w                               │  ← Input line
│  ────────────────────────────────   │
│  Suggestions:                       │  ← Suggestions area
│    :write [filename]                │
│    :wq                              │
│    :wall                            │
├─────────────────────────────────────┤
│  Enter: execute  Esc: cancel        │  ← Footer/hints
└─────────────────────────────────────┘
```

### 5. Integration with Vertical Text

**Rendering Order** (back to front):
1. Vertical text columns
2. Status line (if configured)
3. Floating command bar (TOPMOST LAYER)

**Advantages for Vertical Text**:
- Doesn't interrupt vertical column flow
- Can appear near where you're working
- Can position to avoid covering important text
- Naturally suited for Japanese text input (horizontal input area)

### 6. Configuration

```rust
pub struct FloatingBarConfig {
    /// Position of the floating bar
    pub position: FloatingPosition,
    /// Visual styling
    pub style: FloatingBarStyle,
    /// Auto-hide after command execution
    pub auto_hide: bool,
    /// Show command history
    pub show_history: bool,
    /// Show suggestions
    pub show_suggestions: bool,
    /// Animation (slide in, fade, etc.)
    pub animation: AnimationStyle,
}
```

## Keyboard Shortcuts

### Opening/Closing
- `:` - Open command input mode at floating position
- `/` - Open search mode
- `Esc` - Hide floating bar

### Positioning (vim-style 'z' prefix)
The 'z' prefix follows vim's fold command pattern and is mnemonic for "floating/hovering":

- `zp` - Cycle through preset positions (Center → TopCenter → BottomCenter → NearCursor → Anchored → Center)
- `zt` - Toggle floating bar visibility
- `zk` - Move floating bar up
- `zj` - Move floating bar down
- `zh` - Move floating bar left
- `zl` - Move floating bar right

These commands work in Normal mode and maintain vim's philosophy of composable, memorable commands.

## Implementation Strategy

### Phase 1: Basic Floating Bar
1. Create FloatingCommandBar struct
2. Implement positioning system
3. Add basic rendering with border
4. Integrate with terminal backend

### Phase 2: Styling & Visual Polish
1. Add background colors with alpha
2. Implement border styles
3. Add padding and sizing
4. Implement shadow effect

### Phase 3: Interactive Features
1. Command input mode
2. Command palette with fuzzy search
3. Suggestion system
4. History navigation

### Phase 4: Vertical Text Optimization
1. Position near cursor in vertical text
2. Smart positioning to avoid covering text
3. Special handling for Japanese IME
4. Integration with spatial programming features

## Example Configurations

### Centered Floating Bar
```rust
FloatingBarConfig {
    position: FloatingPosition::Center,
    style: FloatingBarStyle {
        background: Color::new(30, 30, 30, 230), // Semi-transparent dark
        border: BorderStyle::Rounded,
        padding: (2, 2, 1, 1),
        min_width: 40,
        max_width: Some(80),
        shadow: true,
    },
    auto_hide: true,
    show_history: true,
    show_suggestions: true,
    animation: AnimationStyle::SlideDown,
}
```

### Near-Cursor Floating Bar (Vertical Text Optimized)
```rust
FloatingBarConfig {
    position: FloatingPosition::NearCursor {
        offset_x: 3,  // Offset to right of cursor
        offset_y: -2, // Slightly above cursor
    },
    style: FloatingBarStyle {
        background: Color::new(20, 20, 20, 250),
        border: BorderStyle::Single,
        padding: (1, 1, 0, 0),
        min_width: 30,
        max_width: Some(60),
        shadow: false,
    },
    auto_hide: true,
    show_history: false,
    show_suggestions: true,
    animation: AnimationStyle::None,
}
```

### Top-Center Fixed Position (Traditional)
```rust
FloatingBarConfig {
    position: FloatingPosition::TopCenter { offset_y: 1 },
    style: FloatingBarStyle {
        background: Color::new(40, 40, 40, 255),
        border: BorderStyle::Double,
        padding: (2, 2, 1, 1),
        min_width: 50,
        max_width: None,
        shadow: true,
    },
    auto_hide: false,
    show_history: true,
    show_suggestions: true,
    animation: AnimationStyle::FadeIn,
}
```

## Benefits for Vertical Text Editing

1. **Non-intrusive**: Doesn't break vertical column layout
2. **Context-aware**: Can position near where you're editing
3. **Flexible**: Easily movable to accommodate different workflows
4. **Modern UX**: Matches modern editor command palettes (VSCode, Sublime)
5. **Japanese-friendly**: Horizontal input area for IME is more natural
6. **Spatial programming**: Can integrate with spatial metadata display
