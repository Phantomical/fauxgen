#![feature(generators)]

#[fauxgen::generator(yield = i32)]
fn gen() {
    yield;
}

fn main() {}
