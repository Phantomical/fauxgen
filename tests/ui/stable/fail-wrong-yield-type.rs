
#[fauxgen::generator(yield = i32)]
fn gen() {
    r#yield!("test");
}

fn main() {}
