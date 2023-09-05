use std::time::Duration;

use fauxgen::{gen, GeneratorToken};
use futures_util::StreamExt;

#[test]
fn basic_sync() {
    let gen = std::pin::pin!(gen!(|token: GeneratorToken<_>| async move {
        token.yield_(32).await;
        token.yield_(5).await;
    }));

    let vals: Vec<_> = gen.collect();
    assert_eq!(vals, [32, 5]);
}

#[tokio::test]
async fn basic_async() {
    let mut gen = std::pin::pin!(gen!(async |token: GeneratorToken<_>| async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        token.yield_(888).await;
    }));

    while let Some(item) = gen.next().await {
        assert_eq!(item, 888);
    }
}
