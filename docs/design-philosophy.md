# Kakekotoba: Design Philosophy

*"A program is a poem that runs in two directions."*

## The Name

掛詞 (kakekotoba) is a rhetorical device from classical Japanese poetry (和歌, waka) in which a single word or phrase simultaneously carries two meanings. The pivot word belongs to both the phrase before it and the phrase after it, creating a deliberate ambiguity where two readings coexist — neither is "the real one."

For example, in classical poetry the word 松 (matsu) means both "pine tree" and "to wait." A poem about pine trees on the shore is simultaneously a poem about waiting for a lover. The two readings are not a pun or a trick — they are structurally inseparable. The poem means both things at once, and the depth comes from their interaction.

This is the organizing metaphor for the language.

## The Core Idea: Bidirectional Evaluation

Kakekotoba is designed to be a valid program when read in two directions:

- **Left-to-right, top-to-bottom** (横書き, yokogaki) — the modern/Western reading order
- **Top-to-bottom, right-to-left** (縦書き, tategaki) — the traditional Japanese reading order

This is not a rendering feature. It is a **semantic property of the language**. A kakekotoba program, properly written, expresses one computation when read horizontally and a related (possibly different, possibly equivalent) computation when read vertically.

The two readings are connected — they share syntax, they share tokens, and at certain points they *pivot* through shared expressions that participate in both readings simultaneously. Just as a 掛詞 in poetry belongs to two phrases at once, an expression at the intersection of horizontal and vertical flow belongs to two computations at once.

## From Pivot Words to 2D Orthography

### Level 1: Dual-Direction Evaluation

At the simplest level, a program is a grid of tokens. Reading the grid horizontally yields one token stream; reading it vertically yields another. Both must be valid programs.

```
Horizontal reading (→):  Row 1, then Row 2, then Row 3...
Vertical reading (↓):    Column 1, then Column 2, then Column 3...
                          (right-to-left column order for tategaki)
```

A trivial example — a program where both readings produce the same result — is uninteresting. The power comes from programs where the two readings produce *meaningfully different but structurally related* computations.

### Level 2: Pivot Points

Where a horizontal line of code and a vertical line of code intersect, there is a **pivot point** — a token or expression that participates in both computations. This is the direct analogue of 掛詞 in poetry.

```
         ┃
    ━━━━━╋━━━━━  horizontal computation
         ┃
         ┃        vertical computation
         ┃
```

The expression at the pivot must be well-typed in both contexts. This is a powerful constraint — it means the pivot expression is a kind of **natural transformation** between two computations, or more precisely, it is an element that is simultaneously valid in two algebraic structures.

### Level 3: Layers and Intersections

<!-- STATUS: This level is aspirational and under-defined. Needs formal treatment. -->

Beyond two directions, the 2D grid allows for **layers** — multiple computations encoded in the same source text, activated by different reading paths. A single "line" of code (whether horizontal or vertical) might participate in several computations depending on which crossing paths it intersects.

Questions to resolve:
- How many independent computations can a single source text encode?
- What is the formal relationship between reading paths and computation?
- Can reading paths be diagonal, curved, or otherwise non-axis-aligned?
- How does the type system constrain what can appear at multi-path intersections?
- Is there a notion of "depth" — layers that are not spatially separated but semantically stacked at the same position?

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

These are not translations of English keywords. The Japanese words carry mathematical intuitions that the English equivalents lack. 写像 *tells you* that a homomorphism is about preserving image-structure. 型 *tells you* that a type is a mold, not a label. The orthography encodes understanding.

### Vertical Flow and Program Structure

In traditional Japanese typesetting, text flows top-to-bottom within a column, and columns proceed right-to-left. This is not arbitrary — it reflects the physical motion of brush calligraphy, which naturally pulls downward.

For programming, vertical flow changes how you perceive structure:

- **Horizontal code** reads like prose — sequential, left-to-right causality
- **Vertical code** reads like a scroll — temporal, top-to-bottom unfolding

A function definition read horizontally might emphasize the transformation (input → output). The same definition read vertically might emphasize the *descent* through layers of abstraction (general → specific). The two readings illuminate different aspects of the same computation.

### Spatial Semantics

In kakekotoba, the position of code on the 2D plane is not merely formatting — it carries semantic weight.

- **Horizontal alignment** implies parallel structure (things at the same height are "at the same level")
- **Vertical alignment** implies sequential dependency (things in the same column share a thread)
- **Indentation** (in both directions) implies nesting, but nesting in horizontal vs vertical flow means different things
- **Whitespace** is not insignificant — the gap between two code regions is a meaningful boundary

This connects directly to the spatial AST: the abstract syntax tree carries positional metadata not as debug information but as part of the program's meaning.

## The Algebraic Connection

### Pivot Words as Homomorphisms

The 掛詞 metaphor maps precisely onto the mathematical concept at the language's core:

A **group homomorphism** φ: G → H is a mapping that preserves structure. It takes an element of one group and produces an element of another, such that the relationships between elements are maintained.

A **pivot word** is a token that is simultaneously an element of two "groups" (two reading contexts). The constraint that it must be well-typed in both contexts is exactly the constraint that a homomorphism must preserve structure in both groups.

```
Horizontal context (Group G):   ... a  b  [pivot]  c  d ...
                                              │
Vertical context (Group H):     ... x         │
                                    y         │
                                  [pivot]     │
                                    z         │
                                    w         │

The pivot is an element of both G and H.
The type constraint ensures it "means the same thing" in both —
i.e., structure is preserved across readings.
This is a homomorphism.
```

### Reading Paths as Functors

If we formalize a "reading" as a path through the 2D source text that yields a linear token stream, then:

- Each reading path is a **functor** from the 2D source to a 1D computation
- A pivot point is a **natural transformation** between two such functors
- The constraint that pivots are well-typed in both contexts is the **naturality condition**

This gives us a category-theoretic framework for reasoning about 2D programs:

```
                Source2D
               ╱        ╲
    readH     ╱          ╲    readV
             ╱            ╲
            ↓              ↓
      Computation_H ──→ Computation_V
                    pivot
              (natural transformation)
```

### Programs as Algebraic Structures

Combining all of this:

1. A kakekotoba program is a **2D arrangement of tokens**
2. **Reading paths** extract 1D computations from the 2D source
3. **Pivot points** connect different readings, constrained by type
4. The **type system** (HM + homomorphisms) ensures that structure is preserved across readings
5. **Group homomorphisms** in the type system are the formal mechanism for expressing "this transformation preserves structure" — the same guarantee that pivot words provide at the syntactic level

The language eats its own tail: the linguistic device (掛詞) that names it is also the mathematical structure (homomorphism) that defines its type system, which is also the programming paradigm (2D evaluation) that makes it unique.

## Relationship to Shaper

Shaper approaches the same territory from a different direction:

- Where kakekotoba sees **homomorphisms** (structure-preserving maps), Shaper sees **versors** (geometric transformations that preserve magnitude)
- Where kakekotoba sees **pivot words** (shared elements between two readings), Shaper sees **program-blades** (programs encoded as multivectors that can be projected to different grades)
- Where kakekotoba's 2D orthography gives multiple **reading paths** through source text, Shaper's grade system gives multiple **dimensional projections** of program structure

Both languages express the idea that a program has multiple valid interpretations that are connected by algebraic structure. Kakekotoba reaches this through Japanese poetics and abstract algebra. Shaper reaches it through geometric algebra and Clifford theory.

The convergence point is: **structure preservation under transformation is the fundamental operation of computation.** Both languages make this explicit where conventional languages leave it implicit.

## Open Questions

These are areas where the design philosophy is clear but the formal mechanisms need work:

1. **Evaluation semantics for 2D programs**: What exactly happens when you "run" a kakekotoba program? Is one direction primary and the other secondary? Are both evaluated? Is the relationship between readings checked at compile time only?

2. **Pivot typing rules**: What are the formal typing rules for pivot points? A pivot expression must be well-typed in two contexts — is this intersection typing, refinement typing, or something novel?

3. **Reading path algebra**: Can reading paths be composed? Can you define new reading paths beyond horizontal and vertical? What algebraic structure do paths form?

4. **Practical 2D editing**: How do you actually write 2D code? Tategaki-ed supports vertical text, but authoring code that is simultaneously valid in two directions is a much harder UX problem.

5. **Error reporting**: When a program is invalid, which reading produced the error? Can you have a program that is valid horizontally but invalid vertically? What does that mean?

6. **Incremental compilation**: If you change a token at a pivot point, both reading paths are affected. How does incremental compilation handle this?

7. **The multi-layer question**: Beyond two readings (horizontal + vertical), can a source text encode more than two computations? What is the upper bound, and what structures enable or constrain this?
