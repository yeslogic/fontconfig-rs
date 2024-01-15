fontconfig
==========

<div align="center">
  <a href="https://github.com/yeslogic/fontconfig-rs/actions/workflows/ci.yml">
    <img src="https://github.com/yeslogic/fontconfig-rs/actions/workflows/ci.yml/badge.svg" alt="Build Status"></a>
  <a href="https://docs.rs/fontconfig">
    <img src="https://docs.rs/fontconfig/badge.svg" alt="Documentation">
  </a>
  <a href="https://crates.io/crates/fontconfig">
    <img src="https://img.shields.io/crates/v/fontconfig.svg" alt="Version">
  </a>
  <a href="https://github.com/yeslogic/fontconfig-rs/blob/master/LICENSE">
    <img src="https://img.shields.io/crates/l/fontconfig.svg" alt="License">
  </a>
</div>

<br>

A wrapper around [freedesktop.org's Fontconfig library][homepage], for locating fonts on UNIX-like systems such as Linux and BSD. Requires Fontconfig to be installed.

Dependencies
------------

* Alpine Linux: `fontconfig-dev`
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

### Cargo Features

| Feature       | Description                       | Default Enabled | Extra Dependencies    |
|---------------|-----------------------------------|:---------------:|-----------------------|
| `dlopen`      | [dlopen] libfontconfig at runtime |        ❌       |                       |

The `dlopen` feature enables building this crate without dynamically linking to the Fontconfig C library at link time. Instead, Fontconfig will be dynamically loaded at runtime with the [dlopen] function. This can be useful in cross-compiling situations as you don't need to have a version of Fontcofig available for the target platform available at compile time.

Other Fontconfig Crates
-----------------------

* [servo-fontconfig] — This crate provides a low-level interface only. It
  depends on [servo-fontconfig-sys], which will fall back to building a
  vendored version of Fontconfig if a compatible version can't be found. It
  in-turn depends on [expat-sys], which does the same thing regarding a vendored
  version of Expat. This makes it easier if you are distributing a code base
  that needs Fontconfig, but provides less control over the libraries that will
  be used.
* [fontconfig-sys] — superseded by [yeslogic-fontconfig-sys].
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
[dlopen]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dlopen.html
[dlib]: https://crates.io/crates/dlib
