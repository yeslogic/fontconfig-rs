fn main() {
    let dlopen = std::env::var("RUST_FONTCONFIG_DLOPEN").is_ok();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
}
