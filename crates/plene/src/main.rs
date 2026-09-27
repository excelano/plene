//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]

mod render;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::{AutoStream, ColorChoice};
use clap::{Parser, ValueEnum};
use plene_core::{Edition, Glossary, transcribe};
use render::{Layout, Styling, Theme, View};

/// Shows Rust source alongside an expanded transcription: the same code with
/// abbreviations and symbols written out in words.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Rust source file, or `-` to read stdin.
    #[arg(required_unless_present = "dump_glossary")]
    file: Option<PathBuf>,
    /// When to color output.
    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    color: ColorArg,
    /// Rust edition to parse with.
    #[arg(long, value_enum, default_value_t = EditionArg::E2021)]
    edition: EditionArg,
    /// Terminal background the colors are chosen for.
    #[arg(long, value_enum, default_value_t = Theme::Dark)]
    theme: Theme,
    /// Glossary file whose entries override the built-in glossary and the one in
    /// the config directory.
    #[arg(long, value_name = "PATH")]
    glossary: Option<PathBuf>,
    /// Show the source and its expansion in two columns.
    #[arg(long, conflicts_with = "expanded")]
    side_by_side: bool,
    /// Show only the expansion.
    #[arg(long)]
    expanded: bool,
    /// Show only lines the expansion changes, numbered with their source lines.
    #[arg(long)]
    changed_only: bool,
    /// Print the glossary in effect, as a glossary file, and exit.
    #[arg(long, conflicts_with_all = ["file", "side_by_side", "expanded", "changed_only"])]
    dump_glossary: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum ColorArg {
    Auto,
    Always,
    Never,
}

impl From<ColorArg> for ColorChoice {
    fn from(color: ColorArg) -> ColorChoice {
        match color {
            ColorArg::Auto => ColorChoice::Auto,
            ColorArg::Always => ColorChoice::Always,
            ColorArg::Never => ColorChoice::Never,
        }
    }
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

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("plene: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<(), String> {
    let glossary = load_glossary(config_glossary_path().as_deref(), args.glossary.as_deref())?;
    let mut stdout = AutoStream::new(io::stdout().lock(), args.color.into());
    let output = match &args.file {
        Some(file) => {
            let source = read_source(file)?;
            let lines = transcribe(&source, args.edition.into(), &glossary);
            let styling = match stdout.current_choice() {
                ColorChoice::Never => Styling::Plain,
                _ => Styling::Colored(args.theme),
            };
            let layout = if args.side_by_side {
                Layout::SideBySide
            } else if args.expanded {
                Layout::Expanded
            } else {
                Layout::Interleaved
            };
            render::draw(
                &lines,
                View {
                    layout,
                    changed_only: args.changed_only,
                    styling,
                },
            )
        }
        None => glossary.to_toml(),
    };
    match stdout
        .write_all(output.as_bytes())
        .and_then(|()| stdout.flush())
    {
        // The reader went away, as with `plene file.rs | head`; that isn't an error.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| format!("writing output: {error}")),
    }
}

/// The built-in glossary, overridden by the config directory's glossary when there
/// is one, and then by the file named with `--glossary`.
fn load_glossary(config: Option<&Path>, named: Option<&Path>) -> Result<Glossary, String> {
    let mut glossary = Glossary::default();
    for path in config.filter(|path| path.exists()).into_iter().chain(named) {
        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let (overrides, warnings) =
            Glossary::parse(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        for warning in warnings {
            eprintln!("plene: warning: {}: {warning}", path.display());
        }
        glossary.merge(overrides);
    }
    Ok(glossary)
}

/// `$XDG_CONFIG_HOME/plene/glossary.toml`, or `~/.config/plene/glossary.toml` when
/// `XDG_CONFIG_HOME` is unset or not absolute, which the XDG spec says to ignore.
fn config_glossary_path() -> Option<PathBuf> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::home_dir().map(|home| home.join(".config")))?;
    Some(config_home.join("plene").join("glossary.toml"))
}

fn read_source(path: &Path) -> Result<String, String> {
    if path == Path::new("-") {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .map_err(|error| format!("stdin: {error}"))?;
        Ok(source)
    } else {
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
    }
}
