#![deny(missing_docs)]

//! A wrapper around [freedesktop.org's Fontconfig library][homepage], for locating fonts on a UNIX
//! like systems such as Linux and FreeBSD. Requires Fontconfig to be installed. Alternatively,
//! set the environment variable `RUST_FONTCONFIG_DLOPEN=on` or enable the `dlopen` Cargo feature
//! to load the library at runtime rather than link at build time (useful for cross compiling).
//!
//! See the [Fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html
//! [homepage]: https://www.freedesktop.org/wiki/Software/fontconfig/
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
//! -----
//!
//! ```
//! use fontconfig::Fontconfig;
//!
//! fn main() {
//!     let fc = Fontconfig::new().unwrap();
//!     // `Fontconfig::find()` returns `Option` (will rarely be `None` but still could be)
//!     let font = fc.find("freeserif", None).unwrap();
//!     // `name` is a `String`, `path` is a `Path`
//!     println!("Name: {}\nPath: {}", font.name, font.path.display());
//! }
//! ```
//!
//! ### Cargo Features
//!
//! | Feature       | Description                       | Default Enabled | Extra Dependencies    |
//! |---------------|-----------------------------------|:---------------:|-----------------------|
//! | `dlopen`      | [dlopen] libfontconfig at runtime |        ❌       |                       |
//!
//! The `dlopen` feature enables building this crate without dynamically linking to the Fontconfig C
//! library at link time. Instead, Fontconfig will be dynamically loaded at runtime with the
//! [dlopen] function. This can be useful in cross-compiling situations as you don't need to have a
//! version of Fontcofig available for the target platform available at compile time.
//!
//! [dlopen]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dlopen.html

use fontconfig_sys as sys;
use fontconfig_sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::{LIB, LIB_RESULT};
#[cfg(not(feature = "dlopen"))]
use sys::*;

use std::ffi::{c_int, CStr, CString};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;

pub use sys::constants::*;
use sys::{FcBool, FcPattern};

#[allow(non_upper_case_globals)]
const FcTrue: FcBool = 1;
#[allow(non_upper_case_globals, dead_code)]
const FcFalse: FcBool = 0;

/// Handle obtained after Fontconfig has been initialised.
pub struct Fontconfig {
    _initialised: (),
}

/// Error type returned from Pattern::format.
///
/// The error holds the name of the unknown format.
#[derive(Debug)]
pub struct UnknownFontFormat(pub String);

/// The format of a font matched by Fontconfig.
#[derive(Eq, PartialEq)]
#[allow(missing_docs)]
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

impl Fontconfig {
    /// Initialise Fontconfig and return a handle allowing further interaction with the API.
    ///
    /// If Fontconfig fails to initialise, returns `None`.
    pub fn new() -> Option<Self> {
        #[cfg(feature = "dlopen")]
        if LIB_RESULT.is_err() {
            return None;
        }
        if unsafe { ffi_dispatch!(LIB, FcInit,) == FcTrue } {
            Some(Fontconfig { _initialised: () })
        } else {
            None
        }
    }

    /// Find a font of the given `family` (e.g. Dejavu Sans, FreeSerif),
    /// optionally filtering by `style`. Both fields are case-insensitive.
    pub fn find(&self, family: &str, style: Option<&str>) -> Option<Font> {
        Font::find(self, family, style)
    }
}

/// A very high-level view of a font, only concerned with the name and its file location.
///
/// ##Example
/// ```rust
/// use fontconfig::{Font, Fontconfig};
///
/// let fc = Fontconfig::new().unwrap();
/// let font = fc.find("sans-serif", Some("italic")).unwrap();
/// println!("Name: {}\nPath: {}", font.name, font.path.display());
/// ```
pub struct Font {
    /// The true name of this font
    pub name: String,
    /// The location of this font on the filesystem.
    pub path: PathBuf,
    /// The index of the font within the file.
    pub index: Option<i32>,
}

impl Font {
    fn find(fc: &Fontconfig, family: &str, style: Option<&str>) -> Option<Font> {
        let mut pat = Pattern::new(fc);
        let family = CString::new(family).ok()?;
        pat.add_string(FC_FAMILY, &family);

        if let Some(style) = style {
            let style = CString::new(style).ok()?;
            pat.add_string(FC_STYLE, &style);
        }

        let font_match = pat.font_match();

        font_match.name().and_then(|name| {
            font_match.filename().map(|filename| Font {
                name: name.to_owned(),
                path: PathBuf::from(filename),
                index: font_match.face_index(),
            })
        })
    }

    #[allow(dead_code)]
    fn print_debug(&self) {
        println!(
            "Name: {}\nPath: {}\nIndex: {:?}",
            self.name,
            self.path.display(),
            self.index
        );
    }
}

/// A safe wrapper around fontconfig's `FcPattern`.
#[repr(C)]
pub struct Pattern<'fc> {
    /// Raw pointer to `FcPattern`
    pat: *mut FcPattern,
    fc: &'fc Fontconfig,
}

impl<'fc> Pattern<'fc> {
    /// Create a new empty `Pattern`.
    pub fn new(fc: &Fontconfig) -> Pattern {
        let pat = unsafe { ffi_dispatch!(LIB, FcPatternCreate,) };
        assert!(!pat.is_null());

        Pattern { pat, fc }
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer.
    ///
    /// The pattern is referenced.
    ///
    /// **Safety:** The pattern pointer must be valid/non-null.
    pub unsafe fn from_pattern(fc: &Fontconfig, pat: *mut FcPattern) -> Pattern {
        ffi_dispatch!(LIB, FcPatternReference, pat);

        Pattern { pat, fc }
    }

    /// Add a key-value pair of type `String` to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &CStr, val: &CStr) {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternAddString,
                self.pat,
                name.as_ptr(),
                val.as_ptr() as *const u8
            );
        }
    }

    /// Add a key-value pair of type `Int` to this pattern
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_integer(&mut self, name: &CStr, val: c_int) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternAddInteger, self.pat, name.as_ptr(), val);
        }
    }

    /// Get string the value for a key from this pattern.
    pub fn get_string<'a>(&'a self, name: &'a CStr) -> Option<&'a str> {
        unsafe {
            let mut ret: *mut sys::FcChar8 = ptr::null_mut();
            if ffi_dispatch!(
                LIB,
                FcPatternGetString,
                self.pat,
                name.as_ptr(),
                0,
                &mut ret as *mut _
            ) == sys::FcResultMatch
            {
                let cstr = CStr::from_ptr(ret as *const c_char);
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
            if ffi_dispatch!(
                LIB,
                FcPatternGetInteger,
                self.pat,
                name.as_ptr(),
                0,
                &mut ret as *mut i32
            ) == sys::FcResultMatch
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
            ffi_dispatch!(LIB, FcPatternPrint, &*self.pat);
        }
    }

    /// Supplies default values for underspecified font patterns.
    ///
    /// * Patterns without a specified style or weight are set to Medium.
    /// * Patterns without a specified style or slant are set to Roman.
    /// * Patterns without a specified pixel size are given one computed from any specified point size
    ///   (default 12), dpi (default 75) and scale (default 1).
    ///
    /// *Note:* [font_match][Self::font_match] and [sort_fonts][Self::sort_fonts] call this so you
    /// don't need to manually call it when using those methods.
    ///
    /// [Fontconfig reference](https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcdefaultsubstitute.html)
    pub fn default_substitute(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcDefaultSubstitute, self.pat);
        }
    }

    /// Execute substitutions.
    ///
    /// *Note:* [font_match][Self::font_match] and [sort_fonts][Self::sort_fonts] call this so you
    /// don't need to manually call it when using those methods.
    ///
    /// [Fontconfig reference](https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcconfigsubstitute.html)
    pub fn config_substitute(&mut self) {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcConfigSubstitute,
                ptr::null_mut(),
                self.pat,
                sys::FcMatchPattern
            );
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match(&mut self) -> Pattern {
        self.config_substitute();
        self.default_substitute();

        unsafe {
            let mut res = sys::FcResultNoMatch;
            Pattern::from_pattern(
                self.fc,
                ffi_dispatch!(LIB, FcFontMatch, ptr::null_mut(), self.pat, &mut res),
            )
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Option<&str> {
        self.get_string(FC_FULLNAME)
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.get_string(FC_FILE)
    }

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.get_int(FC_INDEX)
    }

    /// Get the "slant" (Italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Option<i32> {
        self.get_int(FC_SLANT)
    }

    /// Get the "weight" (Light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Option<i32> {
        self.get_int(FC_WEIGHT)
    }

    /// Get the "width" (Condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Option<i32> {
        self.get_int(FC_WIDTH)
    }

    /// Get the "fontformat" ("TrueType" "Type 1" "BDF" "PCF" "Type 42" "CID Type 1" "CFF" "PFR" "Windows FNT") of this pattern.
    pub fn format(&self) -> Result<FontFormat, UnknownFontFormat> {
        self.get_string(FC_FONTFORMAT)
            .ok_or_else(|| UnknownFontFormat(String::new()))
            .and_then(|format| format.parse())
    }

    /// Returns a raw pointer to underlying `FcPattern`.
    pub fn as_ptr(&self) -> *const FcPattern {
        self.pat
    }

    /// Returns an unsafe mutable pointer to the underlying `FcPattern`.
    pub fn as_mut_ptr(&mut self) -> *mut FcPattern {
        self.pat
    }
}

impl<'fc> std::fmt::Debug for Pattern<'fc> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { ffi_dispatch!(LIB, FcNameUnparse, self.pat) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const c_char) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { ffi_dispatch!(LIB, FcStrFree, fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl<'fc> Clone for Pattern<'fc> {
    fn clone(&self) -> Self {
        let clone = unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.pat) };
        Pattern {
            pat: clone,
            fc: self.fc,
        }
    }
}

impl<'fc> Drop for Pattern<'fc> {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternDestroy, self.pat);
        }
    }
}

impl Pattern<'_> {
    /// Get the languages set of this pattern.
    pub fn lang_set(&self) -> Option<StrList<'_>> {
        unsafe {
            let mut ret: *mut sys::FcLangSet = ptr::null_mut();
            if ffi_dispatch!(
                LIB,
                FcPatternGetLangSet,
                self.pat,
                FC_LANG.as_ptr(),
                0,
                &mut ret as *mut _
            ) == sys::FcResultMatch
            {
                let ss: *mut sys::FcStrSet = ffi_dispatch!(LIB, FcLangSetGetLangs, ret);
                let lang_strs: *mut sys::FcStrList = ffi_dispatch!(LIB, FcStrListCreate, ss);
                Some(StrList::from_raw(self.fc, lang_strs))
            } else {
                None
            }
        }
    }
}

/// Wrapper around `FcStrList`
///
/// The wrapper implements `Iterator` so it can be iterated directly, filtered etc.
/// **Note:** Any entries in the `StrList` that are not valid UTF-8 will be skipped.
///
/// ```
/// use fontconfig::{Fontconfig, Pattern};
///
/// let fc = Fontconfig::new().expect("unable to init Fontconfig");
///
/// // Find fonts that support japanese
/// let fonts = fontconfig::list_fonts(&Pattern::new(&fc), None);
/// let ja_fonts: Vec<_> = fonts
///     .iter()
///     .filter(|p| p.lang_set().map_or(false, |mut langs| langs.any(|l| l == "ja")))
///     .collect();
/// ```
pub struct StrList<'a> {
    list: *mut sys::FcStrList,
    _life: PhantomData<&'a sys::FcStrList>,
}

impl<'a> StrList<'a> {
    unsafe fn from_raw(_: &Fontconfig, raw_list: *mut sys::FcStrSet) -> Self {
        Self {
            list: raw_list,
            _life: PhantomData,
        }
    }
}

impl<'a> Drop for StrList<'a> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrListDone, self.list) };
    }
}

impl<'a> Iterator for StrList<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let lang_str: *mut sys::FcChar8 = unsafe { ffi_dispatch!(LIB, FcStrListNext, self.list) };
        if lang_str.is_null() {
            None
        } else {
            match unsafe { CStr::from_ptr(lang_str as *const c_char) }.to_str() {
                Ok(s) => Some(s),
                _ => self.next(),
            }
        }
    }
}

/// Wrapper around `FcFontSet`.
pub struct FontSet<'fc> {
    fcset: *mut sys::FcFontSet,
    fc: &'fc Fontconfig,
}

impl<'fc> FontSet<'fc> {
    /// Create a new, empty `FontSet`.
    pub fn new(fc: &Fontconfig) -> FontSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcFontSetCreate,) };
        FontSet { fcset, fc }
    }

    /// Wrap an existing `FcFontSet`.
    ///
    /// The returned wrapper assumes ownership of the `FcFontSet`.
    ///
    /// **Safety:** The font set pointer must be valid/non-null.
    pub unsafe fn from_raw(fc: &Fontconfig, raw_set: *mut sys::FcFontSet) -> FontSet {
        FontSet { fcset: raw_set, fc }
    }

    /// Add a `Pattern` to this `FontSet`.
    pub fn add_pattern(&mut self, pat: Pattern) {
        unsafe {
            ffi_dispatch!(LIB, FcFontSetAdd, self.fcset, pat.pat);
            mem::forget(pat);
        }
    }

    /// Print this `FontSet` to stdout.
    pub fn print(&self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetPrint, self.fcset) };
    }

    /// Iterate the fonts (as `Patterns`) in this `FontSet`.
    pub fn iter(&self) -> impl Iterator<Item = Pattern<'_>> {
        let patterns = unsafe {
            let fontset = self.fcset;
            std::slice::from_raw_parts((*fontset).fonts, (*fontset).nfont as usize)
        };
        patterns
            .iter()
            .map(move |&pat| unsafe { Pattern::from_pattern(self.fc, pat) })
    }
}

impl<'fc> Drop for FontSet<'fc> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetDestroy, self.fcset) }
    }
}

/// Return a `FontSet` containing Fonts that match the supplied `pattern` and `objects`.
pub fn list_fonts<'fc>(pattern: &Pattern<'fc>, objects: Option<&ObjectSet>) -> FontSet<'fc> {
    let os = objects.map(|o| o.fcset).unwrap_or(ptr::null_mut());
    unsafe {
        let raw_set = ffi_dispatch!(LIB, FcFontList, ptr::null_mut(), pattern.pat, os);
        FontSet::from_raw(pattern.fc, raw_set)
    }
}

/// Returns a [`FontSet`] containing fonts sorted by closeness to the supplied `pattern`. If `trim` is true, elements in
/// the list which don't include Unicode coverage not provided by earlier elements in the list are elided.
///
/// See the [fontconfig reference][1] for more details.
///
/// [1]: https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsort.html
pub fn sort_fonts<'fc>(pattern: &Pattern<'fc>, trim: bool) -> FontSet<'fc> {
    // FcFontSort always returns a (possibly empty) set so we don't need to check this.
    let mut res = sys::FcResultNoMatch;
    let unicode_coverage = ptr::null_mut();
    let config = ptr::null_mut();
    unsafe {
        let raw_set = ffi_dispatch!(
            LIB,
            FcFontSort,
            config,
            pattern.pat,
            trim as FcBool,
            unicode_coverage,
            &mut res
        );
        FontSet::from_raw(pattern.fc, raw_set)
    }
}

/// Wrapper around `FcObjectSet`.
pub struct ObjectSet {
    fcset: *mut sys::FcObjectSet,
}

impl ObjectSet {
    /// Create a new, empty `ObjectSet`.
    pub fn new(_: &Fontconfig) -> ObjectSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcObjectSetCreate,) };
        assert!(!fcset.is_null());

        ObjectSet { fcset }
    }

    /// Wrap an existing `FcObjectSet`.
    ///
    /// The `FcObjectSet` must not be null. This method assumes ownership of the `FcObjectSet`.
    ///
    /// **Safety:** The object set pointer must be valid/non-null.
    pub unsafe fn from_raw(_: &Fontconfig, raw_set: *mut sys::FcObjectSet) -> ObjectSet {
        ObjectSet { fcset: raw_set }
    }

    /// Add a string to the `ObjectSet`.
    pub fn add(&mut self, name: &CStr) {
        let res = unsafe { ffi_dispatch!(LIB, FcObjectSetAdd, self.fcset, name.as_ptr()) };
        assert_eq!(res, FcTrue);
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcObjectSetDestroy, self.fcset) }
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
        assert!(Fontconfig::new().is_some())
    }

    #[test]
    fn test_find_font() {
        let fc = Fontconfig::new().unwrap();
        fc.find("dejavu sans", None).unwrap().print_debug();
        fc.find("dejavu sans", Some("oblique"))
            .unwrap()
            .print_debug();
    }

    #[test]
    fn test_iter_and_print() {
        let fc = Fontconfig::new().unwrap();
        let fontset = list_fonts(&Pattern::new(&fc), None);
        for pattern in fontset.iter() {
            println!("{:?}", pattern.name());
        }

        // Ensure that the set can be iterated again
        assert!(fontset.iter().count() > 0);
    }

    #[test]
    fn iter_lang_set() {
        let fc = Fontconfig::new().unwrap();
        let mut pat = Pattern::new(&fc);
        let family = CString::new("dejavu sans").unwrap();
        pat.add_string(FC_FAMILY, &family);
        let pattern = pat.font_match();
        for lang in pattern.lang_set().unwrap() {
            println!("{:?}", lang);
        }

        // Test find
        assert!(pattern
            .lang_set()
            .unwrap()
            .find(|&lang| lang == "za")
            .is_some());

        // Test collect
        let langs = pattern.lang_set().unwrap().collect::<Vec<_>>();
        assert!(langs.iter().find(|&&l| l == "ie").is_some());
    }
}
