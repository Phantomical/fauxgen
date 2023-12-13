#![allow(dead_code)]

mod dummy {
    pub use fauxgen::*;

    pub type Yield = u32;
    pub type Arg = u64;
    pub type Return = &'static str;
}

#[fauxgen::generator(
    crate = crate::dummy,
    yield = crate::dummy::Yield,
    arg = crate::dummy::Arg
)]
fn gen() -> crate::dummy::Return {
    "test"
}

fn main() {}
