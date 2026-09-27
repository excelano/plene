use plene_core::{Edition, HighlightClass, transcribe};

/// The class of the first span whose text is `text`.
fn class_of(source: &str, edition: Edition, text: &str) -> HighlightClass {
    transcribe(source, edition)
        .into_iter()
        .flat_map(|line| line.spans)
        .find(|span| span.original == text)
        .unwrap_or_else(|| panic!("no span {text:?} in {source:?}"))
        .class
}

#[test]
fn async_is_a_keyword_from_2018() {
    let source = "async fn f() {}";
    assert_eq!(
        class_of(source, Edition::E2015, "async"),
        HighlightClass::Identifier
    );
    assert_eq!(
        class_of(source, Edition::E2018, "async"),
        HighlightClass::Keyword
    );
}

#[test]
fn gen_is_a_keyword_from_2024() {
    let source = "fn f() { let gen = 1; }";
    assert_eq!(
        class_of(source, Edition::E2021, "gen"),
        HighlightClass::Identifier
    );
    assert_eq!(
        class_of(source, Edition::E2024, "gen"),
        HighlightClass::Keyword
    );
}

#[test]
fn dyn_in_type_position_is_a_keyword_in_2015() {
    let source = "fn f(x: Box<dyn Draw>) {}";
    assert_eq!(
        class_of(source, Edition::E2015, "dyn"),
        HighlightClass::Keyword
    );
}

#[test]
fn contextual_keywords_are_identifiers_outside_their_context() {
    let source = "union U { a: u8 }\nfn f() { let union = 1; }";
    let lines = transcribe(source, Edition::default());
    let unions: Vec<_> = lines
        .iter()
        .flat_map(|line| &line.spans)
        .filter(|span| span.original == "union")
        .map(|span| span.class)
        .collect();
    assert_eq!(
        unions,
        [HighlightClass::Keyword, HighlightClass::Identifier]
    );
}
