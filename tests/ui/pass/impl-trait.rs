use fauxgen::*;

/// Validate that parameters can use impl Trait types.
#[generator]
fn gen(func: impl FnOnce()) {
    r#yield!(func());
}

fn main() {}
