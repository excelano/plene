pub fn read_config(path: &Path) -> Result<String, io::Error> {
    let mut text = String::new();
    File::open(path)?.read_to_string(&mut text)?;
    Ok(text)
}

impl Config {
    pub fn enabled(&self) -> impl Iterator<Item = &str> + '_ {
        self.flags.iter().filter(|f| !f.is_empty()).map(|f| f.as_str())
    }

    pub fn level(&self) -> &'static str {
        match self.flags.len() {
            0 => "quiet",
            1..=3 => "normal",
            _ => "verbose",
        }
    }
}

#[test]
fn reads_what_was_written() -> io::Result<()> {
    let path = temp_dir().join("plene.toml");
    std::fs::write(&path, "debug = true\n")?;
    assert_eq!(read_config(&path)?, "debug = true\n");
    Ok(())
}
