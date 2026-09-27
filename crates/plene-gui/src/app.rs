//! The window: a toolbar, then the source and its transcription in two columns, one
//! row per source line.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::ops::Range;
use std::path::Path;

use eframe::egui::{
    Align2, CentralPanel, FontId, Key, KeyboardShortcut, Modifiers, Panel, Rect, RichText,
    ScrollArea, Sense, Ui, Vec2, ViewportCommand,
};
use plene_core::{Edition, Glossary, Role, Span};

use crate::document::Document;
use crate::rows::Rows;
use crate::text::{FONT_SIZE, Side, layout_job, span_at};

const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
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
    title: String,
}

/// Where the reader is in the open document: the rows' geometry, the selected row, and
/// the rows last in view.
#[derive(Default)]
struct View {
    rows: Rows,
    selected: Option<usize>,
    in_view: Range<usize>,
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
            title: String::new(),
        };
        if let Some(file) = file {
            app.open(file);
        }
        app
    }

    fn open(&mut self, path: &Path) {
        match Document::open(path, self.edition, &self.glossary) {
            Ok(document) => {
                self.document = Some(document);
                self.view = View::default();
                self.open_error = None;
            }
            Err(error) => self.open_error = Some(error),
        }
    }

    /// The native file dialog needs a display, so this runs by hand, never in a test.
    fn pick_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Rust", &["rs"])
            .pick_file()
        {
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
        self.set_title(ui);
        Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        CentralPanel::default().show(ui, |ui| {
            if let Some(document) = &self.document {
                let sides: &[Side] = if self.show_transcription {
                    &[Side::Source, Side::Transcription]
                } else {
                    &[Side::Source]
                };
                Self::show_rows(ui, document, &mut self.view, &self.glossary, sides);
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
            // Rows are as tall as their tallest side, so they are measured again.
            if ui
                .toggle_value(&mut self.show_transcription, "Transcription")
                .changed()
            {
                self.view.rows = Rows::default();
            }
            if let Some(document) = &self.document {
                ui.label(RichText::new(&document.name).strong());
            }
        });
        let error_color = ui.visuals().error_fg_color;
        for problem in self.glossary_problems.iter().chain(&self.open_error) {
            ui.label(RichText::new(problem).color(error_color));
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
        sides: &[Side],
    ) {
        let lines = &document.lines;
        let font = FontId::monospace(FONT_SIZE);
        let (char_width, row_height) =
            ui.fonts_mut(|fonts| (fonts.glyph_width(&font, '0'), fonts.row_height(&font)));
        let columns = Columns::new(ui.available_width(), char_width, lines.len(), sides.len());
        let column = columns.text_width;
        let visuals = ui.visuals().clone();
        let [up, down, page_up, page_down] =
            [Key::ArrowUp, Key::ArrowDown, Key::PageUp, Key::PageDown]
                .map(|key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key)));
        let moved = (up || down) && !lines.is_empty();
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
                    let job = layout_job(&lines[row], side, column, &visuals);
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
                if response.clicked()
                    && let Some(pointer) = response.interact_pointer_pos()
                {
                    view.selected = rows.row_at(pointer.y - area.min.y);
                }
                let row_rect = |row: usize| {
                    Rect::from_min_max(
                        area.min + Vec2::new(0.0, rows.top(row)),
                        area.min + Vec2::new(columns.width(), rows.bottom(row)),
                    )
                };
                if moved && let Some(row) = view.selected {
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
                        let job = layout_job(&lines[row], side, column, &visuals);
                        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                        let rect = Rect::from_min_size(
                            top + Vec2::new(columns.text_start(side), 0.0),
                            galley.size(),
                        );
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
            });
    }
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

/// The hover card for a token with a glossary role: the token and its expansion, the
/// role, and the glossary's note.
fn hover(ui: &mut Ui, span: &Span, role: Role, glossary: &Glossary) {
    ui.label(RichText::new(format!("{}  →  {}", span.original, span.rendered)).monospace());
    ui.label(RichText::new(role.as_str()).weak());
    if let Some(note) = glossary
        .entry(&span.original, role)
        .and_then(|entry| entry.note.as_deref())
    {
        ui.label(note);
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[cfg(test)]
mod tests;
