
use fauxgen::generator;

#[test]
fn iter_yield() {
    #[generator(yield = i32)]
    pub fn yield_gen() {
        yield 5;
        yield 77;
        yield 256;
    }

    let gen = std::pin::pin!(yield_gen());
    let values: Vec<_> = gen.collect();

    assert_eq!(values, [5, 77, 256]);
}
