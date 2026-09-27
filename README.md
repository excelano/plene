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

## Install and run

    cargo install --git https://github.com/excelano/plene plene
    plene src/main.rs
    plene --color always src/main.rs | less -R

`plene -` reads standard input, and `plene --help` lists the rest. `Cargo.toml` names the Rust toolchain it needs.

The expansions come from a glossary built into plene. Entries can be overridden by token and role in `~/.config/plene/glossary.toml` or in a file passed with `--glossary`; `plene --dump-glossary` prints the glossary in effect as a starting point. `DESIGN.md` describes the glossary and why it reads the way it does.

## The name

Hebrew is usually written without its vowel marks. Plene spelling, *ktiv male* or "full spelling", makes up for them by adding the letters vav and yod to stand for vowels: the same words in the same language, easier to read. plene does that for Rust.

The approach owes a debt to Jeff A. Benner's [Mechanical Translation](https://www.mechanical-translation.org/) of the Hebrew Bible, which renders each Hebrew word the same way every time it appears, in the order it appears, so that a reader without Hebrew can see how the original is built.

## License

MIT. See [LICENSE](LICENSE); security reports go through [SECURITY.md](SECURITY.md).
