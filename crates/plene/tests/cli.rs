use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

fn plene(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_plene"))
        .args(args)
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
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
    std::fs::write(&path, contents).unwrap();
    path
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
