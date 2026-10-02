# plene

plene shows Rust source alongside an expanded transcription: the same code, with Rust's abbreviations and symbols written out as words. Each line that changes is followed by its longhand.

```
  pub fn read_config(path: &Path) -> Result<String, io::Error> {
» public function read_config(path: borrowed Path) returns Result<String, io::Error> {
      let mut text = String::new();
»     let mutable text = String::new();
      File::open(path)?.read_to_string(&mut text)?;
»     File::open(path)?.read_to_string(borrow mutable text) or return early;
      Ok(text)
  }
```

It is for someone who knows some Rust and finds the dense parts slow going. It is not a translator or a tutor: it spells out what is written and does not explain what it means. Words Rust already spells in full stay as they are, and every expansion depends on the role a token plays, so `&` in an expression reads `borrow` while `&` in a type reads `borrowed`. The same token in the same role always gets the same words.

## Install

On Debian and Ubuntu, add the [Excelano apt repository](https://excelano.com/apt/) once, then install it, so `apt upgrade` keeps it current:

```sh
curl -fsSL https://excelano.com/apt/setup.sh | sudo sh
sudo apt install plene
```

With Homebrew:

```sh
brew install excelano/tap/plene
```

From crates.io, with a Rust toolchain at least as new as the one `Cargo.toml` names:

```sh
cargo install plene
```

The window is a separate package, `plene-gui`, from the same apt repository (`sudo apt install plene-gui`) or from crates.io (`cargo install plene-gui`). Homebrew carries the command-line tool only.

## Run

    plene src/main.rs
    plene --color always src/main.rs | less -R

`plene -` reads standard input. `--side-by-side` and `--expanded` change the layout, `--changed-only` keeps just the lines that change, `--lines 40:80` keeps just those source lines, `--keep lifetimes,visibility` leaves those kinds of notation as written (`keep = ["lifetimes", "visibility"]` in `~/.config/plene/config.toml` does so every time, and the window's Expand menu keeps that file for you), and `plene --help` lists the rest.

    plene-gui src/main.rs

`plene-gui` shows the source and its transcription in two columns, one row per source line. Hovering a token with a glossary entry shows its role and what the glossary says about it. Clicking a row selects it, the arrow keys move the selection, and the Transcription button hides the right-hand pane when the source alone will do. A file can also be dropped on the window or opened with Ctrl+O.

The expansions come from a glossary built into plene. Entries can be overridden by token and role in `~/.config/plene/glossary.toml` or in a file passed with `--glossary`; `plene --dump-glossary` prints the glossary in effect as a starting point. `DESIGN.md` describes the glossary and why it reads the way it does.

## The name

Hebrew is usually written without its vowel marks. Plene spelling, *ktiv male* or "full spelling", makes up for them by adding the letters vav and yod to stand for vowels: the same words in the same language, easier to read. plene does that for Rust.

The approach owes a debt to Jeff A. Benner's [Mechanical Translation](https://www.mechanical-translation.org/) of the Hebrew Bible, which renders each Hebrew word the same way every time it appears, in the order it appears, so that a reader without Hebrew can see how the original is built.

## License

MIT. See [LICENSE](LICENSE); security reports go through [SECURITY.md](SECURITY.md).
