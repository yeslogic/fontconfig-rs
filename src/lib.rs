//! A wrapper around freedesktop's `fontconfig` utility, for locating fontfiles on a
//! Linux-based system. Requires `libfontconfig` to be installed.
//!
//! Use `Font` for a high-level search by family name and optional style (e.g. "FreeSerif"
//! and "italic"), and `Pattern` for a more in-depth search.
//!
//! See the [fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html

extern crate fontconfig_sys;
extern crate log;

use fontconfig_sys::FcPattern;

use std::ffi::{CString, CStr};
use std::marker::PhantomData;
use std::mem;
use std::ptr;
use std::sync::{Once, ONCE_INIT};
use std::path::PathBuf;

static FC_INIT: Once = ONCE_INIT;

fn fc_init() {
    FC_INIT.call_once(|| assert_eq!(unsafe { fontconfig_sys::FcInit() }, 1));
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
    pub path: PathBuf,
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

        font_match.name().and_then(|name| {
            font_match.filename().map(|filename| {
                Font {
                    name: name.to_owned(),
                    path: PathBuf::from(filename),
                }
            })
        })
    }

    #[allow(dead_code)]
    fn print_debug(&self) {
        println!("Name: {}\nPath: {}", self.name, self.path.display());
    }
}

/// A safe wrapper around fontconfig's `FcPattern`.
pub struct Pattern<'a> {
    _m: PhantomData<&'a str>,
    pub pat: *mut FcPattern,
    /// This is just to hold the RAII C-strings while the `FcPattern` is alive.
    strings: Vec<CString>,
    should_free: bool,
}

impl<'a> Pattern<'a> {
    pub fn new() -> Pattern<'a> {
        fc_init();

        Pattern {
            _m: PhantomData {},
            pat: unsafe { fontconfig_sys::FcPatternCreate() },
            strings: Vec::new(),
            should_free: true,
        }
    }

    pub fn from_pattern(pat: *mut FcPattern) -> Pattern<'a> {
        unsafe {
            fontconfig_sys::FcPatternReference(pat);
        }

        Pattern {
            _m: PhantomData {},
            pat: pat,
            strings: Vec::new(),
            should_free: false,
        }
    }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &str, val: &str) {
        let c_name = CString::new(name).unwrap();
        let c_val = CString::new(val).unwrap();

        // `val` is copied inside fontconfig so no need to allocate it again.
        unsafe {
            fontconfig_sys::FcPatternAddString(self.pat,
                                               c_name.as_ptr(),
                                               c_val.as_ptr() as *const u8);
        }

        self.strings.push(c_name);
    }

    /// Get the value for a key from this pattern.
    pub fn get_string<'b, 'c>(&'b self, name: &'b str) -> Option<&'b str> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let mut ret: *mut fontconfig_sys::FcChar8 = mem::uninitialized();
            if fontconfig_sys::FcPatternGetString(self.pat,
                                                  c_name.as_ptr(),
                                                  0,
                                                  &mut ret as *mut _) ==
               fontconfig_sys::FcResult::FcResultMatch {
                let cstr = CStr::from_ptr(ret as *const i8);
                Some(cstr.to_str().unwrap())
            } else {
                None
            }
        }
    }

    /// Print this pattern to stdout with all its values.
    #[allow(dead_code)]
    pub fn print(&self) {
        unsafe {
            fontconfig_sys::FcPatternPrint(&*self.pat);
        }
    }

    fn default_substitute(&mut self) {
        unsafe {
            fontconfig_sys::FcDefaultSubstitute(self.pat);
        }
    }

    fn config_substitute(&mut self) {
        unsafe {
            fontconfig_sys::FcConfigSubstitute(ptr::null_mut(),
                                               self.pat,
                                               fontconfig_sys::_FcMatchKind::FcMatchPattern);
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match<'b>(&'b mut self) -> Pattern<'b> {
        self.default_substitute();
        self.config_substitute();

        unsafe {
            let mut res = fontconfig_sys::FcResult::FcResultNoMatch;
            Pattern::from_pattern(fontconfig_sys::FcFontMatch(ptr::null_mut(), self.pat, &mut res))
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

impl<'a> Drop for Pattern<'a> {
    fn drop(&mut self) {
        if self.should_free {
            unsafe {
                fontconfig_sys::FcPatternDestroy(self.pat);
            }
        }
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
