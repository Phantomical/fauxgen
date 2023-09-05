use fauxgen::generator;

#[generator(yield = T)]
fn repeat<T: Clone>(value: T, count: usize) {
    for _ in 0..count {
        r#yield!(value.clone());
    }
}

fn main() {
    for val in std::pin::pin!(repeat("hi there", 3)) {
        println!("{val}");
    }
}
