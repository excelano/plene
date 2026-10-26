# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub Security Advisories at https://github.com/excelano/plene/security/advisories/new. If you would rather not use GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives security fixes. Older releases are not supported.

## What plene can access

plene reads the Rust source it is given, from a file or from standard input; `plene/config.toml` in the config directory (`$XDG_CONFIG_HOME`, or `~/.config`) when it exists and the command line does not say which categories to keep; and up to two glossary files: `plene/glossary.toml` in the same directory when it exists, and the file named with `--glossary`. It parses that source and writes a transcription to standard output. It does not compile, expand or run the code it reads, and it opens no other files.

The source is untrusted as far as the terminal is concerned. Control characters other than tab, which could move the cursor or restyle the screen, and the Unicode bidirectional controls, which could show code in a different order from the order it compiles in, are written as Rust escapes such as `\u{1b}` and `\u{202e}` rather than passed to the terminal.

plene makes no network calls of any kind. It has no auth layer, no telemetry, no analytics, and no remote logging.

## What plene stores

The command-line tool writes no files, keeps no cache, and holds no state between runs. The window, `plene-gui`, saves the categories you choose to keep or expand to the `plene/config.toml` named above, writing a staging file beside it and moving it into place; a config it could not read is reported and not written over, and a session whose categories came from the command line saves nothing.

## Verifying releases

Every GitHub release includes a `.sha256` file next to each archive listing its SHA-256 hash. Verify any download before running it:

    sha256sum plene-x86_64-unknown-linux-gnu.tar.xz
    # compare against the value in plene-x86_64-unknown-linux-gnu.tar.xz.sha256

Release artifacts are built by `build-release`, the release program in excelano/shipping, from a tagged commit using the cargo-dist configuration in this repository (`dist-workspace.toml`). That build configuration is public and auditable.
