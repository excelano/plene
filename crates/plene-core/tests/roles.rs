//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

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

#[test]
fn impl_reads_by_where_it_stands() {
    let roles = roles_in("impl<T> S<T> {}\nfn f(x: impl Tr) -> impl Tr { x }");
    let impls: Vec<_> = roles
        .iter()
        .filter(|(text, _)| text == "impl")
        .map(|(_, role)| *role)
        .collect();
    assert_eq!(
        impls,
        [Role::ImplBlock, Role::ImplTraitType, Role::ImplTraitType]
    );
}

#[test]
fn dyn_is_a_keyword_role_in_2015_type_position() {
    let lines = transcribe(
        "fn f(x: Box<dyn Tr>) {}",
        Edition::E2015,
        &Glossary::default(),
    );
    let role = lines[0]
        .spans
        .iter()
        .find(|span| span.original == "dyn")
        .unwrap()
        .role;
    assert_eq!(role, Some(Role::Keyword));
}

#[test]
fn star_and_bang_read_by_where_they_stand() {
    let roles =
        roles_in("fn f(p: *const u8) -> bool { let n = *p * 2; !(n > 1) }\nimpl !Send for S {}");
    let stars: Vec<_> = roles
        .iter()
        .filter(|(text, _)| text == "*")
        .map(|(_, role)| *role)
        .collect();
    assert_eq!(
        stars,
        [Role::PtrType, Role::Deref],
        "multiplication takes no role"
    );
    let bangs: Vec<_> = roles
        .iter()
        .filter(|(text, _)| text == "!")
        .map(|(_, role)| *role)
        .collect();
    assert_eq!(bangs, [Role::Not, Role::NegativeImpl]);
}

#[test]
fn shifts_are_operators_but_nested_generics_are_not() {
    let roles = roles_in("fn f() -> Vec<Vec<u8>> { x >> 1; Vec::<Vec<u8>>::new() }");
    let texts: Vec<_> = roles.iter().map(|(text, _)| text.as_str()).collect();
    assert_eq!(texts, ["fn", "->", ">>"]);
}

#[test]
fn underscore_reads_by_where_it_stands() {
    for (source, expected) in [
        ("fn f() { let _ = g(); }", Some(Role::Discard)),
        ("fn f() { let _: u8 = g(); }", Some(Role::Discard)),
        ("fn f() { let (a, _) = g(); }", Some(Role::Wildcard)),
        ("fn f() { match x { _ => 0 }; }", Some(Role::Wildcard)),
        ("fn f(_: u8) {}", Some(Role::Wildcard)),
        ("fn f() { let v: Vec<_> = g(); }", Some(Role::InferredType)),
        ("fn f() { _ = g(); }", Some(Role::Discard)),
        ("fn f() { (a, _) = g(); }", Some(Role::Wildcard)),
        ("fn f() { _ += 1; }", Some(Role::Wildcard)),
        ("use std::fmt::Write as _;", None),
    ] {
        let lines = transcribe(source, Edition::default(), &Glossary::default());
        let role = lines[0]
            .spans
            .iter()
            .find(|span| span.original == "_")
            .unwrap()
            .role;
        assert_eq!(role, expected, "in {source:?}");
    }
}

#[test]
fn dot_dot_reads_by_where_it_stands() {
    let roles = roles_in("fn f() { let S { a, .. } = s; let t = S { a: 1, ..s }; }");
    let dots: Vec<_> = roles
        .iter()
        .filter(|(text, _)| text == "..")
        .map(|(_, role)| *role)
        .collect();
    assert_eq!(dots, [Role::RestPattern, Role::StructUpdate]);
}

#[test]
fn ranges_read_by_their_ends() {
    for (source, expected) in [
        ("fn f() { a..b; }", Role::Range),
        ("fn f() { ..b; }", Role::Range),
        ("fn f() { a..; }", Role::RangeFrom),
        ("fn f() { v[..]; }", Role::RangeFull),
        ("fn f() { a..=b; }", Role::RangeInclusive),
        ("fn f() { ..=b; }", Role::RangeInclusive),
        (
            "fn f() { match n { 1..=9 => 0, _ => 1 } }",
            Role::RangeInclusive,
        ),
        ("fn f() { match n { 10..20 => 0, _ => 1 } }", Role::Range),
        ("fn f() { match n { 100.. => 0, _ => 1 } }", Role::RangeFrom),
    ] {
        let roles = roles_in(source);
        let range = roles
            .iter()
            .find(|(text, _)| text.starts_with(".."))
            .unwrap();
        assert_eq!(range.1, expected, "in {source:?}");
    }
}

#[test]
fn colons_are_bounds_only_before_a_bound_list() {
    for (source, expected) in [
        ("fn f<T: Clone>() {}", Some(Role::TraitBound)),
        ("fn f<'a: 'b, 'b>() {}", Some(Role::LifetimeBound)),
        ("fn f<T: 'a + Clone>() {}", Some(Role::LifetimeBound)),
        ("fn f<T>() where T: Clone {}", Some(Role::TraitBound)),
        ("trait A: B {}", Some(Role::TraitBound)),
        ("trait A { type B: Copy; }", Some(Role::TraitBound)),
        ("fn f<I: Iterator<Item: Copy>>() {}", Some(Role::TraitBound)),
        ("fn f(x: u8) {}", None),
        ("struct S { x: u8 }", None),
        ("fn f() { 'a: loop {} }", None),
        ("fn f<const N: usize>() {}", None),
    ] {
        let lines = transcribe(source, Edition::default(), &Glossary::default());
        let role = lines[0]
            .spans
            .iter()
            .find(|span| span.original == ":")
            .unwrap()
            .role;
        assert_eq!(role, expected, "in {source:?}");
    }
}
