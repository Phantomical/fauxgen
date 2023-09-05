# fauxgen - Fake Generators in Stable Rust

This crate allows you to write your own generators in stable rust. It does this
by building its own generators on top of async-await.

## Getting Started
To get the crate run
```bash
cargo add fauxgen
```

## Writing a Generator
This crate provides two different ways to define generators. The first, and
most convenient, is as a named top-level function:
```rust
#[fauxgen::generator(yield = i32)]
fn generator() {
    r#yield!(1);
    r#yield!(2);
}
```

and the second is as a lambda using the `gen!` macro:
```rust
use fauxgen::GeneratorToken;

let generator = fauxgen::gen!(|token: GeneratorToken<i32>| {
    token.yield_(1).await;
    token.yield_(2).await;
});
```

You can also write async generators:
```rust
use std::time::Duration;

#[fauxgen::generator(yield = u32)]
async fn generator() {
    for i in 0u32..10 {
        tokio::time::sleep(Duration::from_millis(50)).await;
        r#yield!(i * 2);
    }
}
```

## Using a Generator
Simple generators will implement either `Iterator` or `Stream` depending on
whether they are async or not. However, in order to be simple enough to do this
the generator must not return a value or take an argument. Most generators will
likely fall in this category.

Note that because generators are based on async you will need to pin them
before they can be used:

```rust
#[fauxgen::generator(yield = &'static str)]
fn yield_some_words() {
    r#yield!("testing");
    r#yield!("one");
    r#yield!("two");
}

let gen = std::pin::pin!(yield_some_words());
let words: Vec<_> = gen.collect();
assert_eq!(words, vec!["testing", "one", "two"]);
```

## More advanced generator usage
Generators are not restricted to only yielding values or acting as iteerators.
There are actually three different ways to pass values into or out of a
generator:
- the first, which we have already seen, is by yielding a value out of the
  generator,
- the second, is by returning a value at the end of the generator,
- the third, is that external code can pass an argument into the generator
  `resume` function. The yield expression then evaluates to this argument.

These can be used together to do more interesting stream processing type work.
See the examples folder for some ways of using them.

As a simpler example, this generator will yield back the same value passed to
the `resume` call:
```rust
#[fauxgen::generator(yield = T, arg = T)]
fn delay<T>() {
    let mut value = argument!();

    loop {
        value = r#yield!(value);
    }
}
```

Note the use of the `argument!` macro in order to grab the first argument to
`resume`. The rest are returned from the `yield` macro but there is no yield
call for the very first argument.

## See Also
- [genawaiter](https://crates.io/crates/genawaiter) is the original "generators
  on top of async" crate.

