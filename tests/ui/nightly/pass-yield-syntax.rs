#![feature(generators)]

#[fauxgen::generator(yield = i32)]
fn gen() {
    yield 5;
    yield 6;
    yield 7;
}

fn main() {
    let gen = std::pin::pin!(gen());
    let values: Vec<_> = gen.collect();

    assert_eq!(values, [5, 6, 7]);
}
