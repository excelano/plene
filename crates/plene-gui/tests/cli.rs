//! Arguments are read before any window opens, so these run without a display.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::process::{Command, Output};

fn plene_gui(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_plene-gui"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap()
}

#[test]
fn version_and_help_need_no_display() {
    let version = plene_gui(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8(version.stdout).unwrap(),
        format!("plene-gui {}\n", env!("CARGO_PKG_VERSION"))
    );
    let help = plene_gui(&["--help"]);
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("--edition")
    );
}

#[test]
fn a_bad_argument_is_an_argument_error() {
    let output = plene_gui(&["--edition", "1999"]);
    assert_eq!(output.status.code(), Some(2));
}
