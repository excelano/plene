//! Prints the ra_ap_syntax tree for a Rust file, for working out token roles.
//!
//! cargo run -p plene-core --example dump_tree -- <FILE>

use ra_ap_syntax::{Edition, SourceFile};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump_tree <FILE>");
    let text = std::fs::read_to_string(&path).expect("read file");
    let parse = SourceFile::parse(&text, Edition::Edition2021);
    for error in parse.errors() {
        eprintln!("parse error: {error:?}");
    }
    print!("{:#?}", parse.syntax_node());
}
