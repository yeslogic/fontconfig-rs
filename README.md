fontconfig-rs
=============

<div align="center">
  <a href="https://travis-ci.com/yeslogic/fontconfig">
    <img src="https://travis-ci.com/yeslogic/fontconfig.svg?token=4GA6ydxNNeb6XeELrMmg&amp;branch=master" alt="Build Status"></a>
  <a href="https://docs.rs/fontconfig">
    <img src="https://docs.rs/fontconfig/badge.svg" alt="Documentation">
  </a>
  <a href="https://crates.io/crates/fontconfig">
    <img src="https://img.shields.io/crates/v/fontconfig.svg" alt="Version">
  </a>
  <a href="https://github.com/yeslogic/fontconfig/blob/master/LICENSE">
    <img src="https://img.shields.io/crates/l/fontconfig.svg" alt="License">
  </a>
</div>

<br>

A wrapper around [freedesktop.org's fontconfig library][homepage], for locating fonts on a UNIX like systems such as Linux and FreeBSD. Requires fontconfig to be installed.

Dependencies
============

* Arch Linux: `fontconfig`
* Debian-based systems: `libfontconfig1-dev`
* FreeBSD: `fontconfig`
* Void Linux: `fontconfig-devel`

Usage
=====

Cargo.toml:

```toml
[dependencies]
fontconfig = "0.1.0"
```

main.rs:

```rust
extern crate fontconfig;

use fontconfig::Font;

fn main() {
    // `Font::find()` returns `Option` (will rarely be `None` but still could be)
    let font = Font::find("freeserif", None).unwrap();
    // `name` is a `String`, `path` is a `Path`
    println!("Name: {}\nPath: {}", font.name, font.path.display());
}
```

You could then, for example, use `font.path` to create a `GlyphCache` from [`opengl_graphics`][gl]
and pass it to [`conrod`][conrod].

[gl]: https://github.com/PistonDevelopers/opengl_graphics
[conrod]: https://github.com/PistonDevelopers/conrod
[homepage]: https://www.freedesktop.org/wiki/Software/fontconfig/
