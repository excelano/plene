//! Terminal rendering of transcribed lines.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::borrow::Cow;
use std::fmt::Write;
use std::str::FromStr;

use anstyle::{Ansi256Color, AnsiColor, Color, Style};
use clap::ValueEnum;
use plene_core::{HighlightClass, Line, Span};
use unicode_width::UnicodeWidthStr;

const SOURCE_GUTTER: &str = "  ";
const EXPANSION_GUTTER: &str = "» ";
const COLUMN_SEPARATOR: &str = " │ ";

/// The terminal background the colors are chosen for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    /// A subtle background band behind expansion lines.
    fn band(self) -> Color {
        match self {
            Theme::Dark => Ansi256Color(236).into(),
            Theme::Light => Ansi256Color(254).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Styling {
    Plain,
    Colored(Theme),
}

impl Styling {
    /// The band behind expansions, when there is color to draw it.
    fn band(self) -> Option<Color> {
        match self {
            Styling::Plain => None,
            Styling::Colored(theme) => Some(theme.band()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Each source line, followed by its expansion when the expansion differs.
    Interleaved,
    /// Source on the left, the whole transcription on the right.
    SideBySide,
    /// The transcription alone.
    Expanded,
}

/// An inclusive span of source lines, counted from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    fn contains(self, number: usize) -> bool {
        (self.start..=self.end).contains(&number)
    }
}

impl FromStr for LineRange {
    type Err = String;

    /// `START:END`, where either end may be left out to run to that end of the file.
    fn from_str(text: &str) -> Result<LineRange, String> {
        let (start, end) = text
            .split_once(':')
            .ok_or("expected START:END, as in 40:80")?;
        let number = |digits: &str, default: usize| {
            if digits.is_empty() {
                Ok(default)
            } else {
                digits
                    .parse::<usize>()
                    .map_err(|_| format!("`{digits}` is not a line number"))
            }
        };
        let range = LineRange {
            start: number(start, 1)?,
            end: number(end, usize::MAX)?,
        };
        if range.start == 0 {
            Err("lines are counted from 1".to_string())
        } else if range.start > range.end {
            Err(format!("the range ends at {} before it starts", range.end))
        } else {
            Ok(range)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct View {
    pub layout: Layout,
    /// Only lines the expansion changes.
    pub changed_only: bool,
    /// Only these source lines.
    pub lines: Option<LineRange>,
    pub styling: Styling,
}

impl View {
    /// Rows carry their source line numbers whenever some lines are left out.
    fn numbered(self) -> bool {
        self.changed_only || self.lines.is_some()
    }
}

/// Draws the transcribed lines. With color, expansions sit on a background band
/// padded to the widest one drawn, so the bands form an even block.
pub fn draw(lines: &[Line], view: View) -> String {
    let rows: Vec<(usize, &Line)> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| (index + 1, line))
        .filter(|(number, _)| view.lines.is_none_or(|range| range.contains(*number)))
        .filter(|(_, line)| !view.changed_only || is_changed(line))
        .collect();
    let number_width = match rows.last() {
        Some((number, _)) if view.numbered() => number.to_string().len(),
        _ => 0,
    };
    let band_width = rows
        .iter()
        .filter(|(_, line)| is_changed(line))
        .map(|(_, line)| text_width(line, |span| &span.rendered))
        .max()
        .unwrap_or(0);
    let mut canvas = Canvas {
        out: String::new(),
        view,
        number_width,
        band_width,
    };

    match view.layout {
        Layout::Interleaved => {
            for (number, line) in &rows {
                canvas.number(Some(*number));
                if !line.spans.is_empty() {
                    canvas.out.push_str(SOURCE_GUTTER);
                }
                canvas.source(line);
                canvas.out.push('\n');
                if is_changed(line) {
                    canvas.number(None);
                    let gutter_style = Style::new().dimmed().bg_color(view.styling.band());
                    canvas.paint(EXPANSION_GUTTER, gutter_style);
                    canvas.expansion(line);
                    canvas.out.push('\n');
                }
            }
        }
        Layout::SideBySide => {
            let left_width = rows
                .iter()
                .map(|(_, line)| text_width(line, |span| &span.original))
                .max()
                .unwrap_or(0);
            for (number, line) in &rows {
                canvas.number(Some(*number));
                canvas.source(line);
                let padding = left_width - text_width(line, |span| &span.original);
                canvas.out.push_str(&" ".repeat(padding));
                if is_changed(line) {
                    canvas.paint(COLUMN_SEPARATOR, Style::new().dimmed());
                    canvas.expansion(line);
                } else if text_width(line, |span| &span.original) > 0 {
                    canvas.paint(COLUMN_SEPARATOR, Style::new().dimmed());
                    canvas.source(line);
                } else {
                    canvas.paint(COLUMN_SEPARATOR.trim_end(), Style::new().dimmed());
                }
                canvas.out.push('\n');
            }
        }
        Layout::Expanded => {
            for (number, line) in &rows {
                canvas.number(Some(*number));
                for span in &line.spans {
                    canvas.paint(&visible(&span.rendered), span_style(span));
                }
                canvas.out.push('\n');
            }
        }
    }
    canvas.out
}

/// The output being drawn, and what every row of it shares.
struct Canvas {
    out: String,
    view: View,
    number_width: usize,
    band_width: usize,
}

impl Canvas {
    fn paint(&mut self, text: &str, style: Style) {
        paint(&mut self.out, text, style, self.view.styling);
    }

    /// A source line number, or blanks of the same width, when lines are numbered.
    fn number(&mut self, number: Option<usize>) {
        if self.view.numbered() {
            let text = match number {
                Some(number) => format!("{number:>width$} ", width = self.number_width),
                None => " ".repeat(self.number_width + 1),
            };
            self.paint(&text, Style::new().dimmed());
        }
    }

    fn source(&mut self, line: &Line) {
        for span in &line.spans {
            self.paint(&visible(&span.original), span_style(span));
        }
    }

    /// A line's expansion on the band, padded to the band's width.
    fn expansion(&mut self, line: &Line) {
        let band = self.view.styling.band();
        for span in &line.spans {
            self.paint(&visible(&span.rendered), span_style(span).bg_color(band));
        }
        if band.is_some() {
            let padding = " ".repeat(self.band_width - text_width(line, |span| &span.rendered));
            self.paint(&padding, Style::new().bg_color(band));
        }
    }
}

fn is_changed(line: &Line) -> bool {
    line.spans.iter().any(|span| span.rendered != span.original)
}

/// Display width of one side of a line, as drawn.
fn text_width(line: &Line, side: impl Fn(&Span) -> &str) -> usize {
    line.spans
        .iter()
        .map(|span| visible(side(span)).width())
        .sum()
}

/// Source text with every character that could act on the terminal or on how the
/// line displays written as a Rust escape instead: control characters other than
/// tab, which could move the cursor or restyle the screen, and the Unicode
/// bidirectional controls, which could show code in a different order from the
/// order it compiles in.
fn visible(text: &str) -> Cow<'_, str> {
    if !text.chars().any(is_hazard) {
        return Cow::Borrowed(text);
    }
    let mut escaped = String::with_capacity(text.len() + 8);
    for c in text.chars() {
        if is_hazard(c) {
            write!(escaped, "\\u{{{:x}}}", u32::from(c)).unwrap();
        } else {
            escaped.push(c);
        }
    }
    Cow::Owned(escaped)
}

fn is_hazard(c: char) -> bool {
    (c.is_control() && c != '\t') || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn paint(out: &mut String, text: &str, style: Style, styling: Styling) {
    match styling {
        Styling::Plain => out.push_str(text),
        Styling::Colored(_) => write!(out, "{style}{text}{style:#}").unwrap(),
    }
}

/// The class color, underlined for tokens with a glossary role so that a token and its
/// expansion pair up across the two lines.
fn span_style(span: &Span) -> Style {
    let color = match span.class {
        HighlightClass::Keyword => Some(AnsiColor::Magenta),
        HighlightClass::Type => Some(AnsiColor::Yellow),
        HighlightClass::Function => Some(AnsiColor::Blue),
        HighlightClass::Macro => Some(AnsiColor::Cyan),
        HighlightClass::Lifetime => Some(AnsiColor::Red),
        HighlightClass::String => Some(AnsiColor::Green),
        HighlightClass::Number => Some(AnsiColor::BrightCyan),
        HighlightClass::Comment | HighlightClass::Attribute => Some(AnsiColor::BrightBlack),
        HighlightClass::Identifier
        | HighlightClass::Operator
        | HighlightClass::Punctuation
        | HighlightClass::Plain => None,
    };
    let style = Style::new().fg_color(color.map(Into::into));
    if span.role.is_some() {
        style.underline()
    } else {
        style
    }
}

#[cfg(test)]
mod tests {
    use plene_core::{Edition, Glossary, transcribe};

    use super::*;

    fn render(source: &str, styling: Styling) -> String {
        draw_view(source, Layout::Interleaved, false, styling)
    }

    fn draw_view(source: &str, layout: Layout, changed_only: bool, styling: Styling) -> String {
        draw_range(source, layout, changed_only, None, styling)
    }

    fn draw_range(
        source: &str,
        layout: Layout,
        changed_only: bool,
        lines: Option<LineRange>,
        styling: Styling,
    ) -> String {
        let transcribed = transcribe(source, Edition::default(), &Glossary::default());
        draw(
            &transcribed,
            View {
                layout,
                changed_only,
                lines,
                styling,
            },
        )
    }

    fn strip(styled: &str) -> String {
        anstream::adapter::strip_str(styled).to_string()
    }

    const THREE: &str = "use a;\n\nfn f(x: &u8) {}\n";

    #[test]
    fn side_by_side_repeats_unchanged_lines_and_aligns_the_columns() {
        let expected = concat!(
            "use a;          │ use a;\n",
            "                │\n",
            "fn f(x: &u8) {} │ function f(x: borrowed u8) {}\n",
        );
        assert_eq!(
            draw_view(THREE, Layout::SideBySide, false, Styling::Plain),
            expected
        );
    }

    #[test]
    fn expanded_is_the_transcription_alone() {
        assert_eq!(
            draw_view(THREE, Layout::Expanded, false, Styling::Plain),
            "use a;\n\nfunction f(x: borrowed u8) {}\n"
        );
    }

    #[test]
    fn changed_only_numbers_rows_in_every_layout() {
        let source = format!("fn a() {{}}\n{}fn b() {{}}\n", "x;\n".repeat(9));
        assert_eq!(
            draw_view(&source, Layout::Interleaved, true, Styling::Plain),
            " 1   fn a() {}\n   » function a() {}\n11   fn b() {}\n   » function b() {}\n"
        );
        assert_eq!(
            draw_view(&source, Layout::SideBySide, true, Styling::Plain),
            " 1 fn a() {} │ function a() {}\n11 fn b() {} │ function b() {}\n"
        );
        assert_eq!(
            draw_view(&source, Layout::Expanded, true, Styling::Plain),
            " 1 function a() {}\n11 function b() {}\n"
        );
    }

    #[test]
    fn plain_layouts_leave_no_trailing_spaces() {
        for layout in [Layout::Interleaved, Layout::SideBySide, Layout::Expanded] {
            let plain = draw_view(THREE, layout, false, Styling::Plain);
            assert!(
                plain.lines().all(|line| !line.ends_with(' ')),
                "{layout:?}: {plain:?}"
            );
        }
    }

    #[test]
    fn side_by_side_bands_only_changed_cells() {
        let styled = draw_view(
            THREE,
            Layout::SideBySide,
            false,
            Styling::Colored(Theme::Dark),
        );
        let banded: Vec<bool> = styled
            .lines()
            .map(|line| line.contains("48;5;236"))
            .collect();
        assert_eq!(banded, [false, false, true]);
    }

    #[test]
    fn unchanged_lines_print_once() {
        assert_eq!(render("struct S;\n", Styling::Plain), "  struct S;\n");
    }

    #[test]
    fn changed_lines_print_their_expansion_beneath() {
        assert_eq!(
            render("fn f() {\n    g()?;\n}\n", Styling::Plain),
            "  fn f() {\n» function f() {\n      g()?;\n»     g() or return early;\n  }\n"
        );
    }

    #[test]
    fn chained_try_is_kept_and_prints_once() {
        assert_eq!(render("g()?.h();\n", Styling::Plain), "  g()?.h();\n");
    }

    #[test]
    fn roled_tokens_are_underlined_on_both_lines() {
        let styled = render("fn f() {}", Styling::Colored(Theme::Dark));
        let keyword = Style::new()
            .fg_color(Some(AnsiColor::Magenta.into()))
            .underline();
        let band = keyword.bg_color(Some(Theme::Dark.band()));
        assert!(
            styled.contains(&format!("{keyword}fn{keyword:#}")),
            "{styled:?}"
        );
        assert!(
            styled.contains(&format!("{band}function{band:#}")),
            "{styled:?}"
        );
    }

    #[test]
    fn only_expansion_lines_have_the_band() {
        for (theme, band) in [(Theme::Dark, "48;5;236"), (Theme::Light, "48;5;254")] {
            let styled = render("fn f() {\n    x;\n}\n", Styling::Colored(theme));
            for line in styled.lines() {
                let is_expansion = strip(line).starts_with('»');
                assert_eq!(line.contains(band), is_expansion, "{theme:?}: {line:?}");
            }
        }
    }

    #[test]
    fn colored_bands_pad_to_the_widest_expansion() {
        let styled = render(
            "fn f() {\n    let x = &y;\n}\n",
            Styling::Colored(Theme::Dark),
        );
        let widths: Vec<usize> = styled
            .lines()
            .map(strip)
            .filter(|line| line.starts_with('»'))
            .map(|line| line.width())
            .collect();
        let widest = "»     let x = borrow y;".width();
        assert_eq!(widths, [widest, widest]);
    }

    #[test]
    fn terminal_and_bidi_controls_are_shown_escaped() {
        let source = "fn f() { let s = \"\u{1b}[2J\u{7}\u{9b}\u{202e}\u{2066}\tok\"; }\n";
        let rendered = render(source, Styling::Plain);
        let expected = "let s = \"\\u{1b}[2J\\u{7}\\u{9b}\\u{202e}\\u{2066}\tok\";";
        assert_eq!(rendered.matches(expected).count(), 2, "{rendered:?}");
        assert!(
            rendered.lines().all(|line| !line.chars().any(is_hazard)),
            "{rendered:?}"
        );
    }

    #[test]
    fn escaped_text_counts_toward_band_width() {
        let styled = render("fn f() { \"\u{1b}\"; }\n", Styling::Colored(Theme::Dark));
        let expansion = styled
            .lines()
            .map(strip)
            .find(|line| line.starts_with('»'))
            .unwrap();
        assert_eq!(expansion, "» function f() { \"\\u{1b}\"; }");
    }

    #[test]
    fn plain_output_has_no_padding_or_escapes() {
        let plain = render("fn f() {\n    let x = &y;\n}\n", Styling::Plain);
        assert!(!plain.contains('\x1b'));
        assert!(plain.lines().all(|line| !line.ends_with(' ')), "{plain:?}");
    }

    #[test]
    fn colored_classes_are_distinct_except_comment_and_attribute() {
        use HighlightClass::*;
        let color = |class| {
            span_style(&Span {
                original: "x".to_string(),
                rendered: "x".to_string(),
                class,
                role: None,
                source_range: 0..1,
            })
            .get_fg_color()
        };
        for class in [Identifier, Operator, Punctuation, Plain] {
            assert_eq!(color(class), None, "{class:?} is colored");
        }
        let colored = [
            Keyword, Type, Function, Macro, Lifetime, String, Number, Comment,
        ];
        let mut colors: Vec<_> = colored.iter().map(|class| color(*class)).collect();
        assert!(colors.iter().all(Option::is_some));
        colors.sort_by_key(|color| format!("{color:?}"));
        colors.dedup();
        assert_eq!(colors.len(), colored.len(), "two classes share a color");
        assert_eq!(color(Attribute), color(Comment));
    }

    const TEN: &str = "a;\nb;\nc;\nd;\ne;\nf;\ng;\nh;\nfn i() {}\nj;\n";

    fn range(text: &str) -> Option<LineRange> {
        Some(text.parse().unwrap())
    }

    #[test]
    fn line_ranges_parse_with_either_end_open() {
        let parsed = |text: &str| text.parse::<LineRange>();
        assert_eq!(parsed("40:80"), Ok(LineRange { start: 40, end: 80 }));
        assert_eq!(parsed("7:7"), Ok(LineRange { start: 7, end: 7 }));
        assert_eq!(
            parsed("40:"),
            Ok(LineRange {
                start: 40,
                end: usize::MAX
            })
        );
        assert_eq!(parsed(":80"), Ok(LineRange { start: 1, end: 80 }));
        assert_eq!(
            parsed(":"),
            Ok(LineRange {
                start: 1,
                end: usize::MAX
            })
        );
        for (text, message) in [
            ("40", "expected START:END, as in 40:80"),
            ("a:9", "`a` is not a line number"),
            ("1:-2", "`-2` is not a line number"),
            ("0:5", "lines are counted from 1"),
            ("9:3", "the range ends at 3 before it starts"),
        ] {
            assert_eq!(parsed(text), Err(message.to_string()), "{text}");
        }
    }

    #[test]
    fn a_range_numbers_its_rows_with_source_lines_in_every_layout() {
        let drawn = |layout| draw_range(TEN, layout, false, range("9:10"), Styling::Plain);
        assert_eq!(
            drawn(Layout::Interleaved),
            " 9   fn i() {}\n   » function i() {}\n10   j;\n"
        );
        assert_eq!(
            drawn(Layout::SideBySide),
            " 9 fn i() {} │ function i() {}\n10 j;        │ j;\n"
        );
        assert_eq!(drawn(Layout::Expanded), " 9 function i() {}\n10 j;\n");
    }

    #[test]
    fn a_range_past_the_end_stops_at_the_last_line() {
        let drawn = draw_range(TEN, Layout::Expanded, false, range("10:99"), Styling::Plain);
        assert_eq!(drawn, "10 j;\n");
    }

    #[test]
    fn a_range_and_changed_only_both_apply() {
        let source = format!("fn a() {{}}\n{}fn b() {{}}\n", "x;\n".repeat(9));
        let drawn = draw_range(&source, Layout::Expanded, true, range("2:"), Styling::Plain);
        assert_eq!(drawn, "11 function b() {}\n");
    }
}
