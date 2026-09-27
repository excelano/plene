fn ranges(v: &[u8], n: usize) -> u8 {
    for i in 0..n {}
    for i in 1..=n {}
    let tail = &v[1..];
    let head = &v[..n];
    let through = &v[..=n];
    let all = &v[..];
    let count = (2..n).len();
    match n {
        0 => 0,
        1..=9 => 1,
        10..20 => 2,
        100.. => 3,
        _ => 4,
    }
}
