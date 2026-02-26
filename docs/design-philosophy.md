# Kakekotoba: Design Philosophy

*"The kanji carries both meanings at once. So does the code."*

## The Name

掛詞 (kakekotoba) is a rhetorical device from classical Japanese poetry (和歌, waka) in which a single word or phrase simultaneously carries two meanings. The pivot word belongs to both the phrase before it and the phrase after it, creating a deliberate ambiguity where two readings coexist — neither is "the real one."

For example, in classical poetry the word 松 (matsu) means both "pine tree" and "to wait." A poem about pine trees on the shore is simultaneously a poem about waiting for a lover. The two readings are not a pun or a trick — they are structurally inseparable. The poem means both things at once, and the depth comes from their interaction.

This is the organizing metaphor for the language.

## The Core Idea: Multiple Readings Through Homomorphisms

Kakekotoba is a programming language where the same code simultaneously carries multiple algebraic interpretations. The "pivot" is not spatial — it is semantic. A group homomorphism maps one algebraic structure to another while preserving relationships. In kakekotoba, a single program expresses both its computational content and its algebraic structure simultaneously, and the type system verifies that these readings are consistent.

### What Bidirectionality Is (and Isn't)

The language supports both horizontal (横書き, yokogaki) and vertical (縦書き, tategaki) text layout. This is a **presentational choice**, not a semantic one. Vertical and horizontal are display options — the same program, the same AST, the same types. A programmer or editor may prefer one orientation over the other for readability, aesthetic, or cultural reasons, but the meaning does not change.

The semantic "multiple readings" come from the homomorphic type system, not from spatial direction. When a kanji keyword like 写像 (mapping/homomorphism) appears in code, it carries both an operational meaning (this function maps values) and an algebraic meaning (the type checker should verify structure preservation). The kanji is the pivot word. The two readings coexist in the type system, not in the layout.

## Three Layers of Reading

### Layer 1: Surface Reading (表読み)

At the surface, kakekotoba is a statically-typed functional programming language with Japanese keywords, Hindley-Milner type inference, affine ownership, pattern matching, and algebraic data types. A programmer can write and run kakekotoba code without ever thinking about geometric algebra or group theory.

```lisp
(定義 階乗 (数 -> 数)
  (場合 数
    (零 -> 一)
    (n -> (* n (階乗 (- n 1))))))
```

This is a factorial function. It reads as ordinary functional programming — define a function, match on cases, recurse. The Japanese keywords are native, not translations: 定義 means "definition," 場合 means "case/situation," 数 means "number." The surface reading is complete and useful on its own.

### Layer 2: Deep Reading (裏読み)

Certain kanji keywords carry a second meaning that engages the type checker's algebraic verification. This is the kakekotoba — the pivot where one word means two things simultaneously.

For example, 写像 means "mapping" in everyday mathematical Japanese. In kakekotoba, writing 写像 in a type position also instructs the compiler to verify that the mapping preserves algebraic structure — it is simultaneously a function declaration and a homomorphism assertion.

```lisp
(定義 変換 (型A -> 型B)
  (写像 構造を 保つ))
```

At the surface, this defines a function `変換` (transform) from `型A` to `型B`. At the deep reading, 写像...保つ ("mapping that preserves") triggers homomorphism verification — the compiler checks that the transformation actually preserves the algebraic structure between source and target types.

The programmer didn't switch modes or drop into a metalanguage. They wrote the same Japanese. The kanji themselves carry both readings.

### Layer 3: Geometric Reading (幾何読み)

Below the algebraic layer, kakekotoba's runtime uses geometric algebra (via the Amari library) for memory management, ownership transformations, and performance-critical operations. This layer is normally invisible — the programmer sees affine ownership semantics while the runtime expresses them as geometric transformations.

When a programmer needs direct access to the geometric layer (for optimization, low-level memory control, or interop with Shaper), kakekotoba provides it through its own idioms — not by switching to a geometric sublanguage, but by using deeper readings of existing keywords.

The coordinate systems for memory management — 棧座標系 (stack), 堆座標系 (heap), 快取座標系 (cache) — are expressed as algebraic structures with homomorphisms between them. Moving data between memory regions is a structure-preserving transformation, verified by the same type machinery that handles Layer 2.

## Why Orthography Matters

### Kanji as Type-Dense Identifiers

Japanese kanji are logographic — each character carries semantic content far denser than alphabetic words. This is not incidental to the language design; it is structurally important.

| Kakekotoba keyword | Literal meaning | What it encodes |
|---|---|---|
| 関数 (kansuu) | "barrier-number" / "related number" | function — a relationship between numbers |
| 写像 (shazo) | "copy-image" | homomorphism — a structure-preserving mapping (literally, an image that copies structure) |
| 型 (kata) | "mold" / "form" | type — a mold that shapes values |
| もし (moshi) | "if it were the case that..." | conditional — carries a sense of hypotheticality that `if` does not |
| 繰り返し (kurikaeshi) | "repeat-return" | loop — literally the act of turning back and repeating |
| 束縛 (sokubaku) | "bundle-bind" | let binding — bundling a name to a value, with connotations of constraint |
| 場合 (baai) | "place-fit" / "occasion" | pattern match — the situation/case that fits |
| 準同型 (jundoukei) | "quasi-same-form" | homomorphism — literally "nearly the same shape" |
| 群 (gun) | "group/flock" | algebraic group — a collection with structure |

These are not translations of English keywords. The Japanese words carry mathematical intuitions that the English equivalents lack. 写像 *tells you* that a homomorphism is about preserving image-structure. 型 *tells you* that a type is a mold, not a label. 準同型 *tells you* that a homomorphism produces something "nearly the same shape." The orthography encodes understanding.

The kakekotoba design exploits kanji's natural property of multiple readings (音読み on'yomi and 訓読み kun'yomi) — a single character like 型 can be read as "kata" (form/mold, the native Japanese reading) or "kei" (type/model, the Chinese-derived reading). These different readings naturally suggest different levels of abstraction, which the language leverages for its multi-layer reading system.

### Vertical Flow as Presentational Choice

In traditional Japanese typesetting, text flows top-to-bottom within a column, and columns proceed right-to-left. This is not arbitrary — it reflects the physical motion of brush calligraphy, which naturally pulls downward.

For programming, vertical flow changes how you *perceive* structure without changing the computation:

- **Horizontal code** reads like prose — sequential, left-to-right causality
- **Vertical code** reads like a scroll — temporal, top-to-bottom unfolding

A function definition read horizontally might emphasize the transformation (input → output). The same definition read vertically might emphasize the *descent* through layers of abstraction (general → specific). The two orientations illuminate different aspects of the same computation, but they are the same program.

Tategaki-ed, the project's vertical text editor, supports both orientations. The choice is the programmer's.

### Spatial Layout

In kakekotoba, the position of code on the 2D plane carries layout information that the compiler and editor can use for presentation and analysis:

- **Horizontal alignment** implies parallel structure (things at the same height are "at the same level")
- **Vertical alignment** implies sequential dependency (things in the same column share a thread)
- **Indentation** implies nesting depth, which in the geometric memory model maps to memory hierarchy depth
- **Whitespace** boundaries are meaningful for block detection and layout analysis

The spatial AST carries positional metadata that supports the editor, diagnostics, and the eventual geometric memory layer — but this is infrastructure for tooling and optimization, not a mechanism for dual evaluation.

## The Algebraic Connection

### Pivot Words as Homomorphisms

The 掛詞 metaphor maps onto the mathematical concept at the language's core:

A **group homomorphism** φ: G → H is a mapping that preserves structure. It takes an element of one group and produces an element of another, such that the relationships between elements are maintained.

A **pivot word** in kakekotoba is a keyword that simultaneously operates at two levels of the type system. When a programmer writes 写像, they are both defining a mapping (the operational meaning) and asserting structure preservation (the algebraic meaning). The type checker must verify both readings are consistent — this is exactly the constraint that a homomorphism must preserve structure.

```
Surface reading:    写像 = "this function maps A to B"
                      │
                      │  (same kanji, same code)
                      │
Deep reading:       写像 = "this mapping preserves the algebraic
                           structure between A and B"

The pivot word carries both meanings simultaneously.
The type system verifies their consistency.
This IS the homomorphism.
```

### Programs as Algebraic Structures

Combining all of this:

1. A kakekotoba program is a **typed functional program with Japanese keywords**
2. **Kanji keywords** carry multiple readings — operational and algebraic — simultaneously
3. The **type system** (HM + affine types + homomorphism types) verifies that readings are consistent
4. **Group homomorphisms** in the type system are the formal mechanism for expressing "this transformation preserves structure"
5. The **geometric layer** (Amari-powered) provides the runtime substrate for ownership, memory management, and algebraic operations

The language eats its own tail: the literary device (掛詞) that names it is also the mathematical structure (homomorphism) that defines its type system, and the kanji orthography that makes it possible.

## The Accessibility Gradient

Kakekotoba inverts the priorities of its sibling language, Shaper. Where Shaper is designed to be AI-accessible first and human-accessible second, exposing geometric algebra directly as the programming model, kakekotoba is **human-first**.

The model is similar to cliffy-tsukoshi (a geometric state management library): the geometric algebra is the *implementation*, not the *interface*. Users of tsukoshi call `.blend()` and `.applyRotor()` without knowing what a bivector is. Similarly, kakekotoba programmers write typed functional programs with Japanese keywords, and the geometric machinery works behind the scenes.

The gradient:

| Layer | What the programmer sees | What the compiler does |
|---|---|---|
| Surface | Japanese keywords, pattern matching, ownership | Standard compilation with affine type checking |
| Algebraic | 写像 (homomorphism), 群 (group), 準同型 assertions | Verifies structure preservation across type algebras |
| Geometric | 棧座標系/堆座標系/快取座標系 coordinate annotations | Uses Amari rotors and grade-segregated memory |

Most programmers stay at the surface. Those who need algebraic guarantees use the deep reading. Those who need direct geometric control can reach the bottom layer. But the surface is always sufficient — you never *have* to descend.

## Amari as Mathematical Foundation

The geometric algebra operations in kakekotoba are powered by [Amari](https://crates.io/crates/amari), a Rust geometric algebra library. Amari replaces nalgebra (referenced in the project's earliest design discussions) with a purpose-built algebraic toolkit.

### Core Type Mapping

Amari's types map naturally onto kakekotoba's type system:

| Amari type | Const-generic signature | Kakekotoba type | Role |
|---|---|---|---|
| `Multivector<P,Q,R>` | Cl(p,q,r) element | `(型 多元体 (P Q R))` | General Clifford algebra element |
| `Rotor<P,Q,R>` | Even-grade, unit norm | `(型 回転子 (P Q R))` | Rotation/transformation operator |
| `Vector<P,Q,R>` | Grade-1 blade | `(型 ベクトル (P Q R))` | Geometric direction |
| `Bivector<P,Q,R>` | Grade-2 blade | `(型 二重ベクトル (P Q R))` | Rotation plane / area element |
| `Scalar<P,Q,R>` | Grade-0 | Kakekotoba `数` (number) | Magnitude without direction |
| `VerifiedMultivector<T,P,Q,R>` | Phantom-typed | `Type::Constructor` with phantom params | Compile-time algebraic guarantees |
| `Signature<P,Q,R>` | Pure phantom type | Metric encoding in kakekotoba's type system | Zero-cost metric verification |

The key operation is `Rotor::apply(v) → Multivector` — the sandwich product R\*v\*R† — which is the core geometric transform and the runtime implementation of kakekotoba's homomorphic transformations between type algebras.

### Amari Operations as Kakekotoba Homomorphisms

| Amari operation | Algebraic meaning | Kakekotoba homomorphism |
|---|---|---|
| `grade_projection(k)` | Cl → Cl_k | Grade-preserving endomorphism |
| `Rotor::from_bivector` | Bivector → Rotor | Exponential map |
| `geometric_product` | Cl × Cl → Cl | Group operation |
| `reverse()` | Cl → Cl | Anti-automorphism |
| Dual number lift | f(x) → (f(x), f'(x)) | DualNumber homomorphism (amari-dual) |
| Tropical interpretation | (+ ×) → (max +) | TropicalNumber homomorphism (amari-tropical) |

### Amari Crates and Their Kakekotoba Roles

| Amari crate | Role in kakekotoba |
|---|---|
| `amari-core` | Clifford algebra, rotors, Cayley tables — the mathematical backbone for homomorphic transformations between type/runtime algebras |
| `amari-tropical` | Tropical algebra (max-plus semirings) for resource analysis, affine type cost modeling, and type constraint solving |
| `amari-dual` | Dual numbers (ε²=0) for automatic differentiation — sensitivity analysis of type transformations and Jacobian computation |
| `amari-holographic` | Holographic/VSA memory for potential homoiconic code-as-data representation |
| `amari-automata` | Geometric cellular automata, Cayley graphs, inverse design, self-assembly — the most kakekotoba-relevant crate beyond core (see below) |
| `amari-info-geom` | Information geometry (Fisher metric, Bregman divergence, dually flat manifolds) — type distance metrics and statistical type inference |

### amari-automata: The Computational Algebra Bridge

The `amari-automata` crate is uniquely relevant to kakekotoba because it combines three algebraic frameworks — geometric algebra, dual numbers, and tropical algebra — into five subsystems that map directly onto kakekotoba's architecture:

**1. GeometricCA\<P,Q,R\>** — Code as evolving cells. Cells are multivectors, evolution follows algebraic rules. The `RuleType` enum classifies transformations: `GradePreserving` (doesn't change abstraction level), `Reversible` (undoable refactoring), `Conservative` (preserves total program behavior). In kakekotoba, code blocks as CA cells means refactoring is algebraically structured evolution.

**2. CayleyGraph\<P,Q,R\>** — Navigating between program readings. The group structure as a navigable graph where nodes are algebraic interpretations and edges are homomorphisms. `CayleyNavigator` tracks position and path history through the graph. In kakekotoba, this models navigation between the multiple readings of a program — surface → deep → geometric — where each edge is a structure-preserving transformation and the `CayleyTable` caches all pairwise compositions.

**3. InverseDesigner\<T,P,Q,R\>** — Generalized type inference. Uses dual numbers for gradient descent through CA evolution to find seeds that produce target configurations. In kakekotoba, "given a target type/behavior, find the program structure" is exactly inverse design — making it a natural foundation for type inference that goes beyond simple unification.

**4. SelfAssembler\<P,Q,R\> + UIAssembler** — Self-arranging code layout. Components self-arrange via geometric affinities (affinity cache matrix). `UIAssembler` adds `LayoutEngine` for UI-specific assembly. In kakekotoba, related functions attract and unrelated ones repel, so vertical layout in tategaki-ed emerges from algebra rather than manual arrangement.

**5. TropicalSolver\<T,DIM\>** — Type constraint optimization. Linearizes discrete constraint problems via tropical algebra. When multiple valid type assignments exist, tropical optimization finds the best one among candidates — complementing HM unification with an optimization layer.

These five subsystems are not theoretical — they have concrete implementations in Amari with well-defined APIs. The integration path is: kakekotoba's `Type::Constructor` wraps Amari types with `(P,Q,R)` metric signatures encoded as const-generic parameters, and `Type::Homomorphism` wraps Amari operations that preserve algebraic structure.

Amari is a **dependency** of kakekotoba. ShaperOS is **inspiration** — its grade-segregated memory model, register-based bytecode VM (shaper-asm), and geometric garbage collection ("liveness is a subspace") inform kakekotoba's runtime design, but kakekotoba does not depend on ShaperOS infrastructure.

## Relationship to Shaper

Shaper approaches the same mathematical territory from a different direction and with different priorities:

| | Kakekotoba | Shaper |
|---|---|---|
| **Foundation** | Abstract Type Theory + Group Homomorphisms | Geometric Algebra (Clifford algebras) |
| **Core abstraction** | Structure-preserving maps between algebraic objects | Multivectors and the geometric product |
| **Accessibility** | Human-first; geometry behind intuitive surface | AI-first; direct geometric access |
| **Type system** | HM inference + affine types + homomorphism types | Grade-based blade types + Schubert capabilities |
| **Meta-programming** | Homomorphisms as program transformations | Versors (rotors/reflectors) as program transformations |
| **Identity** | Japanese keywords, vertical text, poetic structure | Geometric notation, multi-script, spatial layout |
| **Geometric access** | Available when needed, reached through language idioms | Always present, the default mode of expression |
| **Math library** | Amari (dependency) | Amari (dependency) |

These are **convergent** approaches: group homomorphisms and geometric transformations are deeply related mathematically. The long-term goal is interoperability — kakekotoba programs targeting Shaper-asm, Shaper programs expressing kakekotoba's type structure, and a shared mathematical foundation in Amari.

The key difference is the entry point. A kakekotoba programmer starts with Japanese keywords, pattern matching, and typed functional programming — familiar PL concepts expressed in kanji. They reach geometric depth through the language's own multiple-reading system. A Shaper programmer starts with multivectors, the geometric product, and grade-typed blades — they are always in the geometry.

## Open Questions

These are areas where the design philosophy is clear but the formal mechanisms need work:

1. **The preciousness boundary**: How far can kanji multiple-readings be pushed before the language becomes too clever and fights comprehensibility? Kanji with 11+ readings exist, so the space is wide open — but the line between depth and obscurity needs to be felt out in development.

2. **Homomorphism verification**: How to formalize and verify that homomorphic transformations preserve semantics. What properties must be checked at compile time vs. runtime? What proof obligations does the programmer incur when using 写像?

3. **Reading triggers**: When and how are different readings activated? Is it implicit from kanji choice (using 写像 vs a plain function definition), explicit via annotation, or determined by context? Can the programmer control which readings the compiler checks?

4. **Affine types with closures**: Linear and affine types interact non-trivially with closures and partial application. The ownership model needs to handle captured variables, moved values in lambdas, and partially applied functions cleanly.

5. **Geometric memory depth**: Whether indentation-as-memory-depth is practical. ShaperOS's grade-segregated memory provides a concrete implementation model (bump allocation per grade, per-grade collection), but mapping this to a programming language's ownership system is uncharted territory.

6. **The descent boundary**: When does a programmer need to "reach down" from the surface to the geometric layer? What problems require it? Can the compiler automatically select the geometric representation, or must the programmer annotate?

7. **Self-hosting path**: The north star is writing kakekotoba in kakekotoba. What minimal subset of the language is needed to bootstrap? Which compiler components can be expressed in the language's own type system?

8. **Const-generic metric encoding**: Amari's types are parameterized by `(P, Q, R)` const-generics representing the metric signature of the algebra (e.g., Euclidean 3D is `(3,0,0)`, Minkowski spacetime is `(3,1,0)`). Kakekotoba's type system needs to encode these — either as type-level naturals, as phantom type parameters, or through a dedicated metric kind. The choice affects how expressive the geometric layer can be at compile time.

9. **CayleyGraph as IDE navigation model**: amari-automata's `CayleyNavigator` tracks position and path history through the group structure graph. This maps naturally to an IDE feature where the programmer navigates between different readings of the same code — the editor shows which algebraic interpretation you're viewing, and you can traverse edges (homomorphisms) to see other readings. The question is whether this is practical UX or merely elegant mathematics.
