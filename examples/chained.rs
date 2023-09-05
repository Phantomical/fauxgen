use std::pin::Pin;

use fauxgen::{generator, Generator, GeneratorIter, GeneratorState};
use pin_project::pin_project;

#[pin_project]
pub struct ChainedGenerator<G1, G2> {
    #[pin]
    gen1: G1,
    #[pin]
    gen2: G2,
}

impl<G1, G2> ChainedGenerator<G1, G2> {
    pub fn new(gen1: G1, gen2: G2) -> Self {
        Self { gen1, gen2 }
    }
}

impl<A, R, G1, G2> Generator<A> for ChainedGenerator<G1, G2>
where
    G1: Generator<A, Return = R>,
    G2: Generator<G1::Yield, Return = R>,
{
    type Yield = G2::Yield;
    type Return = R;

    #[inline(never)]
    fn resume(self: Pin<&mut Self>, arg: A) -> GeneratorState<Self::Yield, Self::Return> {
        let this = self.project();
        let value = match this.gen1.resume(arg) {
            GeneratorState::Yielded(value) => value,
            GeneratorState::Complete(value) => return GeneratorState::Complete(value),
        };

        this.gen2.resume(value)
    }
}

#[generator(yield = u64)]
fn powers_of_two() {
    let mut value = 1;

    while value != 0 {
        r#yield!(value);
        value <<= 1;
    }
}

#[generator(yield = u32, arg = u64)]
fn leading_zeros() {
    let mut value = argument!();

    loop {
        value = r#yield!(value.leading_zeros());
    }
}

fn main() {
    let chain = std::pin::pin!(ChainedGenerator::new(powers_of_two(), leading_zeros()));

    for value in GeneratorIter::new(chain) {
        println!("{value}");
    }
}
