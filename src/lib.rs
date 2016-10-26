//! A wrapper around freedesktop's `fontconfig` utility, for locating fontfiles on a
//! Linux-based system. Requires `libfontconfig` to be installed.
//!
//! Use `Font` for a high-level search by family name and optional style (e.g. "FreeSerif"
//! and "italic"), and `Pattern` for a more in-depth search.
//!
//! See the [fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html

#![feature(phase, unsafe_destructor)]

extern crate fontconfig_sys as fontconfig;
#[phase(plugin, link)] extern crate log;

use fontconfig::FcPattern;

use std::c_str::CString;
use std::kinds::marker;
use std::mem;
use std::ptr;
use std::sync::{Once, ONCE_INIT};

static FC_INIT: Once = ONCE_INIT;

fn fc_init() {
    FC_INIT.doit(|| assert_eq!(unsafe { fontconfig::FcInit() }, 1));
}

/// A very high-level view of a font, only concerned with the name and its file location.
///
/// ##Example
/// ```rust,ignore
/// let font = Font::find("freeserif", Some("italic")).unwrap();
/// println!("Name: {}\nPath: {}", font.name, font.path.display());
/// ```
#[allow(dead_code)]
pub struct Font {
    /// The true name of this font
    pub name: String,
    /// The location of this font on the filesystem.
    pub path: Path,
}

impl Font {
    /// Find a font of the given `family` (e.g. Dejavu Sans, FreeSerif),
    /// optionally filtering by `style`. Both fields are case-insensitive.
    pub fn find(family: &str, style: Option<&str>) -> Option<Font> {
        let mut pat = Pattern::new();
        pat.add_string("family", family);

        if let Some(style) = style {
            pat.add_string("style", style);
        }

        let font_match = pat.font_match();

        font_match.name().and_then(|name| font_match.filename().map(|filename|
            Font {
                name: name.into_string(),
                path: Path::new(filename),
            }
        ))
    }

    #[allow(dead_code)]
    fn print_debug(&self) {
        debug!("Name: {}\nPath: {}", self.name, self.path.display());
    }
}

/// A safe wrapper around fontconfig's `FcPattern`.
pub struct Pattern<'a> {
    _m: marker::ContravariantLifetime<'a>,
    pat: *mut FcPattern,
    /// This is just to hold the RAII C-strings while the `FcPattern` is alive.
    strings: Vec<CString>,
}

impl<'a> Pattern<'a> {
    pub fn new() -> Pattern<'a> {
        fc_init();

        Pattern {
            _m: marker::ContravariantLifetime,
            pat: unsafe{ fontconfig::FcPatternCreate() },
            strings: Vec::new(),
        }
    }

    fn from_pattern(pat: *mut FcPattern) -> Pattern<'a> {
        unsafe { fontconfig::FcPatternReference(pat); }

        Pattern {
            _m: marker::ContravariantLifetime,
            pat: pat,
            strings: Vec::new(),
        }
    }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &str, val: &str) {
        let c_name = name.to_c_str();

        // `val` is copied inside fontconfig so no need to allocate it again.
        val.with_c_str(|c_str| unsafe {
            fontconfig::FcPatternAddString(self.pat, c_name.as_ptr(), c_str as *const u8);
        });

        self.strings.push(c_name);
    }

    /// Get the value for a key from this pattern.
    pub fn get_string<'a>(&'a self, name: &str) -> Option<&'a str> {
        name.with_c_str(|c_str| unsafe {
            let ret = mem::uninitialized();
            if fontconfig::FcPatternGetString(&*self.pat, c_str, 0, ret) == 0 {
                Some(std::str::from_c_str(*ret as *const i8))
            } else { None }
        })
    }

    /// Print this pattern to stdout with all its values.
    #[allow(dead_code)]
    pub fn print(&self) {
        unsafe { fontconfig::FcPatternPrint(&*self.pat); }
    }

    fn default_substitute(&mut self) {
        unsafe { fontconfig::FcDefaultSubstitute(self.pat); }
    }

    fn config_substitute(&mut self) {
        unsafe { fontconfig::FcConfigSubstitute(ptr::null_mut(), self.pat, fontconfig::FcMatchPattern); }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match<'a>(&'a mut self) -> Pattern<'a> {
        self.default_substitute();
        self.config_substitute();

        unsafe {
            let mut res = fontconfig::FcResultNoMatch;
            Pattern::from_pattern(
                fontconfig::FcFontMatch(ptr::null_mut(), self.pat, &mut res)
            )
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Option<&str> {
        self.get_string("fullname")
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.get_string("file")
    }
}

#[unsafe_destructor]
impl<'a> Drop for Pattern<'a> {
    fn drop(&mut self) {
        unsafe { fontconfig::FcPatternDestroy(self.pat); }
    }
}

#[test]
fn it_works() {
    fc_init();
}

#[test]
fn test_find_font() {
    Font::find("dejavu sans", None).unwrap().print_debug();
    Font::find("dejavu sans", Some("oblique")).unwrap().print_debug();
}

