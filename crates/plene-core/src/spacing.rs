//! Spaces inserted so that an expansion reads as a separate word.

use crate::{HighlightClass, Span};

/// Characters that attach to a following word without a space, as in `(borrow x)`.
const HUGS_BEFORE: &str = "([{<";
/// Characters that attach to a preceding word without a space, as in `lifetime a,`
/// and `public(crate)`.
const HUGS_AFTER: &str = ")]}>,;:.(";

/// Inserts a space span wherever an expansion's word edge touches a character that is
/// neither whitespace nor punctuation that hugs the word from that side.
pub(crate) fn insert_spaces(spans: &mut Vec<Span>) {
    let mut spaced = Vec::with_capacity(spans.len());
    for span in spans.drain(..) {
        if let Some(previous) = spaced.last()
            && needs_space(previous, &span)
        {
            let at = span.source_range.start;
            spaced.push(Span {
                original: String::new(),
                rendered: " ".to_string(),
                class: HighlightClass::Plain,
                role: None,
                source_range: at..at,
            });
        }
        spaced.push(span);
    }
    *spans = spaced;
}

fn needs_space(left: &Span, right: &Span) -> bool {
    let (Some(left_edge), Some(right_edge)) = (
        left.rendered.chars().next_back(),
        right.rendered.chars().next(),
    ) else {
        return false;
    };
    let word_ends_left = is_expanded(left)
        && is_word(left_edge)
        && !right_edge.is_whitespace()
        && !HUGS_AFTER.contains(right_edge);
    let word_starts_right = is_expanded(right)
        && is_word(right_edge)
        && !left_edge.is_whitespace()
        && !HUGS_BEFORE.contains(left_edge);
    word_ends_left || word_starts_right
}

fn is_expanded(span: &Span) -> bool {
    span.rendered != span.original
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
