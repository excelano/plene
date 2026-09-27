//! Laying out one side of a row: its spans in their highlight colors, and which span
//! lies under a point.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, Galley, Stroke, TextFormat, Vec2, Visuals};
use plene_core::{HighlightClass, Line, Span};

pub const FONT_SIZE: f32 = 14.0;

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

/// One side of `line`, wrapped at `width`. Tokens with a glossary role are underlined
/// on both sides, pairing each token with its expansion.
pub fn layout_job(line: &Line, side: Side, width: f32, visuals: &Visuals) -> LayoutJob {
    let mut job = LayoutJob::default();
    for span in &line.spans {
        let color = color(span.class, visuals);
        let mut format = TextFormat::simple(FontId::monospace(FONT_SIZE), color);
        if span.role.is_some() {
            format.underline = Stroke::new(1.0, color);
        }
        job.append(side.text(span), 0.0, format);
    }
    job.wrap.max_width = width;
    job
}

/// The span of `line` under `pos`, relative to the galley laid out from `line`'s
/// `side`. egui merges neighbouring sections that share a format, so sections do not
/// line up with spans; the span is found from the spans' own lengths.
pub fn span_at<'a>(galley: &Galley, line: &'a Line, side: Side, pos: Vec2) -> Option<&'a Span> {
    let cursor = galley.cursor_from_pos(pos);
    let (byte, _) = galley.job.text.char_indices().nth(cursor.index.0)?;
    let mut end = 0;
    line.spans.iter().find(|span| {
        end += side.text(span).len();
        byte < end
    })
}

#[cfg(test)]
mod tests {
    use eframe::egui;
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
            let job = layout_job(line, side, width, ui.visuals());
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
            let job = layout_job(&empty, Side::Source, 100.0, ui.visuals());
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            assert!(span_at(&galley, &line, Side::Source, Vec2::new(50.0, 5.0)).is_none());
        });
    }

    #[test]
    fn roles_are_underlined_and_colors_follow_the_theme() {
        let line = line("fn f() {}");
        let dark = Visuals::dark();
        let light = Visuals::light();
        let job = layout_job(&line, Side::Transcription, 100.0, &dark);
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
}
