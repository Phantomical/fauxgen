use fakerator::{generator, GeneratorExt};

#[generator(yield = u64)]
fn fibonacci() {
    let mut prev = 0u64;
    let mut current = 1;

    loop {
        r#yield!(current);

        prev = current + prev;
        std::mem::swap(&mut current, &mut prev);
    }
}

#[generator(yield = I::Item)]
fn fibonacci_repeat<I>(iter: I)
where
    I: IntoIterator,
    I::Item: Clone,
{
    let fibonnaci = std::pin::pin!(fibonacci());
    for (item, count) in iter.into_iter().zip(fibonnaci.iter()) {
        for _ in 0..count {
            r#yield!(item.clone());
        }
    }
}

fn main() {
    let data = vec!["hi", "there", "one", "two", "three"];
    let fib = std::pin::pin!(fibonacci_repeat(data));

    for item in fib.iter() {
        println!("{item}")
    }
}
