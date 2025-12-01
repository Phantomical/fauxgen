#![allow(dead_code)]

type Arg<'a> = &'a str;

#[fauxgen::generator(arg = Arg<'_>)]
fn generator() {
    let _arg: &str = argument!();
}

fn main() {}
