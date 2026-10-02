macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {{
        let mut m = HashMap::new();
        $(m.insert($k, $v);)*
        m
    }};
    ($x:ident : $t:ty, $($rest:tt)+) => { let $x: $t; };
    ($v:vis struct $n:ident;) => { $v struct $n; };
    ($b:block $i:item $p:path $l:lifetime $m:meta $lit:literal) => {};
    ($a:pat_param | $s:stmt) => { (1 + 2) * 3 + $a };
}
fn f() { let x = 2 * 3; assert!(a?.b + c * d); }
