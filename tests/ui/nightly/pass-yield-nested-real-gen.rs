#![feature(coroutines, generator_trait)]

#[fauxgen::generator]
fn gen() {
    use std::ops::{Generator, GeneratorState};

    yield;

    let inner = || {
        yield 5;
        return "foo";
    };
    let mut inner = std::pin::pin!(inner);

    assert_eq!(
        inner.as_mut().resume(()),
        GeneratorState::Yielded(5)
    );
    assert_eq!(
        inner.as_mut().resume(()),
        GeneratorState::Complete("foo")
    );
}

fn main() {
    let gen = std::pin::pin!(gen());
    for _ in gen {}
}