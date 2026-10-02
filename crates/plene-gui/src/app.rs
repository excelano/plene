//! The window: a toolbar, then the source and its transcription in two columns, one
//! row per source line.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::mem;
use std::ops::Range;
use std::path::{Path, PathBuf};

use eframe::egui::{
    Align2, CentralPanel, FontId, Key, KeyboardShortcut, Modifiers, OpenUrl, Panel, Rect, RichText,
    ScrollArea, Sense, TextEdit, Ui, Vec2, ViewportCommand,
};
use plene_core::{Category, Edition, Glossary, Role, Span};

use crate::document::Document;
use crate::related::Related;
use crate::rows::Rows;
use crate::search::{Scope, Search};
use crate::text::{FONT_SIZE, Side, combine, layout_job, span_at, span_index_at};

const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
/// What every reference link in the built-in glossary starts with, left off the hover card.
const REFERENCE: &str = "https://doc.rust-lang.org/reference/";
const FIND: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);
/// Space between a pane's line numbers and its text, and between the two panes.
const GAP: f32 = 16.0;
/// Space below each row, so that wrapped rows stay apart.
const ROW_SPACING: f32 = 2.0;
/// The narrowest a column gets, in characters, however narrow the window.
const MIN_COLUMN_CHARS: f32 = 20.0;
/// How strongly the selected row's band shows the theme's selection color, so that
/// the text on it stays readable.
const SELECTION_ALPHA: f32 = 0.35;

pub struct App {
    edition: Edition,
    glossary: Glossary,
    /// What went wrong loading the glossary: shown for the whole session.
    glossary_problems: Vec<String>,
    /// Why the last file could not be opened, until one is.
    open_error: Option<String>,
    document: Option<Document>,
    view: View,
    /// Whether the transcription's pane shows beside the source's. It does at start,
    /// and the choice holds across the files opened.
    show_transcription: bool,
    /// The categories the reader has switched off: left as written in the
    /// transcription. All are on at start, and the choice holds across the files opened.
    kept: Vec<Category>,
    search: Search,
    /// The tokens related to the one last clicked.
    related: Related,
    /// Asks the reader for a file. The native dialog in the window; tests put in a
    /// stand-in, since a dialog needs a display and waits for a person.
    picker: Box<dyn Fn() -> Option<PathBuf>>,
    title: String,
}

/// Where the reader is in the open document: the rows' geometry, the selected row, and
/// the rows last in view.
#[derive(Default)]
struct View {
    rows: Rows,
    selected: Option<usize>,
    in_view: Range<usize>,
    /// The selected row is to be scrolled into view on the next frame.
    reveal: bool,
}

impl App {
    /// `glossary` is what `Glossary::load` returned. A glossary that failed to load is
    /// reported in the window, and the built-in glossary is used in its place.
    pub fn new(
        edition: Edition,
        glossary: Result<(Glossary, Vec<String>), String>,
        file: Option<&Path>,
    ) -> App {
        let (glossary, glossary_problems) = match glossary {
            Ok((glossary, warnings)) => (
                glossary,
                warnings
                    .into_iter()
                    .map(|warning| format!("warning: {warning}"))
                    .collect(),
            ),
            Err(error) => (
                Glossary::default(),
                vec![format!("{error}; using the built-in glossary")],
            ),
        };
        let mut app = App {
            edition,
            glossary,
            glossary_problems,
            open_error: None,
            document: None,
            view: View::default(),
            show_transcription: true,
            kept: Vec::new(),
            search: Search::default(),
            related: Related::default(),
            picker: Box::new(native_picker),
            title: String::new(),
        };
        if let Some(file) = file {
            app.open(file);
        }
        app
    }

    fn open(&mut self, path: &Path) {
        match Document::open(path, self.edition, &self.glossary.without(&self.kept)) {
            Ok(document) => {
                self.document = Some(document);
                self.view = View::default();
                self.search.mark_stale();
                self.related.clear();
                self.open_error = None;
            }
            Err(error) => self.open_error = Some(error),
        }
    }

    fn pick_file(&mut self) {
        if let Some(path) = (self.picker)() {
            self.open(&path);
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let dropped = ui.input(|input| {
            input
                .raw
                .dropped_files
                .first()
                .map(|file| file.path().to_path_buf())
        });
        if let Some(path) = dropped {
            self.open(&path);
        }
        if ui.input_mut(|input| input.consume_shortcut(&OPEN)) {
            self.pick_file();
        }
        if ui.input_mut(|input| input.consume_shortcut(&FIND)) {
            self.search.open = true;
            self.search.focus = true;
        }
        if self.search.open && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape))
        {
            self.search.open = false;
        }
        self.set_title(ui);
        Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        self.find_matches(false);
        if let Some(document) = &self.document {
            self.related.refresh(&document.lines);
        }
        CentralPanel::default().show(ui, |ui| {
            if let Some(document) = &self.document {
                let sides: &[Side] = if self.show_transcription {
                    &[Side::Source, Side::Transcription]
                } else {
                    &[Side::Source]
                };
                Self::show_rows(
                    ui,
                    document,
                    &mut self.view,
                    &self.glossary,
                    &self.search,
                    &mut self.related,
                    sides,
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a Rust file with the button above or Ctrl+O, or drop one here.");
                });
            }
        });
    }

    fn set_title(&mut self, ui: &Ui) {
        let title = match &self.document {
            Some(document) => format!("{} - plene", document.name),
            None => "plene".to_string(),
        };
        if title != self.title {
            ui.ctx()
                .send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Open…").clicked() {
                self.pick_file();
            }
            if ui
                .toggle_value(&mut self.show_transcription, "Transcription")
                .changed()
            {
                self.rows_changed();
            }
            self.expand_menu(ui);
            if let Some(document) = &self.document {
                ui.label(RichText::new(&document.name).strong());
            }
        });
        if self.search.open {
            self.search_bar(ui);
        }
        let error_color = ui.visuals().error_fg_color;
        for problem in self.glossary_problems.iter().chain(&self.open_error) {
            ui.label(RichText::new(problem).color(error_color));
        }
    }

    /// What the rows hold or how tall they are has changed: they are measured again,
    /// and the search has new text to look through.
    fn rows_changed(&mut self) {
        self.view.rows = Rows::default();
        self.search.mark_stale();
        self.related.mark_stale();
    }

    /// A checkbox for each category of expansion. The transcription is made again
    /// when one changes.
    fn expand_menu(&mut self, ui: &mut Ui) {
        ui.menu_button("Expand", |ui| {
            for &category in Category::ALL {
                let mut expanded = !self.kept.contains(&category);
                if ui.checkbox(&mut expanded, category.label()).changed() {
                    if expanded {
                        self.kept.retain(|kept| *kept != category);
                    } else {
                        self.kept.push(category);
                    }
                    let glossary = self.glossary.without(&self.kept);
                    if let Some(document) = &mut self.document {
                        document.transcribe(self.edition, &glossary);
                    }
                    self.rows_changed();
                }
            }
        });
    }

    /// The search field, where to search, the way round the matches, and how many there
    /// are.
    fn search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let field = ui.add(
                TextEdit::singleline(&mut self.search.query)
                    .hint_text("Search")
                    .desired_width(240.0),
            );
            if mem::take(&mut self.search.focus) {
                field.request_focus();
            }
            let mut changed = field.changed();
            if field.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) {
                let backward = ui.input(|input| input.modifiers.shift);
                self.step_search(!backward);
                field.request_focus();
            }
            for (scope, label) in [
                (Scope::Both, "Both"),
                (Scope::Source, "Source only"),
                (Scope::Transcription, "Transcription only"),
            ] {
                if ui
                    .selectable_value(&mut self.search.scope, scope, label)
                    .changed()
                {
                    changed = true;
                    if scope == Scope::Transcription && !self.show_transcription {
                        self.show_transcription = true;
                        self.rows_changed();
                    }
                }
            }
            if ui.button("Previous").clicked() {
                self.step_search(false);
            }
            if ui.button("Next").clicked() {
                self.step_search(true);
            }
            if changed {
                self.search.mark_stale();
                self.find_matches(true);
            }
            let count = self.search.count();
            if let Some(position) = self.search.position() {
                ui.label(format!("{position} of {count}"));
            } else if !self.search.query.is_empty() {
                ui.label("No matches");
            }
        });
    }

    /// Finds the matches again if something they depend on changed. With `jump`, the
    /// reader is taken to the first match from where they are.
    fn find_matches(&mut self, jump: bool) {
        let Some(document) = &self.document else {
            return;
        };
        if !self.search.open {
            return;
        }
        let from = self.view.selected.unwrap_or(self.view.in_view.start);
        self.search
            .refresh(&document.lines, self.show_transcription, from);
        if jump {
            self.go_to(self.search.current_row());
        }
    }

    fn step_search(&mut self, forward: bool) {
        let row = self.search.step(forward);
        self.go_to(row);
    }

    fn go_to(&mut self, row: Option<usize>) {
        if let Some(row) = row {
            self.view.selected = Some(row);
            self.view.reveal = true;
        }
    }

    /// The rows in view, each with a pane for each of `sides`, and each pane with the
    /// line number. A click selects a row; the arrow keys move the selection and Page Up
    /// and Page Down scroll by the height of the view.
    fn show_rows(
        ui: &mut Ui,
        document: &Document,
        view: &mut View,
        glossary: &Glossary,
        search: &Search,
        related: &mut Related,
        sides: &[Side],
    ) {
        let lines = &document.lines;
        let font = FontId::monospace(FONT_SIZE);
        let (char_width, row_height) =
            ui.fonts_mut(|fonts| (fonts.glyph_width(&font, '0'), fonts.row_height(&font)));
        let columns = Columns::new(ui.available_width(), char_width, lines.len(), sides.len());
        let column = columns.text_width;
        let visuals = ui.visuals().clone();
        let typing = ui.ctx().egui_wants_keyboard_input();
        let [up, down, page_up, page_down] =
            [Key::ArrowUp, Key::ArrowDown, Key::PageUp, Key::PageDown].map(|key| {
                !typing && ui.input_mut(|input| input.consume_key(Modifiers::NONE, key))
            });
        let moved = (up || down) && !lines.is_empty();
        let reveal = mem::take(&mut view.reveal) || moved;
        if moved {
            let last = lines.len() - 1;
            view.selected = Some(match view.selected {
                None => view.in_view.start.min(last),
                Some(row) if down => (row + 1).min(last),
                Some(row) => row.saturating_sub(1),
            });
        }
        view.rows.measure(column, lines.len(), |row| {
            sides
                .iter()
                .map(|&side| {
                    let job = layout_job(&lines[row], side, column, &visuals, &[]);
                    ui.fonts_mut(|fonts| fonts.layout_job(job)).size().y
                })
                .fold(row_height, f32::max)
                + ROW_SPACING
        });
        ScrollArea::vertical()
            .auto_shrink(false)
            .show_viewport(ui, |ui, viewport| {
                let rows = &view.rows;
                let (area, response) = ui
                    .allocate_exact_size(Vec2::new(columns.width(), rows.total()), Sense::click());
                let click = response
                    .clicked()
                    .then(|| response.interact_pointer_pos())
                    .flatten();
                if let Some(pointer) = click {
                    view.selected = rows.row_at(pointer.y - area.min.y);
                }
                let mut clicked_span = None;
                let row_rect = |row: usize| {
                    Rect::from_min_max(
                        area.min + Vec2::new(0.0, rows.top(row)),
                        area.min + Vec2::new(columns.width(), rows.bottom(row)),
                    )
                };
                if reveal && let Some(row) = view.selected {
                    ui.scroll_to_rect(row_rect(row), None);
                }
                for (pressed, direction) in [(page_up, 1.0), (page_down, -1.0)] {
                    if pressed {
                        ui.scroll_with_delta(Vec2::new(0.0, direction * viewport.height()));
                    }
                }
                view.in_view = rows.visible(viewport.min.y, viewport.max.y);
                for row in view.in_view.clone() {
                    let top = area.min + Vec2::new(0.0, rows.top(row));
                    if view.selected == Some(row) {
                        let band = visuals.selection.bg_fill.gamma_multiply(SELECTION_ALPHA);
                        ui.painter().rect_filled(row_rect(row), 0.0, band);
                    }
                    for &side in sides {
                        ui.painter().text(
                            top + Vec2::new(columns.number_end(side), 0.0),
                            Align2::RIGHT_TOP,
                            row + 1,
                            font.clone(),
                            visuals.weak_text_color(),
                        );
                        let marks =
                            combine(search.marks(row, side), related.marks(lines, row, side));
                        let job = layout_job(&lines[row], side, column, &visuals, &marks);
                        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                        let rect = Rect::from_min_size(
                            top + Vec2::new(columns.text_start(side), 0.0),
                            galley.size(),
                        );
                        if let Some(pointer) = click
                            && rect.contains(pointer)
                        {
                            clicked_span =
                                span_index_at(&galley, &lines[row], side, pointer - rect.min)
                                    .map(|span| (row, span));
                        }
                        if let Some(pointer) = ui.ctx().pointer_hover_pos()
                            && rect.contains(pointer)
                            && let Some(span) =
                                span_at(&galley, &lines[row], side, pointer - rect.min)
                            && let Some(role) = span.role
                        {
                            ui.interact(rect, ui.id().with((row, side)), Sense::hover())
                                .on_hover_ui_at_pointer(|ui| hover(ui, span, role, glossary));
                        }
                        ui.painter().galley(rect.min, galley, visuals.text_color());
                    }
                }
                if click.is_some() {
                    match clicked_span {
                        Some((row, span)) => related.select(lines, row, span),
                        None => related.clear(),
                    }
                    if ui.input(|input| input.modifiers.command)
                        && let Some((row, span)) = clicked_span
                        && let Some(url) = reference_url(&lines[row].spans[span], glossary)
                    {
                        ui.ctx().open_url(OpenUrl::new_tab(url));
                    }
                }
            });
    }
}

/// The native file dialog. It needs a display, so this runs by hand, never in a test.
fn native_picker() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Rust", &["rs"])
        .pick_file()
}

/// Where each pane sits across a row. A pane is its line numbers, a gap and its text;
/// the source's pane comes first, then a gap, then the transcription's.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Columns {
    number_width: f32,
    text_width: f32,
    panes: usize,
}

impl Columns {
    /// `panes` panes that share `available` width, each text no narrower than
    /// `MIN_COLUMN_CHARS`, with numbers wide enough for `line_count`.
    fn new(available: f32, char_width: f32, line_count: usize, panes: usize) -> Columns {
        let number_width = char_width * line_count.to_string().len() as f32;
        let count = panes as f32;
        let text_width = ((available - count * number_width - (2.0 * count - 1.0) * GAP) / count)
            .max(char_width * MIN_COLUMN_CHARS);
        Columns {
            number_width,
            text_width,
            panes,
        }
    }

    fn pane_start(self, side: Side) -> f32 {
        match side {
            Side::Source => 0.0,
            Side::Transcription => self.number_width + GAP + self.text_width + GAP,
        }
    }

    /// The right edge of a pane's line numbers, which are right-aligned.
    fn number_end(self, side: Side) -> f32 {
        self.pane_start(side) + self.number_width
    }

    fn text_start(self, side: Side) -> f32 {
        self.number_end(side) + GAP
    }

    fn width(self) -> f32 {
        let pane = self.number_width + GAP + self.text_width;
        self.panes as f32 * (pane + GAP) - GAP
    }
}

/// Where the Rust reference covers `span`'s token, from its glossary entry.
fn reference_url<'a>(span: &Span, glossary: &'a Glossary) -> Option<&'a str> {
    glossary
        .entry(&span.original, span.role?)
        .and_then(|entry| entry.url.as_deref())
}

/// The hover card for a token with a glossary role: the token and its expansion, the
/// role, the glossary's note, and the page of the Rust reference that Ctrl+click opens.
fn hover(ui: &mut Ui, span: &Span, role: Role, glossary: &Glossary) {
    ui.label(RichText::new(format!("{}  →  {}", span.original, span.rendered)).monospace());
    ui.label(RichText::new(role.as_str()).weak());
    if let Some(note) = glossary
        .entry(&span.original, role)
        .and_then(|entry| entry.note.as_deref())
    {
        ui.label(note);
    }
    if let Some(url) = reference_url(span, glossary) {
        let key = ui.ctx().format_modifiers(Modifiers::COMMAND);
        let page = url
            .strip_prefix(REFERENCE)
            .unwrap_or(url)
            .replacen(".html", "", 1);
        ui.label(RichText::new(format!("{key}+click: Rust reference, {page}")).weak());
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
mod tests;
