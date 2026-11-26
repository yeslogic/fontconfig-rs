fn main() {
    println!("cargo:rerun-if-env-changed=RUST_FONTCONFIG_DLOPEN");
    let dlopen = std::env::var_os("RUST_FONTCONFIG_DLOPEN").is_some();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
    if !(dlopen || cfg!(feature = "dlopen")) {
        let library = pkg_config::find_library("fontconfig").unwrap();
        // Rerun when a versioned link dir (e.g. a Homebrew Cellar) is removed.
        // Missing ones are skipped, as they would force a rerun on every build.
        for dir in library.link_paths.iter().filter(|d| d.exists()) {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }
}
