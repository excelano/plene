mod parser;
extern crate alloc;

pub trait Shape {}
pub struct Square;

impl Shape for Square {}
impl<T: Clone> Wrapper<T> {}
unsafe impl Send for Square {}

unsafe extern "C" {
    fn abs(x: i32) -> i32;
}

extern "C" fn callback() {}

type Shapes = impl Iterator<Item = u8>;

fn shapes(items: &[Box<dyn Shape>]) -> impl Iterator<Item = u8> {
    let Some(ref mut first) = items.first() else { return std::iter::empty() };
    let ref second = items[1];
    std::iter::empty()
}

fn show(shape: impl Shape + Send, action: &dyn Fn(u8) -> u8) {}
