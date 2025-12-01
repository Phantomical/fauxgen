use fauxgen::*;

/// Validate that parameters can use impl Trait types.
#[generator]
fn generator(func: impl FnOnce()) {
    r#yield!(func());
}

fn main() {}
