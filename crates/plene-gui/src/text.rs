//! Laying out one side of a row: its spans in their highlight colors, and which span
//! lies under a point.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::ops::Range;

use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Galley, Stroke, TextFormat, Vec2, Visuals};
use plene_core::{HighlightClass, Line, Span};

pub const FONT_SIZE: f32 = 14.0;

/// Backgrounds for marked text, translucent so the text stays readable on a dark or a
/// light theme.
const MATCH_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(70, 55, 0, 70);
const CURRENT_MATCH_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(160, 120, 0, 160);
const OCCURRENCE_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0, 40, 70, 70);
const BRACKET_BACKGROUND: Color32 = Color32::from_rgba_premultiplied(0, 90, 40, 110);

/// Why a stretch of text is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkKind {
    /// A search match.
    Match,
    /// The search match the reader is on.
    CurrentMatch,
    /// Another token with the name of the one clicked.
    Occurrence,
    /// A bracket and its partner.
    Bracket,
}

impl MarkKind {
    fn background(self) -> Color32 {
        match self {
            MarkKind::Match => MATCH_BACKGROUND,
            MarkKind::CurrentMatch => CURRENT_MATCH_BACKGROUND,
            MarkKind::Occurrence => OCCURRENCE_BACKGROUND,
            MarkKind::Bracket => BRACKET_BACKGROUND,
        }
    }
}

/// A stretch of one side's text for a line that is marked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mark {
    /// Byte range in the text of the whole line on that side; the marks of a line do
    /// not overlap and are in order.
    pub range: Range<usize>,
    pub kind: MarkKind,
}

/// `first` and `second` as one set of marks in order. Where a mark of `second` overlaps
/// one of `first`, the one in `first` stays and the other is dropped.
pub fn combine(mut first: Vec<Mark>, second: Vec<Mark>) -> Vec<Mark> {
    for mark in second {
        let overlaps = first
            .iter()
            .any(|kept| kept.range.start < mark.range.end && mark.range.start < kept.range.end);
        if !overlaps {
            first.push(mark);
        }
    }
    first.sort_by_key(|mark| mark.range.start);
    first
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Source,
    Transcription,
}

impl Side {
    pub fn text(self, span: &Span) -> &str {
        match self {
            Side::Source => &span.original,
            Side::Transcription => &span.rendered,
        }
    }
}

/// The class color, in the same families as the CLI's terminal colors. Identifiers,
/// operators and punctuation take the theme's text color.
pub fn color(class: HighlightClass, visuals: &Visuals) -> Color32 {
    let [dark, light] = match class {
        HighlightClass::Keyword => [(198, 120, 221), (166, 38, 164)],
        HighlightClass::Type => [(229, 192, 123), (152, 104, 1)],
        HighlightClass::Function => [(97, 175, 239), (64, 120, 242)],
        HighlightClass::Macro => [(86, 182, 194), (1, 132, 188)],
        HighlightClass::Lifetime => [(224, 108, 117), (202, 18, 67)],
        HighlightClass::String => [(152, 195, 121), (80, 161, 79)],
        HighlightClass::Number => [(209, 154, 102), (152, 104, 1)],
        HighlightClass::Comment | HighlightClass::Attribute => [(127, 132, 142), (120, 124, 132)],
        HighlightClass::Identifier
        | HighlightClass::Operator
        | HighlightClass::Punctuation
        | HighlightClass::Plain => return visuals.text_color(),
    };
    let (r, g, b) = if visuals.dark_mode { dark } else { light };
    Color32::from_rgb(r, g, b)
}

/// One side of `line`, wrapped at `width`, with `marks` highlighted. Tokens with a
/// glossary role are underlined on both sides, pairing each token with its expansion.
pub fn layout_job(
    line: &Line,
    side: Side,
    width: f32,
    visuals: &Visuals,
    marks: &[Mark],
) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut start = 0;
    for span in &line.spans {
        let text = side.text(span);
        let color = color(span.class, visuals);
        let mut format = TextFormat::simple(FontId::monospace(FONT_SIZE), color);
        if span.role.is_some() {
            format.underline = Stroke::new(1.0, color);
        }
        for (piece, mark) in pieces(text, start, marks) {
            let mut piece_format = format.clone();
            if let Some(mark) = mark {
                piece_format.background = mark.kind.background();
            }
            job.append(piece, 0.0, piece_format);
        }
        start += text.len();
    }
    job.wrap.max_width = width;
    job
}

/// `text`, which starts `start` bytes into its line, cut where marks begin and end,
/// each piece with the mark that covers it.
fn pieces<'a>(text: &'a str, start: usize, marks: &'a [Mark]) -> Vec<(&'a str, Option<&'a Mark>)> {
    let end = start + text.len();
    let mut pieces = Vec::new();
    let mut at = start;
    for mark in marks
        .iter()
        .filter(|mark| mark.range.start < end && mark.range.end > start)
    {
        let from = mark.range.start.max(start);
        let to = mark.range.end.min(end);
        if from > at {
            pieces.push((&text[at - start..from - start], None));
        }
        pieces.push((&text[from - start..to - start], Some(mark)));
        at = to;
    }
    if at < end || pieces.is_empty() {
        pieces.push((&text[at - start..], None));
    }
    pieces
}

/// The span of `line` under `pos`, relative to the galley laid out from `line`'s
/// `side`. egui merges neighbouring sections that share a format, so sections do not
/// line up with spans; the span is found from the spans' own lengths.
pub fn span_at<'a>(galley: &Galley, line: &'a Line, side: Side, pos: Vec2) -> Option<&'a Span> {
    span_index_at(galley, line, side, pos).map(|index| &line.spans[index])
}

/// Where in `line`'s spans the span under `pos` is, found as `span_at` finds it.
pub fn span_index_at(galley: &Galley, line: &Line, side: Side, pos: Vec2) -> Option<usize> {
    let cursor = galley.cursor_from_pos(pos);
    let (byte, _) = galley.job.text.char_indices().nth(cursor.index.0)?;
    let mut end = 0;
    line.spans.iter().position(|span| {
        end += side.text(span).len();
        byte < end
    })
}

#[cfg(test)]
mod tests {
    use eframe::egui;
    use eframe::egui::text::ByteIndex;
    use plene_core::{Edition, Glossary, Role, transcribe};

    use super::*;

    /// Runs `f` in one headless frame. The frame's texture updates go nowhere, so
    /// they are discarded explicitly, as egui requires.
    fn in_frame(f: impl FnMut(&mut egui::Ui)) {
        egui::Context::default()
            .run_ui(Default::default(), f)
            .drop_without_applying_deltas();
    }

    fn line(source: &str) -> Line {
        transcribe(source, Edition::default(), &Glossary::default()).remove(0)
    }

    /// Lays out `line`'s `side` and returns the span under each character cell of
    /// each visual row, as the span's text on that side.
    fn spans_by_cell(line: &Line, side: Side, width: f32) -> Vec<Vec<String>> {
        let mut found = Vec::new();
        in_frame(|ui| {
            let job = layout_job(line, side, width, ui.visuals(), &[]);
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            found = galley
                .rows
                .iter()
                .map(|row| {
                    let rect = row.rect();
                    let cells = row.char_count_excluding_newline().0;
                    let cell = rect.width() / cells as f32;
                    (0..cells)
                        .filter_map(|n| {
                            let pos =
                                Vec2::new(rect.min.x + cell * (n as f32 + 0.3), rect.center().y);
                            span_at(&galley, line, side, pos)
                                .map(|span| side.text(span).to_string())
                        })
                        .collect()
                })
                .collect();
        });
        found
    }

    #[test]
    fn spans_are_found_across_merged_sections_and_wrapped_rows() {
        // `f(x:` and `u8)` are runs of one color that egui merges into one section.
        let line = line("fn f(x: &u8) -> u8 { *x }");
        let rows = spans_by_cell(&line, Side::Transcription, 120.0);
        assert!(rows.len() > 1, "the line did not wrap: {rows:?}");
        let found: Vec<&str> = rows.iter().flatten().map(String::as_str).collect();
        for expected in [
            "function",
            "f",
            "borrowed",
            "u8",
            "returns",
            "dereference",
            "x",
            "}",
        ] {
            assert!(
                found.contains(&expected),
                "{expected} not found in {found:?}"
            );
        }
        let last_row = rows.last().unwrap();
        assert_eq!(last_row.last().map(String::as_str), Some("}"));
    }

    #[test]
    fn source_side_maps_by_original_text() {
        let line = line("fn f(v: &mut u8) {}");
        let found: Vec<String> = spans_by_cell(&line, Side::Source, 1000.0).concat();
        assert_eq!(found.iter().filter(|text| *text == "&").count(), 1);
        assert!(found.contains(&"mut".to_string()));
    }

    #[test]
    fn a_point_past_the_text_finds_no_span() {
        let line = line("x");
        in_frame(|ui| {
            let empty = transcribe("\n", Edition::default(), &Glossary::default()).remove(0);
            let job = layout_job(&empty, Side::Source, 100.0, ui.visuals(), &[]);
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            assert!(span_at(&galley, &line, Side::Source, Vec2::new(50.0, 5.0)).is_none());
        });
    }

    #[test]
    fn roles_are_underlined_and_colors_follow_the_theme() {
        let line = line("fn f() {}");
        let dark = Visuals::dark();
        let light = Visuals::light();
        let job = layout_job(&line, Side::Transcription, 100.0, &dark, &[]);
        assert!(job.sections[0].format.underline.width > 0.0);
        assert_eq!(line.spans[0].role, Some(Role::Keyword));
        assert_eq!(job.sections.last().unwrap().format.underline.width, 0.0);
        assert_ne!(
            color(HighlightClass::Keyword, &dark),
            color(HighlightClass::Keyword, &light)
        );
        assert_eq!(color(HighlightClass::Identifier, &dark), dark.text_color());
    }

    #[test]
    fn every_class_has_a_color_in_both_themes() {
        for class in [
            HighlightClass::Keyword,
            HighlightClass::Type,
            HighlightClass::Function,
            HighlightClass::Macro,
            HighlightClass::Identifier,
            HighlightClass::Lifetime,
            HighlightClass::String,
            HighlightClass::Number,
            HighlightClass::Comment,
            HighlightClass::Attribute,
            HighlightClass::Operator,
            HighlightClass::Punctuation,
            HighlightClass::Plain,
        ] {
            for visuals in [Visuals::dark(), Visuals::light()] {
                assert_ne!(color(class, &visuals), visuals.panel_fill, "{class:?}");
            }
        }
    }

    #[test]
    fn marks_highlight_the_text_they_cover_even_across_spans() {
        let line = line("fn f(v: &mut u8) {}");
        // `borrowed mutable` is two spans with a space between; the mark runs over all
        // three.
        let text: String = line.spans.iter().map(|span| &*span.rendered).collect();
        let start = text.find("rowed mu").unwrap();
        let range = start..start + "rowed mu".len();
        for kind in [
            MarkKind::Match,
            MarkKind::CurrentMatch,
            MarkKind::Occurrence,
            MarkKind::Bracket,
        ] {
            let expected = kind.background();
            let marks = [Mark {
                range: range.clone(),
                kind,
            }];
            let job = layout_job(&line, Side::Transcription, 1000.0, &Visuals::dark(), &marks);
            assert_eq!(job.text, text, "marking changes no text");
            for byte in 0..text.len() {
                let background = job.format_at_byte(ByteIndex(byte)).background;
                let wanted = if range.contains(&byte) {
                    expected
                } else {
                    Color32::TRANSPARENT
                };
                assert_eq!(background, wanted, "byte {byte} of {text:?}");
            }
        }
    }

    #[test]
    fn pieces_cover_the_text_once_with_or_without_marks() {
        let mark = |range: Range<usize>| Mark {
            range,
            kind: MarkKind::Match,
        };
        let marks = [mark(1..3), mark(7..9)];
        // A span at bytes 2..8 of its line: the first mark covers its start, the
        // second its end.
        let found: Vec<(&str, bool)> = pieces("abcdef", 2, &marks)
            .into_iter()
            .map(|(piece, covered)| (piece, covered.is_some()))
            .collect();
        assert_eq!(found, [("a", true), ("bcde", false), ("f", true)]);
        assert_eq!(pieces("", 4, &marks).len(), 1);
        let none: Vec<&str> = pieces("xyz", 0, &[]).into_iter().map(|(p, _)| p).collect();
        assert_eq!(none, ["xyz"]);
        let earlier = [mark(0..4)];
        let before: Vec<&str> = pieces("xyz", 5, &earlier)
            .into_iter()
            .map(|(piece, _)| piece)
            .collect();
        assert_eq!(
            before,
            ["xyz"],
            "a mark that ends before the text leaves it whole"
        );
    }

    #[test]
    fn every_kind_has_its_own_background() {
        let kinds = [
            MarkKind::Match,
            MarkKind::CurrentMatch,
            MarkKind::Occurrence,
            MarkKind::Bracket,
        ];
        for (index, kind) in kinds.iter().enumerate() {
            assert_ne!(kind.background(), Color32::TRANSPARENT);
            for other in &kinds[index + 1..] {
                assert_ne!(kind.background(), other.background(), "{kind:?} {other:?}");
            }
        }
    }

    #[test]
    fn combining_marks_keeps_the_first_where_they_overlap() {
        let mark = |range: Range<usize>, kind| Mark { range, kind };
        let first = vec![
            mark(4..8, MarkKind::Match),
            mark(20..22, MarkKind::CurrentMatch),
        ];
        let second = vec![
            mark(0..2, MarkKind::Occurrence),
            mark(7..9, MarkKind::Occurrence),
            mark(8..10, MarkKind::Occurrence),
            mark(21..23, MarkKind::Bracket),
            mark(30..31, MarkKind::Bracket),
        ];
        let combined: Vec<(Range<usize>, MarkKind)> = combine(first, second)
            .into_iter()
            .map(|mark| (mark.range, mark.kind))
            .collect();
        assert_eq!(
            combined,
            [
                (0..2, MarkKind::Occurrence),
                (4..8, MarkKind::Match),
                (8..10, MarkKind::Occurrence),
                (20..22, MarkKind::CurrentMatch),
                (30..31, MarkKind::Bracket),
            ]
        );
        assert!(combine(Vec::new(), Vec::new()).is_empty());
    }
}
