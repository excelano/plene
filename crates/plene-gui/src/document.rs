//! An open source file and its transcription.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::path::Path;

use plene_core::{Edition, Fold, Glossary, Line, folds, transcribe};

pub struct Document {
    /// The file's name, for the window title and the toolbar.
    pub name: String,
    source: String,
    pub lines: Vec<Line>,
    /// The bodies that can be folded away, which do not depend on the glossary.
    pub folds: Vec<Fold>,
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
        let folds = folds(&source, edition);
        Ok(Document {
            name,
            source,
            lines,
            folds,
        })
    }

    /// Transcribes the source again, with a glossary that now expands something else.
    pub fn transcribe(&mut self, edition: Edition, glossary: &Glossary) {
        self.lines = transcribe(&self.source, edition, glossary);
    }
}
