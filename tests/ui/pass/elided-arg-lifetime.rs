type Arg<'a> = &'a str;

#[fauxgen::generator(arg = Arg<'_>)]
fn gen() {
    let _arg: &str = r#yield!();
}

fn main() {}
