//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::accesskit::Role as NodeRole;
use eframe::egui::{DroppedFile, Key, Modifiers, Pos2, Vec2};
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

/// Stands where the native file dialog would, which a test must never raise: it needs a
/// display and waits for a person to close it.
fn refuse_dialog() -> Option<PathBuf> {
    panic!("a test raised the native file dialog");
}

fn app_with(file: Option<&Path>) -> App {
    let mut app = App::new(
        Edition::default(),
        Ok((Glossary::default(), Vec::new())),
        file,
    );
    app.picker = Box::new(refuse_dialog);
    app
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
    let mut failed = App::new(
        Edition::default(),
        Err("g.toml: bad".to_string()),
        Some(&file),
    );
    failed.picker = Box::new(refuse_dialog);
    let harness = window(failed, WIDE);
    harness.get_by_label("g.toml: bad; using the built-in glossary");
    let document = harness.state().document.as_ref().unwrap();
    assert_eq!(document.lines[0].spans[0].rendered, "function");

    let mut warned = App::new(
        Edition::default(),
        Ok((
            Glossary::default(),
            vec!["g.toml: unknown role".to_string()],
        )),
        None,
    );
    warned.picker = Box::new(refuse_dialog);
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

/// Opens the search bar with Ctrl+F and types `query` into its field.
fn searching(source: &str, name: &str, query: &str) -> Harness<'static, App> {
    let mut harness = opened(source, name, WIDE);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness.get_by_role(NodeRole::TextInput).type_text(query);
    harness.run();
    harness
}

fn search_position(harness: &Harness<'_, App>) -> Option<usize> {
    harness.state().search.position()
}

/// Two `&mut` rows and two plain ones.
const MUTS: &str = "fn a(x: &mut u8) {}\nfn b() {}\nfn c(y: &mut u8) {}\nfn d() {}\n";

#[test]
fn ctrl_f_opens_the_search_and_escape_closes_it() {
    let mut harness = opened(MUTS, "find.rs", WIDE);
    assert!(harness.query_by_role(NodeRole::TextInput).is_none());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    assert!(harness.state().search.open);
    harness.get_by_role(NodeRole::TextInput);
    harness.key_press(Key::Escape);
    harness.run();
    assert!(!harness.state().search.open);
    assert!(harness.query_by_role(NodeRole::TextInput).is_none());
}

#[test]
fn typing_goes_to_the_first_match_and_enter_steps_round() {
    let mut harness = searching(MUTS, "steps.rs", "borrowed mutable");
    harness.get_by_label("1 of 2");
    assert_eq!(selected(&harness), Some(0));
    press(&mut harness, Key::Enter, 1);
    harness.get_by_label("2 of 2");
    assert_eq!(selected(&harness), Some(2));
    press(&mut harness, Key::Enter, 1);
    harness.get_by_label("1 of 2");
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Enter);
    harness.run();
    harness.get_by_label("2 of 2");
    assert_eq!(selected(&harness), Some(2));
}

#[test]
fn the_buttons_step_too() {
    let mut harness = searching(MUTS, "buttons.rs", "&mut");
    harness.get_by_label("Next").click();
    harness.run();
    assert_eq!(selected(&harness), Some(2));
    harness.get_by_label("Previous").click();
    harness.run();
    assert_eq!(selected(&harness), Some(0));
}

#[test]
fn a_query_without_matches_says_so_and_selects_nothing() {
    let harness = searching(MUTS, "none.rs", "no such text");
    harness.get_by_label("No matches");
    assert_eq!(selected(&harness), None);
    assert_eq!(search_position(&harness), None);
}

#[test]
fn the_scope_picks_the_pane_searched() {
    let mut harness = searching(MUTS, "scope.rs", "&mut");
    harness.get_by_label("1 of 2");
    harness.get_by_label("Transcription only").click();
    harness.run();
    harness.get_by_label("No matches");
    harness.get_by_label("Source only").click();
    harness.run();
    harness.get_by_label("1 of 2");
    harness.get_by_label("Both").click();
    harness.run();
    harness.get_by_label("1 of 2");
}

#[test]
fn searching_the_transcription_shows_a_hidden_pane() {
    let mut harness = searching(MUTS, "hidden.rs", "borrowed");
    harness.get_by_label("Transcription").click();
    harness.run();
    assert!(!harness.state().show_transcription);
    harness.get_by_label("No matches");
    harness.get_by_label("Transcription only").click();
    harness.run();
    assert!(harness.state().show_transcription);
    harness.get_by_label("1 of 2");
}

#[test]
fn a_match_far_down_is_scrolled_to() {
    let mut source: String = (0..200).map(|n| format!("fn f{n}() {{}}\n")).collect();
    source.push_str("fn last(x: &mut u8) {}\n");
    let harness = searching(&source, "far.rs", "borrowed mutable");
    assert_eq!(selected(&harness), Some(200));
    assert!(harness.state().view.in_view.contains(&200));
}

#[test]
fn arrow_keys_edit_the_field_and_leave_the_selection_alone() {
    let mut harness = searching(MUTS, "typing.rs", "&mut");
    assert_eq!(selected(&harness), Some(0));
    press(&mut harness, Key::ArrowDown, 2);
    assert_eq!(selected(&harness), Some(0));
}

#[test]
fn a_new_file_is_searched_for_the_same_query() {
    let mut harness = searching(MUTS, "before_find.rs", "&mut");
    harness.get_by_label("1 of 2");
    let dropped = temp_file(
        "after_find.rs",
        "fn a(x: &mut u8, y: &mut u8, z: &mut u8) {}\n",
    );
    harness.input_mut().dropped_files = vec![Arc::new(Dropped(dropped))];
    harness.run();
    harness.get_by_label("1 of 3");
}

#[test]
fn closing_the_search_ends_the_marks() {
    let mut harness = searching(MUTS, "closed.rs", "&mut");
    assert!(!harness.state().search.marks(0, Side::Source).is_empty());
    harness.key_press(Key::Escape);
    harness.run();
    assert!(harness.state().search.marks(0, Side::Source).is_empty());
}

/// The transcription of the first span of the first line whose source is `original`.
fn rendered(harness: &Harness<'_, App>, original: &str) -> String {
    harness.state().document.as_ref().unwrap().lines[0]
        .spans
        .iter()
        .find(|span| span.original == original)
        .unwrap()
        .rendered
        .clone()
}

/// Opens the Expand menu and clicks the checkbox for `label`.
fn toggle_expansion(harness: &mut Harness<'_, App>, label: &str) {
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label(label).click();
    harness.run();
}

const LIFETIME: &str = "fn f<'a>(x: &'a u8) {}\n";

#[test]
fn the_expand_menu_switches_a_category_off_and_on() {
    let mut harness = opened(LIFETIME, "expand.rs", WIDE);
    assert_eq!(rendered(&harness, "'a"), "lifetime a");
    toggle_expansion(&mut harness, "Lifetimes and labels");
    assert_eq!(harness.state().kept, [Category::Lifetimes]);
    assert_eq!(rendered(&harness, "'a"), "'a");
    assert_eq!(
        rendered(&harness, "fn"),
        "function",
        "other categories stay on"
    );
    assert_eq!(rendered(&harness, "&"), "borrowed");
    toggle_expansion(&mut harness, "Lifetimes and labels");
    assert!(harness.state().kept.is_empty());
    assert_eq!(rendered(&harness, "'a"), "lifetime a");
}

#[test]
fn a_category_left_as_written_shows_no_card() {
    let mut harness = opened("fn f() {}\n", "no_card.rs", WIDE);
    assert!(hover_shows(&mut harness, 450..900, "Declares a function."));
    toggle_expansion(&mut harness, "Keywords");
    assert!(!hover_shows(&mut harness, 0..900, "Declares a function."));
}

#[test]
fn switching_a_category_keeps_the_selection_and_searches_the_new_text() {
    let mut harness = opened(LIFETIME, "keeps.rs", WIDE);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run();
    harness
        .get_by_role(NodeRole::TextInput)
        .type_text("lifetime a");
    harness.run();
    harness.get_by_label("1 of 2");
    assert_eq!(selected(&harness), Some(0));
    toggle_expansion(&mut harness, "Lifetimes and labels");
    assert_eq!(selected(&harness), Some(0));
    harness.get_by_label("No matches");
}

#[test]
fn the_choice_holds_across_the_files_opened() {
    let mut harness = opened(LIFETIME, "first_choice.rs", WIDE);
    toggle_expansion(&mut harness, "Lifetimes and labels");
    let dropped = temp_file("second_choice.rs", LIFETIME);
    harness.input_mut().dropped_files = vec![Arc::new(Dropped(dropped))];
    harness.run();
    harness.get_by_label("second_choice.rs");
    assert_eq!(rendered(&harness, "'a"), "'a");
    assert_eq!(rendered(&harness, "fn"), "function");
}

#[test]
fn every_category_has_a_checkbox() {
    let mut harness = opened(LIFETIME, "menu.rs", WIDE);
    harness.get_by_label("Expand").click();
    harness.run();
    for category in Category::ALL {
        harness.get_by_label(category.label());
    }
}

/// Where, just inside the top of `row`, a click falls: found by clicking down the
/// line numbers until that row is selected. The search starts well clear of the
/// toolbar, whose Open button would raise the native file dialog.
fn top_of_row(harness: &mut Harness<'_, App>, row: usize) -> f32 {
    let clear_of_toolbar = harness.get_by_label("Open…").rect().max.y.ceil() as usize + 16;
    for y in (clear_of_toolbar..300).step_by(3) {
        click(harness, Pos2::new(14.0, y as f32));
        if selected(harness) == Some(row) {
            return y as f32 + 1.0;
        }
    }
    panic!("row {row} is not in view");
}

/// Clicks along `row` over `xs`, a character cell wide at a time or so, until `count`
/// tokens are related to the one clicked. A click is the pointer's press and release
/// in one frame, which keeps the search quick.
fn click_until_related(
    harness: &mut Harness<'_, App>,
    row: usize,
    xs: std::ops::Range<usize>,
    count: usize,
) -> bool {
    let y = top_of_row(harness, row);
    xs.step_by(4).any(|x| {
        let pos = Pos2::new(x as f32, y);
        harness.hover_at(pos);
        harness.drag_at(pos);
        harness.drop_at(pos);
        harness.run();
        harness.state().related.count() == count
    })
}

const NAMES: &str = "fn f(alpha: u8) -> u8 {\n    alpha + alpha\n}\n";

#[test]
fn clicking_a_name_marks_its_namesakes_on_either_pane() {
    for xs in [24..200, 450..700] {
        let mut harness = opened(NAMES, "namesakes.rs", WIDE);
        assert!(
            click_until_related(&mut harness, 0, xs.clone(), 3),
            "{xs:?}"
        );
        assert!(selected(&harness).is_some(), "the row is selected too");
    }
}

#[test]
fn clicking_a_bracket_marks_its_partner() {
    let mut harness = opened("fn f() {\n    g(1);\n}\n", "bracket.rs", WIDE);
    assert!(click_until_related(&mut harness, 1, 24..200, 2));
}

#[test]
fn clicking_away_clears_what_was_marked() {
    let mut harness = opened(NAMES, "away.rs", WIDE);
    assert!(click_until_related(&mut harness, 0, 24..200, 3));
    let y = top_of_row(&mut harness, 0);
    click(&mut harness, Pos2::new(380.0, y));
    assert_eq!(harness.state().related.count(), 0);
}

/// The text of row 0 on `side` that is marked as related to the token clicked.
fn marked_text(harness: &Harness<'_, App>, side: Side) -> Vec<String> {
    let app = harness.state();
    let lines = &app.document.as_ref().unwrap().lines;
    let text: String = lines[0].spans.iter().map(|span| side.text(span)).collect();
    app.related
        .marks(lines, 0, side)
        .into_iter()
        .map(|mark| text[mark.range].to_string())
        .collect()
}

#[test]
fn what_was_marked_survives_a_category_being_switched() {
    let mut harness = opened(LIFETIME, "survives.rs", WIDE);
    assert!(click_until_related(&mut harness, 0, 24..200, 2));
    assert_eq!(marked_text(&harness, Side::Source), ["'a", "'a"]);
    assert_eq!(
        marked_text(&harness, Side::Transcription),
        ["lifetime a", "lifetime a"]
    );
    toggle_expansion(&mut harness, "Lifetimes and labels");
    assert_eq!(rendered(&harness, "'a"), "'a");
    assert_eq!(marked_text(&harness, Side::Source), ["'a", "'a"]);
    assert_eq!(marked_text(&harness, Side::Transcription), ["'a", "'a"]);
    toggle_expansion(&mut harness, "Lifetimes and labels");
    assert_eq!(
        marked_text(&harness, Side::Transcription),
        ["lifetime a", "lifetime a"]
    );
}

#[test]
fn opening_a_file_clears_what_was_marked() {
    let mut harness = opened(NAMES, "before_related.rs", WIDE);
    assert!(click_until_related(&mut harness, 0, 24..200, 3));
    let dropped = temp_file("after_related.rs", NAMES);
    harness.input_mut().dropped_files = vec![Arc::new(Dropped(dropped))];
    harness.run();
    assert_eq!(harness.state().related.count(), 0);
}

/// A window whose file dialog answers with `answer`, and a count of how often it was asked.
fn picking(
    answer: Option<PathBuf>,
) -> (Harness<'static, App>, Arc<std::sync::atomic::AtomicUsize>) {
    let asked = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut app = app_with(None);
    let counter = Arc::clone(&asked);
    app.picker = Box::new(move || {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        answer.clone()
    });
    (window(app, WIDE), asked)
}

fn times_asked(asked: &Arc<std::sync::atomic::AtomicUsize>) -> usize {
    asked.load(std::sync::atomic::Ordering::SeqCst)
}

#[test]
fn the_open_button_asks_for_a_file_and_opens_the_answer() {
    let (mut harness, asked) = picking(Some(temp_file("picked.rs", "fn f() {}\n")));
    harness.get_by_label("Open…").click();
    harness.run();
    assert_eq!(times_asked(&asked), 1);
    harness.get_by_label("picked.rs");
    assert!(harness.state().document.is_some());
}

#[test]
fn ctrl_o_asks_for_a_file_too() {
    let (mut harness, asked) = picking(Some(temp_file("picked_keys.rs", "fn f() {}\n")));
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    harness.run();
    assert_eq!(times_asked(&asked), 1);
    assert_eq!(harness.state().title, "picked_keys.rs - plene");
}

#[test]
fn a_cancelled_dialog_changes_nothing() {
    let (mut harness, asked) = picking(None);
    harness.get_by_label("Open…").click();
    harness.run();
    assert_eq!(times_asked(&asked), 1);
    assert!(harness.state().document.is_none());
    harness.get_by_label_contains("Open a Rust file");
}

#[test]
fn marks_stay_on_their_tokens_when_a_switch_removes_a_space_before_them() {
    // `&x` is `borrow x` with a space inserted between; with references left as written
    // there is no space, so the marked `x`s sit one span earlier.
    let mut harness = opened("fn f() { g(&x, x, x); }\n", "spaces.rs", WIDE);
    assert!(click_until_related(&mut harness, 0, 24..260, 3));
    assert_eq!(marked_text(&harness, Side::Transcription), ["x", "x", "x"]);
    toggle_expansion(&mut harness, "References and pointers");
    assert_eq!(marked_text(&harness, Side::Source), ["x", "x", "x"]);
    assert_eq!(marked_text(&harness, Side::Transcription), ["x", "x", "x"]);
}

/// The URLs the window asked to open over the frames that follow a click at `pos`, made
/// with the command key down when `command` is set. The key stays down through the
/// click, as it does under a real hand, and the pointer events carry it too, since
/// egui takes the modifiers from the latest event.
fn click_and_collect_urls(harness: &mut Harness<'_, App>, pos: Pos2, command: bool) -> Vec<String> {
    let held = if command {
        Modifiers::COMMAND
    } else {
        Modifiers::NONE
    };
    let button = |pressed| eframe::egui::Event::PointerButton {
        pos,
        button: eframe::egui::PointerButton::Primary,
        pressed,
        modifiers: held,
    };
    let key = |held| eframe::egui::Event::ModifiersChanged(held);
    harness.hover_at(pos);
    harness.run();
    harness.input_mut().events.extend([key(held), button(true)]);
    harness.run();
    harness.input_mut().events.push(button(false));
    let mut urls = Vec::new();
    for _ in 0..3 {
        harness.step();
        urls.extend(
            harness
                .output()
                .platform_output
                .commands
                .iter()
                .filter_map(|command| match command {
                    eframe::egui::OutputCommand::OpenUrl(open) => Some(open.url.clone()),
                    _ => None,
                }),
        );
    }
    harness.input_mut().events.push(key(Modifiers::NONE));
    harness.run();
    urls
}

#[test]
fn the_hover_card_names_the_reference_page() {
    let mut harness = opened("fn f(x: &u8) {}\n", "reference_card.rs", WIDE);
    let key = harness.ctx.format_modifiers(Modifiers::COMMAND);
    assert!(hover_shows(
        &mut harness,
        450..900,
        &format!("{key}+click: Rust reference, items/functions")
    ));
    let mut harness = opened("fn f(x: &u8) {}\n", "reference_card_anchor.rs", WIDE);
    assert!(hover_shows(
        &mut harness,
        450..900,
        "Rust reference, types/pointer#shared-references-"
    ));
}

#[test]
fn command_click_opens_the_reference_page_of_a_token() {
    let mut harness = opened("fn f() {}\n", "reference_click.rs", WIDE);
    let pos = hover_at_card(&mut harness, 450..900, "fn  →  function").unwrap();
    let urls = click_and_collect_urls(&mut harness, pos, true);
    assert_eq!(
        urls,
        ["https://doc.rust-lang.org/reference/items/functions.html"]
    );
    assert!(selected(&harness).is_some(), "it still selects the row");
}

#[test]
fn a_plain_click_and_a_token_without_a_role_open_nothing() {
    let mut harness = opened("fn f() {}\n", "reference_plain.rs", WIDE);
    let pos = hover_at_card(&mut harness, 450..900, "fn  →  function").unwrap();
    assert!(click_and_collect_urls(&mut harness, pos, false).is_empty());

    let y = top_of_row(&mut harness, 0);
    let on_name = (24..200)
        .step_by(4)
        .map(|x| Pos2::new(x as f32, y))
        .find(|pos| {
            click(&mut harness, *pos);
            harness.state().related.count() == 1
        })
        .expect("a click lands on the name f");
    assert!(click_and_collect_urls(&mut harness, on_name, true).is_empty());
}
