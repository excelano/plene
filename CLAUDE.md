# CLAUDE.md

plene shows Rust source alongside an expanded transcription: the same code with Rust's
abbreviations and symbol tokens written out as words. `plene-core` parses with
`ra_ap_syntax`, classifies every token's highlight class and syntactic role, and renders
each role through the glossary into per-line spans; its only I/O is `Glossary::load`,
which reads the glossary files. `plene` is the CLI and `plene-gui` the eframe window.
`DESIGN.md` is the authority on what is expanded, how, and why the glossary reads as it
does.

## Commands

    cargo build --all-targets
    cargo test --workspace
    cargo clippy --all-targets -- -D warnings                 # must be silent
    cargo fmt --check
    cargo run -p plene -- path/to/file.rs                     # --color always | less -R
    cargo run -p plene-gui -- path/to/file.rs
    cargo deb -p plene-gui && packaging/linux/check-libraries.sh   # needs a display
    packaging/screenshots/render.sh ~/excelano.com/plene/img       # the site's window shots
    cargo test --release -p plene-core -- --ignored           # smoke test over the corpus
    cargo llvm-cov --workspace --summary-only                 # then read the uncovered lines:
    cargo llvm-cov report --workspace --text | grep -E '^\s+[0-9]+\|\s+0\|'
    INSTA_UPDATE=always cargo test --workspace                # rewrite snapshots, then git diff them
    cargo run -p plene-core --example dump_tree -- file.rs    # the syntax tree, for role work

The smoke test reads every `.rs` file under the colon-separated directories in
`PLENE_SMOKE_DIRS`, defaulting to `~/plene-corpus` and the standard library source. To
set up the default corpus:

    mkdir -p ~/plene-corpus && cd ~/plene-corpus
    git clone --depth 1 https://github.com/BurntSushi/ripgrep
    git clone --depth 1 https://github.com/serde-rs/serde
    git clone --depth 1 https://github.com/rust-lang/rust-analyzer
    rustup component add rust-src

## Rules

Concatenating every span's `original` reproduces the input byte-for-byte, and every
source line yields one rendered line; the invariant tests hold this over the fixtures in
both line endings and over the corpus, and a change that breaks it is wrong however good
the rendering looks. An expansion replaces exactly one token and is keyed by token and
role, never by token text alone: a new expansion is a new role in `role.rs`, an entry in
`glossary.toml`, and a fixture, and the tests fail until all three exist. The public API
exposes no `ra_ap_syntax` type. `ra_ap_syntax` is pinned to an exact version because its
API breaks between releases; upgrading it is deliberate and is followed by the smoke test.

Tests land in the same change as the code they cover. Coverage is read line by line
before a commit, because a match arm with several patterns counts as covered when any
one of them is hit; an uncovered line is either tested or shown to be unreachable. A new
test is broken deliberately once to watch it fail, and the break is confirmed to have
applied first: rustfmt reflows lines, so a text substitution can match nothing and leave
a passing run that proves nothing. A test's Rust source has to parse: a top-level `let`
is an error node, so nothing in it takes a role and a test about roles passes for the
wrong reason. Snapshot changes are read before they are accepted: in the span snapshots,
a non-whitespace token classed `Plain` is a token kind the highlighter misses.

Releases: the apps in excelano/shipping, run from this directory; `ship-crates`
publishes the crates in dependency order. There is no release document.

Every crate is `forbid(unsafe_code)`, and nothing in the dependency tree compiles C
(`~/notes/pure_rust_preference.md`). The window's display libraries are opened by name
at run time, so the plene-gui package names them in `Depends` by hand; a change to
eframe or its features is followed by `check-libraries.sh` on both display backends.

Coverage has one exception: code that needs a display. The native file dialog
(`native_picker`), `eframe::App::ui` and `main` run by hand, never in a test, and are
the only uncovered lines `plene-gui` is allowed. Tests put a stand-in in the window in
place of the dialog, one that panics if asked, since a real dialog waits on a person.
