//! An open source file and its transcription.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::path::Path;

use plene_core::{Edition, Glossary, Line, transcribe};

pub struct Document {
    /// The file's name, for the window title and the toolbar.
    pub name: String,
    source: String,
    pub lines: Vec<Line>,
}

impl Document {
    pub fn open(path: &Path, edition: Edition, glossary: &Glossary) -> Result<Document, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let name = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy()
            .into_owned();
        let lines = transcribe(&source, edition, glossary);
        Ok(Document {
            name,
            source,
            lines,
        })
    }

    /// Transcribes the source again, with a glossary that now expands something else.
    pub fn transcribe(&mut self, edition: Edition, glossary: &Glossary) {
        self.lines = transcribe(&self.source, edition, glossary);
    }
}
