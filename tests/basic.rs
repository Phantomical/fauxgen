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
#[cfg(not(all()))]
fn iter_with_lifetime() {
    #[generator(yield = &str)]
    pub fn gen_with_lifetim(data: &str) {
        r#yield!(data);
        r#yield!(data);
        r#yield!(data);
    }

    pub fn gen_with_lifetim2(
        data: &str,
    ) -> ::fauxgen::export::SyncGenerator<impl ::fauxgen::__private::Future<Output = ()>, &str, ()>
    {
        let __token = ::fauxgen::__private::token();
        ::fauxgen::__private::gen_sync(__token.marker(), async move {
            let __token = std::pin::pin!(__token);
            let __token = __token.as_ref();
            ::fauxgen::__private::register(__token).await;
            {
                __token.yield_(data).await;
                __token.yield_(data).await;
                __token.yield_(data).await;
            }
        })
    }
}
