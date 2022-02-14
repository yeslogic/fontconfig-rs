fn main() {
    let dlopen = std::env::var("RUST_FONTCONFIG_DLOPEN").is_ok();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
    if !(dlopen || cfg!(feature = "dlopen")) {
        pkg_config::find_library("fontconfig").unwrap();
    }
}
