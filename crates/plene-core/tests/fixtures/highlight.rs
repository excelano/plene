#![allow(dead_code)]

use std::io::{self, Read};

/// A documented struct.
#[derive(Debug, Clone)]
pub struct Point<T> {
    x: T,
}

macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

impl<T: Clone> Point<T> {
    fn read_all(reader: &mut impl Read) -> io::Result<String> {
        let mut text = String::new();
        reader.read_to_string(&mut text)?;
        let big = 3.5 > 2.0 && 'c' < 'd';
        let raw = square!(4) + Vec::<u8>::with_capacity(1).len();
        println!("{} {big} {raw}", self::VALUE);
        Ok(text)
    }
}
