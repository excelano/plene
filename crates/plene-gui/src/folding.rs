//! Which bodies are folded away, and so which rows are shown.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::collections::HashSet;

use plene_core::Fold;

/// The bodies the reader has folded. A fold hides the rows after the one holding its
/// opening brace, through the one holding its closing brace.
#[derive(Default)]
pub struct Folding {
    folded: HashSet<Fold>,
}

impl Folding {
    pub fn is_folded(&self, fold: &Fold) -> bool {
        self.folded.contains(fold)
    }

    /// The document rows still shown, in order.
    pub fn shown_rows(&self, folds: &[Fold], line_count: usize) -> Vec<usize> {
        let mut hidden = vec![false; line_count];
        for fold in folds.iter().filter(|fold| self.folded.contains(fold)) {
            let end = fold.last.min(line_count.saturating_sub(1));
            if fold.first < end {
                hidden[fold.first + 1..=end].fill(true);
            }
        }
        (0..line_count).filter(|row| !hidden[*row]).collect()
    }

    pub fn toggle(&mut self, fold: Fold) {
        if !self.folded.remove(&fold) {
            self.folded.insert(fold);
        }
    }

    /// Folds every function body, which leaves the `impl`, trait and module around them
    /// open, so that the file reads as its signatures.
    pub fn fold_functions(&mut self, folds: &[Fold]) {
        self.folded
            .extend(folds.iter().filter(|fold| fold.function).copied());
    }

    pub fn unfold_all(&mut self) {
        self.folded.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.folded.is_empty()
    }

    /// What a click on the marker beside `row` does: unfolds the outermost folded body
    /// that opens there, or folds the outermost body when none is folded. Says whether
    /// any body opens on `row`.
    pub fn toggle_at(&mut self, folds: &[Fold], row: usize) -> bool {
        let opening = folds_opening_at(folds, row);
        let Some(fold) = self
            .to_unfold(folds, row)
            .or_else(|| opening.first().copied())
        else {
            return false;
        };
        self.toggle(fold);
        true
    }

    /// Unfolds whatever hides `row`, and says whether anything did.
    pub fn reveal(&mut self, row: usize) -> bool {
        let before = self.folded.len();
        self.folded
            .retain(|fold| !(fold.first < row && row <= fold.last));
        self.folded.len() != before
    }

    /// The fold that Left folds from `row`: the innermost body, not yet folded, that has
    /// `row` in it, its opening line included.
    pub fn to_fold(&self, folds: &[Fold], row: usize) -> Option<Fold> {
        folds
            .iter()
            .filter(|fold| fold.first <= row && row <= fold.last && !self.folded.contains(fold))
            .min_by_key(|fold| fold.last - fold.first)
            .copied()
    }

    /// The fold that Right unfolds at `row`: a folded body that opens there, the
    /// outermost if several do.
    pub fn to_unfold(&self, folds: &[Fold], row: usize) -> Option<Fold> {
        folds_opening_at(folds, row)
            .iter()
            .find(|fold| self.folded.contains(fold))
            .copied()
    }
}

/// The folds that open on `row`, outermost first.
pub fn folds_opening_at(folds: &[Fold], row: usize) -> &[Fold] {
    let start = folds.partition_point(|fold| fold.first < row);
    let end = folds.partition_point(|fold| fold.first <= row);
    &folds[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fold(first: usize, last: usize, function: bool) -> Fold {
        Fold {
            first,
            last,
            function,
        }
    }

    /// An `impl` over rows 0 to 9 with a function at 1 to 4 and another at 5 to 8, and a
    /// struct at 10 to 12.
    fn file() -> Vec<Fold> {
        vec![
            fold(0, 9, false),
            fold(1, 4, true),
            fold(5, 8, true),
            fold(10, 12, false),
        ]
    }

    #[test]
    fn nothing_folded_shows_every_row() {
        let folding = Folding::default();
        assert_eq!(folding.shown_rows(&file(), 14), (0..14).collect::<Vec<_>>());
        assert!(folding.is_empty());
    }

    #[test]
    fn a_fold_hides_the_rows_after_its_opening_through_its_closing_brace() {
        let mut folding = Folding::default();
        folding.toggle(fold(1, 4, true));
        assert_eq!(
            folding.shown_rows(&file(), 14),
            [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13]
        );
        assert!(folding.is_folded(&fold(1, 4, true)));
        folding.toggle(fold(1, 4, true));
        assert!(!folding.is_folded(&fold(1, 4, true)));
        assert_eq!(folding.shown_rows(&file(), 14).len(), 14);
    }

    #[test]
    fn a_folded_body_hides_the_folds_inside_it() {
        let mut folding = Folding::default();
        folding.toggle(fold(1, 4, true));
        folding.toggle(fold(0, 9, false));
        assert_eq!(folding.shown_rows(&file(), 14), [0, 10, 11, 12, 13]);
        folding.toggle(fold(0, 9, false));
        assert_eq!(
            folding.shown_rows(&file(), 14).len(),
            11,
            "the inner one stays"
        );
    }

    #[test]
    fn a_fold_past_the_end_hides_to_the_last_row() {
        let mut folding = Folding::default();
        folding.toggle(fold(2, 40, false));
        assert_eq!(folding.shown_rows(&[fold(2, 40, false)], 5), [0, 1, 2]);
        assert!(folding.shown_rows(&[], 0).is_empty());
    }

    #[test]
    fn fold_functions_leaves_the_rest_open_and_unfold_all_opens_everything() {
        let mut folding = Folding::default();
        folding.fold_functions(&file());
        assert_eq!(
            folding.shown_rows(&file(), 14),
            [0, 1, 5, 9, 10, 11, 12, 13]
        );
        folding.unfold_all();
        assert!(folding.is_empty());
    }

    #[test]
    fn revealing_a_row_unfolds_what_hides_it_and_nothing_else() {
        let mut folding = Folding::default();
        folding.fold_functions(&file());
        folding.toggle(fold(10, 12, false));
        assert!(folding.reveal(3), "inside the first function");
        assert_eq!(
            folding.shown_rows(&file(), 14),
            [0, 1, 2, 3, 4, 5, 9, 10, 13]
        );
        assert!(!folding.reveal(3), "already shown");
        assert!(!folding.reveal(10), "an opening line is shown");
        assert!(folding.reveal(12), "a closing line is hidden");
        assert!(!folding.reveal(99));
    }

    #[test]
    fn left_folds_the_innermost_open_body_around_the_row() {
        let mut folding = Folding::default();
        let folds = file();
        assert_eq!(
            folding.to_fold(&folds, 1),
            Some(fold(1, 4, true)),
            "its own line"
        );
        assert_eq!(
            folding.to_fold(&folds, 3),
            Some(fold(1, 4, true)),
            "inside it"
        );
        assert_eq!(
            folding.to_fold(&folds, 4),
            Some(fold(1, 4, true)),
            "its closing line"
        );
        assert_eq!(folding.to_fold(&folds, 0), Some(fold(0, 9, false)));
        assert_eq!(folding.to_fold(&folds, 13), None, "outside every body");
        folding.toggle(fold(1, 4, true));
        assert_eq!(
            folding.to_fold(&folds, 1),
            Some(fold(0, 9, false)),
            "a folded body is skipped for the one around it"
        );
        folding.toggle(fold(0, 9, false));
        assert_eq!(folding.to_fold(&folds, 0), None);
    }

    #[test]
    fn right_unfolds_a_folded_body_opening_on_the_row() {
        let mut folding = Folding::default();
        let folds = file();
        assert_eq!(folding.to_unfold(&folds, 1), None);
        folding.toggle(fold(1, 4, true));
        assert_eq!(folding.to_unfold(&folds, 1), Some(fold(1, 4, true)));
        assert_eq!(folding.to_unfold(&folds, 2), None);
        assert_eq!(folding.to_unfold(&folds, 5), None);
    }

    #[test]
    fn the_marker_unfolds_a_folded_body_or_folds_the_outermost() {
        let folds = [fold(0, 3, false), fold(0, 2, true), fold(4, 6, true)];
        let mut folding = Folding::default();
        assert!(folding.toggle_at(&folds, 0));
        assert!(folding.is_folded(&fold(0, 3, false)), "the outermost folds");
        assert!(folding.toggle_at(&folds, 0));
        assert!(folding.is_empty(), "then it unfolds");
        folding.toggle(fold(0, 2, true));
        assert!(folding.toggle_at(&folds, 0));
        assert!(folding.is_empty(), "a folded inner body is unfolded first");
        assert!(!folding.toggle_at(&folds, 1), "no body opens on row 1");
        assert!(folding.is_empty());
    }

    #[test]
    fn bodies_opening_on_one_line_come_outermost_first() {
        let folds = [fold(0, 3, false), fold(0, 2, true), fold(4, 6, true)];
        assert_eq!(folds_opening_at(&folds, 0), &folds[..2]);
        assert_eq!(folds_opening_at(&folds, 4), &folds[2..]);
        assert!(folds_opening_at(&folds, 1).is_empty());
        let mut folding = Folding::default();
        folding.toggle(fold(0, 2, true));
        assert_eq!(folding.to_unfold(&folds, 0), Some(fold(0, 2, true)));
        folding.toggle(fold(0, 3, false));
        assert_eq!(
            folding.to_unfold(&folds, 0),
            Some(fold(0, 3, false)),
            "the outermost first"
        );
    }
}
