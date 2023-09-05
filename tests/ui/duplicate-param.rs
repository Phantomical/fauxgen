use fauxgen::generator;

#[generator(yield = u32, yield = u64)]
fn mygen() {}

fn main() {}
