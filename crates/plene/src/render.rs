//! Terminal rendering of transcribed lines.

use std::fmt::Write;

use anstyle::{AnsiColor, Style};
use plene_core::{HighlightClass, Line, Span};

const SOURCE_GUTTER: &str = "  ";
const EXPANSION_GUTTER: &str = "» ";
const GUTTER_STYLE: Style = Style::new().dimmed();

/// Each source line, followed by its expansion when the expansion differs. Output is
/// always styled; the output stream strips styles when color is off.
pub fn interleaved(lines: &[Line]) -> String {
    let mut out = String::new();
    for line in lines {
        out.push_str(SOURCE_GUTTER);
        push_spans(&mut out, &line.spans, |span| &span.original);
        out.push('\n');
        if line.spans.iter().any(|span| span.rendered != span.original) {
            write!(out, "{GUTTER_STYLE}{EXPANSION_GUTTER}{GUTTER_STYLE:#}").unwrap();
            push_spans(&mut out, &line.spans, |span| &span.rendered);
            out.push('\n');
        }
    }
    out
}

fn push_spans(out: &mut String, spans: &[Span], text: impl Fn(&Span) -> &str) {
    for span in spans {
        let style = style(span);
        write!(out, "{style}{}{style:#}", text(span)).unwrap();
    }
}

/// The class color, underlined for tokens with a glossary role so that a token and its
/// expansion pair up across the two lines.
fn style(span: &Span) -> Style {
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

    fn plain(source: &str) -> String {
        let lines = transcribe(source, Edition::default(), &Glossary::default());
        anstream::adapter::strip_str(&interleaved(&lines)).to_string()
    }

    #[test]
    fn unchanged_lines_print_once() {
        assert_eq!(plain("struct S;\n"), "  struct S;\n");
    }

    #[test]
    fn changed_lines_print_their_expansion_beneath() {
        assert_eq!(
            plain("fn f() {\n    g()?;\n}\n"),
            "  fn f() {\n» function f() {\n      g()?;\n»     g() or return early;\n  }\n"
        );
    }

    #[test]
    fn chained_try_is_kept_and_prints_once() {
        assert_eq!(plain("g()?.h();\n"), "  g()?.h();\n");
    }

    #[test]
    fn roled_tokens_are_underlined_on_both_lines() {
        let lines = transcribe("fn f() {}", Edition::default(), &Glossary::default());
        let styled = interleaved(&lines);
        let keyword = Style::new()
            .fg_color(Some(AnsiColor::Magenta.into()))
            .underline();
        assert!(
            styled.contains(&format!("{keyword}fn{keyword:#}")),
            "{styled:?}"
        );
        assert!(
            styled.contains(&format!("{keyword}function{keyword:#}")),
            "{styled:?}"
        );
    }

    #[test]
    fn colored_classes_are_distinct_except_comment_and_attribute() {
        use HighlightClass::*;
        let color = |class| {
            style(&Span {
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
}
