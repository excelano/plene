# Security Policy

## Reporting a vulnerability

Please report suspected vulnerabilities privately through GitHub Security Advisories at https://github.com/excelano/plene/security/advisories/new. If you would rather not use GitHub, email david.anderson@excelano.com instead. I aim to respond within seven days.

Please do not open public issues for security problems.

## Supported versions

The latest release receives security fixes. Older releases are not supported. A repository with no releases is supported at its default branch.

## What plene can access

plene reads the Rust source it is given, from a file or from standard input, and a glossary file when one is named with `--glossary`. It parses that source and writes a transcription to standard output. It does not compile, expand or run the code it reads, and it opens no other files.

The source is untrusted as far as the terminal is concerned. Control characters other than tab, which could move the cursor or restyle the screen, and the Unicode bidirectional controls, which could show code in a different order from the order it compiles in, are written as Rust escapes such as `\u{1b}` and `\u{202e}` rather than passed to the terminal.

plene makes no network calls of any kind. It has no auth layer, no telemetry, no analytics, and no remote logging.

## What plene stores

Nothing. plene writes no files, keeps no cache, and holds no state between runs.

## Verifying releases

plene has no published releases and no prebuilt archives, so there is nothing to verify a download against. It is built from this repository, which means what you run is what you built from source you can read.
