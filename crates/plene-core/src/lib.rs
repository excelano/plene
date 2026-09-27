//! Parses Rust source and produces its expanded transcription as per-line spans.

use std::ops::Range;

use ra_ap_syntax::SourceFile;

/// The Rust edition to parse with. The edition changes which words lex as keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Edition {
    E2015,
    E2018,
    #[default]
    E2021,
    E2024,
}

impl Edition {
    fn to_ra(self) -> ra_ap_syntax::Edition {
        match self {
            Edition::E2015 => ra_ap_syntax::Edition::Edition2015,
            Edition::E2018 => ra_ap_syntax::Edition::Edition2018,
            Edition::E2021 => ra_ap_syntax::Edition::Edition2021,
            Edition::E2024 => ra_ap_syntax::Edition::Edition2024,
        }
    }
}

/// One piece of a source line: a token, or the part of a multi-line token on this line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Exact source text. Never contains a line ending.
    pub original: String,
    /// The expansion, or the same text as `original`.
    pub rendered: String,
    /// Byte offsets of `original` in the input.
    pub source_range: Range<usize>,
}

/// One source line. Concatenating every span's `original` and then `ending`
/// reproduces the source line byte-for-byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub spans: Vec<Span>,
    /// `"\n"`, `"\r\n"`, or `""` for a last line without a line ending.
    pub ending: String,
}

/// Transcribes `source` into one `Line` per source line, matching `str::lines`:
/// a trailing line ending does not start an extra empty line.
pub fn transcribe(source: &str, edition: Edition) -> Vec<Line> {
    let parse = SourceFile::parse(source, edition.to_ra());
    let mut lines = Vec::new();
    let mut spans = Vec::new();

    let tokens = parse
        .syntax_node()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token());
    for token in tokens {
        let mut offset = usize::from(token.text_range().start());
        for piece in token.text().split_inclusive('\n') {
            let (text, ending) = split_line_ending(piece);
            if !text.is_empty() {
                spans.push(Span {
                    original: text.to_string(),
                    rendered: text.to_string(),
                    source_range: offset..offset + text.len(),
                });
            }
            if !ending.is_empty() {
                lines.push(finish_line(std::mem::take(&mut spans), ending));
            }
            offset += piece.len();
        }
    }
    if !spans.is_empty() {
        lines.push(Line {
            spans,
            ending: String::new(),
        });
    }
    lines
}

/// A line comment's token stops at `\n`, so in CRLF source its `\r` arrives on the
/// comment rather than with the newline. Moves it back into the line ending.
fn finish_line(mut spans: Vec<Span>, ending: &str) -> Line {
    if ending == "\n"
        && let Some(last) = spans.last_mut()
        && last.original.ends_with('\r')
    {
        last.original.pop();
        last.rendered.pop();
        last.source_range.end -= 1;
        if last.original.is_empty() {
            spans.pop();
        }
        return Line {
            spans,
            ending: "\r\n".to_string(),
        };
    }
    Line {
        spans,
        ending: ending.to_string(),
    }
}

fn split_line_ending(piece: &str) -> (&str, &str) {
    if let Some(text) = piece.strip_suffix("\r\n") {
        (text, "\r\n")
    } else if let Some(text) = piece.strip_suffix('\n') {
        (text, "\n")
    } else {
        (piece, "")
    }
}
