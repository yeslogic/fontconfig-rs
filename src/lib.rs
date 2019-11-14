//! A wrapper around freedesktop's `fontconfig` utility, for locating fontfiles on a
//! Linux-based system. Requires `libfontconfig` to be installed.
//!
//! Use `Font` for a high-level search by family name and optional style (e.g. "FreeSerif"
//! and "italic"), and `Pattern` for a more in-depth search.
//!
//! See the [fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html

extern crate fontconfig_sys as fontconfig;

use crate::fontconfig::fontconfig as fontconfig_sys;

use std::ffi::{CStr, CString};
use std::mem;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr;

use fontconfig::fontconfig::{FcBool, FcPattern};

#[allow(non_upper_case_globals)]
const FcTrue: FcBool = 1;
#[allow(non_upper_case_globals)]
const FcFalse: FcBool = 0;

pub fn init() -> bool {
    unsafe { fontconfig_sys::FcInit() == FcTrue }
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
            font_match.filename().map(|filename| Font {
                name: name.to_owned(),
                path: PathBuf::from(filename),
            })
        })
    }

    #[allow(dead_code)]
    fn print_debug(&self) {
        println!("Name: {}\nPath: {}", self.name, self.path.display());
    }
}

/// A safe wrapper around fontconfig's `FcPattern`.
#[repr(C)]
pub struct Pattern {
    pub pat: *mut FcPattern,
}

impl Pattern {
    pub fn new() -> Pattern {
        Pattern {
            pat: unsafe { fontconfig_sys::FcPatternCreate() },
        }
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer. The pattern is referenced.
    pub fn from_pattern(pat: *mut FcPattern) -> Pattern {
        unsafe {
            fontconfig_sys::FcPatternReference(pat);
        }

        Pattern { pat: pat }
    }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &str, val: &str) {
        let c_name = CString::new(name).unwrap();
        let c_val = CString::new(val).unwrap();

        unsafe {
            fontconfig_sys::FcPatternAddString(
                self.pat,
                c_name.as_ptr(),
                c_val.as_ptr() as *const u8,
            );
        }
    }

    /// Get string the value for a key from this pattern.
    pub fn get_string<'b, 'c>(&'b self, name: &'b str) -> Option<&'b str> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let mut ret: *mut fontconfig_sys::FcChar8 = ptr::null_mut();
            if fontconfig_sys::FcPatternGetString(self.pat, c_name.as_ptr(), 0, &mut ret as *mut _)
                == fontconfig_sys::FcResultMatch
            {
                let cstr = CStr::from_ptr(ret as *const i8);
                Some(cstr.to_str().unwrap())
            } else {
                None
            }
        }
    }

    /// Get the integer value for a key from this pattern.
    pub fn get_int(&self, name: &str) -> Option<i32> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let mut ret: i32 = 0;
            if fontconfig_sys::FcPatternGetInteger(
                self.pat,
                c_name.as_ptr(),
                0,
                &mut ret as *mut i32,
            ) == fontconfig_sys::FcResultMatch
            {
                Some(ret)
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
            fontconfig_sys::FcConfigSubstitute(
                ptr::null_mut(),
                self.pat,
                fontconfig_sys::FcMatchPattern,
            );
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match(&mut self) -> Pattern {
        self.default_substitute();
        self.config_substitute();

        unsafe {
            let mut res = fontconfig_sys::FcResultNoMatch;
            Pattern::from_pattern(fontconfig_sys::FcFontMatch(
                ptr::null_mut(),
                self.pat,
                &mut res,
            ))
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

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.get_int("index")
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { fontconfig_sys::FcNameUnparse(self.pat) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const i8) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { fontconfig_sys::FcStrFree(fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        let clone = unsafe { fontconfig_sys::FcPatternDuplicate(self.pat) };
        Pattern { pat: clone }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            fontconfig_sys::FcPatternDestroy(self.pat);
        }
    }
}

pub struct FontSet {
    fcset: *mut fontconfig_sys::FcFontSet,
}

impl FontSet {
    pub fn new() -> FontSet {
        let fcset = unsafe { fontconfig_sys::FcFontSetCreate() };
        FontSet { fcset: fcset }
    }

    pub fn from_raw(raw_set: *mut fontconfig_sys::FcFontSet) -> FontSet {
        FontSet { fcset: raw_set }
    }

    pub fn add_pattern(&mut self, pat: Pattern) {
        unsafe {
            fontconfig_sys::FcFontSetAdd(self.fcset, pat.pat);
            mem::forget(pat);
        }
    }

    pub fn print(&self) {
        unsafe { fontconfig_sys::FcFontSetPrint(self.fcset) };
    }
}

impl Deref for FontSet {
    type Target = [Pattern];

    fn deref(&self) -> &[Pattern] {
        unsafe {
            let raw_fs = self.fcset;
            let slce: &[*mut FcPattern] =
                std::slice::from_raw_parts((*raw_fs).fonts, (*raw_fs).nfont as usize);
            mem::transmute(slce)
        }
    }
}

impl Drop for FontSet {
    fn drop(&mut self) {
        unsafe { fontconfig_sys::FcFontSetDestroy(self.fcset) }
    }
}

pub fn list_fonts(pattern: &Pattern) -> FontSet {
    let ptr = unsafe { fontconfig_sys::FcFontList(ptr::null_mut(), pattern.pat, ptr::null_mut()) };
    FontSet::from_raw(ptr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(super::init())
    }

    #[test]
    fn test_find_font() {
        Font::find("dejavu sans", None).unwrap().print_debug();
        Font::find("dejavu sans", Some("oblique"))
            .unwrap()
            .print_debug();
    }

    #[test]
    fn test_print() {
        let fontset = list_fonts(&Pattern::new());
        for pattern in (&fontset).iter() {
            println!("{:?}", pattern.name());
        }
    }
}
