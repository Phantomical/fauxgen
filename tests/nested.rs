use std::future::Future;

use fakerator::{generator, GeneratorState};

#[generator(yield = u32)]
async fn outer() {
    let gen = std::pin::pin!(inner(|| async move { r#yield!(5) }));
    let _ = gen.resume(()).await;
}

#[generator(yield = u32)]
async fn inner<F>(func: impl FnOnce() -> F)
where
    F: Future<Output = ()>,
{
    func().await;
}

#[tokio::test]
#[should_panic]
async fn nested_gens() {
    let mut gen = std::pin::pin!(outer());
    while let GeneratorState::Yield(_) = gen.as_mut().resume(()).await {}
}
