//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

/// Runs plene with its config directory pointed at one that never exists, so a
/// glossary on the machine running the tests cannot change what they see.
fn plene(args: &[&str], stdin: &str) -> Output {
    let config_home = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("no-config");
    plene_in(
        args,
        stdin,
        &[("XDG_CONFIG_HOME", config_home.to_str().unwrap())],
    )
}

fn plene_in(args: &[&str], stdin: &str, env: &[(&str, &str)]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_plene"))
        .args(args)
        .env_remove("XDG_CONFIG_HOME")
        .envs(env.iter().copied())
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // plene can exit before reading its input, as it does on a bad glossary, and
    // then the write meets a closed pipe.
    if let Err(error) = child.stdin.take().unwrap().write_all(stdin.as_bytes()) {
        assert_eq!(error.kind(), ErrorKind::BrokenPipe, "{error}");
    }
    child.wait_with_output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

/// Writes `contents` to a file under cargo's per-target temporary directory.
fn temp_file(name: &str, contents: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, contents).unwrap();
    path
}

/// A fresh, empty directory under cargo's per-target temporary directory.
fn temp_dir(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn fn_entry(text: &str) -> String {
    format!("[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntext = \"{text}\"\n")
}

const SOURCE: &str = "fn f(x: &u8) {\n    g(x)?;\n}\n";
const EXPECTED: &str = "  fn f(x: &u8) {\n» function f(x: borrowed u8) {\n      g(x)?;\n»     g(x) or return early;\n  }\n";

#[test]
fn reads_stdin() {
    let output = plene(&["-"], SOURCE);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), EXPECTED);
}

#[test]
fn reads_a_file() {
    let path = temp_file("reads_a_file.rs", SOURCE);
    let output = plene(&[path.to_str().unwrap()], "");
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(stdout(&output), EXPECTED);
}

#[test]
fn color_is_off_for_pipes_and_never_but_on_for_always() {
    assert!(!stdout(&plene(&["-"], SOURCE)).contains('\x1b'));
    assert!(!stdout(&plene(&["--color", "never", "-"], SOURCE)).contains('\x1b'));
    let colored = stdout(&plene(&["--color", "always", "-"], SOURCE));
    assert!(colored.contains("\x1b[4m"), "no underline in {colored:?}");
}

#[test]
fn theme_picks_the_expansion_band() {
    let dark = stdout(&plene(&["--color", "always", "-"], SOURCE));
    let light = stdout(&plene(
        &["--color", "always", "--theme", "light", "-"],
        SOURCE,
    ));
    assert!(dark.contains("\x1b[48;5;236m"), "{dark:?}");
    assert!(light.contains("\x1b[48;5;254m"), "{light:?}");
    assert!(!light.contains("\x1b[48;5;236m"), "{light:?}");
}

#[test]
fn edition_changes_keyword_lexing() {
    let keyword_async = "\x1b[35masync";
    let in_2015 = stdout(&plene(
        &["--color", "always", "--edition", "2015", "-"],
        "async fn f() {}",
    ));
    let in_2018 = stdout(&plene(
        &["--color", "always", "--edition", "2018", "-"],
        "async fn f() {}",
    ));
    assert!(!in_2015.contains(keyword_async), "{in_2015:?}");
    assert!(in_2018.contains(keyword_async), "{in_2018:?}");

    let keyword_gen = "\x1b[35mgen";
    let source = "fn f() { let gen = 1; }";
    let in_2021 = stdout(&plene(
        &["--color", "always", "--edition", "2021", "-"],
        source,
    ));
    let in_2024 = stdout(&plene(
        &["--color", "always", "--edition", "2024", "-"],
        source,
    ));
    assert!(!in_2021.contains(keyword_gen), "{in_2021:?}");
    assert!(in_2024.contains(keyword_gen), "{in_2024:?}");
}

#[test]
fn rejects_an_unknown_edition() {
    let output = plene(&["--edition", "2019", "-"], SOURCE);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("invalid value '2019'"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn glossary_overrides_apply_and_warn() {
    let glossary = temp_file(
        "overrides.toml",
        "[[expand]]\ntoken = \"fn\"\nrole = \"keyword\"\ntext = \"func\"\n\n\
         [[expand]]\ntoken = \"&\"\nrole = \"no_such_role\"\ntext = \"x\"\n",
    );
    let output = plene(&["--glossary", glossary.to_str().unwrap(), "-"], SOURCE);
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("» func f(x: borrowed u8) {"),
        "{}",
        stdout(&output)
    );
    assert_eq!(
        stderr(&output),
        format!(
            "plene: warning: {}: skipping entry for `&`: unknown role `no_such_role`\n",
            glossary.display()
        )
    );
}

#[test]
fn invalid_glossary_is_an_error() {
    let glossary = temp_file("invalid.toml", "[[expand]\n");
    let output = plene(&["--glossary", glossary.to_str().unwrap(), "-"], SOURCE);
    assert_eq!(output.status.code(), Some(1));
    let expected = format!("plene: {}: invalid glossary: ", glossary.display());
    assert!(
        stderr(&output).starts_with(&expected),
        "{}",
        stderr(&output)
    );
    assert!(stdout(&output).is_empty());
}

#[test]
fn missing_file_is_an_error() {
    let output = plene(&["no_such_file.rs"], "");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).starts_with("plene: no_such_file.rs: "),
        "{}",
        stderr(&output)
    );
}

#[test]
fn non_utf8_input_is_an_error() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("latin1.rs");
    std::fs::write(&path, b"// caf\xe9\n").unwrap();
    let output = plene(&[path.to_str().unwrap()], "");
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr(&output).contains("valid UTF-8"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn closed_reader_is_not_an_error() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_plene"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(SOURCE.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
}

#[test]
fn config_glossary_applies_and_the_named_glossary_wins_over_it() {
    let config_home = temp_dir("layered-config");
    temp_file("layered-config/plene/glossary.toml", &fn_entry("config-fn"));
    let env = [("XDG_CONFIG_HOME", config_home.to_str().unwrap())];

    let from_config = plene_in(&["-"], "fn f() {}", &env);
    assert_eq!(stdout(&from_config), "  fn f() {}\n» config-fn f() {}\n");

    let named = temp_file("named.toml", &fn_entry("named-fn"));
    let both = plene_in(
        &["--glossary", named.to_str().unwrap(), "-"],
        "fn f() {}",
        &env,
    );
    assert_eq!(stdout(&both), "  fn f() {}\n» named-fn f() {}\n");
}

#[test]
fn config_falls_back_to_home_when_xdg_config_home_is_unset_or_relative() {
    let home = temp_dir("home");
    temp_file("home/.config/plene/glossary.toml", &fn_entry("home-fn"));
    let home = home.to_str().unwrap();
    for env in [
        vec![("HOME", home)],
        vec![("HOME", home), ("XDG_CONFIG_HOME", "relative/dir")],
    ] {
        let output = plene_in(&["-"], "fn f() {}", &env);
        assert_eq!(
            stdout(&output),
            "  fn f() {}\n» home-fn f() {}\n",
            "{env:?}"
        );
    }
}

#[test]
fn missing_config_is_silent() {
    let output = plene(&["-"], SOURCE);
    assert!(output.status.success());
    assert!(output.stderr.is_empty(), "{}", stderr(&output));
}

#[test]
fn invalid_config_is_an_error_naming_it() {
    let config_home = temp_dir("invalid-config");
    let path = temp_file("invalid-config/plene/glossary.toml", "[[expand]\n");
    let output = plene_in(
        &["-"],
        SOURCE,
        &[("XDG_CONFIG_HOME", config_home.to_str().unwrap())],
    );
    assert_eq!(output.status.code(), Some(1));
    let expected = format!("plene: {}: invalid glossary: ", path.display());
    assert!(
        stderr(&output).starts_with(&expected),
        "{}",
        stderr(&output)
    );
}

#[test]
fn dump_glossary_prints_a_glossary_that_reads_back() {
    let output = plene(&["--dump-glossary"], "");
    assert!(output.status.success(), "{}", stderr(&output));
    let (dumped, warnings) = plene_core::Glossary::parse(&stdout(&output)).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    let builtin = plene_core::Glossary::default();
    assert_eq!(
        dumped.entries().collect::<Vec<_>>(),
        builtin.entries().collect::<Vec<_>>()
    );
}

#[test]
fn dump_glossary_includes_overrides() {
    let named = temp_file("dump-override.toml", &fn_entry("func"));
    let output = plene(
        &["--dump-glossary", "--glossary", named.to_str().unwrap()],
        "",
    );
    assert!(
        stdout(&output).contains("text = \"func\""),
        "{}",
        stdout(&output)
    );
}

#[test]
fn dump_glossary_takes_no_file_and_a_file_is_otherwise_required() {
    assert_eq!(
        plene(&["--dump-glossary", "x.rs"], "").status.code(),
        Some(2)
    );
    assert_eq!(plene(&[], "").status.code(), Some(2));
}

#[test]
fn layouts_reach_the_output() {
    let side = stdout(&plene(&["--side-by-side", "-"], "fn f() {}\n"));
    assert_eq!(side, "fn f() {} │ function f() {}\n");
    let expanded = stdout(&plene(
        &["--expanded", "--changed-only", "-"],
        "x;\nfn f() {}\n",
    ));
    assert_eq!(expanded, "2 function f() {}\n");
}

#[test]
fn layouts_are_exclusive_and_need_a_file() {
    let rejected: [&[&str]; 6] = [
        &["--dump-glossary", "--keep", "lifetimes"],
        &["--side-by-side", "--expanded", "-"],
        &["--dump-glossary", "--lines", "1:2"],
        &["--dump-glossary", "--side-by-side"],
        &["--dump-glossary", "--expanded"],
        &["--dump-glossary", "--changed-only"],
    ];
    for args in rejected {
        assert_eq!(plene(args, "").status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn lines_selects_a_numbered_range_of_the_source() {
    let source = "x;\nfn f() {}\ny;\nfn g() {}\n";
    let both = stdout(&plene(&["--expanded", "--lines", "2:3", "-"], source));
    assert_eq!(both, "2 function f() {}\n3 y;\n");
    let open_ended = stdout(&plene(&["--expanded", "--lines", "3:", "-"], source));
    assert_eq!(open_ended, "3 y;\n4 function g() {}\n");
    let clamped = stdout(&plene(&["--expanded", "--lines", "4:99", "-"], source));
    assert_eq!(clamped, "4 function g() {}\n");
}

#[test]
fn lines_rejects_a_bad_range_and_a_start_past_the_end() {
    let bad = plene(&["--lines", "9:3", "-"], "x;\n");
    assert_eq!(bad.status.code(), Some(2));
    assert!(stderr(&bad).contains("the range ends at 3 before it starts"));
    let past = plene(&["--lines", "5:", "-"], "x;\ny;\n");
    assert_eq!(past.status.code(), Some(1));
    assert_eq!(
        stderr(&past),
        "plene: --lines starts at 5 but the source has 2 lines\n"
    );
    assert_eq!(stdout(&past), "");
}

#[test]
fn keep_leaves_the_named_categories_as_written() {
    let source = "pub fn f<'a>(x: &'a mut u8) {}\n";
    let expanded = |keep: &[&str]| {
        let mut args = vec!["--expanded"];
        args.extend_from_slice(keep);
        args.push("-");
        stdout(&plene(&args, source))
    };
    assert_eq!(
        expanded(&[]),
        "public function f<lifetime a>(x: borrowed lifetime a mutable u8) {}\n"
    );
    assert_eq!(
        expanded(&["--keep", "lifetimes"]),
        "public function f<'a>(x: borrowed 'a mutable u8) {}\n"
    );
    assert_eq!(
        expanded(&["--keep", "lifetimes,visibility,references"]),
        "pub function f<'a>(x: &'a mutable u8) {}\n"
    );
    assert_eq!(
        expanded(&["--keep", "mutability", "--keep", "keywords"]),
        "public fn f<lifetime a>(x: borrowed lifetime a mut u8) {}\n"
    );
}

#[test]
fn keep_rejects_an_unknown_category_naming_the_known() {
    let output = plene(&["--keep", "lifetimes,nope", "-"], "fn f() {}\n");
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("unknown category `nope`; one of: keywords, visibility"));
}

#[test]
fn help_lists_every_category() {
    let help = stdout(&plene(&["--help"], ""));
    let flat = help.split_whitespace().collect::<Vec<_>>().join(" ");
    for category in plene_core::Category::ALL {
        assert!(flat.contains(category.as_str()), "{category} not in --help");
    }
}

#[test]
fn a_long_functions_closing_brace_is_labelled_unless_ends_are_kept() {
    let source = format!("fn long() {{\n{}}}\n", "x;\n".repeat(20));
    let last = |keep: &[&str]| {
        let mut args = vec!["--expanded"];
        args.extend_from_slice(keep);
        args.push("-");
        stdout(&plene(&args, &source))
            .lines()
            .last()
            .unwrap()
            .to_string()
    };
    assert_eq!(last(&[]), "} end function long");
    assert_eq!(last(&["--keep", "ends"]), "}");
}

/// A config directory holding `config` as plene's config file, and the output of plene
/// run on `source` with `args` and that directory in effect.
fn with_config(name: &str, config: &str, args: &[&str], source: &str) -> Output {
    let home = temp_dir(name);
    temp_file(&format!("{name}/plene/config.toml"), config);
    plene_in(args, source, &[("XDG_CONFIG_HOME", home.to_str().unwrap())])
}

const LIFETIMES: &str = "pub fn f<'a>(x: &'a u8) {}\n";

#[test]
fn the_config_files_keep_list_applies() {
    let output = with_config(
        "config-keep",
        "keep = [\"lifetimes\", \"visibility\"]\n",
        &["--expanded", "-"],
        LIFETIMES,
    );
    assert_eq!(
        stdout(&output),
        "pub function f<'a>(x: borrowed 'a u8) {}\n"
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn keep_on_the_command_line_replaces_the_config_files_list() {
    let output = with_config(
        "config-replaced",
        "keep = [\"lifetimes\"]\n",
        &["--expanded", "--keep", "visibility", "-"],
        LIFETIMES,
    );
    assert_eq!(
        stdout(&output),
        "pub function f<lifetime a>(x: borrowed lifetime a u8) {}\n"
    );
}

#[test]
fn expand_all_ignores_the_config_file() {
    let output = with_config(
        "config-expand-all",
        "keep = [\"lifetimes\", \"visibility\"]\n",
        &["--expanded", "--expand-all", "-"],
        LIFETIMES,
    );
    assert_eq!(
        stdout(&output),
        "public function f<lifetime a>(x: borrowed lifetime a u8) {}\n"
    );
}

#[test]
fn a_config_that_does_not_parse_is_an_error_naming_it_unless_the_command_line_decides() {
    let bad = "keep = 3\n";
    let output = with_config("config-bad", bad, &["--expanded", "-"], LIFETIMES);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stdout(&output), "");
    let message = stderr(&output);
    assert!(message.starts_with("plene: "), "{message}");
    assert!(
        message.contains("config.toml: invalid config: "),
        "{message}"
    );

    for flag in [&["--keep", "visibility"][..], &["--expand-all"][..]] {
        let mut args = vec!["--expanded"];
        args.extend_from_slice(flag);
        args.push("-");
        let output = with_config("config-bad-overridden", bad, &args, LIFETIMES);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{flag:?}: {}",
            stderr(&output)
        );
        assert_eq!(stderr(&output), "");
    }

    let dumped = with_config("config-bad-dump", bad, &["--dump-glossary"], "");
    assert_eq!(dumped.status.code(), Some(0), "the config is not read");
}

#[test]
fn an_unknown_category_in_the_config_is_a_warning() {
    let output = with_config(
        "config-unknown",
        "keep = [\"nope\", \"lifetimes\"]\n",
        &["--expanded", "-"],
        LIFETIMES,
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output),
        "public function f<'a>(x: borrowed 'a u8) {}\n"
    );
    assert!(
        stderr(&output).contains("config.toml: skipping unknown category `nope`; one of: "),
        "{}",
        stderr(&output)
    );
}

#[test]
fn expand_all_conflicts_with_keep_and_with_dumping_the_glossary() {
    for args in [
        &["--expand-all", "--keep", "lifetimes", "-"][..],
        &["--expand-all", "--dump-glossary"][..],
    ] {
        assert_eq!(plene(args, "").status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn help_describes_expand_all() {
    let help = stdout(&plene(&["--help"], ""));
    assert!(help.contains("--expand-all"), "{help}");
}

const ELIDED: &str = "fn f(x: &u8) -> &u8 {}\n";

#[test]
fn elided_lifetimes_start_off_and_are_expanded_on_request() {
    let shown = |args: &[&str]| {
        let mut all = vec!["--expanded"];
        all.extend_from_slice(args);
        all.push("-");
        stdout(&plene(&all, ELIDED))
    };
    let plain = "function f(x: borrowed u8) returns borrowed u8 {}\n";
    let elided = "function f(x: borrowed u8) returns borrowed (from x) u8 {}\n";
    assert_eq!(shown(&[]), plain);
    assert_eq!(shown(&["--expand", "elision"]), elided);
    assert_eq!(shown(&["--expand-all"]), elided);
    assert_eq!(
        shown(&["--keep", "ends"]),
        plain,
        "keeping others does not turn it on"
    );
    assert_eq!(shown(&["--keep", "ends", "--expand", "elision"]), elided);
    assert_eq!(
        shown(&["--keep", "references", "--expand", "elision"]),
        "function f(x: &u8) returns borrowed (from x) u8 {}\n",
        "its own `&` still reads as the elided one while references are kept"
    );
}

#[test]
fn expand_takes_only_a_category_that_starts_off() {
    let output = plene(&["--expand", "lifetimes", "-"], ELIDED);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains(
            "`lifetimes` already expands by default; the categories that start off are: elision"
        ),
        "{}",
        stderr(&output)
    );
    for args in [
        &["--expand", "elision", "--expand-all", "-"][..],
        &["--dump-glossary", "--expand", "elision"][..],
    ] {
        assert_eq!(plene(args, "").status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn the_config_file_expands_what_starts_off() {
    let output = with_config(
        "config-expand",
        "expand = [\"elision\"]\n",
        &["--expanded", "-"],
        ELIDED,
    );
    assert_eq!(
        stdout(&output),
        "function f(x: borrowed u8) returns borrowed (from x) u8 {}\n"
    );
    let warned = with_config(
        "config-expand-warned",
        "expand = [\"lifetimes\"]\n",
        &["--expanded", "-"],
        ELIDED,
    );
    assert!(
        stderr(&warned).contains("config.toml: skipping `lifetimes` already expands by default"),
        "{}",
        stderr(&warned)
    );
    let replaced = with_config(
        "config-expand-replaced",
        "expand = [\"elision\"]\n",
        &["--expanded", "--keep", "ends", "-"],
        ELIDED,
    );
    assert_eq!(
        stdout(&replaced),
        "function f(x: borrowed u8) returns borrowed u8 {}\n",
        "the command line replaces both lists"
    );
}
