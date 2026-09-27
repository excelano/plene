use std::collections::HashMap;

// A line comment.
pub fn count<'a>(words: &[&'a str]) -> HashMap<&'a str, usize> {
    let mut counts = HashMap::new();
    for word in words {
        *counts.entry(*word).or_insert(0) += 1;
    }
    counts
}
