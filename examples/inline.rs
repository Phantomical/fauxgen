use fauxgen::{gen, GeneratorToken};

fn main() {
    let gen = gen!(|token: GeneratorToken<_>| {
        token.yield_(32).await;
        token.yield_(5).await;
    });
    let gen = std::pin::pin!(gen);

    let vals: Vec<_> = gen.collect();
    assert_eq!(vals, [32, 5]);
}

fn test2() {
    use fauxgen::{gen, GeneratorToken};

    let gen = std::pin::pin!(gen!(|token: GeneratorToken<_>| {
        token.yield_(5i32).await;
        token.yield_(6).await;
        token.yield_(77).await;
    }));

    let vals: Vec<i32> = gen.collect();
    assert_eq!(vals, [5, 6, 77]);
}
