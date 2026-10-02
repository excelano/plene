//! Where an item's body can be folded away.
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

use ra_ap_syntax::SourceFile;
use ra_ap_syntax::SyntaxKind::*;

use crate::Edition;
use crate::role::{self, Role};

/// The body of an item that spans lines: the line of its opening brace, which stays in
/// view, and the line of its closing brace, which goes with the lines between.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fold {
    /// Counted from 0, as the lines `transcribe` returns are.
    pub first: usize,
    pub last: usize,
    /// Whether the body is a function's, as opposed to an `impl`, trait, module, struct
    /// or enum.
    pub function: bool,
}

/// The folds in `source`, outermost first where they begin on the same line, in the
/// order they begin. A body inside an error node or a macro is not found.
pub fn folds(source: &str, edition: Edition) -> Vec<Fold> {
    let parse = SourceFile::parse(source, edition.to_ra());
    let line_starts: Vec<usize> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(at, _)| at + 1))
        .collect();
    let line_of = |offset: usize| line_starts.partition_point(|start| *start <= offset) - 1;
    let mut folds = Vec::new();
    for block in parse.syntax_node().descendants() {
        let Some((role, _)) = role::item_body(&block) else {
            continue;
        };
        let Some(close) = block.last_token().filter(|token| token.kind() == R_CURLY) else {
            continue;
        };
        if block.ancestors().any(|node| node.kind() == ERROR) {
            continue;
        }
        let first = line_of(usize::from(block.text_range().start()));
        let last = line_of(usize::from(close.text_range().start()));
        if last > first {
            folds.push(Fold {
                first,
                last,
                function: role == Role::FnEnd,
            });
        }
    }
    folds.sort_by_key(|fold| (fold.first, std::cmp::Reverse(fold.last)));
    folds
}
