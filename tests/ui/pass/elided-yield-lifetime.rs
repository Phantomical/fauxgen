#![allow(dead_code)]

type Yield<'a> = &'a str;

#[fauxgen::generator(yield = Yield<'_>)]
fn gen() {
    r#yield!("test");
}

fn main() {}
