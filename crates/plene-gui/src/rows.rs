//! Where each row sits. Rows wrap, so their heights differ; they are measured once per
//! column width, and only the rows in view are laid out and painted.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use std::ops::Range;

#[derive(Default)]
pub struct Rows {
    /// The column width the rows were measured at.
    width: Option<f32>,
    /// Each row's top, then the bottom of the last row: one more entry than rows.
    tops: Vec<f32>,
}

impl Rows {
    /// Measures `count` rows at `width` with `height`, unless they were already
    /// measured at that width.
    pub fn measure(&mut self, width: f32, count: usize, mut height: impl FnMut(usize) -> f32) {
        if self.width == Some(width) && self.tops.len() == count + 1 {
            return;
        }
        self.width = Some(width);
        self.tops.clear();
        self.tops.push(0.0);
        for row in 0..count {
            let bottom = self.tops[row] + height(row);
            self.tops.push(bottom);
        }
    }

    pub fn total(&self) -> f32 {
        self.tops.last().copied().unwrap_or(0.0)
    }

    pub fn top(&self, row: usize) -> f32 {
        self.tops[row]
    }

    pub fn bottom(&self, row: usize) -> f32 {
        self.tops[row + 1]
    }

    /// The row at height `y`, if there is one.
    pub fn row_at(&self, y: f32) -> Option<usize> {
        let row = self.tops[1..].partition_point(|bottom| *bottom <= y);
        (y >= 0.0 && row + 1 < self.tops.len()).then_some(row)
    }

    /// The rows that overlap the band from `from` to `to`.
    pub fn visible(&self, from: f32, to: f32) -> Range<usize> {
        let count = self.tops.len().saturating_sub(1);
        let first = self.tops[1..].partition_point(|bottom| *bottom <= from);
        let end = self.tops[..count].partition_point(|top| *top < to);
        first..end.max(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured(heights: &[f32]) -> Rows {
        let mut rows = Rows::default();
        rows.measure(100.0, heights.len(), |row| heights[row]);
        rows
    }

    #[test]
    fn tops_accumulate_heights() {
        let rows = measured(&[10.0, 30.0, 20.0]);
        assert_eq!([rows.top(0), rows.top(1), rows.top(2)], [0.0, 10.0, 40.0]);
        assert_eq!(rows.total(), 60.0);
    }

    #[test]
    fn visible_rows_overlap_the_band() {
        let rows = measured(&[10.0, 30.0, 20.0]);
        assert_eq!(rows.visible(0.0, 5.0), 0..1);
        assert_eq!(rows.visible(10.0, 40.0), 1..2);
        assert_eq!(rows.visible(9.0, 41.0), 0..3);
        assert_eq!(rows.visible(60.0, 90.0), 3..3);
    }

    #[test]
    fn a_height_finds_its_row() {
        let rows = measured(&[10.0, 30.0, 20.0]);
        assert_eq!(rows.bottom(1), 40.0);
        assert_eq!(rows.row_at(0.0), Some(0));
        assert_eq!(rows.row_at(10.0), Some(1));
        assert_eq!(rows.row_at(59.9), Some(2));
        assert_eq!(rows.row_at(60.0), None);
        assert_eq!(rows.row_at(-1.0), None);
    }

    #[test]
    fn no_rows_measure_as_empty() {
        let rows = measured(&[]);
        assert_eq!(rows.total(), 0.0);
        assert_eq!(rows.visible(0.0, 100.0), 0..0);
        assert_eq!(Rows::default().total(), 0.0);
    }

    #[test]
    fn measures_again_only_when_width_or_count_changes() {
        let mut rows = measured(&[10.0, 10.0]);
        let calls = std::cell::Cell::new(0);
        let twenty = |_| {
            calls.set(calls.get() + 1);
            20.0
        };
        rows.measure(100.0, 2, twenty);
        assert_eq!((calls.get(), rows.total()), (0, 20.0));
        rows.measure(80.0, 2, twenty);
        assert_eq!((calls.get(), rows.total()), (2, 40.0));
        rows.measure(80.0, 3, |_| 5.0);
        assert_eq!(rows.total(), 15.0);
    }
}
