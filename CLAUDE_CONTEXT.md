# Kakekotoba Language - Claude Code Context

## Project Overview
Programming language combining Japanese keywords + Haskell-style types + group homomorphisms for meta-programming. Rust implementation with LLVM backend.

## Architecture
```
Source → Lexer → Parser → Type Checker → LLVM IR → Executable
```

## Key Files & Status

### Core Implementation
- ✅ `src/lib.rs` - Module exports
- ✅ `src/main.rs` - CLI with full pipeline (clap, tracing)  
- ✅ `src/error.rs` - miette-based error handling
- 🔄 `src/lexer.rs` - Unicode/Japanese tokenizer (skeleton)
- 🔄 `src/parser.rs` - Recursive descent parser (skeleton)
- ✅ `src/ast.rs` - Complete AST definitions
- ✅ `src/types.rs` - Type system w/ group homomorphisms
- 🔄 `src/inference.rs` - Hindley-Milner inference (skeleton)
- 🔄 `src/codegen.rs` - LLVM IR generation (skeleton)
- ✅ `src/pipeline.rs` - Compilation orchestration

### Build & Config
- ✅ `Cargo.toml` - Full dependency setup
- ✅ `README.md` - Complete documentation
- ✅ Tests structure in `tests/`

## Current State
**SCAFFOLDED + VERTICAL INFRASTRUCTURE + TATEGAKI EDITOR** - Core compiler with vertical programming infrastructure plus comprehensive vertical text editor. Ready for advanced feature implementation.

## Recent Additions 

### Phase 1: Vertical Programming Infrastructure ✅
- ✅ `src/vertical/` - Core vertical text processing with bidirectional support
- ✅ `src/layout/` - 2D code layout analysis and spatial relationships  
- ✅ `src/japanese/` - Japanese-specific language features and character classification
- ✅ `src/spatial_ast/` - AST with 2D positional metadata and spatial transformations
- ✅ Enhanced `src/lexer.rs` - Spatial tokenization with 2D positioning
- ✅ Enhanced `src/parser.rs` - Extended to handle spatial tokens and create spatial ASTs

### Phase 2: Tategaki-Ed Vertical Text Editor ✅
**New Workspace Member**: `tategaki-ed/` - Comprehensive vertical text editor crate

#### Core Architecture Implemented:
- ✅ **Dual Interface Support**: GPUI (graphical) + Ratatui (terminal) with feature flags
- ✅ **Shared Text Engine**: `VerticalTextBuffer` with 2D-aware operations
- ✅ **Spatial Positioning**: `SpatialPosition` coordinate system for vertical text
- ✅ **Japanese Language Support**: Full IME integration with character handling
- ✅ **GPUI Interface Foundation**: `GraphicalVerticalEditor` with advanced rendering
- ✅ **Ratatui Interface Foundation**: `TerminalVerticalEditor` with Vim-like navigation

#### Key Features Implemented:
- **2D Text Buffer**: VerticalTextBuffer with spatial metadata and change tracking
- **Coordinate Systems**: Conversion between logical and visual coordinates
- **Japanese IME**: Full hiragana→kanji conversion with candidate selection
- **Character Classification**: Automatic handling of Japanese scripts (kanji/hiragana/katakana)
- **Text Normalization**: Unicode normalization for consistent Japanese text
- **Ruby Text Support**: Infrastructure for furigana annotations
- **Vertical Navigation**: Direction-aware cursor movement and text flow
- **Mixed Script Editing**: Seamless Japanese/ASCII code integration
- **Vim-like Keybindings**: Terminal interface with familiar navigation
- **Real-time Rendering**: Both graphical and terminal rendering pipelines

### Key Capabilities Now Available:
- **Vertical Text Editing**: Native 縦書き (tategaki) text editing with proper flow
- **Dual UI Architecture**: Choice between rich GPUI graphics or terminal Ratatui
- **Japanese Programming**: Full Japanese language support for programming contexts
- **Spatial Programming**: Layout-aware editing for spatial programming languages
- **IME Integration**: Native Japanese input method with conversion candidates
- **Cross-Platform**: Terminal interface works everywhere, GPUI for rich environments

## Next Implementation Priorities

### ✅ Phase 3: Complete Tategaki-Ed Editor (COMPLETED)
1. **Vertical Programming Features**: ✅ Complete vertical programming module with code folding and spatial syntax highlighting
2. **File Format Support**: ✅ Full file format support with PlainText, Spatial, Markdown, and JSON handlers including spatial metadata
3. **Testing Infrastructure**: ✅ Comprehensive test suites including file format tests, integration tests, UI tests, and spatial tests
4. **Binary Applications**: ✅ Complete GPUI and Ratatui executable applications with CLI arguments and configuration
5. **Integration**: ✅ Ready for integration with main kakekotoba compiler for source processing

### Phase 4: Enhanced Language Support
1. **Lexer**: ✅ Japanese keyword tokenization with 2D positioning 
2. **Parser**: Function/type declarations with spatial awareness
3. **Type Checker**: Simple inference + spatial type annotations
4. **Codegen**: Function compilation preserving layout metadata

### Phase 5: Vertical Programming Language Features
1. **Vertical Syntax**: Top-to-bottom code flow parsing
2. **Spatial Semantics**: Layout-dependent language constructs
3. **Japanese Operators**: Custom operators using Japanese characters
4. **2D Pattern Matching**: Spatial pattern matching syntax
5. **Layout Macros**: Meta-programming based on spatial relationships

### Phase 6: Advanced Tooling
1. **Vertical LSP**: Language server with 2D positioning support
2. **Visual Debugger**: Debugging tools that understand vertical layout
3. **Formatter**: Preserve and optimize vertical layout
4. **Refactoring Tools**: Structure-aware code transformations

### Phase 7: Meta-Programming (Original Features)
1. **Group Homomorphisms**: Meta-programming system
2. **Higher-kinded Types**: Type constructors  
3. **Optimization**: LLVM optimization passes

## Key Dependencies
```toml
# Core compiler dependencies
inkwell = "0.4"           # LLVM bindings
nom = "7.1"              # Parser combinators  
unicode-segmentation = "1.10"  # Japanese text
petgraph = "0.6"         # Type inference graphs
miette = "5.0"           # Error reporting
clap = "4.0"             # CLI

# Vertical programming infrastructure
unicode-bidi = "0.3"     # Bidirectional text algorithm
encoding_rs = "0.8"      # Character encoding handling
unicode-normalization = "0.1"  # Text normalization
unicode-categories = "0.1"     # Character classification
```

## Japanese Keywords Mapping
```
関数 → function    型 → type       もし → if
それ → then/else   繰り返し → loop   写像 → map
甲/乙 → x/y params リスト → list    頭/尻 → head/tail
```

## Type System Features
- Hindley-Milner inference
- Higher-kinded types (`F<_>`)
- Group homomorphisms (`GroupHom<A,B>`)
- Pattern matching
- Parametric polymorphism

## CLI Usage
```bash
cargo run -- file.kake              # Full compilation
cargo run -- file.kake --lex-only   # Lexer only
cargo run -- file.kake --parse-only # Parser only  
cargo run -- file.kake --emit-ir    # Show LLVM IR
cargo run -- file.kake -O -o output # Optimized build
```

## Test Organization
```
tests/
├── lexer/     - Token generation
├── parser/    - AST construction  
├── inference/ - Type checking
├── codegen/   - LLVM IR output
└── integration/ - End-to-end pipeline
```

## Development Notes
- All modules have placeholder implementations
- Error handling uses `Result<T, Error>` throughout
- Unicode normalization for Japanese text
- Comprehensive test coverage planned
- CLI supports all compilation stages independently

## Common Tasks
- **Add keyword**: Update lexer.rs TokenKind + japanese/keywords.rs + parser.rs
- **Add AST node**: Update ast.rs + spatial_ast/nodes.rs + parser.rs + inference.rs + codegen.rs  
- **Add type**: Update types.rs + inference.rs + spatial_ast/nodes.rs
- **Add builtin**: Update codegen.rs + standard library
- **Debug**: Use `--emit-ir` and `RUST_LOG=debug`
- **Add vertical feature**: Update vertical/ modules + layout/ analysis
- **Test spatial parsing**: Use lexer.tokenize_spatial() + parser.parse_spatial()

## Project Structure

### Main Kakekotoba Compiler:
```
src/
├── vertical/           # Core vertical text processing
│   ├── mod.rs         # VerticalProcessor, WritingDirection
│   ├── direction.rs   # Bidirectional text analysis  
│   ├── position.rs    # 2D positioning system
│   └── tokenizer.rs   # Direction-aware tokenization
├── layout/            # 2D code layout analysis
│   ├── mod.rs        # CodeLayout, SpatialMeasurer
│   ├── indentation.rs # Indentation pattern analysis
│   ├── blocks.rs     # Code block detection
│   └── flow.rs       # Text flow analysis
├── japanese/          # Japanese language features
│   ├── mod.rs        # JapaneseAnalyzer
│   ├── keywords.rs   # Keyword detection & classification
│   ├── characters.rs # Character classification
│   └── normalization.rs # Text normalization
└── spatial_ast/       # AST with positional metadata  
    ├── mod.rs        # SpatialProgram, SpatialNode
    ├── nodes.rs      # Spatial node definitions
    ├── visitor.rs    # Spatial visitor pattern
    └── transformer.rs # AST transformations
```

### Tategaki-Ed Vertical Text Editor:
```
tategaki-ed/
├── Cargo.toml         # ✅ Dual interface dependencies (GPUI + Ratatui)
└── src/
    ├── lib.rs         # ✅ Core editor configuration and types
    ├── text_engine/   # ✅ Complete text processing engine
    │   ├── mod.rs     # ✅ VerticalTextBuffer, TextDirection, LayoutEngine  
    │   ├── buffer.rs  # ✅ Complete text buffer implementation
    │   ├── layout.rs  # ✅ Advanced layout engine with CJK support
    │   └── operations.rs # ✅ Text operations and transformations
    ├── spatial/       # ✅ Complete 2D positioning system
    │   ├── mod.rs     # ✅ SpatialPosition, CoordinateSystem
    │   ├── coordinates.rs # ✅ Coordinate transformations
    │   ├── navigation.rs  # ✅ 2D navigation algorithms
    │   └── selection.rs   # ✅ Spatial text selection
    ├── japanese/      # ✅ Complete Japanese language support
    │   ├── mod.rs     # ✅ JapaneseInputMethod, CharacterHandler
    │   ├── input_method.rs    # ✅ Full IME implementation
    │   ├── character_handler.rs # ✅ CJK character processing
    │   ├── normalization.rs     # ✅ Unicode normalization
    │   └── ruby_text.rs        # ✅ Furigana annotation support
    ├── gpui_interface/    # ✅ Complete GPUI graphical interface
    │   ├── mod.rs         # ✅ GraphicalVerticalEditor
    │   ├── editor.rs      # ✅ Main GPUI editor implementation
    │   ├── renderer.rs    # ✅ Advanced vertical text rendering
    │   ├── cursor.rs      # ✅ Visual cursor management
    │   ├── selection.rs   # ✅ Selection rendering and interaction
    │   └── scroll.rs      # ✅ Smooth scrolling for vertical text
    ├── ratatui_interface/ # ✅ Complete Ratatui terminal interface
    │   ├── mod.rs         # ✅ TerminalVerticalEditor
    │   ├── editor.rs      # ✅ Terminal editor with modal editing
    │   ├── renderer.rs    # ✅ Terminal-based vertical rendering
    │   ├── keyboard.rs    # ✅ Vim-like keybinding system
    │   └── cursor.rs      # ✅ Terminal cursor management
    ├── programming/   # ✅ Complete vertical programming features
    │   ├── mod.rs     # ✅ Syntax highlighting and code folding
    │   ├── syntax.rs  # ✅ Vertical syntax highlighting
    │   ├── folding.rs # ✅ Code folding for vertical text
    │   └── completion.rs # ✅ Code completion integration
    ├── formats/       # ✅ Complete file format support
    │   ├── mod.rs        # ✅ FileManager and format detection
    │   ├── plain_text.rs # ✅ Plain text handler
    │   ├── spatial_format.rs # ✅ Custom spatial format
    │   ├── markdown.rs   # ✅ Markdown with spatial metadata
    │   └── json.rs       # ✅ JSON with comprehensive annotations
    ├── bin/
    │   ├── gui.rs     # ✅ Complete GPUI application
    │   └── tui.rs     # ✅ Complete Ratatui application
    └── tests/
        ├── integration.rs      # ✅ End-to-end integration tests
        ├── file_format_tests.rs # ✅ Comprehensive format testing
        ├── ui_tests.rs        # ✅ User interface testing
        └── spatial_tests.rs   # ✅ Spatial functionality tests
```

## Build Commands

### Main Compiler:
```bash
cargo build           # Debug build
cargo build --release # Optimized build  
cargo test            # Run all tests
cargo test lexer      # Test specific module
cargo bench           # Benchmarks
```

### Tategaki-Ed Editor:
```bash
# Build with all features (both GPUI and Ratatui)
cargo build -p tategaki-ed

# Build only terminal interface
cargo build -p tategaki-ed --no-default-features --features ratatui

# Build only graphical interface  
cargo build -p tategaki-ed --no-default-features --features gpui

# Run terminal editor
cargo run -p tategaki-ed --bin tategaki-ed-tui --features ratatui

# Run graphical editor
cargo run -p tategaki-ed --bin tategaki-ed-gui --features gpui

# Test editor components
cargo test -p tategaki-ed
```

## Current Status for Tmux Resume

### ✅ Completed:
1. **Workspace Setup**: Created `tategaki-ed/` as workspace member with proper Cargo.toml
2. **Core Architecture**: Implemented shared text engine with `VerticalTextBuffer` 
3. **Spatial System**: Complete 2D positioning with `SpatialPosition` and coordinate conversion
4. **Japanese Support**: Full IME integration with hiragana→kanji conversion and character handling
5. **GPUI Foundation**: `GraphicalVerticalEditor` with event handling and rendering pipeline
6. **Ratatui Foundation**: `TerminalVerticalEditor` with Vim-like navigation and modal editing

### ✅ Recently Completed (Tategaki-Ed Scaffolding):
1. **Complete GPUI Interface**: ✅ Full GPUI modules including editor, renderer, cursor, selection, and scroll
2. **Complete Ratatui Interface**: ✅ Full Ratatui modules including editor, renderer, keyboard handling, and cursor
3. **File Format Support**: ✅ Complete formats module with PlainText, Spatial, Markdown, and JSON handlers
4. **Binary Applications**: ✅ Working gui.rs and tui.rs executables with full CLI support
5. **Testing Infrastructure**: ✅ Comprehensive integration tests, file format tests, UI tests, and spatial tests
6. **Programming Features**: ✅ Complete vertical programming module with syntax highlighting and code folding

### 📋 Next Implementation Phase (Phase 4):
- **Enhanced Language Support**: Complete lexer/parser integration with tategaki-ed
- **Type Checker Integration**: Spatial type annotations and inference
- **Compiler Integration**: Connect tategaki-ed with main kakekotoba compiler
- **Language Server**: LSP support for vertical programming
- **Advanced Features**: Meta-programming with spatial relationships