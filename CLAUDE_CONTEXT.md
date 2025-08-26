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
**SCAFFOLDED + VERTICAL INFRASTRUCTURE** - All core modules exist with vertical programming infrastructure added. Ready for feature implementation with 2D spatial support.

## Recent Additions (Vertical Programming Infrastructure)

### New Modules Added:
- ✅ `src/vertical/` - Core vertical text processing with bidirectional support
- ✅ `src/layout/` - 2D code layout analysis and spatial relationships  
- ✅ `src/japanese/` - Japanese-specific language features and character classification
- ✅ `src/spatial_ast/` - AST with 2D positional metadata and spatial transformations

### Enhanced Existing Modules:
- ✅ `src/lexer.rs` - Now supports spatial tokenization with 2D positioning
- ✅ `src/parser.rs` - Extended to handle spatial tokens and create spatial ASTs
- ✅ `Cargo.toml` - Added dependencies for Unicode bidirectional text processing

### Key Capabilities Now Available:
- **Spatial Tokenization**: Tokens preserve 2D coordinates (row, column)
- **Bidirectional Text**: Full Unicode bidirectional algorithm support
- **Japanese Integration**: Character classification, keyword detection, text normalization
- **Layout Analysis**: Indentation analysis, block detection, flow analysis
- **Spatial AST**: AST nodes with positional metadata and layout-aware transformations

## Next Implementation Priorities

### Phase 1: Basic Functionality (Enhanced with Vertical Support)
1. **Lexer**: ✅ Japanese keyword tokenization with 2D positioning 
2. **Parser**: Function/type declarations with spatial awareness
3. **Type Checker**: Simple inference + spatial type annotations
4. **Codegen**: Function compilation preserving layout metadata

### Phase 2: Vertical Programming Features
1. **Vertical Syntax**: Top-to-bottom code flow parsing
2. **Spatial Semantics**: Layout-dependent language constructs
3. **Japanese Operators**: Custom operators using Japanese characters
4. **Mixed-Script Support**: Seamless Japanese/ASCII code integration

### Phase 3: Advanced Vertical Features  
1. **2D Pattern Matching**: Spatial pattern matching syntax
2. **Layout Macros**: Meta-programming based on spatial relationships
3. **Vertical LSP**: Language server with 2D positioning support
4. **Visual Debugger**: Debugging tools that understand vertical layout

### Phase 4: Meta-Programming (Original Features)
1. **Group Homomorphisms**: Meta-programming system
2. **Higher-kinded Types**: Type constructors  
3. **Optimization**: LLVM optimization passes
4. **Advanced Tooling**: Formatter, refactoring tools

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

## New Vertical Programming Infrastructure

### Module Structure:
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

## Build Commands
```bash
cargo build           # Debug build
cargo build --release # Optimized build  
cargo test            # Run all tests
cargo test lexer      # Test specific module
cargo bench           # Benchmarks
```