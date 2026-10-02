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
use plene_core::{Category, Config, Edition, Glossary};

use app::{App, Settings};

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
    /// Leave these categories as written instead of expanding them, comma-separated:
    /// keywords, visibility, mutability, references, operators, ranges, patterns,
    /// lifetimes, bounds, flow, ends, macros. Replaces the config file's lists for this
    /// run, and the Expand menu's changes are not saved.
    #[arg(long, value_delimiter = ',', value_name = "CATEGORY")]
    keep: Vec<Category>,
    /// Expand the categories that start off, comma-separated: elision. Replaces the
    /// config file's lists for this run, like --keep.
    #[arg(long, value_delimiter = ',', value_name = "CATEGORY", value_parser = Category::from_opt_in_str)]
    expand: Vec<Category>,
    /// Expand every category, those that start off included, ignoring the config file
    /// for this run.
    #[arg(long, conflicts_with_all = ["keep", "expand"])]
    expand_all: bool,
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

/// The settings to start with: what the command line names, or else the config file's,
/// which the window then saves its changes to. A config that could not be read is
/// reported and not saved over.
fn settings(
    args: &Args,
    config: impl FnOnce() -> Result<(Config, Vec<String>), String>,
    config_path: Option<PathBuf>,
) -> Settings {
    if args.expand_all {
        return Settings {
            kept: Vec::new(),
            ..Settings::default()
        };
    }
    if !args.keep.is_empty() || !args.expand.is_empty() {
        return Settings {
            kept: Category::kept(&args.keep, &args.expand),
            ..Settings::default()
        };
    }
    match config() {
        Ok((config, warnings)) => Settings {
            kept: config.kept(),
            save_to: config_path,
            problems: warnings
                .into_iter()
                .map(|warning| format!("warning: {warning}"))
                .collect(),
        },
        Err(error) => Settings {
            problems: vec![format!("{error}; this session's settings are not saved")],
            ..Settings::default()
        },
    }
}

// Opening the window needs a display, so these lines run by hand, never in a test.
fn main() -> eframe::Result {
    let args = Args::parse();
    let app = App::new(
        args.edition.into(),
        Glossary::load(args.glossary.as_deref()),
        settings(&args, Config::load, Config::path()),
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

    fn parsed(args: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("plene-gui").chain(args.iter().copied())).unwrap()
    }

    fn saved_config() -> Result<(Config, Vec<String>), String> {
        Ok((
            Config {
                keep: vec![Category::Lifetimes],
                ..Config::default()
            },
            vec!["c.toml: skipping unknown category `x`".to_string()],
        ))
    }

    fn path() -> Option<PathBuf> {
        Some(PathBuf::from("/config/plene/config.toml"))
    }

    #[test]
    fn the_config_files_settings_are_the_start_and_where_changes_are_saved() {
        let settings = settings(&parsed(&[]), saved_config, path());
        assert_eq!(settings.kept, [Category::Lifetimes, Category::Elision]);
        assert_eq!(settings.save_to, path());
        assert_eq!(
            settings.problems,
            ["warning: c.toml: skipping unknown category `x`"]
        );
    }

    #[test]
    fn the_command_line_decides_for_the_run_and_nothing_is_saved() {
        let unread = || -> Result<(Config, Vec<String>), String> { panic!("the config was read") };
        let kept = settings(&parsed(&["--keep", "visibility,ends"]), unread, path());
        assert_eq!(
            kept.kept,
            [Category::Visibility, Category::Ends, Category::Elision]
        );
        assert_eq!(kept.save_to, None);
        assert!(kept.problems.is_empty());

        let all = settings(&parsed(&["--expand-all"]), unread, path());
        assert!(all.kept.is_empty());
        assert_eq!(all.save_to, None);
    }

    #[test]
    fn a_config_that_cannot_be_read_is_reported_and_not_saved_over() {
        let failed = || Err("c.toml: invalid config: bad".to_string());
        let settings = settings(&parsed(&[]), failed, path());
        assert_eq!(settings.kept, [Category::Elision], "the defaults");
        assert_eq!(settings.save_to, None);
        assert_eq!(
            settings.problems,
            ["c.toml: invalid config: bad; this session's settings are not saved"]
        );
    }

    #[test]
    fn keep_and_expand_all_exclude_each_other_and_keep_rejects_an_unknown_category() {
        assert!(Args::try_parse_from(["plene-gui", "--keep", "ends", "--expand-all"]).is_err());
        let error = Args::try_parse_from(["plene-gui", "--keep", "nope"])
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("unknown category `nope`"), "{error}");
    }

    #[test]
    fn expand_turns_on_what_starts_off_for_the_run_and_nothing_is_saved() {
        let unread = || -> Result<(Config, Vec<String>), String> { panic!("the config was read") };
        let expanded = settings(&parsed(&["--expand", "elision"]), unread, path());
        assert!(expanded.kept.is_empty());
        assert_eq!(expanded.save_to, None);
        let both = settings(
            &parsed(&["--keep", "ends", "--expand", "elision"]),
            unread,
            path(),
        );
        assert_eq!(both.kept, [Category::Ends]);
        let all = settings(&parsed(&["--expand-all"]), unread, path());
        assert!(all.kept.is_empty(), "elision included");
    }

    #[test]
    fn expand_takes_only_a_category_that_starts_off_and_not_with_expand_all() {
        let error = Args::try_parse_from(["plene-gui", "--expand", "lifetimes"])
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("already expands by default"), "{error}");
        assert!(
            Args::try_parse_from(["plene-gui", "--expand", "elision", "--expand-all"]).is_err()
        );
    }

    #[test]
    fn the_config_files_expand_list_is_the_start() {
        let config = || {
            Ok((
                Config {
                    expand: vec![Category::Elision],
                    ..Config::default()
                },
                Vec::new(),
            ))
        };
        let settings = settings(&parsed(&[]), config, path());
        assert!(settings.kept.is_empty());
        assert_eq!(settings.save_to, path());
    }
}
