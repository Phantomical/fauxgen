
#[fauxgen::generator(arg = i32)]
fn gen() {
    let value: &str = argument!();
}

fn main() {}
