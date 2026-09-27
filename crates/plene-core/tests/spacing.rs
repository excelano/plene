//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use plene_core::{Edition, Glossary, HighlightClass, transcribe};

/// The rendered text of the single line inside `fn f() { … }` wrapping `body`.
fn render_body(body: &str) -> String {
    let source = format!("fn f() {{ {body} }}");
    let rendered = render(&source);
    let inner = rendered
        .strip_prefix("function f() { ")
        .and_then(|rest| rest.strip_suffix(" }"));
    inner
        .unwrap_or_else(|| panic!("unexpected rendering {rendered:?}"))
        .to_string()
}

fn render(source: &str) -> String {
    transcribe(source, Edition::default(), &Glossary::default())
        .iter()
        .flat_map(|line| &line.spans)
        .map(|span| span.rendered.as_str())
        .collect()
}

#[test]
fn spaces_separate_words() {
    for (body, expected) in [
        ("x?;", "x or return early;"),
        ("foo()?;", "foo() or return early;"),
        ("&&x;", "borrow borrow x;"),
        ("&mut x;", "borrow mutable x;"),
        ("&[1];", "borrow [1];"),
        ("&*x;", "borrow dereference x;"),
        ("&-x;", "borrow -x;"),
        ("x=&y;", "x= borrow y;"),
        ("let c = &|x| x;", "let c = borrow closure(x) x;"),
        ("!(a > b);", "not (a > b);"),
        ("&(a);", "borrow (a);"),
        ("x[a..(b)];", "x[a up to (b)];"),
    ] {
        assert_eq!(render_body(body), expected, "for {body:?}");
    }
}

#[test]
fn hugging_punctuation_takes_no_space() {
    for (body, expected) in [
        ("(&x);", "(borrow x);"),
        ("[&x];", "[borrow x];"),
        ("{&x};", "{borrow x};"),
        ("g(x?, y);", "g(x or return early, y);"),
        ("g()?.h();", "g()?.h();"),
    ] {
        assert_eq!(render_body(body), expected, "for {body:?}");
    }
}

#[test]
fn spaces_in_signatures() {
    for (source, expected) in [
        ("pub(crate) fn f() {}", "public(crate) function f() {}"),
        ("fn f()->u8 {}", "function f() returns u8 {}"),
        (
            "fn f<'a>(x: &'a T) {}",
            "function f<lifetime a>(x: borrowed lifetime a T) {}",
        ),
        ("fn f(x: fn(u8)) {}", "function f(x: function(u8)) {}"),
        ("fn f(&self) {}", "function f(borrowed self) {}"),
        ("impl<T> S<T> {}", "implement<T> S<T> {}"),
        (
            "fn f() -> <T as Tr>::X {}",
            "function f() returns <T as Tr>::X {}",
        ),
    ] {
        assert_eq!(render(source), expected, "for {source:?}");
    }
}

#[test]
fn inserted_space_is_an_empty_plain_span_at_the_next_token() {
    let lines = transcribe("fn f() { x?; }", Edition::default(), &Glossary::default());
    let spans = &lines[0].spans;
    let question = spans.iter().position(|span| span.original == "?").unwrap();
    let space = &spans[question - 1];
    assert_eq!(space.original, "");
    assert_eq!(space.rendered, " ");
    assert_eq!(space.class, HighlightClass::Plain);
    assert_eq!(space.role, None);
    let at = spans[question].source_range.start;
    assert_eq!(space.source_range, at..at);
}

#[test]
fn empty_override_hides_the_token_without_spacing() {
    let mut glossary = Glossary::default();
    let (overrides, _) =
        Glossary::parse("[[expand]]\ntoken = \"&\"\nrole = \"ref_expr\"\ntext = \"\"\n").unwrap();
    glossary.merge(overrides);
    let rendered: String = transcribe("fn f() { g(&x); }", Edition::default(), &glossary)
        .iter()
        .flat_map(|line| &line.spans)
        .map(|span| span.rendered.as_str())
        .collect();
    assert_eq!(rendered, "function f() { g(x); }");
}
