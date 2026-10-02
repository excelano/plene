//! The glossary: the expansion text and hover note for each token in each role.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::category::Category;
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
    /// Where the Rust reference covers the token.
    pub url: Option<String>,
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

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct GlossaryFile {
    #[serde(default)]
    expand: Vec<EntryFile>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EntryFile {
    token: String,
    role: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// The built-in glossary.
impl Default for Glossary {
    fn default() -> Glossary {
        let (glossary, _) = Glossary::parse(BUILTIN).expect("the built-in glossary parses");
        glossary
    }
}

impl Glossary {
    /// The glossary in effect: the built-in one, overridden by the config directory's
    /// `plene/glossary.toml` when it exists, and then by the file at `named`. Each
    /// warning and error names the file it came from. This is the only part of
    /// plene-core that reads files.
    pub fn load(named: Option<&Path>) -> Result<(Glossary, Vec<String>), String> {
        let mut glossary = Glossary::default();
        let mut warnings = Vec::new();
        let config = config_path();
        for path in config
            .as_deref()
            .filter(|path| path.exists())
            .into_iter()
            .chain(named)
        {
            let text = std::fs::read_to_string(path)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            let (overrides, file_warnings) =
                Glossary::parse(&text).map_err(|error| format!("{}: {error}", path.display()))?;
            warnings.extend(
                file_warnings
                    .into_iter()
                    .map(|warning| format!("{}: {warning}", path.display())),
            );
            glossary.merge(overrides);
        }
        Ok((glossary, warnings))
    }

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
                url: entry.url,
            };
            entries.insert(key, entry);
        }
        Ok((Glossary { entries }, warnings))
    }

    /// Applies `overrides` on top of this glossary. An override without a note or a
    /// link keeps the note or link it replaces.
    pub fn merge(&mut self, overrides: Glossary) {
        for (key, mut entry) in overrides.entries {
            let base = self.entries.get(&key);
            if entry.note.is_none() {
                entry.note = base.and_then(|base| base.note.clone());
            }
            if entry.url.is_none() {
                entry.url = base.and_then(|base| base.url.clone());
            }
            self.entries.insert(key, entry);
        }
    }

    /// This glossary without the entries of `categories`, so that what they cover is
    /// kept as written.
    pub fn without(&self, categories: &[Category]) -> Glossary {
        let mut entries = self.entries.clone();
        entries.retain(|_, entry| !categories.contains(&Category::of(&entry.token, entry.role)));
        Glossary { entries }
    }

    /// The glossary as a file `parse` reads back to the same entries, ordered by role
    /// and then token.
    pub fn to_toml(&self) -> String {
        let mut entries: Vec<&GlossaryEntry> = self.entries().collect();
        entries.sort_by(|a, b| (a.role, &a.token).cmp(&(b.role, &b.token)));
        let file = GlossaryFile {
            expand: entries
                .into_iter()
                .map(|entry| EntryFile {
                    token: entry.token.clone(),
                    role: entry.role.to_string(),
                    text: entry.text.clone(),
                    note: entry.note.clone(),
                    url: entry.url.clone(),
                })
                .collect(),
        };
        toml::to_string(&file).expect("a glossary serializes")
    }

    pub fn entries(&self) -> impl Iterator<Item = &GlossaryEntry> {
        self.entries.values()
    }

    pub fn entry(&self, token: &str, role: Role) -> Option<&GlossaryEntry> {
        self.entries.get(&key(token, role))
    }

    /// The expansion of `token` in `role`, with `{name}` replaced by `name` when the
    /// role takes one.
    pub fn expand(&self, token: &str, role: Role, name: Option<&str>) -> Option<String> {
        let entry = self.entry(token, role)?;
        match name {
            Some(name) if role.takes_name() => Some(entry.text.replace("{name}", name)),
            _ => Some(entry.text.clone()),
        }
    }
}

/// `$XDG_CONFIG_HOME/plene/glossary.toml`, or `~/.config/plene/glossary.toml` when
/// `XDG_CONFIG_HOME` is unset or not absolute, which the XDG spec says to ignore.
fn config_path() -> Option<PathBuf> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::home_dir().map(|home| home.join(".config")))?;
    Some(config_home.join("plene").join("glossary.toml"))
}

/// Entries for roles that take a name are keyed by role alone.
fn key(token: &str, role: Role) -> (String, Role) {
    if role.takes_name() {
        (String::new(), role)
    } else {
        (token.to_string(), role)
    }
}
