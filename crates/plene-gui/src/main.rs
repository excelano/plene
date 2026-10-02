//! A window showing Rust source beside its expanded transcription.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]

mod app;
mod document;
mod folding;
mod related;
mod rows;
mod search;
mod text;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eframe::egui::ViewportBuilder;
use plene_core::{Edition, Glossary};

use app::App;

/// Shows Rust source beside an expanded transcription: the same code with
/// abbreviations and symbols written out in words.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Rust source file to open.
    file: Option<PathBuf>,
    /// Rust edition to parse with.
    #[arg(long, value_enum, default_value_t = EditionArg::E2021)]
    edition: EditionArg,
    /// Glossary file whose entries override the built-in glossary and the one in
    /// the config directory.
    #[arg(long, value_name = "PATH")]
    glossary: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum EditionArg {
    #[value(name = "2015")]
    E2015,
    #[value(name = "2018")]
    E2018,
    #[value(name = "2021")]
    E2021,
    #[value(name = "2024")]
    E2024,
}

impl From<EditionArg> for Edition {
    fn from(edition: EditionArg) -> Edition {
        match edition {
            EditionArg::E2015 => Edition::E2015,
            EditionArg::E2018 => Edition::E2018,
            EditionArg::E2021 => Edition::E2021,
            EditionArg::E2024 => Edition::E2024,
        }
    }
}

// Opening the window needs a display, so these lines run by hand, never in a test.
fn main() -> eframe::Result {
    let args = Args::parse();
    let app = App::new(
        args.edition.into(),
        Glossary::load(args.glossary.as_deref()),
        args.file.as_deref(),
    );
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_app_id("plene-gui"),
        ..Default::default()
    };
    eframe::run_native("plene", options, Box::new(|_| Ok(Box::new(app))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editions_parse_by_year() {
        for (year, edition) in [
            ("2015", Edition::E2015),
            ("2018", Edition::E2018),
            ("2021", Edition::E2021),
            ("2024", Edition::E2024),
        ] {
            let args = Args::try_parse_from(["plene-gui", "--edition", year]).unwrap();
            assert_eq!(Edition::from(args.edition), edition);
        }
    }
}
