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
- Everything else is byte-for-byte unchanged: identifiers, std types, full-word keywords, `::`, generics, macro names, the arguments of macros other than the standard ones listed under Architecture, attributes, comments, strings, braces, indentation.
- Each expansion is consistent: the same token in the same syntactic role always gets the same text.
- Expansions are keyed by **token + syntactic role**, never by token text alone.
- **Every expansion replaces exactly one token.** Multi-token forms compose: `&mut x` is `&` → `borrow` plus `mut` → `mutable`, giving `borrow mutable x`; `*const T` gives `raw pointer const T`.

## Glossary

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
| `..` | `rest_pattern` | `[a, ..]`, `Foo { a, .. }`, and the same in a destructuring assignment | `and the rest` |
| `..` | `rest_binding` | `rest @ ..` | `the rest` |
| `=>` | `match_arm` | | `gives` |
| `\|` | `pattern_or` | pattern alternative | `or` |
| `@` | `pattern_binding` | | `bound as` |
| `_` | `wildcard` | wildcard pattern | `anything` |
| `_` | `discard` | `_` as the whole pattern of a `let`, or the whole left side of an assignment | `discard` |
| `_` | `inferred_type` | `Vec<_>` | `inferred` |
| `:` | `trait_bound` | generics, `where`, supertraits, when the first bound is a trait | `implementing` |
| `:` | `lifetime_bound` | `'a: 'b`, `T: 'a`, when the first bound is a lifetime | `outliving` |
| `+` | `bound_separator` | between bounds | `and` |

Examples and notes:
- `&'a T` → `borrowed lifetime a T`; `&'a mut T` → `borrowed lifetime a mutable T`; `&mut self` → `borrowed mutable self`.
- `move |x| …` → `move closure(x) …`.
- The range roles cover range patterns as well as range expressions: `1..=9 =>` → `1 through 9 gives`.
- `'outer: loop` → `label outer: loop`; `break 'outer` → `break label outer`.
- `T: Clone + 'a` → `T implementing Clone and lifetime a`.
- `write!(f, "x")?;` → `write!(f, "x") or return early;`, but `foo()?.bar()` keeps its `?`, since `foo() or return early.bar()` doesn't read.
- `&&x`, `&&T` and `&&pat` parse as two `&` tokens in nested nodes, so a double borrow needs no special handling. An empty closure's `||` likewise parses as two pipes in the closure's parameter list, while logical `||` is a single token.
- `_x`-style identifiers are identifiers; leave them alone.
- Syntax alone can't see types, so some expansions are approximate. `&`, `|` and `^` on `bool` operands are non-short-circuit logical operators, not bitwise ones, and `!` on an integer is bitwise not. The glossary uses the common reading.

## Categories

A reader who has grown used to some of the notation can leave it as written. The roles are grouped into categories, each switched on or off as a whole, and every role belongs to exactly one: `Category::of` matches on the role, and on the token for the `keyword` role, which covers several words. Adding a role fails to compile until it is placed.

| Category | Id | Covers |
|---|---|---|
| Keywords | `keywords` | `fn`, `mod`, `extern`, `dyn`, `impl` |
| Visibility | `visibility` | `pub` |
| Mutability | `mutability` | `mut` |
| References and pointers | `references` | `&`, `*` as dereference or raw pointer, `ref` |
| Operators | `operators` | bitwise and shift operators and their compound assignments, prefix `!` |
| Ranges | `ranges` | `..` and `..=` as ranges |
| Patterns and wildcards | `patterns` | `\|` between alternatives, `@`, `_`, `..` as a rest or a struct update |
| Lifetimes and labels | `lifetimes` | lifetimes, labels, `'a: 'b` |
| Trait bounds | `bounds` | `:` and `+` in bounds, `?Sized`, `impl !Trait` |
| Match arms, returns, closures and `?` | `flow` | `=>`, `->`, the closure pipes, `?` |

Switching a category off removes its entries from the glossary that transcribes, with `Glossary::without`, so the tokens it covers come out as written and carry no role: no underline and no hover card. The glossary the hover cards read is unchanged. The invariants hold for any set of categories off, and with all of them off the transcription is the source.

## Glossary file format

The default glossary lives at `crates/plene-core/glossary.toml` and is embedded via `include_str!`. It sits inside the crate so `cargo publish` packages it. Two files may override it, merging by `(token, role)`: the config directory's `plene/glossary.toml`, then the file named with `--glossary`, so the named file wins. The config directory is `$XDG_CONFIG_HOME`, or `~/.config` when that is unset or not an absolute path, on every platform. A missing config file is silent; one that exists and fails to parse is an error, as a bad `--glossary` file is.

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
- `plene --dump-glossary` prints the effective glossary, all layers merged, as a glossary file that reads back to the same entries; it is the starting point for writing overrides.
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
- **Whitespace:** never alter source whitespace or indentation. Where an expansion's word edge touches a character that is not whitespace, insert a space, unless that character is punctuation that hugs the word from its side: `( [ { <` before it, or `) ] } > , ; : .` after it. After an expanded keyword, `(` and `<` also attach, as they do in the source. So `foo()?;` → `foo() or return early;`, `&mut` → `borrow mutable`, `&[u8]` → `borrowed [u8]`, `()->u8` → `() returns u8` and `!(a > b)` → `not (a > b)`, but `(&x)` → `(borrow x)`, `<'a>` → `<lifetime a>`, `pub(crate)` → `public(crate)` and `impl<T>` → `implement<T>`. An inserted space is its own span with an empty `original`, so each expansion's `rendered` is exactly its glossary text.
- **Line correspondence is 1:1.** Every source line produces exactly one rendered line. Tokens that span lines (block comments, multi-line and raw strings) are split into one span per line. Line endings (LF or CRLF) are preserved.
- **Macros:** the arguments of standard macros that take expressions are parsed as Rust and expanded like any other code: `assert`, `assert_eq`, `assert_ne`, `debug_assert`, `debug_assert_eq`, `debug_assert_ne`, `format`, `format_args`, `print`, `println`, `eprint`, `eprintln`, `write`, `writeln`, `panic`, `todo`, `unimplemented`, `unreachable`, `vec`, `dbg`, `addr_of` and `addr_of_mut`, matched by the last segment of the path, so `std::format!` counts. The arguments are parsed as the elements of an array, which also covers `vec![x; n]`, and an invocation expands only when they parse without error; otherwise it stays untouched. So `assert_eq!(&a, &b)` reads `assert_eq!(borrow a, borrow b)`, while `format!("{}", type = &a)` stays as written. The parsed arguments are the source's own bytes split into tokens again, so the invariants hold as they do elsewhere. Every other macro's token tree is left untouched, since syntax alone can't tell whether its arguments are Rust: `stringify!(&a)`, `quote!` bodies, and patterns in `matches!` would all read wrongly. Format strings are strings and stay unchanged. Attributes are untouched too. `macro_rules!` bodies are never expanded.
- **Parse errors:** render everything; tokens inside error nodes are not expanded.
- No CLI/GUI dependencies, and no I/O except `Glossary::load`, which reads the glossary files so that the CLI and the GUI find them by the same rules.

### plene (CLI)

```
plene [OPTIONS] <FILE|->

  (default)          interleaved: each source line, then its expansion beneath when it differs
  --side-by-side     two columns: the source, and the whole transcription
  --expanded         the transcription alone
  --changed-only     only lines that differ, numbered with their source lines
  --lines <START:END>  only these source lines, numbered; either end may be left out
  --keep <CATEGORY,...>  leave these categories as written, not expanded
  --color <auto|always|never>
  --theme <dark|light>              (default dark)
  --edition <2015|2018|2021|2024>   (default 2021)
  --glossary <PATH>
  --dump-glossary
```

- Interleaved output marks lines in a two-column gutter: blank for source lines, `» ` for expansion lines, so the two are distinguishable without color and indentation stays aligned. With color, expansion lines also sit on a subtle background band, padded with spaces to the widest expansion line so the bands form an even block. Padding uses spaces rather than an erase-to-end-of-line escape, which `less -R` would print literally. `--theme <dark|light>` (default dark) picks the band for the terminal's background.
- `--keep` takes categories, comma-separated or repeated, and leaves them as written. An unknown name is an argument error that lists the known ones.
- The three layouts are exclusive; `--changed-only` and `--lines` filter any of them, and both number the rows they keep with their source lines. The whole file is parsed whatever the range, so a line's roles do not depend on it. A range's end is clamped to the file; a start past the last line is an error. Side by side, the left column is padded to the widest source line and never truncated, and changed lines carry the band on the right; `less -RS` scrolls a wide result. Blank lines print as empty lines in every layout, leaving no trailing spaces.
- ANSI color via the highlight classes, from the terminal's 16-color palette so it follows the user's theme. Tokens with a glossary role are underlined on both lines, pairing each token with its expansion.
- A closed reader (`plene file.rs | head`) ends output quietly. Errors print `plene: …` and exit 1; argument errors exit 2.
- `-` reads stdin.
- Control characters other than tab, and the Unicode bidirectional controls, are printed as Rust escapes (`\u{1b}`), so source cannot drive the terminal or reorder how a line displays.
- Should work cleanly with `less -R`.

### plene-gui (eframe/egui)

```
plene-gui [OPTIONS] [FILE]

  --edition <2015|2018|2021|2024>   (default 2021)
  --glossary <PATH>
```

- Opens a `.rs` file named on the command line, dropped on the window, or chosen with the Open button or Ctrl+O. The file dialog is the XDG portal on Linux, so nothing links GTK.
- The glossary is found by the CLI's rules, through `Glossary::load`. A glossary that fails to load, or a file that can't be opened, is reported in the window rather than ending it; a bad glossary falls back to the built-in one.
- Two panes, original on the left and expansion on the right, each with its own line numbers. A toggle in the toolbar hides the transcription's pane and gives the source the whole width; it shows at start, and the choice holds across the files opened.
- A click selects a row, marked by a band across both panes. The Up and Down arrows move the selection, starting from the first row in view, and scroll to keep it there. Page Up and Page Down scroll by the height of the view and leave the selection where it is. Opening a file clears the selection.
- One scroll area holding one row per source line, each row with two cells. Rows stay aligned and scroll together by construction: long lines wrap, and each row's height is the taller of its two cells. Row heights are measured once per column width and kept with each row's top, so only the rows in view are laid out and painted; a change of width measures them again.
- Syntax highlighting on both sides uses matching colors, from a dark and a light palette in the CLI's color families. Tokens with a glossary role are underlined on both sides, as in the CLI.
- Hovering a token with a glossary role, on either side, shows the token and its expansion, its role, and the glossary `note`. egui merges neighbouring text of one format, so the token under the pointer is found from the spans' own lengths, not from the laid-out sections.
- Ctrl+F opens a search bar under the toolbar; Escape closes it. The query is a substring, with ASCII letters matching in either case, searched in the text of each line as a pane shows it, so `borrowed mutable` finds what the source spells `&mut` and a match may run across tokens. The bar chooses the source, the transcription or both, and searches only the panes on screen; choosing the transcription shows its pane. Matches are highlighted in the panes, the current one more strongly. Enter and Shift+Enter, or the Next and Previous buttons, step through the matches in row order, source before transcription within a row, wrapping at the ends; a step selects the match's row and scrolls to it. Typing a query goes to the first match from the selected row, or from the first row in view. While the field has the keyboard, the arrow and page keys leave the rows alone.
- An Expand menu in the toolbar holds a checkbox for each category; unchecking one leaves what it covers as written. The transcription is made again, so the selection stays and the search runs over the new text. All are on at start, and the choice holds across the files opened.
- A toggle for changed-only lines.
- Light and dark themes, with a switch; on Linux the system theme comes from the XDG portal, since winit reports none there.
- Reload when the file changes on disk (nice to have).

## Testing

- **Snapshot tests** (`insta`) over a fixture corpus covering every glossary role. Each role needs at least one fixture. CRLF input and multi-line tokens get fixtures too.
- **Invariant:** concatenating `original` across all spans reproduces the input byte-for-byte. Test this on every fixture and on real crates.
- **Smoke test** (`#[ignore]`; `cargo test --release -- --ignored`): run over every `.rs` file under `PLENE_SMOKE_DIRS` (colon-separated) and assert the invariants, no panics, and 1:1 line counts. The default corpus is `~/plene-corpus` (shallow clones of ripgrep, serde and rust-analyzer, whose `crates/parser/test_data` holds deliberately malformed edge cases) plus the standard library from `rustup component add rust-src`.
- **Role coverage test:** every role in the default glossary is exercised by at least one fixture.
- **GUI tests** run headless with `egui_kittest`, with no display and no GPU: they drive the pointer and input and check the app's state and the labels it shows. Painted text has no accessibility node, so rows are checked through state and hover cards, not by reading the columns.
