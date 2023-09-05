#![feature(generators, generator_trait)]
#![deny(deprecated)]

macro_rules! delay {
    ($value:expr) => { () };
}

#[fauxgen::generator]
fn gen() {
    let _ = delay!(yield);
}

fn main() {}
