//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use plene_core::{Edition, Glossary, Line, transcribe};

mod common;
use common::fixtures;

fn reassemble(lines: &[Line]) -> String {
    let mut out = String::new();
    for line in lines {
        for span in &line.spans {
            out.push_str(&span.original);
        }
        out.push_str(&line.ending);
    }
    out
}

/// Checks byte-for-byte reassembly, 1:1 line correspondence, span ranges, and that
/// no span carries a line ending.
fn assert_invariants(source: &str, label: &str) {
    let lines = transcribe(source, Edition::default(), &Glossary::default());
    assert_eq!(
        reassemble(&lines),
        source,
        "{label}: reassembly differs from source"
    );
    assert_eq!(
        lines.len(),
        source.lines().count(),
        "{label}: line count differs"
    );
    for (source_line, line) in source.lines().zip(&lines) {
        let text: String = line
            .spans
            .iter()
            .map(|span| span.original.as_str())
            .collect();
        assert_eq!(text, source_line, "{label}: line text differs");
        for span in &line.spans {
            assert_eq!(
                &source[span.source_range.clone()],
                span.original,
                "{label}: bad source_range"
            );
            assert!(
                !span.original.contains('\n'),
                "{label}: span contains a newline"
            );
            assert!(
                !span.rendered.contains('\n'),
                "{label}: rendering contains a newline"
            );
        }
    }
}

#[test]
fn fixtures_hold_invariants() {
    for (name, source) in fixtures() {
        assert_invariants(&source, &name);
    }
}

#[test]
fn fixtures_hold_invariants_with_crlf() {
    for (name, source) in fixtures() {
        assert_invariants(&source.replace('\n', "\r\n"), &format!("{name} (CRLF)"));
    }
}

#[test]
fn edge_cases_hold_invariants() {
    for source in [
        "",
        "\n",
        "\n\n",
        "fn f() {}",
        "fn f() {}\n",
        "\r\n",
        "fn f() {} // end",
        "a\rb\n",
    ] {
        assert_invariants(source, &format!("{source:?}"));
    }
}

#[test]
fn crlf_ending_moves_out_of_line_comment() {
    let lines = transcribe(
        "// comment\r\nfn f() {}\r\n",
        Edition::default(),
        &Glossary::default(),
    );
    assert_eq!(lines[0].ending, "\r\n");
    assert_eq!(lines[0].spans.last().unwrap().original, "// comment");
}

/// Runs over every `.rs` file under the directories in `PLENE_SMOKE_DIRS`
/// (colon-separated), defaulting to `~/plene-corpus` and the `rust-src` standard
/// library. `cargo test --release -- --ignored` to run.
#[test]
#[ignore]
fn smoke_corpus() {
    let dirs: Vec<PathBuf> = match std::env::var("PLENE_SMOKE_DIRS") {
        Ok(dirs) => std::env::split_paths(&dirs).collect(),
        Err(_) => default_smoke_dirs(),
    };
    let mut files = 0;
    for dir in &dirs {
        assert!(dir.is_dir(), "smoke directory not found: {}", dir.display());
        files += smoke_dir(dir);
    }
    assert!(files > 0, "no .rs files found in {dirs:?}");
    eprintln!("smoke-tested {files} files");
}

fn default_smoke_dirs() -> Vec<PathBuf> {
    let home = PathBuf::from(std::env::var("HOME").expect("HOME is set"));
    let mut dirs = vec![home.join("plene-corpus")];
    let sysroot = Command::new("rustc").args(["--print", "sysroot"]).output();
    if let Ok(output) = sysroot {
        let library = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
            .join("lib/rustlib/src/rust/library");
        if library.is_dir() {
            dirs.push(library);
        }
    }
    dirs
}

/// Skips symlinks, so linked directories aren't tested twice and can't loop.
fn smoke_dir(dir: &Path) -> usize {
    let mut files = 0;
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let path = entry.path();
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            files += smoke_dir(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && let Ok(source) = fs::read_to_string(&path)
        {
            assert_invariants(&source, &path.display().to_string());
            files += 1;
        }
    }
    files
}
