#![allow(dead_code)]

type Return<'a> = &'a str;

#[fauxgen::generator]
fn generator() -> Return<'_> {
    "test"
}

fn main() {}
