//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use plene_core::{Category, Edition, Glossary, Line, Role, Span, UnknownCategory, transcribe};

mod common;
use common::fixtures;

fn spans(lines: &[Line]) -> impl Iterator<Item = &Span> {
    lines
        .iter()
        .flat_map(|line| &line.spans)
        .filter(|span| !span.original.is_empty())
}

#[test]
fn every_category_covers_a_builtin_entry_and_every_entry_has_one() {
    let glossary = Glossary::default();
    for category in Category::ALL {
        assert!(
            glossary
                .entries()
                .any(|entry| Category::of(&entry.token, entry.role) == *category),
            "no entry in {category}"
        );
    }
    let covered: usize = Category::ALL
        .iter()
        .map(|category| {
            glossary
                .entries()
                .filter(|entry| Category::of(&entry.token, entry.role) == *category)
                .count()
        })
        .sum();
    assert_eq!(covered, glossary.entries().count());
}

#[test]
fn keyword_tokens_split_by_what_they_are() {
    let of = |token| Category::of(token, Role::Keyword);
    assert_eq!(of("pub"), Category::Visibility);
    assert_eq!(of("mut"), Category::Mutability);
    assert_eq!(of("ref"), Category::References);
    for token in ["fn", "mod", "dyn", "extern"] {
        assert_eq!(of(token), Category::Keywords, "{token}");
    }
}

#[test]
fn ids_round_trip_and_an_unknown_one_lists_the_known() {
    for category in Category::ALL {
        assert_eq!(category.as_str().parse::<Category>(), Ok(*category));
        assert_eq!(category.to_string(), category.as_str());
        assert!(!category.label().is_empty());
    }
    let unknown: UnknownCategory = "nope".parse::<Category>().unwrap_err();
    let message = unknown.to_string();
    assert!(message.starts_with("unknown category `nope`; one of: keywords, visibility, "));
    assert!(message.ends_with(", flow, ends, macros"));
}

#[test]
fn without_drops_exactly_the_entries_of_its_categories() {
    let glossary = Glossary::default();
    let total = glossary.entries().count();
    assert_eq!(glossary.without(&[]).entries().count(), total);
    assert_eq!(glossary.without(Category::ALL).entries().count(), 0);

    let kept = glossary.without(&[Category::Lifetimes, Category::Ranges]);
    assert!(kept.entries().all(|entry| {
        !matches!(
            Category::of(&entry.token, entry.role),
            Category::Lifetimes | Category::Ranges
        )
    }));
    let dropped = glossary
        .entries()
        .filter(|entry| {
            matches!(
                Category::of(&entry.token, entry.role),
                Category::Lifetimes | Category::Ranges
            )
        })
        .count();
    assert_eq!(kept.entries().count(), total - dropped);
    assert_eq!(kept.expand("'a", Role::Lifetime, Some("a")), None);
    assert_eq!(
        kept.expand("fn", Role::Keyword, None).as_deref(),
        Some("function")
    );
}

#[test]
fn a_switched_off_category_is_left_as_written_and_nothing_else_changes() {
    let full = Glossary::default();
    for category in Category::ALL {
        let reduced = full.without(&[*category]);
        for (name, source) in fixtures() {
            let with = transcribe(&source, Edition::default(), &full);
            let without = transcribe(&source, Edition::default(), &reduced);
            assert_eq!(with.len(), without.len(), "{category} in {name}");
            for (a, b) in spans(&with).zip(spans(&without)) {
                assert_eq!(a.original, b.original, "{category} in {name}");
                let in_category = a
                    .role
                    .is_some_and(|role| Category::of(&a.original, role) == *category);
                if in_category {
                    assert_eq!(b.rendered, b.original, "{category} in {name}");
                    assert_eq!(b.role, None, "{category} in {name}");
                } else {
                    assert_eq!((&a.rendered, a.role), (&b.rendered, b.role));
                }
            }
        }
    }
}

#[test]
fn the_source_survives_any_switched_off_categories() {
    let full = Glossary::default();
    let mut subsets: Vec<Vec<Category>> = Category::ALL.iter().map(|c| vec![*c]).collect();
    subsets.push(Category::ALL.to_vec());
    for subset in subsets {
        let glossary = full.without(&subset);
        for (name, source) in fixtures() {
            for text in [source.clone(), source.replace('\n', "\r\n")] {
                let lines = transcribe(&text, Edition::default(), &glossary);
                let rebuilt: String = lines
                    .iter()
                    .map(|line| {
                        let original: String = line.spans.iter().map(|s| &*s.original).collect();
                        original + &line.ending
                    })
                    .collect();
                assert_eq!(rebuilt, text, "{subset:?} in {name}");
                assert_eq!(lines.len(), text.lines().count(), "{subset:?} in {name}");
            }
        }
    }
}

#[test]
fn with_every_category_off_the_transcription_is_the_source() {
    let glossary = Glossary::default().without(Category::ALL);
    for (name, source) in fixtures() {
        for line in transcribe(&source, Edition::default(), &glossary) {
            for span in &line.spans {
                assert_eq!(span.rendered, span.original, "{name}");
                assert_eq!(span.role, None, "{name}");
            }
        }
    }
}
