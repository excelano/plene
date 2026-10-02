impl Reader {
    fn first(&self) -> &str {
        self.text
    }

    fn pick(&mut self, y: &u8) -> &mut u8 {
        &mut self.byte
    }

    fn named<'a>(&'a self) -> &u8 {
        &self.byte
    }

    fn typed(self: &Self) -> Option<&u8> {
        None
    }
}

fn one(x: &u8) -> &u8 {
    x
}

fn nested(x: Vec<&u8>) -> Option<&u8> {
    x.first()
}

fn through(x: Foo<'_>, n: usize) -> impl Iterator<Item = &u8> {
    x.items()
}

fn static_one(name: &'static str) -> &str {
    name
}

fn pointer(x: &u8, f: fn(&u8) -> &u8) -> &u8 {
    f(x)
}

fn two(x: &u8, y: &u8) -> &u8 {
    x
}

fn none() -> &'static u8 {
    &0
}

fn untouched(x: Foo) -> &u8 {
    x.get()
}

fn destructured((a, b): (&u8, u8)) -> &u8 {
    a
}

fn returns_pointer(x: &u8) -> fn(&u8) -> &u8 {
    helper
}

fn closure_bound(f: impl Fn(&u8) -> &u8) {}
