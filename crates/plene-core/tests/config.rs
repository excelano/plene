//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::path::{Path, PathBuf};

use plene_core::{Category, Config};

fn parse(text: &str) -> (Config, Vec<String>) {
    Config::parse(text).unwrap()
}

/// A directory of its own under cargo's per-target temporary directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("config-tests")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn keep_lists_the_categories_left_as_written() {
    let (config, warnings) = parse("keep = [\"lifetimes\", \"visibility\"]\n");
    assert_eq!(config.keep, [Category::Lifetimes, Category::Visibility]);
    assert!(warnings.is_empty());
}

#[test]
fn an_empty_file_or_no_keep_keeps_nothing() {
    for text in ["", "keep = []\n"] {
        let (config, warnings) = parse(text);
        assert_eq!(config, Config::default(), "{text:?}");
        assert!(warnings.is_empty());
    }
}

#[test]
fn an_unknown_category_is_skipped_with_a_warning_and_a_repeat_counts_once() {
    let (config, warnings) = parse("keep = [\"ends\", \"nope\", \"ends\", \"macros\"]\n");
    assert_eq!(config.keep, [Category::Ends, Category::Macros]);
    assert_eq!(warnings.len(), 1);
    assert!(
        warnings[0].starts_with("skipping unknown category `nope`; one of: keywords, "),
        "{warnings:?}"
    );
}

#[test]
fn malformed_files_are_errors() {
    for (text, reason) in [
        ("keep = \"lifetimes\"\n", "invalid type: string"),
        ("keep = [1]\n", "invalid type: integer"),
        ("other = []\n", "unknown field `other`"),
        ("keep = [\n", "unclosed array"),
    ] {
        let error = Config::parse(text).unwrap_err().to_string();
        assert!(error.starts_with("invalid config: "), "{error}");
        assert!(error.contains(reason), "expected {reason:?} in {error}");
    }
}

#[test]
fn to_toml_writes_the_categories_in_their_usual_order_and_reads_back() {
    let config = Config {
        keep: vec![Category::Macros, Category::Keywords, Category::Lifetimes],
        ..Config::default()
    };
    let text = config.to_toml();
    assert_eq!(text, "keep = [\"keywords\", \"lifetimes\", \"macros\"]\n");
    let (read_back, warnings) = parse(&text);
    assert!(warnings.is_empty());
    assert_eq!(
        read_back.keep,
        [Category::Keywords, Category::Lifetimes, Category::Macros]
    );
    assert_eq!(Config::default().to_toml(), "keep = []\n");
}

#[test]
fn save_makes_the_directory_and_load_reads_the_file_back() {
    let path = scratch("save").join("plene").join("config.toml");
    let config = Config {
        keep: vec![Category::Visibility],
        ..Config::default()
    };
    config.save(&path).unwrap();
    assert_eq!(Config::load_from(&path).unwrap(), (config, Vec::new()));
    assert!(
        !path.with_extension("toml.tmp").exists(),
        "no staging file is left"
    );
}

#[test]
fn save_replaces_what_was_there() {
    let path = scratch("replace").join("config.toml");
    Config {
        keep: vec![Category::Ends],
        ..Config::default()
    }
    .save(&path)
    .unwrap();
    Config {
        keep: vec![Category::Flow],
        ..Config::default()
    }
    .save(&path)
    .unwrap();
    assert_eq!(Config::load_from(&path).unwrap().0.keep, [Category::Flow]);
    Config::default().save(&path).unwrap();
    assert!(Config::load_from(&path).unwrap().0.keep.is_empty());
}

#[test]
fn errors_name_the_file() {
    let dir = scratch("errors");
    std::fs::create_dir_all(&dir).unwrap();

    let missing = dir.join("missing.toml");
    let error = Config::load_from(&missing).unwrap_err();
    assert!(
        error.starts_with(&format!("{}: ", missing.display())),
        "{error}"
    );

    let bad = dir.join("bad.toml");
    std::fs::write(&bad, "other = 1\n").unwrap();
    let error = Config::load_from(&bad).unwrap_err();
    assert!(
        error.starts_with(&format!("{}: invalid config: ", bad.display())),
        "{error}"
    );

    let warned = dir.join("warned.toml");
    std::fs::write(&warned, "keep = [\"nope\"]\n").unwrap();
    let (config, warnings) = Config::load_from(&warned).unwrap();
    assert!(config.keep.is_empty());
    assert!(warnings[0].starts_with(&format!("{}: skipping ", warned.display())));

    // A file where the directory should be: neither the directory nor the file can be made.
    let blocker = dir.join("blocker");
    std::fs::write(&blocker, "").unwrap();
    let error = Config::default()
        .save(&blocker.join("config.toml"))
        .unwrap_err();
    assert!(error.contains("blocker"), "{error}");

    // A path with no parent has no directory to make, and nowhere to write either.
    assert!(Config::default().save(Path::new("")).is_err());
}

#[test]
fn the_config_lives_beside_the_glossary() {
    let path = Config::path().unwrap();
    assert_eq!(path.file_name().unwrap(), "config.toml");
    assert_eq!(path.parent().unwrap().file_name().unwrap(), "plene");
}

#[test]
fn expand_turns_on_a_category_that_starts_off() {
    let (config, warnings) = parse("expand = [\"elision\"]\n");
    assert_eq!(config.expand, [Category::Elision]);
    assert!(config.keep.is_empty());
    assert!(warnings.is_empty());
    assert!(config.kept().is_empty(), "elision is no longer kept");
    assert_eq!(Config::default().kept(), [Category::Elision]);
}

#[test]
fn expand_skips_a_category_that_already_expands_and_one_that_is_unknown() {
    let (config, warnings) =
        parse("expand = [\"lifetimes\", \"nope\", \"elision\", \"elision\"]\n");
    assert_eq!(config.expand, [Category::Elision]);
    assert_eq!(warnings.len(), 2);
    assert_eq!(
        warnings[0],
        "skipping `lifetimes` already expands by default; the categories that start off are: elision"
    );
    assert!(warnings[1].starts_with("skipping unknown category `nope`; one of: "));
}

#[test]
fn keeping_other_categories_does_not_turn_elision_on() {
    let (config, _) = parse("keep = [\"lifetimes\"]\n");
    assert_eq!(config.kept(), [Category::Lifetimes, Category::Elision]);
}

#[test]
fn expand_is_written_only_when_there_is_something_to_say_and_reads_back() {
    let config = Config {
        keep: vec![Category::Ends],
        expand: vec![Category::Elision],
    };
    let text = config.to_toml();
    assert_eq!(text, "keep = [\"ends\"]\nexpand = [\"elision\"]\n");
    assert_eq!(parse(&text).0, config);
    assert_eq!(Config::default().to_toml(), "keep = []\n");
}

#[test]
fn a_set_of_kept_categories_is_made_into_the_config_that_gives_it_back() {
    for kept in [
        vec![],
        vec![Category::Elision],
        vec![Category::Lifetimes, Category::Elision],
        vec![Category::Lifetimes, Category::Visibility],
        Category::ALL.to_vec(),
    ] {
        let config = Config::from_kept(&kept);
        assert!(config.keep.iter().all(|category| !category.opt_in()));
        assert!(config.expand.iter().all(|category| category.opt_in()));
        let mut sorted = kept.clone();
        sorted.sort();
        assert_eq!(config.kept(), sorted, "{kept:?}");
    }
    assert_eq!(Config::from_kept(&[]).expand, [Category::Elision]);
    assert!(Config::from_kept(&[Category::Elision]).expand.is_empty());
}
