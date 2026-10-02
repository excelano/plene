//! The tokens related to the one clicked: every token with the same name, or the
//! bracket that matches.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use plene_core::{HighlightClass, Line};

use crate::text::{Mark, MarkKind, Side};

/// Where the clicked token is, kept by its place in the source so that it survives the
/// transcription being made again, which moves spans about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Anchor {
    row: usize,
    source_start: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    row: usize,
    span: usize,
    kind: MarkKind,
}

#[derive(Default)]
pub struct Related {
    anchor: Option<Anchor>,
    /// In order of row, then span.
    positions: Vec<Position>,
    stale: bool,
}

const OPENING: &str = "([{";
const CLOSING: &str = ")]}";

impl Related {
    pub fn clear(&mut self) {
        *self = Related::default();
    }

    /// The spans may have moved: find the clicked token again before it is next shown.
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Takes the span at `span` of `row` as the clicked token. A name relates to every
    /// token with the same name, a bracket to its partner, and anything else to nothing.
    pub fn select(&mut self, lines: &[Line], row: usize, span: usize) {
        self.clear();
        let clicked = &lines[row].spans[span];
        let positions = if is_bracket(&clicked.original, clicked.class) {
            partner(lines, row, span)
                .map(|partner| {
                    let at = |row, span| Position {
                        row,
                        span,
                        kind: MarkKind::Bracket,
                    };
                    let mut pair = vec![at(row, span), at(partner.0, partner.1)];
                    pair.sort_by_key(|position| (position.row, position.span));
                    pair
                })
                .unwrap_or_default()
        } else {
            names_like(lines, &clicked.original)
        };
        if !positions.is_empty() {
            self.anchor = Some(Anchor {
                row,
                source_start: clicked.source_range.start,
            });
            self.positions = positions;
        }
    }

    /// Finds the clicked token again if the transcription was made again.
    pub fn refresh(&mut self, lines: &[Line]) {
        if !self.stale {
            return;
        }
        self.stale = false;
        let Some(anchor) = self.anchor else {
            return;
        };
        let span = lines.get(anchor.row).and_then(|line| {
            line.spans.iter().position(|span| {
                span.source_range.start == anchor.source_start && !span.original.is_empty()
            })
        });
        match span {
            Some(span) => self.select(lines, anchor.row, span),
            None => self.clear(),
        }
    }

    /// The marks on `side` of `row`.
    pub fn marks(&self, lines: &[Line], row: usize, side: Side) -> Vec<Mark> {
        let first = self
            .positions
            .partition_point(|position| position.row < row);
        self.positions[first..]
            .iter()
            .take_while(|position| position.row == row)
            .filter_map(|position| {
                let spans = &lines[row].spans;
                let start: usize = spans[..position.span]
                    .iter()
                    .map(|span| side.text(span).len())
                    .sum();
                let len = side.text(&spans[position.span]).len();
                (len > 0).then_some(Mark {
                    range: start..start + len,
                    kind: position.kind,
                })
            })
            .collect()
    }

    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.positions.len()
    }
}

/// Whether tokens of `class` are names that refer to something, so that the same name
/// elsewhere is worth showing.
fn is_name(class: HighlightClass) -> bool {
    matches!(
        class,
        HighlightClass::Identifier
            | HighlightClass::Type
            | HighlightClass::Function
            | HighlightClass::Macro
            | HighlightClass::Lifetime
    )
}

fn is_bracket(text: &str, class: HighlightClass) -> bool {
    class == HighlightClass::Punctuation
        && text.len() == 1
        && (OPENING.contains(text) || CLOSING.contains(text))
}

/// Every span that is a name spelled `name`; nothing, when `name` is not one.
fn names_like(lines: &[Line], name: &str) -> Vec<Position> {
    lines
        .iter()
        .enumerate()
        .flat_map(|(row, line)| {
            line.spans
                .iter()
                .enumerate()
                .filter(|(_, span)| is_name(span.class) && span.original == name)
                .map(move |(span, _)| Position {
                    row,
                    span,
                    kind: MarkKind::Occurrence,
                })
        })
        .collect()
}

/// The bracket that matches the one at `span` of `row`, as `(row, span)`. Brackets are
/// matched by kind, nesting as they come; a bracket nothing matches has no partner.
fn partner(lines: &[Line], row: usize, span: usize) -> Option<(usize, usize)> {
    let mut open: Vec<(char, usize, usize)> = Vec::new();
    for (line_row, line) in lines.iter().enumerate() {
        for (line_span, candidate) in line.spans.iter().enumerate() {
            if !is_bracket(&candidate.original, candidate.class) {
                continue;
            }
            let bracket = candidate.original.chars().next()?;
            if let Some(kind) = OPENING.find(bracket) {
                open.push((CLOSING.as_bytes()[kind] as char, line_row, line_span));
                continue;
            }
            // A closer ends the innermost opener of its kind; openers inside it that
            // never closed are dropped with it.
            let Some(depth) = open.iter().rposition(|(closer, ..)| *closer == bracket) else {
                continue;
            };
            let (_, open_row, open_span) = open[depth];
            open.truncate(depth);
            if (open_row, open_span) == (row, span) {
                return Some((line_row, line_span));
            }
            if (line_row, line_span) == (row, span) {
                return Some((open_row, open_span));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use plene_core::{Category, Edition, Glossary, transcribe};

    use super::*;

    fn lines(source: &str) -> Vec<Line> {
        transcribe(source, Edition::default(), &Glossary::default())
    }

    /// The index of the first span on `row` whose source text is `original`.
    fn span_of(lines: &[Line], row: usize, original: &str) -> usize {
        lines[row]
            .spans
            .iter()
            .position(|span| span.original == original)
            .unwrap()
    }

    fn select(lines: &[Line], row: usize, original: &str) -> Related {
        let mut related = Related::default();
        related.select(lines, row, span_of(lines, row, original));
        related
    }

    const SOURCE: &str =
        "fn area(w: u8, h: u8) -> u8 {\n    let w2 = w;\n    w * h + area(w, h)\n}\n";

    #[test]
    fn a_name_relates_to_every_token_spelled_the_same() {
        let lines = lines(SOURCE);
        let related = select(&lines, 0, "w");
        // `w` is a whole token, so `w2` is another name: w in the parameters, in
        // `let w2 = w`, in `w * h`, and in `area(w, h)`.
        assert_eq!(related.count(), 4);
        assert_eq!(related.marks(&lines, 1, Side::Source).len(), 1);
        assert_eq!(related.marks(&lines, 2, Side::Source).len(), 2);
        assert!(related.marks(&lines, 3, Side::Source).is_empty());
        let function = select(&lines, 0, "area");
        assert_eq!(function.count(), 2, "the definition and the call");
    }

    #[test]
    fn marks_cover_the_name_on_either_side_at_its_own_offset() {
        let lines = lines("fn f(x: &u8) -> u8 { *x }\n");
        let related = select(&lines, 0, "x");
        let source = &"fn f(x: &u8) -> u8 { *x }";
        let text = |side: Side, row: usize| -> String {
            lines[row]
                .spans
                .iter()
                .map(|span| side.text(span))
                .collect()
        };
        assert_eq!(text(Side::Source, 0), *source);
        for side in [Side::Source, Side::Transcription] {
            let line_text = text(side, 0);
            let marks = related.marks(&lines, 0, side);
            assert_eq!(marks.len(), 2);
            for mark in marks {
                assert_eq!(&line_text[mark.range], "x", "{side:?}");
            }
        }
    }

    #[test]
    fn a_lifetime_is_marked_by_its_rendered_text_on_the_transcription_side() {
        let lines = lines("fn f<'a>(x: &'a u8) {}\n");
        let related = select(&lines, 0, "'a");
        let text: String = lines[0].spans.iter().map(|span| &*span.rendered).collect();
        let marks = related.marks(&lines, 0, Side::Transcription);
        assert_eq!(marks.len(), 2);
        assert_eq!(&text[marks[0].range.clone()], "lifetime a");
        assert_eq!(marks[0].kind, MarkKind::Occurrence);
    }

    #[test]
    fn only_names_relate_to_their_namesakes() {
        let lines = lines("fn f() { let n = 1 + 1; \"n\"; g(1, 2); }\n");
        for (original, class) in [
            ("fn", HighlightClass::Keyword),
            ("1", HighlightClass::Number),
            ("+", HighlightClass::Operator),
            (",", HighlightClass::Punctuation),
            (";", HighlightClass::Punctuation),
            ("\"n\"", HighlightClass::String),
        ] {
            let span = &lines[0].spans[span_of(&lines, 0, original)];
            assert_eq!(span.class, class, "{original}");
            assert_eq!(select(&lines, 0, original).count(), 0, "{original}");
        }
        let space = lines[0]
            .spans
            .iter()
            .position(|span| span.original == " ")
            .unwrap();
        let mut related = Related::default();
        related.select(&lines, 0, space);
        assert_eq!(related.count(), 0);
    }

    #[test]
    fn a_bracket_relates_to_its_partner_across_lines() {
        let lines = lines("fn f() {\n    g(a[0], {\n        1\n    });\n}\n");
        let related = select(&lines, 0, "{");
        let pair: Vec<(usize, usize)> = related
            .positions
            .iter()
            .map(|position| (position.row, position.span))
            .collect();
        assert_eq!(pair.len(), 2);
        assert_eq!(pair[0].0, 0);
        assert_eq!(pair[1].0, 4, "the closing brace of the function");
        assert!(
            related
                .positions
                .iter()
                .all(|p| p.kind == MarkKind::Bracket)
        );
        assert_eq!(related.marks(&lines, 4, Side::Source).len(), 1);

        let inner = select(&lines, 1, "[");
        let rows: Vec<usize> = inner.positions.iter().map(|p| p.row).collect();
        assert_eq!(rows, [1, 1]);
        let from_closer = {
            let mut related = Related::default();
            let closer = lines[3]
                .spans
                .iter()
                .position(|s| s.original == "}")
                .unwrap();
            related.select(&lines, 3, closer);
            related
        };
        let rows: Vec<usize> = from_closer.positions.iter().map(|p| p.row).collect();
        assert_eq!(rows, [1, 3], "the closer finds its opener");
    }

    #[test]
    fn brackets_match_by_kind_and_unmatched_ones_have_no_partner() {
        let unclosed = lines("fn f() {\n    g(\n}\n");
        assert_eq!(select(&unclosed, 1, "(").count(), 0);
        let stray = lines("fn f() {\n    )\n}\n");
        assert_eq!(select(&stray, 1, ")").count(), 0);
        let crossed = lines("fn f() {\n    ( ]\n}\n");
        assert_eq!(select(&crossed, 1, "(").count(), 0);
        assert_eq!(select(&crossed, 1, "]").count(), 0);
    }

    #[test]
    fn brackets_inside_strings_and_comments_are_not_brackets() {
        let lines = lines("fn f() {\n    g(\"(\"); // )\n}\n");
        let related = select(&lines, 1, "(");
        let rows: Vec<usize> = related.positions.iter().map(|p| p.row).collect();
        assert_eq!(rows, [1, 1]);
        assert_eq!(related.positions[1].span - related.positions[0].span, 2);
    }

    #[test]
    fn the_clicked_token_is_found_again_after_the_transcription_changes() {
        let source = "fn f<'a>(x: &'a u8) -> &'a u8 { x }\n";
        let full = Glossary::default();
        let before = transcribe(source, Edition::default(), &full);
        let mut related = select(&before, 0, "'a");
        assert_eq!(related.count(), 3);
        let after = transcribe(
            source,
            Edition::default(),
            &full.without(&[Category::References, Category::Lifetimes]),
        );
        assert_ne!(before[0].spans.len(), after[0].spans.len());
        related.mark_stale();
        related.refresh(&after);
        assert_eq!(related.count(), 3);
        let marks = related.marks(&after, 0, Side::Transcription);
        let text: String = after[0].spans.iter().map(|s| &*s.rendered).collect();
        assert!(marks.iter().all(|mark| &text[mark.range.clone()] == "'a"));
    }

    #[test]
    fn the_clicked_token_is_found_past_the_spaces_an_expansion_inserts() {
        // An inserted space starts where the token after it does, so a lookup by
        // position has to skip it. Here the second `'a` gains a space before it.
        let source = "fn f<'a>(x: &'a u8) {}\n";
        let full = Glossary::default();
        let plain = full.without(Category::ALL);
        let before = transcribe(source, Edition::default(), &plain);
        let second = before[0]
            .spans
            .iter()
            .rposition(|span| span.original == "'a")
            .unwrap();
        let mut related = Related::default();
        related.select(&before, 0, second);
        let after = transcribe(source, Edition::default(), &full);
        assert!(after[0].spans.len() > before[0].spans.len());
        related.mark_stale();
        related.refresh(&after);
        assert_eq!(related.count(), 2);
        let text: String = after[0].spans.iter().map(|s| &*s.rendered).collect();
        for mark in related.marks(&after, 0, Side::Transcription) {
            assert_eq!(&text[mark.range], "lifetime a");
        }
    }

    #[test]
    fn refreshing_waits_for_a_stale_mark_and_loses_a_vanished_token() {
        let lines = lines("fn f(x: u8) { x }\n");
        let mut related = select(&lines, 0, "x");
        related.refresh(&[]);
        assert_eq!(related.count(), 2, "not stale, so left alone");
        related.mark_stale();
        related.refresh(&[]);
        assert_eq!(related.count(), 0);
        let mut empty = Related::default();
        empty.mark_stale();
        empty.refresh(&lines);
        assert_eq!(empty.count(), 0);
    }

    #[test]
    fn selecting_again_replaces_what_was_selected() {
        let lines = lines("fn f(x: u8, y: u8) { x + y }\n");
        let mut related = select(&lines, 0, "x");
        assert_eq!(related.count(), 2);
        related.select(&lines, 0, span_of(&lines, 0, "fn"));
        assert_eq!(related.count(), 0);
    }
}
