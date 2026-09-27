//! Renders plene-gui screenshots, light and dark, from plene-gui's own modules:
//! `sample` opened, one row selected, and one hover card open.
//!
//!     plene-screenshots <sample.rs> <out-dir>
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

// plene-gui's modules, compiled here as they are there. rustfmt formats them in
// their own crate, and cannot follow `app`'s test module from here.
#[path = "../../../crates/plene-gui/src/app.rs"]
#[rustfmt::skip]
mod app;
#[path = "../../../crates/plene-gui/src/document.rs"]
#[rustfmt::skip]
mod document;
#[path = "../../../crates/plene-gui/src/rows.rs"]
#[rustfmt::skip]
mod rows;
#[path = "../../../crates/plene-gui/src/text.rs"]
#[rustfmt::skip]
mod text;

use std::path::Path;

use eframe::egui::{Key, Pos2, Theme, Vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use plene_core::{Edition, Glossary};

/// The window, in logical pixels; rendered at twice that.
const SIZE: Vec2 = Vec2::new(1500.0, 600.0);
/// Rows selected by pressing Down this many times from the top.
const SELECTED_ROW: usize = 3;
/// The hover card to open, by its first line.
const CARD: &str = "&  →  borrow";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, sample, out] = args.as_slice() else {
        eprintln!("usage: plene-screenshots <sample.rs> <out-dir>");
        std::process::exit(2);
    };
    for (theme, name) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
        let app = app::App::new(
            Edition::default(),
            Ok((Glossary::default(), Vec::new())),
            Some(Path::new(sample)),
        );
        let mut harness = Harness::builder()
            .with_size(SIZE)
            .with_pixels_per_point(2.0)
            .with_theme(theme)
            .wgpu()
            .build_ui_state(|ui, app: &mut app::App| app.show(ui), app);
        harness.run();
        for _ in 0..SELECTED_ROW {
            harness.key_press(Key::ArrowDown);
            harness.run();
        }
        let card = open_card(&mut harness);
        park_pointer(&mut harness, card);
        let path = Path::new(out).join(format!("plene-gui-{name}.png"));
        harness.render().unwrap().save(&path).unwrap();
        eprintln!("wrote {}", path.display());
    }
}

fn shows_card(harness: &mut Harness<'_, app::App>, pos: Pos2) -> bool {
    harness.hover_at(pos);
    harness.run();
    harness.query_by_label(CARD).is_some()
}

/// Sweeps the pointer over the transcription until `CARD` opens.
fn open_card(harness: &mut Harness<'_, app::App>) -> Pos2 {
    for y in (40..SIZE.y as usize).step_by(2) {
        for x in (SIZE.x as usize / 2..SIZE.x as usize).step_by(3) {
            let pos = Pos2::new(x as f32, y as f32);
            if shows_card(harness, pos) {
                return pos;
            }
        }
    }
    panic!("no hover card reads {CARD:?}");
}

/// Moves the pointer to the bottom right of the hovered word, so the arrow the
/// renderer draws for it covers the gap after the word rather than the word.
fn park_pointer(harness: &mut Harness<'_, app::App>, mut pos: Pos2) {
    while shows_card(harness, pos + Vec2::new(1.0, 0.0)) {
        pos.x += 1.0;
    }
    while shows_card(harness, pos + Vec2::new(0.0, 1.0)) {
        pos.y += 1.0;
    }
    assert!(shows_card(harness, pos - Vec2::new(2.0, 2.0)));
}
