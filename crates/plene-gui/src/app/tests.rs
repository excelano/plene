//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{DroppedFile, Key, Pos2, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;

use super::*;

/// Writes `contents` to a file of its own under the system's temporary directory.
fn temp_file(name: &str, contents: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("plene-gui-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path
}

fn app_with(file: Option<&Path>) -> App {
    App::new(
        Edition::default(),
        Ok((Glossary::default(), Vec::new())),
        file,
    )
}

fn window(app: App, size: Vec2) -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(size)
        .build_ui_state(|ui, app: &mut App| app.show(ui), app);
    harness.run();
    harness
}

fn opened(source: &str, name: &str, size: Vec2) -> Harness<'static, App> {
    window(app_with(Some(&temp_file(name, source))), size)
}

/// Moves the pointer over the columns until a hover card holding `text` shows, and
/// returns where it showed.
fn hover_at_card(
    harness: &mut Harness<'_, App>,
    xs: std::ops::Range<usize>,
    text: &str,
) -> Option<Pos2> {
    for y in (0..160).step_by(4) {
        for x in xs.clone().step_by(3) {
            let pos = Pos2::new(x as f32, y as f32);
            harness.hover_at(pos);
            harness.run();
            if harness.query_by_label_contains(text).is_some() {
                return Some(pos);
            }
        }
    }
    None
}

fn hover_shows(harness: &mut Harness<'_, App>, xs: std::ops::Range<usize>, text: &str) -> bool {
    hover_at_card(harness, xs, text).is_some()
}

fn click(harness: &mut Harness<'_, App>, pos: Pos2) {
    harness.hover_at(pos);
    harness.run();
    harness.drag_at(pos);
    harness.run();
    harness.drop_at(pos);
    harness.run();
}

fn press(harness: &mut Harness<'_, App>, key: Key, times: usize) {
    for _ in 0..times {
        harness.key_press(key);
        harness.run();
    }
}

fn selected(harness: &Harness<'_, App>) -> Option<usize> {
    harness.state().view.selected
}

/// A document of `count` numbered functions, one per line.
fn numbered(count: usize, name: &str, size: Vec2) -> Harness<'static, App> {
    let source: String = (0..count).map(|n| format!("fn f{n}() {{}}\n")).collect();
    opened(&source, name, size)
}

const WIDE: Vec2 = Vec2::new(900.0, 400.0);

#[test]
fn empty_window_says_how_to_open_a_file() {
    let harness = window(app_with(None), WIDE);
    harness.get_by_label_contains("Open a Rust file");
    assert_eq!(harness.state().title, "plene");
}

#[test]
fn file_named_at_start_opens() {
    let harness = opened("fn f() {}\n", "named.rs", WIDE);
    harness.get_by_label("named.rs");
    assert_eq!(harness.state().title, "named.rs - plene");
    let document = harness.state().document.as_ref().unwrap();
    assert_eq!(document.lines[0].spans[0].rendered, "function");
}

#[derive(Debug)]
struct Dropped(PathBuf);

impl DroppedFile for Dropped {
    fn path(&self) -> &Path {
        &self.0
    }

    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|error| error.to_string())
    }
}

#[test]
fn unreadable_file_is_reported_until_one_opens() {
    let missing = std::env::temp_dir().join("plene-gui-tests/missing.rs");
    let mut harness = window(app_with(Some(&missing)), WIDE);
    harness.get_by_label_contains("missing.rs: ");
    assert!(harness.state().document.is_none());

    let dropped = temp_file("dropped.rs", "fn f() {}\n");
    harness.input_mut().dropped_files = vec![Arc::new(Dropped(dropped))];
    harness.run();
    harness.get_by_label("dropped.rs");
    assert!(harness.query_by_label_contains("missing.rs: ").is_none());
}

#[test]
fn glossary_problems_are_shown_and_the_built_in_glossary_used() {
    let file = temp_file("glossary.rs", "fn f() {}\n");
    let failed = App::new(
        Edition::default(),
        Err("g.toml: bad".to_string()),
        Some(&file),
    );
    let harness = window(failed, WIDE);
    harness.get_by_label("g.toml: bad; using the built-in glossary");
    let document = harness.state().document.as_ref().unwrap();
    assert_eq!(document.lines[0].spans[0].rendered, "function");

    let warned = App::new(
        Edition::default(),
        Ok((
            Glossary::default(),
            vec!["g.toml: unknown role".to_string()],
        )),
        None,
    );
    window(warned, WIDE).get_by_label("warning: g.toml: unknown role");
}

#[test]
fn hovering_either_side_shows_the_note() {
    let mut harness = opened("fn f() {}\n", "hover.rs", WIDE);
    assert!(hover_shows(&mut harness, 450..900, "Declares a function."));
    harness.get_by_label("fn  →  function");
    harness.get_by_label("keyword");

    let mut harness = opened("fn f() {}\n", "hover_source.rs", WIDE);
    assert!(hover_shows(&mut harness, 0..450, "Declares a function."));
}

#[test]
fn a_lifetime_finds_its_note_by_role() {
    let mut harness = opened("fn f<'a>() {}\n", "lifetime.rs", WIDE);
    assert!(hover_shows(&mut harness, 450..900, "'a  →  lifetime a"));
    let note = Glossary::default()
        .entry("'a", Role::Lifetime)
        .and_then(|entry| entry.note.clone())
        .unwrap();
    harness.get_by_label(&note);
}

#[test]
fn tokens_without_a_role_show_no_card() {
    let mut harness = opened("const N: u8 = 1;\n", "plain.rs", WIDE);
    let document = harness.state().document.as_ref().unwrap();
    assert!(
        document.lines[0]
            .spans
            .iter()
            .all(|span| span.role.is_none())
    );
    assert!(!hover_shows(&mut harness, 0..900, "→"));
}

#[test]
fn spans_past_a_wrap_are_hovered() {
    let source = "fn f() -> u8 { first_long_name + second_long_name + third_long_name(&value) }\n";
    let mut harness = opened(source, "wrapped.rs", Vec2::new(420.0, 300.0));
    assert!(hover_shows(&mut harness, 210..420, "&  →  borrow"));
}

#[test]
fn each_pane_has_its_numbers_and_the_panes_fill_the_width() {
    let columns = Columns::new(1000.0, 8.0, 250, 2);
    assert_eq!(columns.number_width, 24.0);
    assert_eq!(columns.number_end(Side::Source), 24.0);
    assert_eq!(columns.text_start(Side::Source), 24.0 + GAP);
    let source_end = columns.text_start(Side::Source) + columns.text_width;
    assert_eq!(columns.pane_start(Side::Transcription), source_end + GAP);
    assert_eq!(
        columns.text_start(Side::Transcription),
        columns.number_end(Side::Transcription) + GAP
    );
    assert_eq!(columns.width(), 1000.0);
}

#[test]
fn a_narrow_window_keeps_each_text_readable() {
    let columns = Columns::new(100.0, 8.0, 9, 2);
    assert_eq!(columns.text_width, 8.0 * MIN_COLUMN_CHARS);
    assert!(columns.width() > 100.0);
}

#[test]
fn a_click_selects_the_row_under_it() {
    let mut harness = numbered(30, "click.rs", WIDE);
    click(&mut harness, Pos2::new(300.0, 100.0));
    let upper = selected(&harness).unwrap();
    click(&mut harness, Pos2::new(700.0, 200.0));
    let lower = selected(&harness).unwrap();
    assert!(lower > upper, "{upper} then {lower}");
}

#[test]
fn a_click_on_a_token_with_a_card_still_selects() {
    let mut harness = numbered(3, "click_token.rs", WIDE);
    let pos = hover_at_card(&mut harness, 450..900, "fn  →  function").unwrap();
    click(&mut harness, pos);
    assert!(selected(&harness).is_some());
}

#[test]
fn arrows_move_the_selection_within_the_document() {
    let mut harness = numbered(5, "arrows.rs", WIDE);
    press(&mut harness, Key::ArrowDown, 1);
    assert_eq!(
        selected(&harness),
        Some(0),
        "nothing selected: the first row in view"
    );
    press(&mut harness, Key::ArrowDown, 2);
    assert_eq!(selected(&harness), Some(2));
    press(&mut harness, Key::ArrowUp, 1);
    assert_eq!(selected(&harness), Some(1));
    press(&mut harness, Key::ArrowUp, 3);
    assert_eq!(selected(&harness), Some(0));
    press(&mut harness, Key::ArrowDown, 9);
    assert_eq!(selected(&harness), Some(4));
}

#[test]
fn the_selection_scrolls_into_view() {
    let mut harness = numbered(200, "follow.rs", WIDE);
    press(&mut harness, Key::ArrowDown, 60);
    assert_eq!(selected(&harness), Some(59));
    assert!(harness.state().view.in_view.contains(&59));
    assert!(!harness.state().view.in_view.contains(&0));
}

#[test]
fn page_keys_scroll_by_the_view() {
    let mut harness = numbered(200, "pages.rs", WIDE);
    let first = harness.state().view.in_view.clone();
    press(&mut harness, Key::PageDown, 1);
    let next = harness.state().view.in_view.clone();
    assert!(next.start >= first.end - 2, "{first:?} then {next:?}");
    press(&mut harness, Key::PageUp, 1);
    assert_eq!(harness.state().view.in_view, first);
    assert_eq!(selected(&harness), None);
}

#[test]
fn an_empty_file_has_nothing_to_select() {
    let mut harness = opened("", "empty.rs", WIDE);
    press(&mut harness, Key::ArrowDown, 1);
    assert_eq!(selected(&harness), None);
}

#[test]
fn opening_a_file_clears_the_selection() {
    let mut harness = numbered(5, "before.rs", WIDE);
    press(&mut harness, Key::ArrowDown, 2);
    let dropped = temp_file("after.rs", "fn f() {}\n");
    harness.input_mut().dropped_files = vec![Arc::new(Dropped(dropped))];
    harness.run();
    assert_eq!(selected(&harness), None);
}

#[test]
fn one_pane_takes_the_whole_width() {
    let columns = Columns::new(1000.0, 8.0, 250, 1);
    assert_eq!(columns.text_start(Side::Source), 24.0 + GAP);
    assert_eq!(columns.width(), 1000.0);
    assert_eq!(
        columns.text_start(Side::Source) + columns.text_width,
        1000.0
    );
}

#[test]
fn the_transcription_hides_and_shows_again() {
    let mut harness = numbered(3, "toggle.rs", WIDE);
    assert!(harness.state().show_transcription);
    harness.get_by_label("Transcription").click();
    harness.run();
    assert!(!harness.state().show_transcription);
    // The source's tokens still show their cards.
    assert!(hover_shows(&mut harness, 0..450, "fn  →  function"));

    harness.get_by_label("Transcription").click();
    harness.run();
    assert!(harness.state().show_transcription);
}

#[test]
fn hiding_the_transcription_measures_rows_by_the_source_alone() {
    // A window so narrow that one pane or two, each text is its narrowest, so the
    // width alone does not say the rows need measuring again. The source fits that
    // width and its transcription wraps, so the rows of both sides are taller.
    let source = "fn f(a: &mut u8) {}\n";
    let mut harness = opened(source, "heights.rs", Vec2::new(150.0, 300.0));
    let both = harness.state().view.rows.total();
    harness.get_by_label("Transcription").click();
    harness.run();
    let source_only = harness.state().view.rows.total();
    assert!(source_only < both, "{source_only} is not below {both}");
}

#[test]
fn the_selected_row_is_painted_with_a_band() {
    use eframe::egui::Shape;
    let mut harness = numbered(3, "band.rs", WIDE);
    let bands = |harness: &Harness<'_, App>| {
        let band = harness
            .ctx
            .global_style()
            .visuals
            .selection
            .bg_fill
            .gamma_multiply(SELECTION_ALPHA);
        harness
            .output()
            .shapes
            .iter()
            .filter(|clipped| matches!(&clipped.shape, Shape::Rect(rect) if rect.fill == band))
            .count()
    };
    assert_eq!(bands(&harness), 0);
    press(&mut harness, Key::ArrowDown, 1);
    assert_eq!(bands(&harness), 1);
}
