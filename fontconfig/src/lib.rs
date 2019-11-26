//! A wrapper around [freedesktop.org's fontconfig library][homepage], for locating fonts on a UNIX like systems such as Linux and FreeBSD. Requires fontconfig to be installed.
//!
//! See the [fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html
//!
//! Dependencies
//! ============
//!
//! * Arch Linux: `fontconfig`
//! * Debian-based systems: `libfontconfig1-dev`
//! * FreeBSD: `fontconfig`
//! * Void Linux: `fontconfig-devel`
//!
//! Usage
//! =====
//!
//! Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! fontconfig = "0.1.0"
//! ```
//!
//! main.rs:
//!
//! ```
//! extern crate fontconfig;
//!
//! use fontconfig::Font;
//!
//! fn main() {
//!     // `Font::find()` returns `Option` (will rarely be `None` but still could be)
//!     let font = Font::find("freeserif", None).unwrap();
//!     // `name` is a `String`, `path` is a `Path`
//!     println!("Name: {}\nPath: {}", font.name, font.path.display());
//! }
//! ```

extern crate fontconfig_sys;

use crate::fontconfig_sys::fontconfig as sys;

use std::ffi::{CStr, CString};
use std::mem;
use std::ops::Deref;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

use sys::{FcBool, FcPattern};

pub use sys::constants::*;

#[allow(non_upper_case_globals)]
const FcTrue: FcBool = 1;
#[allow(non_upper_case_globals, dead_code)]
const FcFalse: FcBool = 0;

/// Error type returned from Pattern::format.
///
/// The error holds the name of the unknown format.
#[derive(Debug)]
pub struct UnknownFontFormat(pub String);

/// The format of a font matched by Fontconfig.
#[derive(Eq, PartialEq)]
pub enum FontFormat {
    TrueType,
    Type1,
    BDF,
    PCF,
    Type42,
    CIDType1,
    CFF,
    PFR,
    WindowsFNT,
}

/// Initialise Fontconfig.
///
/// Can be safely called more than once. If initialisation fails, returns `false`.
pub fn init() -> bool {
    unsafe { sys::FcInit() == FcTrue }
}

/// A very high-level view of a font, only concerned with the name and its file location.
///
/// ##Example
/// ```rust
/// use fontconfig::Font;
///
/// let font = Font::find("sans-serif", Some("italic")).unwrap();
/// println!("Name: {}\nPath: {}", font.name, font.path.display());
/// ```
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
        let family = CString::new(family).ok()?;
        pat.add_string(FC_FAMILY.as_cstr(), &family);

        if let Some(style) = style {
            let style = CString::new(style).ok()?;
            pat.add_string(FC_STYLE.as_cstr(), &style);
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
    /// Raw pointer to `FcPattern`
    pub pat: *mut FcPattern,
}

impl Pattern {
    /// Create a new `Pattern`.
    pub fn new() -> Pattern {
        let pat = unsafe { sys::FcPatternCreate() };
        assert!(!pat.is_null());

        Pattern { pat }
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer. The pattern is referenced.
    pub fn from_pattern(pat: *mut FcPattern) -> Pattern {
        unsafe {
            sys::FcPatternReference(pat);
        }

        Pattern { pat: pat }
    }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &CStr, val: &CStr) {
        unsafe {
            sys::FcPatternAddString(self.pat, name.as_ptr(), val.as_ptr() as *const u8);
        }
    }

    /// Get string the value for a key from this pattern.
    pub fn get_string<'a>(&'a self, name: &'a CStr) -> Option<&'a str> {
        unsafe {
            let mut ret: *mut sys::FcChar8 = ptr::null_mut();
            if sys::FcPatternGetString(self.pat, name.as_ptr(), 0, &mut ret as *mut _)
                == sys::FcResultMatch
            {
                let cstr = CStr::from_ptr(ret as *const i8);
                Some(cstr.to_str().unwrap())
            } else {
                None
            }
        }
    }

    /// Get the integer value for a key from this pattern.
    pub fn get_int(&self, name: &CStr) -> Option<i32> {
        unsafe {
            let mut ret: i32 = 0;
            if sys::FcPatternGetInteger(self.pat, name.as_ptr(), 0, &mut ret as *mut i32)
                == sys::FcResultMatch
            {
                Some(ret)
            } else {
                None
            }
        }
    }

    /// Print this pattern to stdout with all its values.
    pub fn print(&self) {
        unsafe {
            sys::FcPatternPrint(&*self.pat);
        }
    }

    fn default_substitute(&mut self) {
        unsafe {
            sys::FcDefaultSubstitute(self.pat);
        }
    }

    fn config_substitute(&mut self) {
        unsafe {
            sys::FcConfigSubstitute(ptr::null_mut(), self.pat, sys::FcMatchPattern);
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match(&mut self) -> Pattern {
        self.default_substitute();
        self.config_substitute();

        unsafe {
            let mut res = sys::FcResultNoMatch;
            Pattern::from_pattern(sys::FcFontMatch(ptr::null_mut(), self.pat, &mut res))
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Option<&str> {
        self.get_string(FC_FULLNAME.as_cstr())
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.get_string(FC_FILE.as_cstr())
    }

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.get_int(FC_INDEX.as_cstr())
    }

    /// Get the "slant" (Italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Option<i32> {
        self.get_int(FC_SLANT.as_cstr())
    }

    /// Get the "weight" (Light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Option<i32> {
        self.get_int(FC_WEIGHT.as_cstr())
    }

    /// Get the "width" (Condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Option<i32> {
        self.get_int(FC_WIDTH.as_cstr())
    }

    /// Get the "fontformat" ("TrueType" "Type 1" "BDF" "PCF" "Type 42" "CID Type 1" "CFF" "PFR" "Windows FNT") of this pattern.
    pub fn format(&self) -> Result<FontFormat, UnknownFontFormat> {
        self.get_string(FC_FONTFORMAT.as_cstr())
            .ok_or_else(|| UnknownFontFormat(String::new()))
            .and_then(|format| format.parse())
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { sys::FcNameUnparse(self.pat) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const i8) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { sys::FcStrFree(fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        let clone = unsafe { sys::FcPatternDuplicate(self.pat) };
        Pattern { pat: clone }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            sys::FcPatternDestroy(self.pat);
        }
    }
}

/// Wrapper around `FcFontSet`.
pub struct FontSet {
    fcset: *mut sys::FcFontSet,
}

impl FontSet {
    /// Create a new, empty `FontSet`.
    pub fn new() -> FontSet {
        let fcset = unsafe { sys::FcFontSetCreate() };
        FontSet { fcset: fcset }
    }

    /// Wrap an existing `FcFontSet`.
    ///
    /// This returned wrapper assumes ownership of the `FcFontSet`.
    pub fn from_raw(raw_set: *mut sys::FcFontSet) -> FontSet {
        FontSet { fcset: raw_set }
    }

    /// Add a `Pattern` to this `FontSet`.
    pub fn add_pattern(&mut self, pat: Pattern) {
        unsafe {
            sys::FcFontSetAdd(self.fcset, pat.pat);
            mem::forget(pat);
        }
    }

    /// Print this `FontSet` to stdout.
    pub fn print(&self) {
        unsafe { sys::FcFontSetPrint(self.fcset) };
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
        unsafe { sys::FcFontSetDestroy(self.fcset) }
    }
}

/// Return a `FontSet` containing Fonts that match the supplied `pattern` and `objects`.
pub fn list_fonts(pattern: &Pattern, objects: Option<&ObjectSet>) -> FontSet {
    let os = objects.map(|o| o.fcset).unwrap_or(ptr::null_mut());
    let ptr = unsafe { sys::FcFontList(ptr::null_mut(), pattern.pat, os) };
    FontSet::from_raw(ptr)
}

/// Wrapper around `FcObjectSet`.
pub struct ObjectSet {
    fcset: *mut sys::FcObjectSet,
}

impl ObjectSet {
    /// Create a new, empty `ObjectSet`.
    pub fn new() -> ObjectSet {
        let fcset = unsafe { sys::FcObjectSetCreate() };
        assert!(!fcset.is_null());

        ObjectSet { fcset }
    }

    /// Wrap an existing `FcObjectSet`.
    ///
    /// The `FcObjectSet` must not be null. This method assumes ownership of the `FcObjectSet`.
    pub fn from_raw(raw_set: *mut sys::FcObjectSet) -> ObjectSet {
        assert!(!raw_set.is_null());
        ObjectSet { fcset: raw_set }
    }

    /// Add a string to the `ObjectSet`.
    pub fn add(&mut self, name: &CStr) {
        let res = unsafe { sys::FcObjectSetAdd(self.fcset, name.as_ptr()) };
        assert_eq!(res, FcTrue);
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { sys::FcObjectSetDestroy(self.fcset) }
    }
}

impl FromStr for FontFormat {
    type Err = UnknownFontFormat;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TrueType" => Ok(FontFormat::TrueType),
            "Type 1" => Ok(FontFormat::Type1),
            "BDF" => Ok(FontFormat::BDF),
            "PCF" => Ok(FontFormat::PCF),
            "Type 42" => Ok(FontFormat::Type42),
            "CID Type 1" => Ok(FontFormat::CIDType1),
            "CFF" => Ok(FontFormat::CFF),
            "PFR" => Ok(FontFormat::PFR),
            "Windows FNT" => Ok(FontFormat::WindowsFNT),
            _ => Err(UnknownFontFormat(s.to_string())),
        }
    }
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
        let fontset = list_fonts(&Pattern::new(), None);
        for pattern in (&fontset).iter() {
            println!("{:?}", pattern.name());
        }
    }
}
