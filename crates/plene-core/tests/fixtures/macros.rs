// The arguments of standard macros that take expressions are expanded.
fn listed(a: &mut Vec<u8>, f: &mut Formatter<'_>) -> Result {
    assert!(!a.is_empty(), "{} left", a.len());
    assert_eq!(a.first(), Some(&1));
    debug_assert_ne!(*a, vec![0; 3]);
    let halves = vec![&a[..2], &a[2..]];
    let masks = vec! { |x: u8| x & 1, |x: u8| x >> 1 };
    println!();
    let total = std::format!("{n}", n = a.iter().map(|b| *b as u32).sum::<u32>());
    write!(
        f,
        "{}", // the count
        a.len()
    )?;
    Ok(())
}

// Arguments that do not parse as expressions, and other macros, are left alone.
fn unlisted(a: &u8) {
    format!("{}", type = &a);
    stringify!(&a);
    assert_eq!(&a, T![_]);
    assert!(matches!(a, &1 | &2));
}
