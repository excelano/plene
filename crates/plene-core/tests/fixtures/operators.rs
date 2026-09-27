fn bits(a: u32, b: u32, flags: &mut u32) -> u32 {
    let masked = a & b;
    let merged = a | b;
    let toggled = a ^ b;
    let shifted = (a << 2) >> 1;
    *flags &= !a;
    *flags |= b;
    *flags ^= 0xff;
    *flags <<= 1;
    *flags >>= 2;
    let product = a * b;
    let negated = -(a as i32);
    let unequal = a != b && !(a > b) || b == 0;
    masked + merged + toggled + shifted + product
}

fn pointers(p: *const u8, q: *mut u8, r: &&u8) -> u8 {
    unsafe { *p + *q + **r }
}

struct Handle;
impl !Send for Handle {}

fn nested() -> Vec<Vec<u8>> {
    Vec::<Vec<u8>>::new()
}
