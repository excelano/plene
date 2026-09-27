//! Inner doc comment.

enum Shape { Circle }
union Bits { int: u32 }
trait Draw {}
type Id = u64;
macro m($x:expr) { $x }

fn literals() {
    let raw = b'a';
    let bytes = b"bytes";
    let c = c"c string";
    let _ = raw;
    let default = 1;
}
