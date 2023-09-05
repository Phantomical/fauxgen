use std::time::Duration;

use fauxgen::generator;

#[generator(yield = u32, arg = u32)]
async fn delay(start: u32) {
    let mut current = start;

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;

        current = r#yield!(current);
        current = r#yield!(current);
    }
}

#[tokio::test]
async fn test_async_gen() {
    let mut gen = std::pin::pin!(delay(32));

    for i in 0..10 {
        println!("{:?}", gen.as_mut().resume(i).await);
    }
}
