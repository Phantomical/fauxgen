use fauxgen::generator;

#[test]
fn iter_basic() {
    #[generator(yield = i32)]
    pub fn basic_gen() {
        r#yield!(5);
        r#yield!(77);
        r#yield!(256);
    }

    let gen = std::pin::pin!(basic_gen());
    let values: Vec<_> = gen.collect();

    assert_eq!(values, [5, 77, 256]);
}

#[test]
fn iter_with_lifetime() {
    #[generator(yield = &str)]
    pub fn gen_with_lifetim(data: &str) {
        r#yield!(data);
        r#yield!(data);
        r#yield!(data);
    }
}
