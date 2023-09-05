#![feature(generators)]

use fauxgen::Generator;

#[fauxgen::generator(arg = i32)]
fn printer() {
    for _ in 0..3 {
        let _value = yield;
    }
}

fn main() {
    let mut gen = std::pin::pin!(printer());

    gen.as_mut().resume(5);
    gen.as_mut().resume(6);
}
