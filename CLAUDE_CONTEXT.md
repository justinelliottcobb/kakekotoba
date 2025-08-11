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
**SCAFFOLDED** - All modules exist with placeholder implementations. Ready for feature implementation.

## Next Implementation Priorities

### Phase 1: Basic Functionality
1. **Lexer**: Japanese keyword tokenization (`関数`, `型`, `もし`, etc.)
2. **Parser**: Function/type declarations, basic expressions
3. **Type Checker**: Simple inference for primitives + functions
4. **Codegen**: Function compilation to LLVM

### Phase 2: Language Features  
1. **Pattern Matching**: AST + parser + codegen
2. **Higher-order Functions**: Lambda expressions
3. **Algebraic Types**: Sum/product types
4. **Standard Library**: Built-in functions

### Phase 3: Advanced Features
1. **Group Homomorphisms**: Meta-programming system
2. **Higher-kinded Types**: Type constructors  
3. **Optimization**: LLVM optimization passes
4. **Tooling**: LSP, formatter, etc.

## Key Dependencies
```toml
inkwell = "0.4"           # LLVM bindings
nom = "7.1"              # Parser combinators  
unicode-segmentation = "1.10"  # Japanese text
petgraph = "0.6"         # Type inference graphs
miette = "5.0"           # Error reporting
clap = "4.0"             # CLI
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
- **Add keyword**: Update lexer.rs TokenKind + parser.rs
- **Add AST node**: Update ast.rs + parser.rs + inference.rs + codegen.rs  
- **Add type**: Update types.rs + inference.rs
- **Add builtin**: Update codegen.rs + standard library
- **Debug**: Use `--emit-ir` and `RUST_LOG=debug`

## Build Commands
```bash
cargo build           # Debug build
cargo build --release # Optimized build  
cargo test            # Run all tests
cargo test lexer      # Test specific module
cargo bench           # Benchmarks
```