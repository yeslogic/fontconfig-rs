extern crate pkg_config;

const FONTCONFIG_VERSION: &str = "2.11.1";

fn main() {
    pkg_config::Config::new()
        .atleast_version(FONTCONFIG_VERSION)
        .find("fontconfig")
        .expect(&format!(
            "unable to find fontconfig {} or newer with pkg-config",
            FONTCONFIG_VERSION
        ));
}
