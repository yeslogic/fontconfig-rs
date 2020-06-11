fontconfig-rs
=============

<div align="center">
  <a href="https://travis-ci.com/yeslogic/fontconfig-rs">
    <img src="https://travis-ci.com/yeslogic/fontconfig-rs.svg?branch=master" alt="Build Status"></a>
  <a href="https://docs.rs/yeslogic-fontconfig">
    <img src="https://docs.rs/yeslogic-fontconfig/badge.svg" alt="Documentation">
  </a>
  <a href="https://crates.io/crates/yeslogic-fontconfig">
    <img src="https://img.shields.io/crates/v/yeslogic-fontconfig.svg" alt="Version">
  </a>
  <a href="https://github.com/yeslogic/fontconfig-rs/blob/master/LICENSE">
    <img src="https://img.shields.io/crates/l/yeslogic-fontconfig.svg" alt="License">
  </a>
</div>

<br>

A wrapper around [freedesktop.org's Fontconfig library][homepage], for locating fonts on a UNIX like systems such as Linux and FreeBSD. Requires Fontconfig to be installed.

Dependencies
------------

* Arch Linux: `fontconfig`
* Debian-based systems: `libfontconfig1-dev`
* FreeBSD: `fontconfig`
* Void Linux: `fontconfig-devel`

Usage
-----

`main.rs`:

```rust
use fontconfig::Fontconfig;

fn main() {
    let fc = Fontconfig::new().unwrap();
    // `Fontconfig::find()` returns `Option` (will rarely be `None` but still could be)
    let font = fc.find("freeserif", None).unwrap();
    // `name` is a `String`, `path` is a `Path`
    println!("Name: {}\nPath: {}", font.name, font.path.display());
}
```

You could then, for example, use `font.path` to create a `GlyphCache` from [`opengl_graphics`][gl] and pass it to [`conrod`][conrod].

Other Fontconfig Crates
-----------------------

* [servo-fontconfig] — This crate provides a low-level interface only. It
  depends on [servo-fontconfig-sys], which will fall back to building a
  vendored version of Fontconfig if a compatible version can't be found. It
  in-turn depends on [expat-sys], which does the same thing regarding a vendored
  version of Expat. This makes it easier if you are distributing a code base
  that needs Fontconfig, but provides less control over the libraries that will
  be used.
* [fontconfig-sys] — superceded by [yeslogic-fontconfig-sys].
* [yeslogic-fontconfig] — This crate was previously published under this name before we were given to publish it as [fontconfig].

For our needs in [Prince] we wanted higher-level bindings that did not fall back on vendored versions of libraries, which is what the crates in this repo provide.

Credits
-------

Thanks to [Austin Bonander][abonander] for originally creating the
`fontconfig` crate and [allowing us to publish ours under that
name](https://github.com/abonander/fontconfig-rs/issues/9).

[conrod]: https://github.com/PistonDevelopers/conrod
[expat-sys]: https://crates.io/crates/expat-sys
[fontconfig-sys]: https://crates.io/crates/fontconfig-sys
[fontconfig]: https://crates.io/crates/fontconfig
[gl]: https://github.com/PistonDevelopers/opengl_graphics
[homepage]: https://www.freedesktop.org/wiki/Software/fontconfig/
[Prince]: https://www.princexml.com/
[servo-fontconfig-sys]: https://crates.io/crates/servo-fontconfig-sys
[servo-fontconfig]: https://crates.io/crates/servo-fontconfig
[yeslogic-fontconfig]: https://crates.io/crates/yeslogic-fontconfig
[yeslogic-fontconfig-sys]: https://crates.io/crates/yeslogic-fontconfig-sys
[abonander]: https://github.com/abonander
