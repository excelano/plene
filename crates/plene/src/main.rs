mod render;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anstream::{AutoStream, ColorChoice};
use clap::{Parser, ValueEnum};
use plene_core::{Edition, Glossary, transcribe};

/// Shows Rust source alongside an expanded transcription: the same code with
/// abbreviations and symbols written out in words.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Rust source file, or `-` to read stdin.
    file: PathBuf,
    /// When to color output.
    #[arg(long, value_enum, default_value_t = ColorArg::Auto)]
    color: ColorArg,
    /// Rust edition to parse with.
    #[arg(long, value_enum, default_value_t = EditionArg::E2021)]
    edition: EditionArg,
    /// Glossary file whose entries override the built-in glossary.
    #[arg(long, value_name = "PATH")]
    glossary: Option<PathBuf>,
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
    let glossary = load_glossary(args.glossary.as_deref())?;
    let source = read_source(&args.file)?;
    let lines = transcribe(&source, args.edition.into(), &glossary);
    let output = render::interleaved(&lines);

    let mut stdout = AutoStream::new(io::stdout().lock(), args.color.into());
    match stdout
        .write_all(output.as_bytes())
        .and_then(|()| stdout.flush())
    {
        // The reader went away, as with `plene file.rs | head`; that isn't an error.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result.map_err(|error| format!("writing output: {error}")),
    }
}

fn load_glossary(path: Option<&Path>) -> Result<Glossary, String> {
    let mut glossary = Glossary::default();
    if let Some(path) = path {
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
