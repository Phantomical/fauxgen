use rustc_version::Channel;

fn main() {
    let version = rustc_version::version_meta().unwrap();

    if version.channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }
}
