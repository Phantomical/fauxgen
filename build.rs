use rustc_version::Channel;

fn main() {
    let version = rustc_version::version_meta().unwrap();

    if cfg!(feature = "unstable_nightly") && version.channel == Channel::Nightly {
        println!("cargo:rustc-cfg=nightly");
    }

    println!("cargo:rustc-check-cfg=cfg(nightly)");
    println!("cargo:rustc-check-cfg=cfg(std_generators)");
}
