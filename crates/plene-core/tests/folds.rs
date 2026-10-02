//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use plene_core::{Edition, Fold, folds};

fn found(source: &str) -> Vec<(usize, usize, bool)> {
    folds(source, Edition::default())
        .into_iter()
        .map(|fold| (fold.first, fold.last, fold.function))
        .collect()
}

#[test]
fn a_body_that_spans_lines_folds_from_its_opening_brace() {
    assert_eq!(found("fn f() {\n    x;\n}\n"), [(0, 2, true)]);
    assert_eq!(found("fn f() {\n}"), [(0, 1, true)], "no final newline");
    assert_eq!(
        found("fn f(\n    a: u8,\n) {\n    x;\n}\n"),
        [(2, 4, true)],
        "the signature stays in view"
    );
    assert_eq!(
        found("fn f()\nwhere\n    T: Clone,\n{\n    x;\n}\n"),
        [(3, 5, true)]
    );
}

#[test]
fn a_body_on_one_line_has_nothing_to_hide() {
    assert!(found("fn f() {}\n").is_empty());
    assert!(found("fn f() { x; }\n").is_empty());
    assert!(found("struct S { a: u8 }\n").is_empty());
    assert!(found("").is_empty());
}

#[test]
fn every_kind_of_item_body_folds_and_only_functions_say_so() {
    let source = "\
mod m {
    x;
}
trait T {
    fn a(&self);
}
struct S {
    a: u8,
}
enum E {
    A,
}
impl S {
    fn b(&self) {
        x;
    }
}
";
    assert_eq!(
        found(source),
        [
            (0, 2, false),
            (3, 5, false),
            (6, 8, false),
            (9, 11, false),
            (12, 16, false),
            (13, 15, true)
        ]
    );
}

#[test]
fn nested_bodies_come_in_the_order_they_begin() {
    let source = "impl S {\n    fn a() {\n        x;\n    }\n    fn b() {\n        y;\n    }\n}\n";
    assert_eq!(found(source), [(0, 7, false), (1, 3, true), (4, 6, true)]);
}

#[test]
fn bodies_that_begin_on_one_line_come_outermost_first() {
    let source = "mod m { fn f() {\n    x;\n}\n}\n";
    assert_eq!(found(source), [(0, 3, false), (0, 2, true)]);
}

#[test]
fn blocks_that_are_not_an_items_body_do_not_fold() {
    let source = "\
fn f() {
    if a {
        x;
    }
    loop {
        y;
    }
    let c = || {
        z;
    };
    match a {
        _ => {
            w;
        }
    }
}
enum E {
    V {
        a: u8,
    },
}
const C: u8 = {
    1
};
";
    assert_eq!(found(source), [(0, 15, true), (16, 20, false)]);
}

#[test]
fn bodies_in_macros_and_after_a_parse_error_are_not_found() {
    assert!(found("macro_rules! m {\n    () => {\n        fn f() {\n            x;\n        }\n    };\n}\n").is_empty());
    assert!(found("fn f() {\n    x;\n").is_empty(), "never closed");
    assert!(
        found("match x { fn g() {\n    y;\n}\n}").is_empty(),
        "inside an error node"
    );
}

#[test]
fn lines_are_counted_as_transcribe_counts_them() {
    let plain = "// é ✓\nfn f() {\n    x;\n}\n";
    assert_eq!(found(plain), [(1, 3, true)]);
    assert_eq!(found(&plain.replace('\n', "\r\n")), [(1, 3, true)]);
    let lines = plene_core::transcribe(plain, Edition::default(), &plene_core::Glossary::default());
    assert_eq!(lines.len(), 4);
    let fold: Fold = folds(plain, Edition::default())[0];
    assert!(fold.last < lines.len());
}
