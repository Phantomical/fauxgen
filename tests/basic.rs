use genawaiter::{generator, GeneratorExt};

#[generator(yield = i32)]
pub fn basic_gen() {
    r#yield!(5);
    r#yield!(77);
    r#yield!(256);
}

#[test]
fn iter_basic() {
    let gen = std::pin::pin!(basic_gen());
    let values: Vec<_> = gen.iter().collect();

    assert_eq!(values, [5, 77, 256]);
}
