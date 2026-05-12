#![deny(missing_docs)]

//! A wrapper around [freedesktop.org's Fontconfig library][homepage], for locating fonts on
//! UNIX-like systems such as Linux and FreeBSD. Requires Fontconfig to be installed. Alternatively,
//! set the environment variable `RUST_FONTCONFIG_DLOPEN=on` or enable the `dlopen` Cargo feature to
//! load the library at runtime rather than link at build time (useful for cross compiling).
//!
//! See the [Fontconfig developer reference][1] for more information.
//!
//! [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/t1.html
//! [homepage]: https://www.freedesktop.org/wiki/Software/fontconfig/
//!
//! Dependencies
//! ============
//!
//! To use this crate, you need to have Fontconfig installed on your system.
//! For example, install the package:
//!
//! * Arch Linux: `fontconfig`
//! * Debian-based systems: `libfontconfig1-dev`
//! * FreeBSD: `fontconfig`
//! * Void Linux: `fontconfig-devel`
//!
//! Usage
//! -----
//!
//! ### Example
//!
//! ```
//! use fontconfig::{Fontconfig, FontconfigError};
//!
//! fn main() -> Result<(), FontconfigError> {
//!     let fc = Fontconfig::new().expect("unable to init Fontconfig");
//!     // `Fontconfig::find()` returns `Result` (will rarely be `Err` but still could be)
//!     let font = fc.find("freeserif", None)?;
//!     // `name` is a `String`, `path` is a `Path`
//!     println!("Name: {}\nPath: {}", font.name, font.path.display());
//!     Ok(())
//! }
//! ```
//!
//! For more advanced usage, see [list_fonts] and the [Pattern] type.
//!
//! See the [examples directory in the repository](https://github.com/yeslogic/fontconfig-rs/blob/master/examples/fc-list.rs)
//! for more examples.
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
//! version of Fontconfig available for the target platform available at compile time. This can also
//! be enabled by setting the `RUST_FONTCONFIG_DLOPEN` environment variable.
//!
//! [dlopen]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dlopen.html
//!
//! ### Thread Safety
//!
//! **Note:**

use fontconfig_sys as sys;
use fontconfig_sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::{LIB, LIB_RESULT};
#[cfg(not(feature = "dlopen"))]
use sys::*;

use std::ffi::{self, c_char, c_int, CStr, CString};
use std::marker::PhantomData;
use std::path::PathBuf;
use std::str::{self, FromStr};
use std::{fmt, ptr};

pub use sys::constants::*;
use sys::{FcBool, FcCharSet, FcPattern, FcResult};

#[allow(non_upper_case_globals)]
const FcTrue: FcBool = 1;
#[allow(non_upper_case_globals, dead_code)]
const FcFalse: FcBool = 0;

/// Handle obtained after Fontconfig has been initialised.
pub struct Fontconfig {
    _initialised: (),
}

/// Fontconfig error
#[derive(Debug)]
pub enum FontconfigError {
    /// A Fontconfig operation failed with FcFalse or NULL pointer.
    ///
    /// Fontconfig does not provide more information in these cases.
    Failed,
    /// An attempt to convert a Rust string to C string failed due to it containing one or more NUL bytes.
    NulError,
    /// An attempt to convert a C string to Rust string failed because it was not valid UTF-8.
    Utf8Error,
    // FcResult variants, minus FcResultMatch
    /// A find/match operation returned no matches.
    NoMatch,
    /// There was a type mismatch between expected and supplied data.
    TypeMismatch,
    /// There is no item with the supplied id.
    NoId,
    /// Out of memory.
    OutOfMemory,
}

/// Error type returned from [Pattern::format].
///
/// The error holds the name of the unknown format.
#[derive(Debug)]
pub struct UnknownFontFormat(pub String);

/// The format of a font matched by Fontconfig.
#[derive(Eq, PartialEq, Debug)]
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

/// Flag indicating whether results from [Pattern::sort_fonts] should be trimmed
/// to exclude fonts that do not cover the Unicode coverage in the pattern.
#[derive(Eq, PartialEq, Debug)]
pub enum UnicodeCoverage {
    /// Trim results that don't include Unicode coverage.
    Trim,
    /// Don't trim results.
    NoTrim,
}

trait ToResult {
    fn to_result(self) -> Result<(), FontconfigError>;
}

impl Fontconfig {
    /// Initialise Fontconfig and return a handle allowing further interaction with the API.
    ///
    /// If Fontconfig fails to initialise, returns `None`.
    ///
    /// ## Example
    ///
    /// ```
    /// use fontconfig::{Fontconfig};
    ///
    /// let fc = Fontconfig::new().expect("unable to initialise Fontconfig");
    /// ```
    #[doc(alias = "FcInit")]
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

    /// Find a font of the given `family` and `style`.
    ///
    /// Results can optionally be filtered by `style`. Both fields are case-insensitive.
    pub fn find(&self, family: &str, style: Option<&str>) -> Result<Font, FontconfigError> {
        Font::find(self, family, style)
    }
}

// There were issues with calling FcFini more than once in Fontconfig versions prior
// to 2.17.0. As of 14 May 2026 earlier versions are still commonly used, so we don't
// call it automatically.
// https://gitlab.freedesktop.org/fontconfig/fontconfig/-/merge_requests/410
// impl Drop for Fontconfig {
//     fn drop(&mut self) {
//         unsafe { ffi_dispatch!(LIB, FcFini,) };
//     }
// }

/// A  high-level view of a font, only concerned with the name and its file location.
///
/// ## Example
///
/// ```rust
/// use fontconfig::{Font, Fontconfig};
///
/// let fc = Fontconfig::new().unwrap();
/// let font = fc.find("sans-serif", Some("italic")).unwrap();
/// println!("Name: {}\nPath: {}", font.name, font.path.display());
/// ```
pub struct Font {
    /// The true name of this font.
    pub name: String,
    /// The location of this font on the filesystem.
    pub path: PathBuf,
    /// The index of the font within the file.
    pub index: Option<i32>,
}

impl Font {
    fn find(fc: &Fontconfig, family: &str, style: Option<&str>) -> Result<Font, FontconfigError> {
        let mut pat = Pattern::new(fc)?;
        let family = CString::new(family)?;
        pat.add_string(FC_FAMILY, &family)?;

        if let Some(style) = style {
            let style = CString::new(style)?;
            pat.add_string(FC_STYLE, &style)?;
        }

        let font_match = pat.font_match()?;
        let name = font_match.name()?;
        let filename = font_match.filename()?;

        Ok(Font {
            name: name.to_owned(),
            path: PathBuf::from(filename),
            index: font_match.face_index().ok(),
        })
    }

    #[cfg(test)]
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
///
/// See the [fc-match example](https://github.com/yeslogic/fontconfig-rs/blob/master/examples/fc-match.rs)
/// and
/// [fc-list example](https://github.com/yeslogic/fontconfig-rs/blob/master/examples/fc-list.rs)
/// for an example of using `Pattern`.
///
#[repr(C)]
#[doc(alias = "FcPattern")]
pub struct Pattern<'fc> {
    /// Raw pointer to `FcPattern`
    pat: *mut FcPattern,
    fc: &'fc Fontconfig,
}

impl<'fc> Pattern<'fc> {
    /// Create a new empty `Pattern`.
    ///
    /// The [Fontconfig] handle is obtained from [Fontconfig::new].
    #[doc(alias = "FcPatternCreate")]
    pub fn new(fc: &Fontconfig) -> Result<Pattern<'_>, FontconfigError> {
        let pat = unsafe { ffi_dispatch!(LIB, FcPatternCreate,) };
        is_non_null(pat)
            .then_some(Pattern { pat, fc })
            .ok_or(FontconfigError::Failed)
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer.
    ///
    /// The pattern is referenced.
    ///
    /// **Safety:** The pattern pointer must be valid/non-null.
    pub unsafe fn from_pattern(fc: &Fontconfig, pat: *mut FcPattern) -> Pattern<'_> {
        assert!(is_non_null(pat));
        ffi_dispatch!(LIB, FcPatternReference, pat);

        Pattern { pat, fc }
    }

    /// Add a key-value pair of type `String` to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    #[doc(alias = "FcPatternAddString")]
    pub fn add_string(&mut self, name: &CStr, val: &CStr) -> Result<(), FontconfigError> {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternAddString,
                self.pat,
                name.as_ptr(),
                val.as_ptr() as *const u8
            )
            .to_result()
        }
    }

    /// Add a key-value pair of type `Int` to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    #[doc(alias = "FcPatternAddInteger")]
    pub fn add_integer(&mut self, name: &CStr, val: c_int) -> Result<(), FontconfigError> {
        unsafe { ffi_dispatch!(LIB, FcPatternAddInteger, self.pat, name.as_ptr(), val).to_result() }
    }

    /// Add a charset to this pattern.
    #[doc(alias = "FcPatternAddCharSet")]
    pub fn add_charset(&mut self, val: CharSet) -> Result<(), FontconfigError> {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternAddCharSet,
                self.pat,
                FC_CHARSET.as_ptr(),
                val.char_set
            )
            .to_result()
        }
    }

    /// Get string the value for a key from this pattern.
    #[doc(alias = "FcPatternGetString")]
    pub fn get_string<'a>(&'a self, name: &'a CStr) -> Result<&'a str, FontconfigError> {
        unsafe {
            let mut ret: *mut sys::FcChar8 = ptr::null_mut();
            ffi_dispatch!(
                LIB,
                FcPatternGetString,
                self.pat,
                name.as_ptr(),
                0,
                &mut ret as *mut _
            )
            .to_result()?;
            let cstr = CStr::from_ptr(ret as *const c_char);
            cstr.to_str().map_err(FontconfigError::from)
        }
    }

    /// Get the integer value for a key from this pattern.
    #[doc(alias = "FcPatternGetInteger")]
    pub fn get_int(&self, name: &CStr) -> Result<i32, FontconfigError> {
        unsafe {
            let mut ret: i32 = 0;
            ffi_dispatch!(
                LIB,
                FcPatternGetInteger,
                self.pat,
                name.as_ptr(),
                0,
                &mut ret as *mut i32
            )
            .to_result()?;
            Ok(ret)
        }
    }

    /// Print this pattern to stdout with all its values.
    #[doc(alias = "FcPatternPrint")]
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
    #[doc(alias = "FcDefaultSubstitute")]
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
    #[doc(alias = "FcConfigSubstitute")]
    pub fn config_substitute(&mut self) -> Result<(), FontconfigError> {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcConfigSubstitute,
                ptr::null_mut(),
                self.pat,
                sys::FcMatchPattern
            )
            .to_result()
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    ///
    /// See [the fc-match example](https://github.com/yeslogic/fontconfig-rs/blob/master/examples/fc-match.rs)
    /// for an example of using this function.
    #[doc(alias = "FcFontMatch")]
    pub fn font_match(&mut self) -> Result<Pattern<'_>, FontconfigError> {
        self.config_substitute()?;
        self.default_substitute();

        unsafe {
            let mut res = sys::FcResultNoMatch;
            let pattern_ptr = ffi_dispatch!(LIB, FcFontMatch, ptr::null_mut(), self.pat, &mut res);
            is_non_null(pattern_ptr)
                .then(|| Pattern::from_pattern(self.fc, pattern_ptr))
                .ok_or(FontconfigError::Failed)
        }
    }

    /// Returns a [`FontSet`] containing fonts sorted by closeness to this pattern.
    ///
    /// If `trim` is [UnicodeCoverage::Trim], elements in the list which don't include Unicode
    /// coverage provided by earlier elements in the list are elided.
    ///
    /// See the [Fontconfig reference][1] for more details.
    ///
    /// [1]: https://www.freedesktop.org/software/fontconfig/fontconfig-devel/fcfontsort.html
    #[doc(alias = "FcFontSort")]
    pub fn sort_fonts(&mut self, trim: UnicodeCoverage) -> Result<FontSet<'fc>, FontconfigError> {
        // Docs: This function should be called only after FcConfigSubstitute and FcDefaultSubstitute
        // have been called for p; otherwise the results will not be correct.
        self.config_substitute()?;
        self.default_substitute();

        // FcFontSort always returns a (possibly empty) set so we don't need to check this.
        let mut res = sys::FcResultNoMatch;
        let unicode_coverage = ptr::null_mut();
        let config = ptr::null_mut();
        unsafe {
            let raw_set = ffi_dispatch!(
                LIB,
                FcFontSort,
                config,
                self.pat,
                trim as FcBool,
                unicode_coverage,
                &mut res
            );
            Ok(FontSet::from_raw(self.fc, raw_set))
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Result<&str, FontconfigError> {
        self.get_string(FC_FULLNAME)
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Result<&str, FontconfigError> {
        self.get_string(FC_FILE)
    }

    /// Get the "index" (the index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Result<i32, FontconfigError> {
        self.get_int(FC_INDEX)
    }

    /// Get the "slant" (italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Result<i32, FontconfigError> {
        self.get_int(FC_SLANT)
    }

    /// Get the "weight" (light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Result<i32, FontconfigError> {
        self.get_int(FC_WEIGHT)
    }

    /// Get the "width" (condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Result<i32, FontconfigError> {
        self.get_int(FC_WIDTH)
    }

    /// Get the "fontformat" of this pattern.
    ///
    /// If Fontconfig returns an unknown font format, [`UnknownFontFormat`] is returned,
    /// which contains the format returned by Fontconfig.
    pub fn format(&self) -> Result<FontFormat, UnknownFontFormat> {
        self.get_string(FC_FONTFORMAT)
            .map_err(|_| UnknownFontFormat(String::new()))
            .and_then(|format| format.parse())
    }

    /// Get the language set of this pattern.
    #[doc(alias = "FcPatternGetLangSet")]
    pub fn lang_set(&self) -> Result<StrList<'_>, FontconfigError> {
        unsafe {
            let mut lang_set: *mut sys::FcLangSet = ptr::null_mut();
            ffi_dispatch!(
                LIB,
                FcPatternGetLangSet,
                self.pat,
                FC_LANG.as_ptr(),
                0,
                &mut lang_set as *mut _
            )
            .to_result()?;
            let ss: *mut sys::FcStrSet = ffi_dispatch!(LIB, FcLangSetGetLangs, lang_set);
            if ss.is_null() {
                return Err(FontconfigError::Failed);
            }
            let lang_strs: *mut sys::FcStrList = ffi_dispatch!(LIB, FcStrListCreate, ss);
            if lang_strs.is_null() {
                return Err(FontconfigError::Failed);
            }
            Ok(StrList::from_raw(self.fc, lang_strs))
        }
    }

    /// Returns a raw pointer to underlying `FcPattern`.
    pub fn as_ptr(&self) -> *const FcPattern {
        self.pat
    }

    /// Returns a mutable pointer to the underlying `FcPattern`.
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
    /// Clones a `Pattern` using `FcPatternDuplicate`.
    ///
    /// # Panics
    ///
    /// Panics if the `Pattern` cannot be allocated.
    #[doc(alias = "FcPatternDuplicate")]
    fn clone(&self) -> Self {
        let clone = unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.pat) };
        if clone.is_null() {
            panic!("Unable to clone. FcPatternDuplicate returned NULL")
        }

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

/// Wrapper around `FcStrList`.
///
/// The wrapper implements [Iterator] so it can be iterated directly, filtered etc.
///
/// **Note:** Any entries in the `StrList` that are not valid UTF-8 will be skipped.
///
/// ```
/// use fontconfig::{Fontconfig, FontconfigError, Pattern};
///
/// # fn main() -> Result<(), FontconfigError> {
/// let fc = Fontconfig::new().expect("unable to init Fontconfig");
///
/// // Find fonts that support japanese
/// let fonts = fontconfig::list_fonts(&Pattern::new(&fc)?, None)?;
/// let ja_fonts: Vec<_> = fonts
///     .iter()
///     .filter(|p| p.lang_set().map_or(false, |mut langs| langs.any(|l| l == "ja")))
///     .collect();
/// # Ok(())
/// # }
/// ```
#[doc(alias = "FcStrList")]
pub struct StrList<'a> {
    list: *mut sys::FcStrList,
    _life: PhantomData<&'a sys::FcStrList>,
}

impl<'a> StrList<'a> {
    /// Wrap an existing `FcStrSet`.
    ///
    /// The returned wrapper assumes ownership of the `FcStrSet`.
    ///
    /// **Safety:** The string list pointer must be valid/non-null.
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

/// Wrapper around `FcFontSet` representing a set of fonts.
///
/// The `FontSet` can be iterated, yielding the [Pattern]s in the set.
#[doc(alias = "FcFontSet")]
pub struct FontSet<'fc> {
    fcset: *mut sys::FcFontSet,
    fc: &'fc Fontconfig,
}

impl<'fc> FontSet<'fc> {
    /// Create a new, empty `FontSet`.
    pub fn new(fc: &Fontconfig) -> Result<FontSet<'_>, FontconfigError> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcFontSetCreate,) };
        is_non_null(fcset)
            .then_some(FontSet { fcset, fc })
            .ok_or(FontconfigError::Failed)
    }

    /// Wrap an existing `FcFontSet`.
    ///
    /// The returned wrapper assumes ownership of the `FcFontSet`.
    ///
    /// **Safety:** The font set pointer must be valid/non-null.
    pub unsafe fn from_raw(fc: &Fontconfig, raw_set: *mut sys::FcFontSet) -> FontSet<'_> {
        FontSet { fcset: raw_set, fc }
    }

    /// Add a `Pattern` to this `FontSet`.
    #[doc(alias = "FcFontSetAdd")]
    pub fn add_pattern(&mut self, pat: Pattern) -> Result<(), FontconfigError> {
        unsafe { ffi_dispatch!(LIB, FcFontSetAdd, self.fcset, pat.pat).to_result() }
    }

    /// Print this `FontSet` to stdout.
    #[doc(alias = "FcFontSetPrint")]
    pub fn print(&self) {
        unsafe { ffi_dispatch!(LIB, FcFontSetPrint, self.fcset) };
    }

    /// Iterate the fonts (as `Patterns`) in this `FontSet`.
    pub fn iter(&self) -> impl Iterator<Item = Pattern<'_>> {
        let patterns = unsafe {
            let fontset = self.fcset;
            // The set may be empty, in which case .fonts is NULL, but slices require
            // the pointers to be non-NULL.
            if (*fontset).nfont > 0 {
                std::slice::from_raw_parts((*fontset).fonts, (*fontset).nfont as usize)
            } else {
                &[]
            }
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

/// Returns a [FontSet] containing [Font]s that match the supplied `pattern` and `objects`.
///
/// Creates patterns from those fonts containing only the objects in `objects` and returns
/// the set of unique such patterns. In other words, `objects` indicates that properties
/// of the font you want populated. The values of these on the matched fonts can be retrieved
/// using [Pattern::get_string] or [Pattern::get_int] on the returned patterns as wells as the
/// convenience functions such as [Pattern::name] and [Pattern::filename].
///
/// See [the fc-list example](https://github.com/yeslogic/fontconfig-rs/blob/master/examples/fc-list.rs)
/// for an example of using this function.
///
#[doc(alias = "FcFontList")]
pub fn list_fonts<'fc>(
    pattern: &Pattern<'fc>,
    objects: Option<&ObjectSet>,
) -> Result<FontSet<'fc>, FontconfigError> {
    let os = objects.map(|o| o.fcset).unwrap_or(ptr::null_mut());
    unsafe {
        let raw_set = ffi_dispatch!(LIB, FcFontList, ptr::null_mut(), pattern.pat, os);
        is_non_null(raw_set)
            .then(|| FontSet::from_raw(pattern.fc, raw_set))
            .ok_or(FontconfigError::Failed)
    }
}

/// Wrapper around `FcObjectSet`.
#[doc(alias = "FcObjectSet")]
pub struct ObjectSet {
    fcset: *mut sys::FcObjectSet,
}

impl ObjectSet {
    /// Create a new, empty `ObjectSet`.
    #[doc(alias = "FcObjectSetCreate")]
    pub fn new(_: &Fontconfig) -> Result<ObjectSet, FontconfigError> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcObjectSetCreate,) };
        is_non_null(fcset)
            .then(|| ObjectSet { fcset })
            .ok_or(FontconfigError::Failed)
    }

    /// Wrap an existing `FcObjectSet`.
    ///
    /// The `FcObjectSet` must not be null. This method assumes ownership of the `FcObjectSet`.
    ///
    /// **Safety:** The object set pointer must be valid/non-null.
    pub unsafe fn from_raw(_: &Fontconfig, raw_set: *mut sys::FcObjectSet) -> ObjectSet {
        assert!(!raw_set.is_null());
        ObjectSet { fcset: raw_set }
    }

    /// Add a string to the `ObjectSet`.
    #[doc(alias = "FcObjectSetAdd")]
    pub fn add(&mut self, name: &CStr) -> Result<(), FontconfigError> {
        unsafe { ffi_dispatch!(LIB, FcObjectSetAdd, self.fcset, name.as_ptr()).to_result() }
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

impl fmt::Display for FontFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontFormat::TrueType => write!(f, "TrueType"),
            FontFormat::Type1 => write!(f, "Type 1"),
            FontFormat::BDF => write!(f, "BDF"),
            FontFormat::PCF => write!(f, "PCF"),
            FontFormat::Type42 => write!(f, "Type 42"),
            FontFormat::CIDType1 => write!(f, "CID Type 1"),
            FontFormat::CFF => write!(f, "CFF"),
            FontFormat::PFR => write!(f, "PFR"),
            FontFormat::WindowsFNT => write!(f, "Windows FNT"),
        }
    }
}

/// Wrapper around `FcCharSet`.
#[repr(C)]
pub struct CharSet {
    char_set: *mut FcCharSet,
}

impl<'fc> CharSet {
    /// Create a new, empty `CharSet`.
    #[doc(alias = "FcCharSetCreate")]
    pub fn new(_: &'fc Fontconfig) -> Result<Self, FontconfigError> {
        let char_set = unsafe { ffi_dispatch!(LIB, FcCharSetCreate,) };
        is_non_null(char_set)
            .then_some(CharSet { char_set })
            .ok_or(FontconfigError::Failed)
    }

    /// Add a char to the `CharSet`.
    #[doc(alias = "FcCharSetAddChar")]
    pub fn add_char(&mut self, c: char) -> Result<(), FontconfigError> {
        unsafe { ffi_dispatch!(LIB, FcCharSetAddChar, self.char_set, c as u32).to_result() }
    }
}

impl Drop for CharSet {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcCharSetDestroy, self.char_set);
        }
    }
}

impl fmt::Display for FontconfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FontconfigError::Failed => f.write_str("operation failed"),
            FontconfigError::NulError => f.write_str("string contained NUL byte"),
            FontconfigError::Utf8Error => f.write_str("string contained invalid UTF-8"),
            FontconfigError::NoMatch => f.write_str("no matches"),
            FontconfigError::TypeMismatch => f.write_str("type mismatch"),
            FontconfigError::NoId => f.write_str("no matching id"),
            FontconfigError::OutOfMemory => f.write_str("out of memory"),
        }
    }
}

impl std::error::Error for FontconfigError {}

impl From<str::Utf8Error> for FontconfigError {
    fn from(_err: str::Utf8Error) -> Self {
        FontconfigError::Utf8Error
    }
}

impl From<ffi::NulError> for FontconfigError {
    fn from(_err: ffi::NulError) -> Self {
        FontconfigError::NulError
    }
}

impl ToResult for FcBool {
    fn to_result(self) -> Result<(), FontconfigError> {
        if self == FcTrue {
            Ok(())
        } else {
            Err(FontconfigError::Failed)
        }
    }
}

impl ToResult for FcResult {
    fn to_result(self) -> Result<(), FontconfigError> {
        match self {
            sys::FcResultMatch => Ok(()),
            sys::FcResultNoMatch => Err(FontconfigError::NoMatch),
            sys::FcResultTypeMismatch => Err(FontconfigError::TypeMismatch),
            sys::FcResultNoId => Err(FontconfigError::NoId),
            sys::FcResultOutOfMemory => Err(FontconfigError::OutOfMemory),
            _ => Err(FontconfigError::Failed),
        }
    }
}

fn is_non_null<T>(ptr: *mut T) -> bool {
    !ptr.is_null()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(Fontconfig::new().is_some())
    }

    #[test]
    fn test_find_font() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        fc.find("dejavu sans", None)?.print_debug();
        fc.find("dejavu sans", Some("oblique"))?.print_debug();
        Ok(())
    }

    #[test]
    fn test_iter_and_print() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let fontset = list_fonts(&Pattern::new(&fc)?, None)?;
        for pattern in fontset.iter() {
            println!("{:?}", pattern.name());
        }

        // Ensure that the set can be iterated again
        assert!(fontset.iter().count() > 0);
        Ok(())
    }

    #[test]
    fn test_empty_font_set() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let mut pat = Pattern::new(&fc)?;
        let family = CString::new("xxx yyy zzz does not exist")?;
        pat.add_string(FC_FAMILY, &family)?;

        let fontset = list_fonts(&pat, None)?;
        // Prior to a bug fix this would fail when the set was empty due to trying
        // to create a slice from a NULL pointer (FcSet.fonts).
        assert_eq!(fontset.iter().count(), 0);
        Ok(())
    }

    #[test]
    fn iter_lang_set() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let mut pat = Pattern::new(&fc)?;
        let family = CString::new("dejavu sans")?;
        pat.add_string(FC_FAMILY, &family)?;
        let pattern = pat.font_match()?;
        for lang in pattern.lang_set()? {
            println!("{:?}", lang);
        }

        // Test find
        assert!(pattern.lang_set()?.find(|&lang| lang == "za").is_some());

        // Test collect
        let langs = pattern.lang_set()?.collect::<Vec<_>>();
        assert!(langs.iter().find(|&&l| l == "ie").is_some());
        Ok(())
    }

    #[test]
    fn test_sort_fonts() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let mut pat = Pattern::new(&fc)?;
        let family = CString::new("dejavu sans")?;
        pat.add_string(FC_FAMILY, &family)?;

        let style = CString::new("oblique")?;
        pat.add_string(FC_STYLE, &style)?;

        let font_set = pat.sort_fonts(UnicodeCoverage::NoTrim)?;

        for pattern in font_set.iter() {
            println!("{:?}", pattern.name());
        }

        // Ensure that the set can be iterated again
        assert!(font_set.iter().count() > 0);
        Ok(())
    }

    #[test]
    fn finds_font_containing_charset() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let mut pat = Pattern::new(&fc)?;
        let mut char_set = CharSet::new(&fc)?;
        char_set.add_char('a')?;
        pat.add_charset(char_set)?;
        let font_set = list_fonts(&pat, None)?;

        // Should find at least one font
        assert!(font_set.iter().count() > 0);
        Ok(())
    }

    #[test]
    fn does_not_find_missing_charset() -> Result<(), FontconfigError> {
        let fc = Fontconfig::new().expect("FcInit");
        let mut pat = Pattern::new(&fc)?;
        let mut char_set = CharSet::new(&fc)?;
        // DejaVu Sans does not support CJK so try finding it for the following U+5317
        char_set.add_char('北')?;
        let family = CString::new("dejavu sans")?;
        pat.add_string(FC_FAMILY, &family)?;
        pat.add_charset(char_set)?;
        let font_set = list_fonts(&pat, None)?;

        // Font set should be empty
        assert_eq!(font_set.iter().count(), 0);
        Ok(())
    }
}
