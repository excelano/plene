pub fn roles<'a>(&mut self, text: &'a str, items: &[u8]) -> Result<&'static str, Error> {
    let mut total = 0;
    let &first = &items[0];
    let bytes = read(text)?;
    let length = parse(text)?.len();
    let field = parse(text)?.value;
    let entry = lookup(text)?[0];
    let joined = fetch(text)?.await;
    let add = |x: u8, y: u8| -> u8 { x + y };
    let noop = || {};
    let any: &'_ str = text;
    'outer: for item in items {
        if *item == 0 {
            continue 'outer;
        }
        break 'outer;
    }
    total
}

// Tokens in macro and attribute arguments take no role.
fn unroled(a: u8) {
    println!("{}", &a);
    #[route(&mut state, fn() -> u8)]
    let attributed = a;
}
