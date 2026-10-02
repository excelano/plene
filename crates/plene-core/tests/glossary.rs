//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use plene_core::{Glossary, Role};

const BUILTIN: &str = include_str!("../glossary.toml");

fn parse(text: &str) -> (Glossary, Vec<String>) {
    Glossary::parse(text).unwrap()
}

#[test]
fn builtin_parses_without_warnings() {
    let (_, warnings) = parse(BUILTIN);
    assert!(warnings.is_empty(), "{warnings:?}");
}

#[test]
fn builtin_covers_every_role_with_a_note() {
    let glossary = Glossary::default();
    for role in Role::ALL {
        assert!(
            glossary.entries().any(|entry| entry.role == *role),
            "no entry for {role}"
        );
    }
    for entry in glossary.entries() {
        assert!(
            entry.note.as_deref().is_some_and(|note| !note.is_empty()),
            "no note for {entry:?}"
        );
    }
}

#[test]
fn builtin_links_every_entry_to_the_rust_reference() {
    let glossary = Glossary::default();
    for entry in glossary.entries() {
        let url = entry.url.as_deref().unwrap_or("");
        assert!(
            url.starts_with("https://doc.rust-lang.org/reference/")
                && (url.ends_with(".html") || url.contains(".html#")),
            "no reference link for {entry:?}"
        );
        assert!(!url.contains(' '), "{url}");
    }
}

#[test]
fn expands_by_token_and_role() {
    let glossary = Glossary::default();
    assert_eq!(
        glossary.expand("fn", Role::Keyword, None).as_deref(),
        Some("function")
    );
    assert_eq!(
        glossary.expand("&", Role::RefExpr, None).as_deref(),
        Some("borrow")
    );
    assert_eq!(
        glossary.expand("&", Role::RefType, None).as_deref(),
        Some("borrowed")
    );
    assert_eq!(
        glossary.expand("?", Role::TryChained, None).as_deref(),
        Some("?")
    );
    assert_eq!(glossary.expand("pub", Role::RefExpr, None), None);
    assert_eq!(glossary.expand("let", Role::Keyword, None), None);
}

#[test]
fn named_roles_fill_in_the_name() {
    let glossary = Glossary::default();
    assert_eq!(
        glossary.expand("'a", Role::Lifetime, Some("a")).as_deref(),
        Some("lifetime a")
    );
    assert_eq!(
        glossary
            .expand("'static", Role::Lifetime, Some("static"))
            .as_deref(),
        Some("lifetime static")
    );
    assert_eq!(
        glossary
            .expand("'outer", Role::Label, Some("outer"))
            .as_deref(),
        Some("label outer")
    );
    assert_eq!(
        glossary
            .expand("'_", Role::LifetimeAnonymous, None)
            .as_deref(),
        Some("lifetime inferred")
    );
}

#[test]
fn unknown_role_warns_and_skips() {
    let (glossary, warnings) = parse(
        r#"
        [[expand]]
        token = "&"
        role = "no_such_role"
        text = "x"
        "#,
    );
    assert_eq!(glossary.entries().count(), 0);
    assert_eq!(
        warnings,
        ["skipping entry for `&`: unknown role `no_such_role`"]
    );
}

#[test]
fn duplicate_warns_and_later_wins() {
    let (glossary, warnings) = parse(
        r#"
        [[expand]]
        token = "fn"
        role = "keyword"
        text = "first"

        [[expand]]
        token = "fn"
        role = "keyword"
        text = "second"
        "#,
    );
    assert_eq!(
        glossary.expand("fn", Role::Keyword, None).as_deref(),
        Some("second")
    );
    assert_eq!(
        warnings,
        ["duplicate entry for `fn` as keyword; the later one wins"]
    );
}

#[test]
fn malformed_files_are_errors() {
    for (text, reason) in [
        (
            "[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\n",
            "missing field `text`",
        ),
        (
            "[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntxt = \"function\"\n",
            "unknown field `txt`",
        ),
        ("[[expand]\n", "unclosed array table"),
        (
            "expand = 1\n",
            "invalid type: integer `1`, expected a sequence",
        ),
        ("other = []\n", "unknown field `other`"),
    ] {
        let error = Glossary::parse(text).unwrap_err().to_string();
        assert!(error.starts_with("invalid glossary: "), "{error}");
        assert!(error.contains(reason), "expected {reason:?} in {error}");
    }
}

#[test]
fn empty_file_is_an_empty_glossary() {
    let (glossary, warnings) = parse("");
    assert_eq!(glossary.entries().count(), 0);
    assert!(warnings.is_empty());
}

#[test]
fn merge_replaces_text_and_keeps_notes_unless_given() {
    let mut glossary = Glossary::default();
    let default_note = glossary.entry("fn", Role::Keyword).unwrap().note.clone();
    let (overrides, _) = parse(
        r#"
        [[expand]]
        token = "fn"
        role = "keyword"
        text = "func"

        [[expand]]
        token = "mut"
        role = "keyword"
        text = "changeable"
        note = "Custom note."

        [[expand]]
        token = "'b"
        role = "lifetime"
        text = "for {name}"
        "#,
    );
    glossary.merge(overrides);

    let fn_entry = glossary.entry("fn", Role::Keyword).unwrap();
    assert_eq!(fn_entry.text, "func");
    assert_eq!(fn_entry.note, default_note);
    assert_eq!(
        glossary
            .entry("mut", Role::Keyword)
            .unwrap()
            .note
            .as_deref(),
        Some("Custom note.")
    );
    assert_eq!(
        glossary.expand("'a", Role::Lifetime, Some("a")).as_deref(),
        Some("for a")
    );
    assert_eq!(
        glossary.expand("pub", Role::Keyword, None).as_deref(),
        Some("public")
    );
    assert_eq!(
        glossary
            .entries()
            .filter(|entry| entry.role == Role::Lifetime)
            .count(),
        1
    );
}

#[test]
fn merge_keeps_the_link_unless_given_another() {
    let mut glossary = Glossary::default();
    let base = glossary.entry("fn", Role::Keyword).unwrap().url.clone();
    assert!(base.is_some());
    let (overrides, _) = parse(
        r#"
        [[expand]]
        token = "fn"
        role = "keyword"
        text = "func"

        [[expand]]
        token = "mut"
        role = "keyword"
        text = "changeable"
        url = "https://example.com/mutable"

        [[expand]]
        token = "'b"
        role = "lifetime"
        text = "for {name}"
        "#,
    );
    glossary.merge(overrides);
    assert_eq!(glossary.entry("fn", Role::Keyword).unwrap().url, base);
    assert_eq!(
        glossary.entry("mut", Role::Keyword).unwrap().url.as_deref(),
        Some("https://example.com/mutable")
    );
    assert!(
        glossary
            .entry("'a", Role::Lifetime)
            .unwrap()
            .url
            .as_deref()
            .is_some_and(|url| url.ends_with("items/generics.html")),
        "a lifetime override with no link keeps the lifetime link"
    );
}

#[test]
fn a_link_is_written_and_read_back_and_an_absent_one_is_not_written() {
    let (glossary, _) = parse(
        "[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntext = \"function\"\nurl = \"https://example.com/fn\"\n\n\
         [[expand]]\ntoken = \"mut\"\nrole = \"keyword\"\ntext = \"mutable\"\n",
    );
    let written = glossary.to_toml();
    assert_eq!(written.matches("url = ").count(), 1, "{written}");
    let (read_back, _) = parse(&written);
    assert_eq!(
        read_back.entry("fn", Role::Keyword).unwrap().url.as_deref(),
        Some("https://example.com/fn")
    );
    assert_eq!(read_back.entry("mut", Role::Keyword).unwrap().url, None);
}

#[test]
fn to_toml_reads_back_to_the_same_entries() {
    let mut glossary = Glossary::default();
    let (overrides, _) =
        parse("[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntext = \"func \\\"quoted\\\"\"\n");
    glossary.merge(overrides);
    let (read_back, warnings) = parse(&glossary.to_toml());
    assert!(warnings.is_empty(), "{warnings:?}");
    let original: Vec<_> = glossary.entries().collect();
    let round_tripped: Vec<_> = read_back.entries().collect();
    assert_eq!(original, round_tripped);
    assert_eq!(
        read_back.expand("fn", Role::Keyword, None).as_deref(),
        Some("func \"quoted\"")
    );
}

#[test]
fn to_toml_omits_missing_notes_and_orders_by_role() {
    let (glossary, _) = parse(
        "[[expand]]\ntoken = \"->\"\nrole = \"ret_type\"\ntext = \"returns\"\n\n\
         [[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntext = \"function\"\n",
    );
    let toml = glossary.to_toml();
    assert!(!toml.contains("note"), "{toml}");
    assert!(
        toml.find("\"fn\"").unwrap() < toml.find("\"->\"").unwrap(),
        "{toml}"
    );
}
