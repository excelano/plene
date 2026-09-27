use std::collections::BTreeSet;

use plene_core::{Edition, Glossary, Role, transcribe};

mod common;
use common::fixtures;

fn roles_in(source: &str) -> Vec<(String, Role)> {
    transcribe(source, Edition::default(), &Glossary::default())
        .into_iter()
        .flat_map(|line| line.spans)
        .filter_map(|span| span.role.map(|role| (span.original, role)))
        .collect()
}

#[test]
fn every_role_has_a_fixture() {
    let covered: BTreeSet<Role> = fixtures()
        .iter()
        .flat_map(|(_, source)| roles_in(source))
        .map(|(_, role)| role)
        .collect();
    let missing: Vec<_> = Role::ALL
        .iter()
        .filter(|role| !covered.contains(role))
        .collect();
    assert!(missing.is_empty(), "roles without a fixture: {missing:?}");
}

#[test]
fn role_ids_round_trip() {
    for role in Role::ALL {
        assert_eq!(role.as_str().parse::<Role>(), Ok(*role));
    }
    let unknown = "no_such_role".parse::<Role>().unwrap_err();
    assert_eq!(unknown.to_string(), "unknown role `no_such_role`");
}

#[test]
fn try_is_chained_only_as_a_receiver() {
    for (source, expected) in [
        ("fn f() { g()?; }", Role::Try),
        ("fn f() { h(g()?); }", Role::Try),
        ("fn f() { g()??; }", Role::Try),
        ("fn f() { g()?.h(); }", Role::TryChained),
        ("fn f() { g()?.field; }", Role::TryChained),
        ("fn f() { g()?[0]; }", Role::TryChained),
        ("async fn f() { g()?.await; }", Role::TryChained),
        ("fn f() { v[g()?]; }", Role::Try),
    ] {
        let roles = roles_in(source);
        let question = roles.iter().find(|(text, _)| text == "?").unwrap();
        assert_eq!(question.1, expected, "in {source:?}");
    }
}

#[test]
fn closure_pipes_open_and_close() {
    let roles = roles_in("fn f() { let c = |x| x; let d = || 0; let e = a || b; }");
    let pipes: Vec<_> = roles
        .iter()
        .filter(|(text, _)| text == "|")
        .map(|(_, role)| *role)
        .collect();
    assert_eq!(
        pipes,
        [
            Role::ClosureOpen,
            Role::ClosureClose,
            Role::ClosureOpen,
            Role::ClosureClose
        ]
    );
    assert!(
        !roles.iter().any(|(text, _)| text == "||"),
        "logical || has no role"
    );
}

#[test]
fn tokens_in_error_nodes_have_no_role() {
    let valid = roles_in("struct S { a: &mut u8 }");
    assert_eq!(
        valid.len(),
        2,
        "control: `&` and `mut` in a field type have roles"
    );
    let broken = roles_in("struct S { a: u8, &mut b }");
    assert!(
        broken.is_empty(),
        "`&mut` in an error node took roles: {broken:?}"
    );
}
