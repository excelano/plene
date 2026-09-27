//! The glossary: the expansion text and hover note for each token in each role.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use crate::role::Role;

const BUILTIN: &str = include_str!("../glossary.toml");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryEntry {
    /// The token as written. For roles that take a name, a representative form such as `'a`.
    pub token: String,
    pub role: Role,
    /// The expansion. Equal to `token` when the token is kept as written.
    pub text: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Glossary {
    entries: BTreeMap<(String, Role), GlossaryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryError(String);

impl fmt::Display for GlossaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid glossary: {}", self.0)
    }
}

impl std::error::Error for GlossaryError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GlossaryFile {
    #[serde(default)]
    expand: Vec<EntryFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntryFile {
    token: String,
    role: String,
    text: String,
    note: Option<String>,
}

/// The built-in glossary.
impl Default for Glossary {
    fn default() -> Glossary {
        let (glossary, _) = Glossary::parse(BUILTIN).expect("the built-in glossary parses");
        glossary
    }
}

impl Glossary {
    /// Parses a glossary file. An entry with an unknown role, or one that repeats an
    /// earlier entry's token and role, produces a warning rather than an error.
    pub fn parse(text: &str) -> Result<(Glossary, Vec<String>), GlossaryError> {
        let file: GlossaryFile =
            toml::from_str(text).map_err(|error| GlossaryError(error.to_string()))?;
        let mut entries = BTreeMap::new();
        let mut warnings = Vec::new();
        for entry in file.expand {
            let role = match entry.role.parse::<Role>() {
                Ok(role) => role,
                Err(unknown) => {
                    warnings.push(format!("skipping entry for `{}`: {unknown}", entry.token));
                    continue;
                }
            };
            let key = key(&entry.token, role);
            if entries.contains_key(&key) {
                warnings.push(format!(
                    "duplicate entry for `{}` as {role}; the later one wins",
                    entry.token
                ));
            }
            let entry = GlossaryEntry {
                token: entry.token,
                role,
                text: entry.text,
                note: entry.note,
            };
            entries.insert(key, entry);
        }
        Ok((Glossary { entries }, warnings))
    }

    /// Applies `overrides` on top of this glossary. An override without a note keeps
    /// the note it replaces.
    pub fn merge(&mut self, overrides: Glossary) {
        for (key, mut entry) in overrides.entries {
            if entry.note.is_none() {
                entry.note = self.entries.get(&key).and_then(|base| base.note.clone());
            }
            self.entries.insert(key, entry);
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = &GlossaryEntry> {
        self.entries.values()
    }

    pub fn entry(&self, token: &str, role: Role) -> Option<&GlossaryEntry> {
        self.entries.get(&key(token, role))
    }

    /// The expansion of `token` in `role`, with `{name}` filled in.
    pub fn expand(&self, token: &str, role: Role) -> Option<String> {
        let entry = self.entry(token, role)?;
        if role.takes_name() {
            let name = token.strip_prefix('\'').unwrap_or(token);
            Some(entry.text.replace("{name}", name))
        } else {
            Some(entry.text.clone())
        }
    }
}

/// Entries for roles that take a name are keyed by role alone.
fn key(token: &str, role: Role) -> (String, Role) {
    if role.takes_name() {
        (String::new(), role)
    } else {
        (token.to_string(), role)
    }
}
