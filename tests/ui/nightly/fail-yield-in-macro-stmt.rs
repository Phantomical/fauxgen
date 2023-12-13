#![feature(coroutines, generator_trait)]
#![deny(deprecated)]

macro_rules! delay {
    ($value:expr) => { () };
}

#[fauxgen::generator]
fn gen() {
    delay!(yield);
}

fn main() {}
