use fauxgen::{gen, GeneratorToken};

fn main() {
    let gen = gen!(|token: GeneratorToken<_>| async move {
        token.yield_(32).await;
        token.yield_(5).await;
    });
    let gen = std::pin::pin!(gen);

    let vals: Vec<_> = gen.collect();
    assert_eq!(vals, [32, 5]);
}
