//! Searching the panes' text: which lines match a query on which side, and which match
//! the reader is on.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::ops::Range;

use plene_core::Line;

use crate::text::{Mark, Side};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Both,
    Source,
    Transcription,
}

impl Scope {
    /// The sides to search: those this scope names that are on screen.
    fn sides(self, transcription_shown: bool) -> &'static [Side] {
        match self {
            Scope::Source => &[Side::Source],
            Scope::Transcription if transcription_shown => &[Side::Transcription],
            Scope::Transcription => &[],
            Scope::Both if transcription_shown => &[Side::Source, Side::Transcription],
            Scope::Both => &[Side::Source],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Match {
    row: usize,
    side: Side,
    /// Byte range in the text of the line on `side`.
    range: Range<usize>,
}

pub struct Search {
    pub open: bool,
    /// Set when the field should take the keyboard on the next frame.
    pub focus: bool,
    pub query: String,
    pub scope: Scope,
    matches: Vec<Match>,
    current: Option<usize>,
    stale: bool,
}

impl Default for Search {
    fn default() -> Search {
        Search {
            open: false,
            focus: false,
            query: String::new(),
            scope: Scope::Both,
            matches: Vec::new(),
            current: None,
            stale: true,
        }
    }
}

impl Search {
    /// The matches no longer fit the document, the query, the scope or the panes shown.
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Finds the matches again if they are stale, and puts the current one on the first
    /// match at or after `from_row`, or the first of all when none follows.
    pub fn refresh(&mut self, lines: &[Line], transcription_shown: bool, from_row: usize) {
        if !self.stale {
            return;
        }
        self.stale = false;
        self.matches = find_all(lines, &self.query, self.scope.sides(transcription_shown));
        self.current = if self.matches.is_empty() {
            None
        } else {
            let after = self.matches.partition_point(|found| found.row < from_row);
            Some(after % self.matches.len())
        };
    }

    /// Moves to the next or previous match, wrapping round the document, and returns
    /// the row it is on.
    pub fn step(&mut self, forward: bool) -> Option<usize> {
        let count = self.matches.len();
        let current = self.current?;
        let next = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.current = Some(next);
        self.current_row()
    }

    pub fn current_row(&self) -> Option<usize> {
        self.current.map(|current| self.matches[current].row)
    }

    pub fn count(&self) -> usize {
        self.matches.len()
    }

    /// The current match's place among the matches, counted from 1.
    pub fn position(&self) -> Option<usize> {
        self.current.map(|current| current + 1)
    }

    /// The matches on `side` of `row`, to be shown. None while the search is closed.
    pub fn marks(&self, row: usize, side: Side) -> Vec<Mark> {
        if !self.open {
            return Vec::new();
        }
        let first = self.matches.partition_point(|found| found.row < row);
        self.matches[first..]
            .iter()
            .enumerate()
            .take_while(|(_, found)| found.row == row)
            .filter(|(_, found)| found.side == side)
            .map(|(offset, found)| Mark {
                range: found.range.clone(),
                current: self.current == Some(first + offset),
            })
            .collect()
    }
}

/// Every non-overlapping occurrence of `query` in each line's text on each of `sides`,
/// in row order, the source's before the transcription's. Letters match whatever their
/// ASCII case, which leaves byte positions where they were.
fn find_all(lines: &[Line], query: &str, sides: &[Side]) -> Vec<Match> {
    if query.is_empty() {
        return Vec::new();
    }
    let needle = query.to_ascii_lowercase();
    let mut found = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        for &side in sides {
            let text: String = line.spans.iter().map(|span| side.text(span)).collect();
            let haystack = text.to_ascii_lowercase();
            let mut from = 0;
            while let Some(at) = haystack[from..].find(&needle) {
                let start = from + at;
                from = start + needle.len();
                found.push(Match {
                    row,
                    side,
                    range: start..from,
                });
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use plene_core::{Edition, Glossary, transcribe};

    use super::*;

    const SOURCE: &str = "fn f(a: &mut u8, b: &mut u8) {}\nfn g() {}\nlet Borrow = 1;\n";

    fn lines() -> Vec<Line> {
        transcribe(SOURCE, Edition::default(), &Glossary::default())
    }

    fn found(query: &str, scope: Scope, transcription_shown: bool) -> Vec<(usize, Side)> {
        let mut search = Search {
            query: query.to_string(),
            scope,
            ..Search::default()
        };
        search.refresh(&lines(), transcription_shown, 0);
        search.matches.iter().map(|m| (m.row, m.side)).collect()
    }

    #[test]
    fn the_transcription_is_found_by_its_words_and_the_source_by_its_symbols() {
        assert_eq!(
            found("borrowed mutable", Scope::Both, true),
            [(0, Side::Transcription), (0, Side::Transcription)]
        );
        assert_eq!(
            found("&mut", Scope::Both, true),
            [(0, Side::Source), (0, Side::Source)]
        );
    }

    #[test]
    fn a_scope_narrows_the_sides() {
        assert_eq!(found("fn g", Scope::Source, true), [(1, Side::Source)]);
        assert_eq!(found("fn g", Scope::Transcription, true), []);
        assert_eq!(
            found("function g", Scope::Transcription, true),
            [(1, Side::Transcription)]
        );
        assert_eq!(found("function g", Scope::Source, true), []);
    }

    #[test]
    fn a_hidden_transcription_is_not_searched() {
        assert_eq!(found("function", Scope::Both, false), []);
        assert_eq!(found("function", Scope::Transcription, false), []);
        assert_eq!(found("fn", Scope::Both, false)[0], (0, Side::Source));
    }

    #[test]
    fn a_line_lists_its_source_matches_before_its_transcription_matches() {
        let both = found("f", Scope::Both, true);
        let row_zero: Vec<Side> = both
            .iter()
            .filter(|(row, _)| *row == 0)
            .map(|(_, side)| *side)
            .collect();
        assert_eq!(row_zero.first(), Some(&Side::Source));
        assert_eq!(row_zero.last(), Some(&Side::Transcription));
    }

    #[test]
    fn letter_case_is_ignored_for_ascii() {
        assert_eq!(found("BORROW", Scope::Source, true), [(2, Side::Source)]);
        assert_eq!(found("borrow", Scope::Source, true), [(2, Side::Source)]);
    }

    #[test]
    fn matches_do_not_overlap_and_an_empty_query_finds_nothing() {
        let lines = transcribe("aaaa\n", Edition::default(), &Glossary::default());
        let ranges: Vec<Range<usize>> = find_all(&lines, "aa", &[Side::Source])
            .into_iter()
            .map(|found| found.range)
            .collect();
        assert_eq!(ranges, [0..2, 2..4]);
        assert!(find_all(&lines, "", &[Side::Source]).is_empty());
    }

    #[test]
    fn non_ascii_text_keeps_its_byte_positions() {
        let lines = transcribe(
            "// İstanbul café\nfn f() {}\n",
            Edition::default(),
            &Glossary::default(),
        );
        let found = find_all(&lines, "café", &[Side::Source]);
        assert_eq!(found.len(), 1);
        let text = "// İstanbul café";
        assert_eq!(&text[found[0].range.clone()], "café");
    }

    #[test]
    fn refresh_starts_at_the_first_match_from_a_row_and_wraps() {
        let mut search = Search {
            query: "fn".to_string(),
            scope: Scope::Source,
            ..Search::default()
        };
        search.refresh(&lines(), true, 1);
        assert_eq!(search.current_row(), Some(1));
        search.mark_stale();
        search.refresh(&lines(), true, 2);
        assert_eq!(search.current_row(), Some(0), "none follows row 2");
    }

    #[test]
    fn refresh_waits_for_a_stale_mark() {
        let mut search = Search {
            query: "fn".to_string(),
            ..Search::default()
        };
        search.refresh(&lines(), true, 0);
        let count = search.count();
        search.query = "u8".to_string();
        search.refresh(&lines(), true, 0);
        assert_eq!(search.count(), count, "not stale, so not searched again");
        search.mark_stale();
        search.refresh(&lines(), true, 0);
        assert_ne!(search.count(), count);
    }

    #[test]
    fn stepping_wraps_in_both_directions() {
        let mut search = Search {
            query: "fn".to_string(),
            scope: Scope::Source,
            ..Search::default()
        };
        assert_eq!(search.step(true), None, "nothing searched yet");
        search.refresh(&lines(), true, 0);
        assert_eq!(search.position(), Some(1));
        assert_eq!(search.step(true), Some(1));
        assert_eq!(search.position(), Some(2));
        assert_eq!(search.step(true), Some(0));
        assert_eq!(search.step(false), Some(1));
        assert_eq!(search.step(false), Some(0));
        assert_eq!(search.count(), 2);
    }

    #[test]
    fn marks_belong_to_a_row_and_side_and_flag_the_current_match() {
        let mut search = Search {
            open: true,
            query: "&mut".to_string(),
            ..Search::default()
        };
        search.refresh(&lines(), true, 0);
        let marks = search.marks(0, Side::Source);
        assert_eq!(marks.len(), 2);
        assert!(marks[0].current && !marks[1].current);
        assert!(search.marks(0, Side::Transcription).is_empty());
        assert!(search.marks(1, Side::Source).is_empty());
        search.step(true);
        let marks = search.marks(0, Side::Source);
        assert!(!marks[0].current && marks[1].current);
        search.open = false;
        assert!(search.marks(0, Side::Source).is_empty());
    }
}
