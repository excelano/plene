use std::fmt::Write;
use std::fs;

use plene_core::{Edition, transcribe};

/// One line per source line: each span as `text:class` or `text:class/role`, skipping whitespace. Showing
/// non-whitespace `Plain` spans makes any token kind the classifier misses visible.
fn render_spans(source: &str) -> String {
    let mut out = String::new();
    for (number, line) in transcribe(source, Edition::default()).iter().enumerate() {
        write!(out, "{:>3} |", number + 1).unwrap();
        for span in line
            .spans
            .iter()
            .filter(|span| !span.original.trim().is_empty())
        {
            write!(out, " {}:{:?}", span.original, span.class).unwrap();
            if let Some(role) = span.role {
                write!(out, "/{role}").unwrap();
            }
        }
        out.push('\n');
    }
    out
}

#[test]
fn spans() {
    insta::glob!("fixtures/*.rs", |path| {
        let source = fs::read_to_string(path).unwrap();
        insta::assert_snapshot!(render_spans(&source));
    });
}
