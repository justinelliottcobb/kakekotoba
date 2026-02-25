# Kakekotoba Roadmap

*Last updated: February 2026*

## Vision

Kakekotoba (掛詞) is a programming language where **algebraic structure is the fundamental abstraction**. Where traditional languages organize computation around functions and data, kakekotoba organizes it around **groups, homomorphisms, and type-theoretic structure** — the mathematics of structure-preserving transformations.

The name means "pivot word" — a classical Japanese poetic device where a word simultaneously carries two meanings. This duality mirrors the project's core idea: programs express both their computational content and their algebraic structure simultaneously.

### Relationship to Shaper

Kakekotoba is a sibling language to Shaper (ShaperOS's native language). Both are algebraic programming languages, but they approach algebra from different directions:

| | Kakekotoba | Shaper |
|---|---|---|
| **Foundation** | Abstract Type Theory + Group Homomorphisms | Geometric Algebra (Clifford algebras) |
| **Core abstraction** | Structure-preserving maps between algebraic objects | Multivectors and the geometric product |
| **Accessibility** | Human-first; geometry behind intuitive surface | AI-first; direct geometric access |
| **Type system** | HM inference + affine types + homomorphism types | Grade-based blade types + Schubert capabilities |
| **Meta-programming** | Homomorphisms as program transformations | Versors (rotors/reflectors) as program transformations |
| **Identity** | Japanese keywords, vertical text, poetic structure | Geometric notation, multi-script, spatial layout |
| **Math library** | Amari (dependency) | Amari (dependency) |

These are **convergent** approaches: group homomorphisms and geometric transformations are deeply related mathematically. The long-term goal is interoperability — kakekotoba programs targeting Shaper-asm, Shaper programs expressing kakekotoba's type structure, and shared tooling between the two ecosystems. Both languages share Amari as their geometric algebra foundation.

### Self-Hosting Goal

The north star is **self-hosting**: writing kakekotoba code in tategaki-ed (the project's vertical text editor), compiled by a kakekotoba compiler. This drives priorities — the editor must be usable, the language must be expressive enough, and the compiler must produce running code.

---

## Current State (February 2026)

### What Exists

**Compiler (kakekotoba crate)**
- Lexer: spatial tokenization with 2D positions works; regular `next_token()` is incomplete (only handles parens). Japanese keyword detection is fully implemented (41 keywords) but disconnected from the regular lexer path — only wired through spatial tokenization.
- Parser: recursive descent, parses function and type declarations. Expression parsing is minimal (literals and identifiers only — no operators, lambdas, application, let, or match).
- AST: complete definition including sum/product types, homomorphism declarations, pattern matching, lambda, let, if, match, binary/unary operators.
- Type system: HM with polymorphism, higher-kinded types, group homomorphism types, affine ownership types (complete definition in `types.rs`).
- Type inference: HM unification partially working — constraint generation, occurs check, basic solving implemented. Let-polymorphism and pattern matching inference stubbed. Binary/unary operator type checking works.
- Code generation: stubbed (LLVM/inkwell removed; bytecode VM approach planned per Phase 3).
- CLI: full pipeline orchestration with `--lex-only`, `--parse-only`, `--type-check-only`.
- Error handling: miette-based diagnostics with source spans.

**Vertical/Spatial Infrastructure**
- Bidirectional text processing (LTR/RTL/vertical) via unicode-bidi
- 2D code layout analysis (indentation, blocks, flow)
- Japanese keyword detection (41 keywords across 8 categories) and character classification (kanji, hiragana, katakana, ascii)
- Unicode normalization for Japanese text
- Spatial AST with 2D positional metadata
- Note: vertical/horizontal is a **presentational** choice, not semantic (see `docs/design-philosophy.md`)

**Editor (tategaki-ed)**
- Notcurses terminal backend with vertical text rendering
- Vim-like modal editing (Normal/Insert/Visual/Command)
- Full keyboard handler with multi-key commands (dd, yy, counts)
- File I/O in 4 formats (plain, JSON, spatial, markdown)
- Floating command bar with configurable positioning
- Blocked on system notcurses version (3.0.7 vs required 3.0.11+)

**Build Status**
- Workspace compiles with 0 errors (`cargo build --workspace`)
- 69/80 lib tests pass (11 pre-existing failures in scaffolded test expectations)
- `cargo fmt --check --all` passes

### Branch Structure (Gitflow)
- `main` — stable releases
- `develop` — integration branch
- Feature/chore/fix branches → PR to `develop` → release PR → `main`

---

## Phase 1: Editor Polish

**Goal**: Make tategaki-ed a genuinely usable text editor.

The editor is the eventual home for writing kakekotoba code. It needs to work well as a general-purpose editor before we layer on language-specific features.

### 1.0 Unblock Runtime Testing
- [ ] Resolve notcurses version requirement (build 3.0.11+ from source or update system package)
- [ ] Verify terminal editor launches and renders correctly
- [ ] Smoke test all vim modes (Normal, Insert, Visual, Command)

### 1.1 Core Editing Gaps
- [ ] Undo/redo stack (currently stubbed in the editor)
- [ ] Search with `/pattern` in normal mode
- [ ] Replace with `:%s/old/new/g` in command mode
- [ ] Line numbers in gutter

### 1.2 Productivity
- [ ] Multiple buffers (`:e file`, `:bn`, `:bp`, `:ls`)
- [ ] Command history (up/down in `:` mode)
- [ ] Visual mode selection completion (partial today)
- [ ] Yank registers (named registers `"a`, `"b`, etc.)

### 1.3 Code Editing Foundations
- [ ] Basic syntax highlighting framework (initially for kakekotoba source)
- [ ] Auto-indent on newline
- [ ] Bracket/paren matching and jump (`%`)

### 1.4 Stability
- [ ] Edge case handling for wide characters (CJK double-width)
- [ ] Terminal resize handling
- [ ] Graceful degradation when notcurses features unavailable

---

## Phase 2: Language Design Solidification

**Goal**: Nail down the kakekotoba language design before investing in backends.

The language has a strong type system definition but the surface syntax and semantics need to be fully specified. This phase is about writing a language reference and completing the parser.

### 2.1 Language Reference
- [ ] Write `docs/language-reference.md` covering:
  - Lexical structure (keywords in Japanese/ASCII, operators, literals)
  - Declarations (functions, types, imports)
  - Expressions (application, lambda, let, if, match, binary/unary ops)
  - Type system (primitives, constructors, polymorphism, higher-kinded types)
  - Group homomorphism declarations and types
  - Pattern matching (wildcards, constructors, guards, or-patterns)
  - Module system design
- [ ] Define operator precedence table (especially the Compose operator)
- [ ] Specify how group homomorphism properties are checked vs declared

### 2.2 Complete the Parser
- [ ] Full expression grammar (currently partial)
  - Lambda expressions
  - Match expressions with guards
  - Binary/unary operators with precedence climbing
  - Block expressions
- [ ] Homomorphism declaration syntax
- [ ] Import declarations
- [ ] Sum/product type definitions
- [ ] Error recovery for better diagnostics

### 2.3 Type System Formalization
- [ ] Specify the interaction between HM inference and homomorphism types
- [ ] Define when homomorphism properties are verified (compile-time vs runtime)
- [ ] Specify the kind system for higher-kinded types
- [ ] Define type class / trait mechanism (if any — or are homomorphisms sufficient?)

### 2.4 Example Programs
- [ ] Write 5-10 example programs that exercise the language's features
- [ ] "Hello world" equivalent
- [ ] A program demonstrating group homomorphisms
- [ ] A program using pattern matching and algebraic types
- [ ] A program showing higher-kinded type usage

---

## Phase 2.5: Minimal End-to-End Pipeline

**Goal**: Get the surface reading working — a minimal kakekotoba program runs and produces a result.

This is the "surface reading works" milestone. Before investing in a full bytecode VM, we need the lexer → parser → type checker → evaluation chain wired end-to-end for a minimal language subset. A tree-walking interpreter is acceptable here — the goal is to validate the language design, not optimize execution.

### Target
Run this and get a result:
```lisp
(定義 二倍 (数 -> 数) (* 2 数))
```

### 2.5.1 Wire Japanese Keywords into Lexer
- [ ] Connect `KeywordDetector` (41 keywords, fully implemented) to the regular `next_token()` path
- [ ] Tokenize: 定義, 場合, 数, 真, 偽, 無, 単位, and arithmetic operators
- [ ] S-expression tokenization: `(`, `)`, identifiers, literals, keywords

### 2.5.2 Complete Expression Parsing
- [ ] S-expression parsing (the initial syntax form, per the homoiconic design)
- [ ] Function application
- [ ] Binary/unary operators with precedence
- [ ] Lambda expressions
- [ ] Let bindings
- [ ] If expressions
- [ ] Match/場合 expressions

### 2.5.3 Tree-Walking Interpreter
- [ ] Evaluate literals, arithmetic, comparison, boolean operators
- [ ] Function definition and application
- [ ] Let bindings and closures
- [ ] Pattern matching (literal and identifier patterns)
- [ ] Basic I/O (print)
- [ ] REPL for interactive evaluation

### 2.5.4 Example Programs
- [ ] Factorial (場合/match, recursion)
- [ ] Fibonacci (二重再帰, double recursion)
- [ ] Map over a list (写像, higher-order functions)
- [ ] Simple algebraic type with pattern matching

---

## Phase 3: Bytecode VM

**Goal**: Get kakekotoba programs running via a bytecode interpreter.

Rather than keeping the tree-walking interpreter from Phase 2.5, kakekotoba targets a bytecode IR. This is inspired by (but not identical to) ShaperOS's shaper-asm architecture, providing a foundation that can later be compiled to native code, extended toward sasm compatibility, or target WASM.

### Design Principles (shaper-asm-informed)
- **Register-based VM** — avoids stack shuffling, maps well to native compilation later. ShaperOS's shaper-asm uses 256 general registers + specialized registers (grade, scalar, predicate) — kakekotoba's VM may use general + type + ownership registers.
- **Algebraic data in registers** — registers hold typed algebraic values, not just scalars. Algebraic operations powered by Amari (amari-core for Clifford algebra, amari-tropical for resource analysis).
- **Grade-segregated memory** — inspired by ShaperOS's memory model: memory organized by algebraic grade, bump allocation per grade, per-grade collection. In kakekotoba, "grade" maps to ownership tier (stack/heap/cache coordinate systems).
- **Conventional control flow** — branches, calls, returns (no need to reinvent this)
- **Interpretable and compilable** — same bytecode works for both modes
- **Self-contained** — no dependency on Shaper infrastructure (namespace, sprites, capabilities)

### Open Decision: Custom Bytecode vs WASM Extension
The bytecode format is not yet finalized. Two paths under consideration:

**Option A: Custom bytecode (sasm-inspired)**
- Full control over instruction set design
- Can express algebraic operations natively (homomorphism application, group operations)
- Natural path to sasm compatibility later
- More implementation effort

**Option B: WASM extension**
- Leverage existing WASM tooling (debuggers, profilers, wasm-bindgen)
- Portable by default
- Algebraic operations would be library calls rather than native instructions
- May constrain the design

This decision can be deferred until Phase 2 (language design) is complete enough to understand what the instruction set needs to express.

### 3.1 IR Design
- [ ] Define the kakekotoba bytecode instruction set
- [ ] Register model (general-purpose, typed algebraic, scalar, predicate)
- [ ] Module format (function table, type section, code section)
- [ ] Text format for debugging (`.kkir` or similar)

### 3.2 Compiler Frontend → IR
- [ ] Implement type inference engine (constraint generation + solving)
- [ ] Lower typed AST to bytecode IR
- [ ] Handle closures and captured variables
- [ ] Handle pattern matching compilation (decision trees)

### 3.3 Interpreter
- [ ] Register file implementation
- [ ] Core dispatch loop
- [ ] Algebraic operations (group operations, homomorphism application)
- [ ] Standard operations (arithmetic, comparison, control flow)
- [ ] Function calls with register windows
- [ ] Memory/heap for algebraic data structures

### 3.4 Standard Library (Minimal)
- [ ] Basic I/O (print, read)
- [ ] String operations
- [ ] List operations
- [ ] Core algebraic structures (integers as a group under addition, etc.)

### 3.5 REPL
- [ ] Interactive read-eval-print loop using the interpreter
- [ ] Expression evaluation with type display
- [ ] `:type` command for type inspection

---

## Phase 4: Native Compilation

**Goal**: Compile kakekotoba programs to native executables.

### 4.1 Backend Selection
- [ ] Evaluate options given the bytecode IR design:
  - LLVM (via inkwell) — maximum optimization, complex FFI
  - Cranelift — Rust-native, simpler, faster compilation
  - Custom native codegen — full control, significant effort
  - Shaper-asm target — emit sasm bytecode for the Shaper VM
- [ ] Implement chosen backend
- [ ] Benchmark against interpreter

### 4.2 Optimization
- [ ] Algebraic identity simplification at the IR level
- [ ] Homomorphism fusion (compose consecutive homomorphisms)
- [ ] Dead code elimination
- [ ] Inlining for small functions

---

## Phase 5: Self-Hosting

**Goal**: Write kakekotoba code in tategaki-ed, compiled by kakekotoba.

### 5.1 Editor ↔ Language Integration
- [ ] Kakekotoba syntax highlighting in tategaki-ed
- [ ] Inline type display / hover information
- [ ] Go-to-definition for kakekotoba source
- [ ] Error display from the compiler in the editor
- [ ] Compile-and-run from within the editor

### 5.2 Language Maturity
- [ ] Module system implementation
- [ ] Package/dependency management
- [ ] Standard library expansion
- [ ] FFI mechanism for calling Rust/C

### 5.3 Bootstrapping
- [ ] Rewrite components of the compiler/editor in kakekotoba
- [ ] Kakekotoba programs that express their own algebraic structure via homomorphisms

---

## Phase 6: Convergence with Shaper

**Goal**: Enable interoperability between kakekotoba and the Shaper ecosystem.

### 6.1 Shaper-asm Backend
- [ ] Emit Shaper-asm bytecode from kakekotoba programs
- [ ] Map kakekotoba's algebraic types to sasm's geometric types where applicable
- [ ] Homomorphism application → versor sandwich where the algebras align

### 6.2 Cross-Language Interop
- [ ] Import Shaper modules from kakekotoba
- [ ] Shared type representations for common algebraic structures
- [ ] Foreign function interface between the two languages

### 6.3 Theoretical Unification
- [ ] Formal relationship between group homomorphisms and geometric versors
- [ ] Shared algebraic optimization passes
- [ ] Unified program-space representation
- [ ] Amari as the shared mathematical foundation — both languages depend on it for their algebraic operations, making it the natural interop layer

---

## Principles

1. **Algebra first** — Every design decision should respect the algebraic foundations. Group homomorphisms are not a feature bolted on; they are the organizing principle.

2. **Japanese nativity** — Japanese keywords are not translations of English. The language is designed for expression in Japanese, with ASCII as the fallback.

3. **Vertical text as presentation** — Tategaki (vertical writing) and yokogaki (horizontal writing) are display choices, not semantic differences. The spatial AST and layout system treat 2D text arrangement as a first-class concern for tooling and readability, but the program's meaning is independent of orientation.

4. **Accessible depth** — The geometric machinery (Amari-powered rotors, grade-segregated memory, subspace liveness) is available when needed but hidden behind ownership semantics and Japanese keywords by default. Most programmers stay at the surface. The geometry is there when you need it.

5. **Convergent, not identical** — Kakekotoba and Shaper approach the same mathematical territory (algebraic structure, transformation, preservation) from different entry points. They should converge through interoperability, not become the same language.

6. **Working software over perfect design** — Each phase should produce something that runs. The interpreter before the optimizing compiler. The editor before the IDE. Working examples before the formal specification. Surface reading before deep reading.
