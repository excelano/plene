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
fn expands_by_token_and_role() {
    let glossary = Glossary::default();
    assert_eq!(
        glossary.expand("fn", Role::Keyword).as_deref(),
        Some("function")
    );
    assert_eq!(
        glossary.expand("&", Role::RefExpr).as_deref(),
        Some("borrow")
    );
    assert_eq!(
        glossary.expand("&", Role::RefType).as_deref(),
        Some("borrowed")
    );
    assert_eq!(glossary.expand("?", Role::TryChained).as_deref(), Some("?"));
    assert_eq!(glossary.expand("pub", Role::RefExpr), None);
    assert_eq!(glossary.expand("let", Role::Keyword), None);
}

#[test]
fn named_roles_fill_in_the_name() {
    let glossary = Glossary::default();
    assert_eq!(
        glossary.expand("'a", Role::Lifetime).as_deref(),
        Some("lifetime a")
    );
    assert_eq!(
        glossary.expand("'static", Role::Lifetime).as_deref(),
        Some("lifetime static")
    );
    assert_eq!(
        glossary.expand("'outer", Role::Label).as_deref(),
        Some("label outer")
    );
    assert_eq!(
        glossary.expand("'_", Role::LifetimeAnonymous).as_deref(),
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
        glossary.expand("fn", Role::Keyword).as_deref(),
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
        glossary.expand("'a", Role::Lifetime).as_deref(),
        Some("for a")
    );
    assert_eq!(
        glossary.expand("pub", Role::Keyword).as_deref(),
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
