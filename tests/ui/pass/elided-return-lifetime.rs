#![allow(dead_code)]

type Return<'a> = &'a str;

#[fauxgen::generator]
fn gen() -> Return<'_> {
    "test"
}

fn main() {}
