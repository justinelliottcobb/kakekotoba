# Kakekotoba — Development Guidelines

## Project Overview

Kakekotoba (掛詞) is a programming language built on Abstract Type Theory and Group Homomorphisms, with Japanese-native orthography and bidirectional evaluation semantics. The workspace contains the compiler (`kakekotoba`) and a vertical text editor (`tategaki-ed`).

See `docs/ROADMAP.md` for project roadmap and `docs/design-philosophy.md` for the design thesis.

## Coding Conventions (Industrial Algebra Rust Standards)

### 1. Idiomatic Rust
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Prefer `impl Trait` over `dyn Trait` where monomorphization is acceptable
- Use `#[must_use]` on functions that return values that should not be ignored
- Prefer iterators and combinators over manual loops
- Use `From`/`Into` for type conversions; `TryFrom`/`TryInto` for fallible ones
- Minimize `unsafe`; when required, document the safety invariant

### 2. Test-Driven Development
- Write tests before or alongside implementation, not after
- Unit tests go in `#[cfg(test)] mod tests` within the source file
- Integration tests go in `tests/`
- Use `proptest` for property-based testing (especially for parser, type system)
- Use `insta` for snapshot testing (parser output, IR, error messages)
- All PRs must include tests for new behavior and regressions for bug fixes
- Run `cargo test --workspace` before committing

### 3. Phantom Types and Algebraic Patterns
- Use phantom types to encode state at the type level (e.g., compilation phases)
- Prefer `enum` for sum types; use exhaustive matching (avoid `_ =>` catch-alls where possible)
- Use newtype wrappers for domain-specific identifiers (e.g., `TypeVar(u32)` not bare `u32`)
- Leverage `std::marker::PhantomData` for zero-cost type-level distinctions
- Encode invariants in the type system rather than runtime checks where practical

### 4. Concurrency
- Use `rayon` for CPU-bound parallel work (batch compilation, parallel testing, etc.)
- Use `tokio` for async I/O where already present (tategaki-ed)
- Use `wgpu` for GPU acceleration where applicable (future: geometric algebra computations)

### 5. TUI
- Use `ratatui` for TUI interfaces by default
- Exception: `tategaki-ed` terminal backend uses `notcurses` for vertical text rendering

### 6. Error Handling
- Use `thiserror` for library error types with structured variants
- Use `anyhow` only in binary entry points and tests
- Use `miette` for user-facing compiler diagnostics with source spans
- Never use `.unwrap()` in library code; `.expect("reason")` only where panic is truly impossible to avoid

### 7. Dependencies
- Prefer well-maintained, minimal-dependency crates
- Pin major versions in `Cargo.toml`
- Audit new dependencies before adding

## Git Workflow

This project follows a **gitflow-like** branching model:

```
feature/*, chore/*, fix/*  →  PR to develop  →  develop  →  release PR  →  main
```

### Branches
- `main` — stable releases only; protected
- `develop` — integration branch; all feature work merges here
- `feature/*` — new features (e.g., `feature/undo-redo`, `feature/bytecode-vm`)
- `chore/*` — maintenance work (e.g., `chore/update-deps`, `chore/ci-setup`)
- `fix/*` — bug fixes (e.g., `fix/parser-panic`)
- `release/*` — release preparation (version bumps, changelog)

### Commit Messages
- Use imperative mood: "Add undo stack" not "Added undo stack"
- Keep the first line under 72 characters
- Reference issues where applicable: "Fix parser panic on empty input (#42)"

### Pre-Commit Hooks
A pre-commit hook runs automatically on `git commit`:
1. `cargo fmt --check` — formatting
2. `cargo clippy --workspace -- -D warnings` — lints
3. `cargo test --workspace` — all tests pass

Install hooks after cloning:
```bash
git config core.hooksPath .githooks
```

## Build Commands

```bash
# Full workspace build
cargo build --workspace

# Terminal editor (requires notcurses >= 3.0.11)
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/14/include"
cargo build --no-default-features --features notcurses --bin tategaki-ed-terminal

# Run all tests
cargo test --workspace

# Run with clippy lints
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

## Project Structure

```
kakekotoba/
├── src/                    # Compiler crate
│   ├── main.rs            # CLI entry point
│   ├── lib.rs             # Library exports
│   ├── lexer.rs           # Unicode/Japanese tokenizer
│   ├── parser.rs          # Recursive descent parser
│   ├── ast.rs             # AST definitions
│   ├── types.rs           # Type system (HM + homomorphisms)
│   ├── inference.rs       # Type inference
│   ├── codegen.rs         # Code generation
│   ├── pipeline.rs        # Compilation orchestration
│   ├── error.rs           # Diagnostic errors
│   ├── vertical/          # Vertical text processing
│   ├── layout/            # 2D code layout analysis
│   ├── japanese/          # Japanese language support
│   └── spatial_ast/       # AST with 2D positioning
├── tategaki-ed/           # Vertical text editor (workspace member)
├── tests/                 # Integration tests
├── docs/                  # Project documentation
│   ├── ROADMAP.md         # Project roadmap
│   └── design-philosophy.md
└── .github/workflows/     # CI configuration
```
