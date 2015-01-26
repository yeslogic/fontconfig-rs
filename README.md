fontconfig-rs
=============

A wrapper around freedesktop's fontconfig utility, for locating fontfiles on a Linux-based system. Requires libfontconfig to be installed.

Prerequisites
========

####Ubuntu-based system:
```shell
sudo apt-get install libfontconfig libfontconfig1-dev
```

Usage
=====

Cargo.toml:
```toml
[dependencies]
fontconfig = "*"
```

main.rs:
```rust
extern crate fontconfig;

use fontconfig::Font;

fn main() {
    `Font::find()` returns `Option` (will rarely be `None` but still could be)
    let font = Font::find("freeserif", None).unwrap();
    // `name` is a `String`, `path` is a `Path`
    println!("Name: {}\nPath: {}", font.name, font.path.display());
}
```

You could then, for example, use `font.path` to create a `GlyphCache` from [`opengl_graphics`][gl]
and pass it to [`conrod`][conrod].

[gl]: https://github.com/PistonDevelopers/opengl_graphics
[conrod]: https://github.com/PistonDevelopers/conrod

Documentation
=============

**TODO**: RustCI integration

```shell
git clone https://github.com/cybergeek94/fontconfig-rs
cd fontconfig-rs
cargo doc --open
```
