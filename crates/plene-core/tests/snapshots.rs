use std::fmt::Write;
use std::fs;

use plene_core::{Edition, HighlightClass, transcribe};

/// One line per source line: each non-plain span as `text:class`.
fn render_highlights(source: &str) -> String {
    let mut out = String::new();
    for (number, line) in transcribe(source, Edition::default()).iter().enumerate() {
        write!(out, "{:>3} |", number + 1).unwrap();
        for span in line
            .spans
            .iter()
            .filter(|span| span.class != HighlightClass::Plain)
        {
            write!(out, " {}:{:?}", span.original, span.class).unwrap();
        }
        out.push('\n');
    }
    out
}

#[test]
fn highlights() {
    insta::glob!("fixtures/*.rs", |path| {
        let source = fs::read_to_string(path).unwrap();
        insta::assert_snapshot!(render_highlights(&source));
    });
}
