# Kakekotoba (掛詞) Programming Language

A modern programming language that combines Japanese keywords with Haskell-style type systems and group homomorphisms for meta-programming.

## Features

- **Japanese Keywords**: Natural programming using Japanese language constructs
- **Haskell-style Type System**: Strong static typing with type inference
- **Group Homomorphisms**: Advanced meta-programming capabilities through algebraic structures
- **Unicode Support**: First-class support for Japanese text and Unicode normalization
- **LLVM Backend**: High-performance code generation via LLVM IR

## Getting Started

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- LLVM 18.0+ development libraries

#### Installing LLVM on Ubuntu/Debian

```bash
# Add LLVM APT repository
wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | sudo apt-key add -
sudo add-apt-repository "deb http://apt.llvm.org/focal/ llvm-toolchain-focal-18 main"
sudo apt update

# Install LLVM development packages
sudo apt install llvm-18-dev libllvm18 llvm-18-runtime
```

#### Installing LLVM on macOS

```bash
brew install llvm@18
export LLVM_SYS_180_PREFIX=$(brew --prefix llvm@18)
```

#### Installing LLVM on Windows

Download and install LLVM from the [official releases page](https://releases.llvm.org/) or use chocolatey:

```powershell
choco install llvm --version=18.1.8
```

### Installation

1. Clone the repository:

```bash
git clone https://github.com/yourusername/kakekotoba.git
cd kakekotoba
```

2. Build the project:

```bash
cargo build --release
```

3. Run tests to verify everything works:

```bash
cargo test
```

### Basic Usage

#### Command Line Interface

```bash
# Compile a kakekotoba source file
cargo run -- input.kake -o output

# Show available options
cargo run -- --help

# Only perform lexical analysis
cargo run -- input.kake --lex-only

# Only perform parsing
cargo run -- input.kake --parse-only

# Only perform type checking
cargo run -- input.kake --type-check-only

# Output LLVM IR
cargo run -- input.kake --emit-ir

# Enable optimizations
cargo run -- input.kake -O -o optimized_output
```

#### Language Syntax Examples

**Function Declarations (Japanese Keywords):**

```kakekotoba
// Simple function using Japanese keywords
関数 足す(甲: Int, 乙: Int): Int = 甲 + 乙

// Function with type parameters
関数 写像<A, B>(関数: A -> B, リスト: [A]): [B] = ...

// Pattern matching
関数 長さ(リスト: [A]): Int = 
  もし リスト が
    [] -> 0
    頭::尻 -> 1 + 長さ(尻)
```

**Type Definitions:**

```kakekotoba
// Sum type (algebraic data type)
型 Option<A> = 
  | Some(A)
  | None

// Product type (record)
型 Point = { x: Float, y: Float }

// Group homomorphism for meta-programming
型 GroupHom<G1, G2> = Homomorphism {
  source: Group<G1>,
  target: Group<G2>,
  mapping: G1 -> G2,
  preserves_operation: True
}
```

**Advanced Features:**

```kakekotoba
// Higher-kinded types
型 Functor<F<_>> = {
  map: <A, B>(A -> B) -> F<A> -> F<B>
}

// Group homomorphism usage
関数 apply_homomorphism<G1, G2>(
  h: GroupHom<G1, G2>,
  elements: [G1]
): [G2] = 
  写像(h.mapping, elements)
```

## Tategaki-Ed Vertical Text Editor

The kakekotoba project includes **tategaki-ed** (縦書きエディタ), a specialized text editor designed for vertical Japanese text editing with vim-like keybindings.

### Editor Features

- **Vertical Japanese Text**: Native support for top-to-bottom, right-to-left tategaki layout
- **Multiple Backends**: GPUI (GPU-accelerated), notcurses (terminal), and ratatui support
- **Vim-like Modal Editing**: Complete Normal/Insert/Visual/Command mode system
- **Unicode Vertical Forms**: Automatic conversion of punctuation (、→︑ 。→︒)
- **Direction-Aware Navigation**: hjkl navigation adapted for vertical text flow
- **Bilingual UI**: Status bar and messages in both English and Japanese

### Editor Binaries

```bash
# Terminal editor with notcurses backend (requires notcurses ≥ 3.0.11)
cargo build -p tategaki-ed --bin tategaki-ed-terminal --features notcurses

# Terminal editor with ratatui backend
cargo build -p tategaki-ed --bin tategaki-ed-tui --features ratatui

# GUI editor with GPUI
cargo build -p tategaki-ed --bin tategaki-ed-gui --features gpui
```

### Quick Start

```bash
# Run terminal editor
./target/debug/tategaki-ed-terminal myfile.kake

# Vim keybindings
# Normal mode: hjkl (navigate), i (insert), dd (delete line), yy (yank)
# Command mode: :w (save), :q (quit), :wq (save & quit)
# Global: Ctrl+S (save), Ctrl+Q (quit)
```

### Documentation

- **NOTCURSES_BACKEND.md**: Detailed backend implementation documentation
- **BUILD_STATUS.md**: Build troubleshooting and dependency issues
- **QUICK_BUILD.md**: Quick reference for building and using the editor

## Project Architecture

### Compiler Pipeline

The kakekotoba compiler follows a traditional multi-stage architecture:

```
Source Code → Lexer → Parser → Type Checker → Code Generator → Executable
     ↓           ↓        ↓          ↓             ↓
   Unicode    Tokens    AST    Typed AST      LLVM IR
```

### Module Organization

- **`src/lexer.rs`**: Unicode-aware tokenization with Japanese keyword support
- **`src/parser.rs`**: Recursive descent parser generating AST
- **`src/ast.rs`**: Abstract syntax tree definitions
- **`src/types.rs`**: Type system including group homomorphisms
- **`src/inference.rs`**: Hindley-Milner type inference engine
- **`src/codegen.rs`**: LLVM IR generation
- **`src/pipeline.rs`**: Compilation orchestration
- **`src/error.rs`**: Comprehensive error handling with source locations
- **`tategaki-ed/`**: Vertical text editor with multiple backend support

### Key Dependencies

- **`inkwell`**: Safe LLVM bindings for code generation
- **`nom`**: Parser combinator library
- **`unicode-segmentation`**: Proper Unicode text handling
- **`petgraph`**: Graph algorithms for type inference
- **`miette`**: Beautiful error reporting
- **`clap`**: Command-line argument parsing

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific module
cargo test lexer
cargo test parser
cargo test inference
cargo test codegen
cargo test integration

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Code Organization

The project follows Rust best practices:

- **Modules**: Each compiler stage is a separate module
- **Error Handling**: Uses `Result<T, Error>` throughout with comprehensive error types
- **Testing**: Unit tests for each module plus integration tests
- **Documentation**: Rustdoc comments on public APIs

### Contributing

1. **Language Design**: Extend AST nodes and parser for new syntax
2. **Type System**: Add new type constructors and inference rules
3. **Code Generation**: Implement missing LLVM IR patterns
4. **Standard Library**: Add built-in functions and types
5. **Tooling**: Improve error messages, add language server support

### Adding New Language Features

1. **Lexer**: Add new keywords/tokens in `src/lexer.rs`
2. **AST**: Define syntax tree nodes in `src/ast.rs`
3. **Parser**: Implement parsing rules in `src/parser.rs`
4. **Types**: Add type system support in `src/types.rs`
5. **Inference**: Update type checking in `src/inference.rs`
6. **Codegen**: Generate LLVM IR in `src/codegen.rs`
7. **Tests**: Add comprehensive tests in `tests/`

## Language Reference

### Japanese Keywords

| Japanese | English | Usage |
|----------|---------|-------|
| 関数 | function | Function declaration |
| 型 | type | Type definition |
| もし | if | Conditional expression |
| それ | then/else | Conditional branches |
| 繰り返し | loop | Iteration construct |

### Type System Features

- **Type Inference**: Hindley-Milner algorithm with constraint solving
- **Polymorphism**: Parametric types with quantification
- **Higher-Kinded Types**: Types that take type parameters
- **Group Homomorphisms**: Algebraic structures for meta-programming
- **Pattern Matching**: Exhaustive pattern matching on algebraic types

## Examples

See the `examples/` directory for complete programs:

- **`examples/hello_world.kake`**: Basic program structure
- **`examples/fibonacci.kake`**: Recursive functions
- **`examples/list_processing.kake`**: Higher-order functions
- **`examples/type_classes.kake`**: Advanced type system features
- **`examples/homomorphisms.kake`**: Group theory applications

## Performance

Kakekotoba generates efficient native code through LLVM:

- **Zero-cost abstractions**: Higher-level constructs compile to optimal machine code
- **Aggressive optimization**: LLVM optimization passes enabled in release mode
- **Memory safety**: Static analysis prevents common memory errors
- **Unicode efficiency**: Optimized Japanese text processing

## Roadmap

- [ ] **Phase 1**: Complete basic language implementation
- [ ] **Phase 2**: Standard library development  
- [ ] **Phase 3**: Package manager and tooling
- [ ] **Phase 4**: IDE integration and language server
- [ ] **Phase 5**: Advanced meta-programming features

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

**掛詞** (kakekotoba): A traditional Japanese rhetorical technique using words with multiple meanings - fitting for a language that bridges Japanese natural language with formal type theory.