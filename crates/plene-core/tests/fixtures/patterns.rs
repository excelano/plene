fn classify(value: Option<u8>, point: Point, items: &[u8]) -> u8 {
    let _ = cleanup();
    let (first, _) = pair();
    let numbers: Vec<_> = items.iter().collect();
    _ = flush();
    match value {
        Some(0) | None => 0,
        | Some(1) | Some(2) => 1,
        Some(n @ 3..=9) => n,
        Some(_) => 2,
    }
    let Point { x, .. } = point;
    let (head, ..) = triple();
    let [start, rest @ ..] = items else { return 0 };
    let moved = Point { x: 1, ..point };
    for _ in items {}
    items.iter().map(|_| 1).count();
    x
}

fn ignored(_: u8, _unused: u8) {}
use std::fmt::Write as _;
