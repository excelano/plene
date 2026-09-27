trait Shape: Clone + Send {
    type Corner: Copy;
}

fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
    items[0]
}

fn longest<'a, 'b: 'a>(x: &'a str, y: &'b str) -> &'a str {
    x
}

fn hold<T: ?Sized + 'static>(value: Box<T>) {}

fn show<T>(value: T) -> String
where
    T: std::fmt::Display + 'static,
    for<'c> &'c T: Into<String>,
{
    value.to_string()
}

fn boxed(error: Box<dyn std::error::Error + Send + Sync>) {}

fn items() -> impl Iterator<Item = u8> + Clone {
    [1].into_iter()
}

struct Holder<'a, T: 'a> {
    value: &'a T,
}

fn assoc<I: Iterator<Item: Copy>>(iter: I) {}
