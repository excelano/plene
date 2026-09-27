# plene — Design

**plene** shows Rust source alongside an *expanded transcription*: the same code with abbreviations and symbol tokens written out in longhand, color-matched to the original.

The name comes from Hebrew *plene spelling* (ktiv male): the same words in the same language, written with fuller spelling to make them easier to read.

## Goal and non-goals

**Goal:** make Rust easier to read for someone with basic Rust knowledge by expanding the dense parts (abbreviated keywords, operators, punctuation) into words.

**Non-goals:**
- Not a new language, not a translation, not a teaching tool.
- No round-trip. Output is display-only. Rust's own tools do all real work.
- Does not explain semantics. It only spells out what is written.

## Principle: expand, don't translate

- Expand abbreviations that are specific to Rust (`fn`, `pub`, `mut`, `impl`, `mod`, `dyn`, `ref`, `extern`) and operator/punctuation tokens.
- Keep abbreviations that are standard vocabulary across programming languages (`struct`, `enum`, `const`, `async`, `str`, `i32`, `Vec`). Expanding them to `structure` or `enumeration` makes code harder to read, not easier.
- Never expand into another abbreviation.
- Everything else is byte-for-byte unchanged: identifiers, std types, full-word keywords, `::`, generics, macros, attributes, comments, strings, braces, indentation.
- Each expansion is consistent: the same token in the same syntactic role always gets the same text.
- Expansions are keyed by **token + syntactic role**, never by token text alone.
- **Every expansion replaces exactly one token.** Multi-token forms compose: `&mut x` is `&` → `borrow` plus `mut` → `mutable`, giving `borrow mutable x`; `*const T` gives `raw pointer const T`.

## Glossary v1

### Keep as-is

Identifiers; std types (`Vec`, `Box`, `str`, `i32`…); keywords not listed below (`let`, `match`, `move`, `if`, `else`, `for`, `in`, `loop`, `while`, `return`, `where`, `type`, `struct`, `enum`, `trait`, `use`, `as`, `self`, `Self`, `static`, `const`, `unsafe`, `async`, `await`, `crate`, `super`); `Some`/`None`/`Ok`/`Err`; arithmetic and comparison operators; `=` and arithmetic compound assignment; `&&`; `||` (logical); `.`; `::`; `:` in plain type annotations; `;`; `,`; all brackets; `#[…]` and `#![…]` attributes; the `!` in macro names; the never type `!`.

### Abbreviated keywords

| Token | Role | Expansion |
|---|---|---|
| `fn` | `keyword` | `function` |
| `pub` | `keyword` | `public` (so `pub(crate)` → `public(crate)`) |
| `mut` | `keyword` | `mutable` |
| `impl` | `impl_block` | `implement` |
| `impl` | `impl_trait_type` (argument or return position) | `some` |
| `mod` | `keyword` | `module` |
| `dyn` | `keyword` | `dynamic` |
| `ref` | `keyword` | `by reference` (so `ref mut x` → `by reference mutable x`) |
| `extern` | `keyword` | `external` |

### Operators and punctuation, by role

| Token | Role | Context | Expansion |
|---|---|---|---|
| `&` | `ref_expr` | borrow expression | `borrow` |
| `&` | `ref_type` | reference type | `borrowed` |
| `&` | `ref_pattern` | reference pattern | `borrowed` |
| `&` | `self_param` | `&self`, `&mut self` | `borrowed` |
| `&` | `binary` | bitwise and | `bitwise and` |
| `\|` | `binary` | bitwise or | `bitwise or` |
| `^` | `binary` | bitwise xor | `bitwise xor` |
| `<<` / `>>` | `binary` | shifts | `shift left` / `shift right` |
| `&=` `\|=` `^=` `<<=` `>>=` | `compound_assign` | | `bitwise and=`, `bitwise or=`, `bitwise xor=`, `shift left=`, `shift right=` |
| `*` | `deref` | dereference | `dereference` |
| `*` | `ptr_type` | raw pointer type | `raw pointer` |
| `*` | `binary` | multiplication | *(keep)* |
| `?` | `try` | try operator | `or return early` |
| `?` | `try_chained` | try operator whose result is used by a method call, field access or index (`foo()?.bar()`) | *(keep; expanded-span style and hover note)* |
| `?` | `maybe_bound` | `?Sized` | `maybe` |
| `->` | `ret_type` | return type (fn, closure, `Fn` types) | `returns` |
| `\|` | `closure_open` | opening closure pipe | `closure(` |
| `\|` | `closure_close` | closing closure pipe | `)` (so `\|\|` → `closure()`) |
| `'a` | `lifetime` | any named lifetime, including `'static` | `lifetime {name}` |
| `'_` | `lifetime_anonymous` | anonymous lifetime | `lifetime inferred` |
| `'label` | `label` | loop label, definition or use | `label {name}` |
| `!` | `not` | prefix logical not | `not` |
| `!` | `negative_impl` | `impl !Send for T` | `not` |
| `..` | `range` | `a..b`, `..b` | `up to` |
| `..` | `range_from` | `a..` | `onward` |
| `..` | `range_full` | `..` alone, as in `&v[..]` | `all` |
| `..=` | `range_inclusive` | `a..=b`, `..=b` | `through` |
| `..` | `struct_update` | `..base` | `rest from` |
| `..` | `rest_pattern` | `[a, ..]`, `Foo { a, .. }` | `and the rest` |
| `=>` | `match_arm` | | `gives` |
| `\|` | `pattern_or` | pattern alternative | `or` |
| `@` | `pattern_binding` | | `bound as` |
| `_` | `wildcard` | wildcard pattern | `anything` |
| `_` | `let_discard` | `_` as the whole pattern of a `let` | `discard` |
| `_` | `inferred_type` | `Vec<_>` | `inferred` |
| `:` | `trait_bound` | generics, `where`, supertraits, when the first bound is a trait | `implementing` |
| `:` | `lifetime_bound` | `'a: 'b`, `T: 'a`, when the first bound is a lifetime | `outliving` |
| `+` | `bound_separator` | between bounds | `and` |

Examples and notes:
- `&'a T` → `borrowed lifetime a T`; `&'a mut T` → `borrowed lifetime a mutable T`; `&mut self` → `borrowed mutable self`.
- `move |x| …` → `move closure(x) …`.
- `'outer: loop` → `label outer: loop`; `break 'outer` → `break label outer`.
- `T: Clone + 'a` → `T implementing Clone and lifetime a`.
- `write!(f, "x")?;` → `write!(f, "x") or return early;`, but `foo()?.bar()` keeps its `?`, since `foo() or return early.bar()` doesn't read.
- `&&x`, `&&T` and `&&pat` parse as two `&` tokens in nested nodes, so a double borrow needs no special handling. An empty closure's `||` likewise parses as two pipes in the closure's parameter list, while logical `||` is a single token.
- `_x`-style identifiers are identifiers; leave them alone.
- Syntax alone can't see types, so some expansions are approximate. `&`, `|` and `^` on `bool` operands are non-short-circuit logical operators, not bitwise ones, and `!` on an integer is bitwise not. The glossary uses the common reading.

## Glossary file format

The default glossary lives at `crates/plene-core/glossary.toml` and is embedded via `include_str!`. It sits inside the crate so `cargo publish` packages it. The user may override entries with `~/.config/plene/glossary.toml` or `--glossary <path>`; overrides merge by `(token, role)`.

```toml
[[expand]]
token = "&"
role = "ref_expr"
text = "borrow"
note = "Takes a shared reference to a value without moving it."

[[expand]]
token = "'a"
role = "lifetime"
text = "lifetime {name}"
note = "A named lifetime: how long a reference is valid."
```

- Role names are the stable string identifiers in the tables above, defined in `plene-core` and mapped from `ra_ap_syntax` node/token kinds.
- `{name}` in `text` is replaced with the lifetime or label name without its leading `'`. The `token` field for these roles is a representative form (`'a`, `'label`).
- `note` is shown as hover text in the GUI (a built-in dictionary).
- `plene --dump-glossary` prints the effective glossary.
- Unknown roles in a user file produce a warning, not an error.

## Architecture

Cargo workspace, three crates:

```
plene/
├── Cargo.toml              # [workspace]
└── crates/
    ├── plene-core/         # library: parse, classify, expand → spans
    │   └── glossary.toml   # default glossary
    ├── plene/              # CLI binary
    └── plene-gui/          # eframe/egui app
```

### plene-core

- Parses with `ra_ap_syntax`, which is lossless and error-tolerant. Pin it to an exact version (`=0.0.x`); it releases weekly and breaks its API between releases.
- Parses with an explicit edition (default 2021), since the edition changes keyword lexing.
- Walks every token and determines its role from the parent node kind.
- Emits, per source line, a list of spans:

```rust
pub struct Span {
    pub original: String,        // exact source text
    pub rendered: String,        // expansion, or same as original
    pub class: HighlightClass,   // keyword, operator, ident, type, lifetime, string, comment, ...
    pub role: Option<Role>,      // set when the token has a glossary role
    pub source_range: Range<usize>, // byte offsets into the input
}
```

- The public API exposes no `ra_ap_syntax` types, so the CLI and GUI don't depend on it.
- **Highlighting is driven by the same token kinds that drive expansion.** Do not use a separate highlighter such as syntect. An expanded token keeps its original token's highlight class, so `&mut` and `borrow mutable` share a color by construction.
- **Whitespace:** never alter source whitespace or indentation. Where an expansion's word edge touches a character that is not whitespace, insert a space, unless that character is punctuation that hugs the word from its side: `( [ { <` before it, or `) ] } > , ; : . (` after it. So `foo()?;` → `foo() or return early;`, `&mut` → `borrow mutable`, `&[u8]` → `borrowed [u8]`, and `()->u8` → `() returns u8`, but `(&x)` → `(borrow x)`, `<'a>` → `<lifetime a>`, and `pub(crate)` → `public(crate)`. The cost of `(` hugging is that the rare `&(a, b)` reads `borrow(a, b)`. An inserted space is its own span with an empty `original`, so each expansion's `rendered` is exactly its glossary text.
- **Line correspondence is 1:1.** Every source line produces exactly one rendered line. Tokens that span lines (block comments, multi-line and raw strings) are split into one span per line. Line endings (LF or CRLF) are preserved.
- **Macros:** v1 leaves the token trees of macro invocations untouched, since they aren't parsed as Rust. Attributes are untouched too. `macro_rules!` bodies are never expanded.
- **Parse errors:** render everything; tokens inside error nodes are not expanded.
- No I/O and no CLI/GUI dependencies.

### plene (CLI)

```
plene [OPTIONS] <FILE|->

  (default)          interleaved: each source line, then its expansion beneath when it differs
  --side-by-side     two columns (for wide terminals)
  --changed-only     only lines that differ, prefixed with line numbers
  --expanded         expanded text only
  --color <auto|always|never>
  --theme <dark|light>              (default dark)
  --edition <2015|2018|2021|2024>   (default 2021)
  --glossary <PATH>
  --dump-glossary
```

- Interleaved output marks lines in a two-column gutter: blank for source lines, `» ` for expansion lines, so the two are distinguishable without color and indentation stays aligned. With color, expansion lines also sit on a subtle background band, padded with spaces to the widest expansion line so the bands form an even block. Padding uses spaces rather than an erase-to-end-of-line escape, which `less -R` would print literally. `--theme <dark|light>` (default dark) picks the band for the terminal's background.
- ANSI color via the highlight classes, from the terminal's 16-color palette so it follows the user's theme. Tokens with a glossary role are underlined on both lines, pairing each token with its expansion.
- A closed reader (`plene file.rs | head`) ends output quietly. Errors print `plene: …` and exit 1; argument errors exit 2.
- `-` reads stdin.
- Should work cleanly with `less -R`.

### plene-gui (eframe/egui)

- Open a `.rs` file via a dialog, drag-and-drop, or CLI argument.
- Two columns, original on the left and expansion on the right, with line numbers.
- One scroll area holding one row per source line, each row with two cells. Rows stay aligned and scroll together by construction: long lines wrap, and each row's height is the taller of its two cells. Wrapped rows rule out `show_rows` virtualization, which is acceptable for v1.
- Syntax highlighting on both sides uses matching colors. Expanded spans are visually marked.
- Hovering an expanded span shows the original token, its role, and the glossary `note`.
- A toggle for changed-only lines.
- Light and dark themes.
- Reload when the file changes on disk (nice to have).

## Testing

- **Snapshot tests** (`insta`) over a fixture corpus covering every glossary role. Each role needs at least one fixture. CRLF input and multi-line tokens get fixtures too.
- **Invariant:** concatenating `original` across all spans reproduces the input byte-for-byte. Test this on every fixture and on real crates.
- **Smoke test** (`#[ignore]`; `cargo test --release -- --ignored`): run over every `.rs` file under `PLENE_SMOKE_DIRS` (colon-separated) and assert the invariants, no panics, and 1:1 line counts. The default corpus is `~/plene-corpus` (shallow clones of ripgrep, serde and rust-analyzer, whose `crates/parser/test_data` holds deliberately malformed edge cases) plus the standard library from `rustup component add rust-src`.
- **Role coverage test:** every role in the default glossary is exercised by at least one fixture.

## Milestones

1. **Core + CLI, minimum subset:** `fn`, `pub`, `mut`, `&` (expression, type, self param), `->`, `?`, closures, lifetimes; interleaved output; snapshot tests; the byte-for-byte invariant.
2. **Full glossary v1:** all roles in the tables, user overrides, `--dump-glossary`, all CLI modes, role coverage test.
3. **Macro arguments:** parse the arguments of function-like macro invocations as comma-separated expressions (`ra_ap_syntax::hacks::parse_expr_from_str`) when they parse cleanly, and expand them. This covers `println!`, `format!`, `vec!`, `assert_eq!` and similar. Invocations that don't parse cleanly stay untouched.
4. **GUI:** two-column view, matching highlights, aligned rows.
5. **GUI polish:** hover dictionary, changed-only toggle, themes, reload on change.
