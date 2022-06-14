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
//! use fontconfig::FontConfig;
//!
//! fn main() {
//!     let mut config = FontConfig::default();
//!     // `FontConfig::find()` returns `Option` (will rarely be `None` but still could be)
//!     let font = config.find("freeserif", None).unwrap();
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

use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_char;
use std::path::PathBuf;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub use sys::constants::*;
use sys::{FcBool, FcPattern};

use thiserror::Error;

mod langset;
pub use langset::{LangSet, LangSetCmp};

///
pub mod fontset;
pub use fontset::FontSet;

///
pub mod stringset;
pub use stringset::StringSet;

mod matrix;
pub use matrix::Matrix;

#[allow(non_upper_case_globals)]
const FcTrue: FcBool = 1;
#[allow(non_upper_case_globals, dead_code)]
const FcFalse: FcBool = 0;

type Result<T> = std::result::Result<T, Error>;

static INITIALIZED: once_cell::sync::Lazy<Arc<Mutex<usize>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(0)));

/// Error type returned from Pattern::format.
///
/// The error holds the name of the unknown format.
#[derive(Debug, Error)]
pub enum Error {
    /// The format is not known.
    #[error("Unknown format {0}")]
    UnknownFontFormat(String),
    /// Out of memory error.
    #[error("malloc failed")]
    OutOfMemory,
    /// Object exists, but has fewer values than specified
    #[error("Object exists, but has fewer values than specified")]
    NoId,
    /// Object exists, but the type doesn't match.
    #[error("Object exists, but the type doesn't match")]
    TypeMismatch,
    /// Object doesn't exist at all.
    #[error("Object doesn't exist at all")]
    NoMatch,
}

trait ToResult {
    fn ok(&self) -> Result<()>;

    fn opt(&self) -> Option<()> {
        self.ok().ok()
    }
}

impl ToResult for sys::FcResult {
    fn ok(&self) -> Result<()> {
        match *self {
            sys::FcResultMatch => Ok(()),
            sys::FcResultNoMatch => Err(Error::NoMatch),
            sys::FcResultTypeMismatch => Err(Error::TypeMismatch),
            sys::FcResultNoId => Err(Error::NoId),
            _ => unreachable!(),
        }
    }
}

///
pub enum MatchKind {
    /// Tagged as pattern operations are applied
    Pattern,
    /// Tagged as font operations are applied and `pat` is used for <test> elements with target=pattern
    Font,
    ///
    Scan,
}

#[doc(hidden)]
impl From<sys::FcMatchKind> for MatchKind {
    fn from(kind: sys::FcMatchKind) -> Self {
        match kind {
            sys::FcMatchPattern => MatchKind::Pattern,
            sys::FcMatchFont => MatchKind::Font,
            sys::FcMatchScan => MatchKind::Scan,
            _ => unreachable!(),
        }
    }
}

#[doc(hidden)]
impl From<MatchKind> for sys::FcMatchKind {
    fn from(kind: MatchKind) -> sys::FcMatchKind {
        match kind {
            MatchKind::Pattern => sys::FcMatchPattern,
            MatchKind::Font => sys::FcMatchFont,
            MatchKind::Scan => sys::FcMatchScan,
        }
    }
}

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

/// Handle obtained after Fontconfig has been initialised.
#[doc(alias = "FcConfig")]
pub struct FontConfig {
    cfg: Option<NonNull<sys::FcConfig>>,
}

///
impl FontConfig {
    /// Create a configuration
    // pub fn new() -> Option<Self> {
    //     #[cfg(feature = "dlopen")]
    //     if LIB_RESULT.is_err() {
    //         return None;
    //     }
    //     let cfg = unsafe { ffi_dispatch!(LIB, FcConfigCreate,) };
    //     Some(FontConfig {
    //         cfg: Some(NonNull::new(cfg)?),
    //     })
    // }

    /// Set configuration as default.
    ///
    /// Sets the current default configuration to config.
    /// Implicitly calls FcConfigBuildFonts if necessary,
    /// and FcConfigReference() to inrease the reference count in config since 2.12.0,
    /// returning FcFalse if that call fails.
    // pub fn set_current(config: &mut FontConfig) {
    //     //
    // }

    /// Execute substitutions
    ///
    /// Calls FcConfigSubstituteWithPat setting p_pat to NULL.
    /// Returns false if the substitution cannot be performed (due to allocation failure).
    /// Otherwise returns true.
    pub fn substitute(&mut self, pat: &mut Pattern, kind: MatchKind) {
        let ret = unsafe {
            ffi_dispatch!(
                LIB,
                FcConfigSubstitute,
                self.as_mut_ptr(),
                pat.as_mut_ptr(),
                kind.into()
            )
        };
        assert_eq!(ret, FcTrue);
    }

    /// Return the best font from a set of font sets
    ///
    /// Finds the font in sets most closely matching pattern and
    /// returns the result of FcFontRenderPrepare for that font and the provided pattern.
    /// This function should be called only after FcConfigSubstitute and FcDefaultSubstitute have been called for pattern;
    /// otherwise the results will not be correct.
    /// If config is NULL, the current configuration is used.
    /// Returns NULL if an error occurs during this process.
    pub fn match_(&mut self, sets: &mut [FontSet], pat: &mut Pattern) -> Pattern {
        pat.default_substitute();
        self.substitute(pat, MatchKind::Font);
        let mut result = sys::FcResultNoMatch;
        let pat = unsafe {
            ffi_dispatch!(
                LIB,
                FcFontSetMatch,
                self.as_mut_ptr(),
                &mut sets.as_mut_ptr().cast(),
                sets.len() as i32,
                pat.as_mut_ptr(),
                &mut result
            )
        };
        result.ok().unwrap();
        Pattern {
            pat: NonNull::new(pat).unwrap(),
        }
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcConfig {
        if let Some(ref mut cfg) = self.cfg {
            cfg.as_ptr()
        } else {
            ptr::null_mut()
        }
    }

    #[allow(dead_code)]
    fn as_ptr(&self) -> *const sys::FcConfig {
        if let Some(ref cfg) = self.cfg {
            cfg.as_ptr()
        } else {
            ptr::null_mut()
        }
    }

    /// Find a font of the given `family` (e.g. Dejavu Sans, FreeSerif),
    /// optionally filtering by `style`. Both fields are case-insensitive.
    pub fn find(&mut self, family: &str, style: Option<&str>) -> Option<Font> {
        let mut pat = Pattern::new();
        let family = CString::new(family).ok()?;
        pat.add_string(FC_FAMILY.as_cstr(), &family);

        if let Some(style) = style {
            let style = CString::new(style).ok()?;
            pat.add_string(FC_STYLE.as_cstr(), &style);
        }

        let font_match = pat.font_match(self);

        font_match.name().and_then(|name| {
            font_match.filename().map(|filename| Font {
                name: name.to_owned(),
                path: PathBuf::from(filename),
            })
        })
    }

    /// Return a `FontSet` containing Fonts that match the supplied `pattern` and `objects`.
    pub fn list_fonts(&mut self, mut pattern: Pattern, objects: Option<&mut ObjectSet>) -> FontSet {
        let os = objects.map(|o| o.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let set =
            unsafe { ffi_dispatch!(LIB, FcFontList, self.as_mut_ptr(), pattern.as_mut_ptr(), os) };
        // NOTE: Referenced by FontSet, should not drop it.
        mem::forget(pattern);
        FontSet {
            fcset: NonNull::new(set).unwrap(),
        }
    }
}

impl Default for FontConfig {
    /// Initialise fontconfig and returns the default config.
    ///
    /// **PANIC** : If fontconfig fails to initialise
    fn default() -> Self {
        let mut guard = INITIALIZED.lock().unwrap();
        #[cfg(feature = "dlopen")]
        if LIB_RESULT.is_err() {
            panic!("Failed to load fontconfig library");
        }
        assert_eq!(FcTrue, unsafe { ffi_dispatch!(LIB, FcInit,) });
        *guard += 1;
        FontConfig { cfg: None }
    }
}

impl Drop for FontConfig {
    fn drop(&mut self) {
        let guard = INITIALIZED.lock().unwrap();
        if guard.checked_sub(1).unwrap() == 0 {
            unsafe { ffi_dispatch!(LIB, FcFini,) };
        }
    }
}

/// A very high-level view of a font, only concerned with the name and its file location.
///
/// ##Example
/// ```rust
/// use fontconfig::{Font, FontConfig};
///
/// let mut config = FontConfig::default();
/// let font = config.find("sans-serif", Some("italic")).unwrap();
/// println!("Name: {}\nPath: {}", font.name, font.path.display());
/// ```
pub struct Font {
    /// The true name of this font
    pub name: String,
    /// The location of this font on the filesystem.
    pub path: PathBuf,
}
impl Font {
    #[allow(dead_code)]
    fn print_debug(&self) {
        println!("Name: {}\nPath: {}", self.name, self.path.display());
    }
}

/// A safe wrapper around fontconfig's `FcPattern`.
pub struct Pattern {
    /// Raw pointer to `FcPattern`
    pat: NonNull<FcPattern>,
}

impl Pattern {
    /// Create a new empty `Pattern`.
    pub fn new() -> Pattern {
        let pat = unsafe { ffi_dispatch!(LIB, FcPatternCreate,) };
        assert!(!pat.is_null());

        Pattern {
            pat: NonNull::new(pat).expect("out of memory"),
        }
    }

    /// Create a `Pattern` from a raw fontconfig FcPattern pointer.
    ///
    /// The pattern is referenced.
    ///
    /// **Panic:** If the pointer is null.
    ///
    /// **Safety:** The pattern pointer must be valid/non-null.
    // pub unsafe fn from_raw(pat: *mut FcPattern) -> Pattern {
    //     // ffi_dispatch!(LIB, FcPatternReference, pat);
    //
    //     Pattern {
    //         pat: NonNull::new(pat).unwrap(),
    //     }
    // }

    /// Add a key-value pair to this pattern.
    ///
    /// See useful keys in the [fontconfig reference][1].
    ///
    /// [1]: http://www.freedesktop.org/software/fontconfig/fontconfig-devel/x19.html
    pub fn add_string(&mut self, name: &CStr, val: &CStr) {
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternAddString,
                self.as_mut_ptr(),
                name.as_ptr(),
                val.as_ptr() as *const u8
            );
        }
    }

    /// Get string the value for a key from this pattern.
    pub fn string<'a>(&'a self, name: &'a CStr) -> Option<&'a str> {
        unsafe {
            let mut ret: *mut sys::FcChar8 = ptr::null_mut();
            ffi_dispatch!(
                LIB,
                FcPatternGetString,
                self.pat.as_ptr(),
                name.as_ptr(),
                0,
                &mut ret as *mut _
            )
            .opt()?;
            let cstr = CStr::from_ptr(ret as *const c_char);
            Some(cstr.to_str().unwrap())
        }
    }

    /// Get the integer value for a key from this pattern.
    pub fn int(&self, name: &CStr) -> Option<i32> {
        unsafe {
            let mut ret: i32 = 0;
            ffi_dispatch!(
                LIB,
                FcPatternGetInteger,
                self.pat.as_ptr(),
                name.as_ptr(),
                0,
                &mut ret as *mut i32
            )
            .opt()?;
            Some(ret)
        }
    }

    /// Print this pattern to stdout with all its values.
    pub fn print(&self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternPrint, self.pat.as_ptr());
        }
    }

    fn default_substitute(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcDefaultSubstitute, self.pat.as_mut());
        }
    }

    /// Get the best available match for this pattern, returned as a new pattern.
    pub fn font_match(&mut self, config: &mut FontConfig) -> Pattern {
        self.default_substitute();
        config.substitute(self, MatchKind::Pattern);

        unsafe {
            let mut res = sys::FcResultNoMatch;
            let pat = ffi_dispatch!(
                LIB,
                FcFontMatch,
                config.as_mut_ptr(),
                self.pat.as_mut(),
                &mut res
            );
            res.ok().unwrap();
            Pattern {
                pat: NonNull::new(pat).unwrap(),
            }
        }
    }

    /// Get the list of fonts sorted by closeness to self.
    /// If trim is `true`, elements in the list which don't include Unicode coverage not provided by earlier elements in the list are elided.
    pub fn font_sort(&mut self, config: &mut FontConfig, trim: bool) -> Option<FontSet> {
        self.default_substitute();
        config.substitute(self, MatchKind::Pattern);
        let mut pat = self.clone();
        unsafe {
            // What is this result actually used for? Seems redundant with
            // return type.
            let mut res = sys::FcResultNoMatch;

            let mut charsets: *mut _ = ptr::null_mut();

            let fcset = ffi_dispatch!(
                LIB,
                FcFontSort,
                config.as_mut_ptr(),
                pat.as_mut_ptr(),
                if trim { FcTrue } else { FcFalse }, // Trim font list.
                &mut charsets,
                &mut res
            );
            res.opt()?;
            Some(FontSet {
                fcset: NonNull::new(fcset).unwrap(),
            })
        }
    }

    /// Prepare pattern for loading font file.
    ///
    /// Creates a new pattern consisting of elements of font not appearing in pat,
    /// elements of pat not appearing in font and the best matching value from pat for elements appearing in both.
    /// The result is passed to FcConfigSubstituteWithPat with kind FcMatchFont and then returned.
    #[doc(alias = "FcFontRenderPrepare")]
    pub fn render_prepare(&self, font: &Self) -> Self {
        let pat = unsafe {
            ffi_dispatch!(
                LIB,
                FcFontRenderPrepare,
                ptr::null_mut(),
                self.pat.as_ptr(),
                font.pat.as_ptr()
            )
        };
        Pattern {
            pat: NonNull::new(pat).unwrap(),
        }
    }

    /// Get character map
    #[doc(alias = "FcPatternGetCharSet")]
    pub fn charset(&self) -> Option<CharSet> {
        unsafe {
            let mut charsets = ffi_dispatch!(LIB, FcCharSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetCharSet,
                self.pat.as_ptr(),
                FC_CHARSET.as_ptr(),
                0,
                &mut charsets
            );
            if charsets.is_null() {
                None
            } else {
                let charsets = ffi_dispatch!(LIB, FcCharSetCopy, charsets);
                NonNull::new(charsets).map(|fcset| CharSet {
                    fcset,
                    _marker: PhantomData,
                })
            }
        }
    }

    /// Get the "fullname" (human-readable name) of this pattern.
    pub fn name(&self) -> Option<&str> {
        self.string(FC_FULLNAME.as_cstr())
    }

    /// Get the "file" (path on the filesystem) of this font pattern.
    pub fn filename(&self) -> Option<&str> {
        self.string(FC_FILE.as_cstr())
    }

    /// Get the "index" (The index of the font within the file) of this pattern.
    pub fn face_index(&self) -> Option<i32> {
        self.int(FC_INDEX.as_cstr())
    }

    /// Get the "slant" (Italic, oblique or roman) of this pattern.
    pub fn slant(&self) -> Option<i32> {
        self.int(FC_SLANT.as_cstr())
    }

    /// Get the "weight" (Light, medium, demibold, bold or black) of this pattern.
    pub fn weight(&self) -> Option<i32> {
        self.int(FC_WEIGHT.as_cstr())
    }

    /// Get the "width" (Condensed, normal or expanded) of this pattern.
    pub fn width(&self) -> Option<i32> {
        self.int(FC_WIDTH.as_cstr())
    }

    /// Get the "fontformat" ("TrueType" "Type 1" "BDF" "PCF" "Type 42" "CID Type 1" "CFF" "PFR" "Windows FNT") of this pattern.
    pub fn format(&self) -> Result<FontFormat> {
        self.string(FC_FONTFORMAT.as_cstr())
            .ok_or_else(|| Error::UnknownFontFormat(String::new()))
            .and_then(|format| format.parse())
    }

    /// Returns a raw pointer to underlying `FcPattern`.
    pub fn as_ptr(&self) -> *const FcPattern {
        self.pat.as_ptr()
    }

    /// Returns an unsafe mutable pointer to the underlying `FcPattern`.
    pub fn as_mut_ptr(&mut self) -> *mut FcPattern {
        self.pat.as_ptr()
    }
}

impl FromStr for Pattern {
    type Err = Error;
    /// Converts `name` from the standard text format described above into a pattern.
    fn from_str(s: &str) -> Result<Self> {
        let c_str = CString::new(s).unwrap();
        unsafe {
            let pat = ffi_dispatch!(LIB, FcNameParse, c_str.as_ptr().cast());
            if let Some(pat) = NonNull::new(pat) {
                Ok(Pattern { pat })
            } else {
                Err(Error::OutOfMemory)
            }
        }
    }
}

impl std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let fcstr = unsafe { ffi_dispatch!(LIB, FcNameUnparse, self.pat.as_ptr()) };
        let fcstr = unsafe { CStr::from_ptr(fcstr as *const c_char) };
        let result = write!(f, "{:?}", fcstr);
        unsafe { ffi_dispatch!(LIB, FcStrFree, fcstr.as_ptr() as *mut u8) };
        result
    }
}

impl Clone for Pattern {
    fn clone(&self) -> Self {
        let cloned = unsafe { ffi_dispatch!(LIB, FcPatternDuplicate, self.pat.as_ptr()) };
        Pattern {
            pat: NonNull::new(cloned).unwrap(),
        }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(LIB, FcPatternDestroy, self.pat.as_ptr());
        }
    }
}

impl Pattern {
    /// Get the languages set of this pattern.
    pub fn lang_set(&mut self) -> Option<LangSet> {
        // let mut langset = LangSet::new();
        let langset = unsafe {
            let mut langset = ffi_dispatch!(LIB, FcLangSetCreate,);
            ffi_dispatch!(
                LIB,
                FcPatternGetLangSet,
                self.as_mut_ptr(),
                FC_LANG.as_ptr(),
                0,
                &mut langset
            )
            .opt()?;
            ffi_dispatch!(LIB, FcLangSetCopy, langset)
        };
        NonNull::new(langset).map(|langset| LangSet { langset })
    }

    /// Get the matrix from this pattern.
    pub fn matrix(&mut self) -> Option<Matrix> {
        let mut matrix = Matrix::new();
        unsafe {
            ffi_dispatch!(
                LIB,
                FcPatternGetMatrix,
                self.as_mut_ptr(),
                FC_MATRIX.as_ptr(),
                0,
                &mut matrix.as_mut_ptr()
            )
            .opt()?;
        }
        Some(matrix)
    }
}

/// Wrapper around `FcObjectSet`.
pub struct ObjectSet {
    fcset: NonNull<sys::FcObjectSet>,
}

impl ObjectSet {
    /// Create a new, empty `ObjectSet`.
    pub fn new() -> ObjectSet {
        let fcset = unsafe { ffi_dispatch!(LIB, FcObjectSetCreate,) };

        ObjectSet {
            fcset: NonNull::new(fcset).unwrap(),
        }
    }

    /// Wrap an existing `FcObjectSet`.
    ///
    /// The `FcObjectSet` must not be null. This method assumes ownership of the `FcObjectSet`.
    ///
    /// **Safety:** The object set pointer must be valid/non-null.
    // pub unsafe fn from_raw(_: &Fontconfig, raw_set: *mut sys::FcObjectSet) -> ObjectSet {
    //     // ObjectSet { fcset: raw_set }
    // }

    /// Add a string to the `ObjectSet`.
    pub fn add(&mut self, name: &CStr) {
        let res = unsafe { ffi_dispatch!(LIB, FcObjectSetAdd, self.as_mut_ptr(), name.as_ptr()) };
        assert_eq!(res, FcTrue);
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcObjectSet {
        self.fcset.as_ptr()
    }

    #[allow(dead_code)]
    fn as_ptr(&self) -> *const sys::FcObjectSet {
        self.fcset.as_ptr()
    }
}

impl Drop for ObjectSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcObjectSetDestroy, self.as_mut_ptr()) }
    }
}

impl FromStr for FontFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
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
            _ => Err(Error::UnknownFontFormat(s.to_string())),
        }
    }
}

/// Wrapper around `FcCharSet`.
pub struct CharSet<'a> {
    fcset: NonNull<sys::FcCharSet>,
    _marker: PhantomData<&'a sys::FcCharSet>,
}

impl<'a> CharSet<'a> {
    /// Count entries in a charset
    pub fn len(&self) -> usize {
        let size = unsafe { ffi_dispatch!(LIB, FcCharSetCount, self.as_ptr()) };
        size as usize
    }

    /// Check if a character is in the `CharSet`.
    pub fn has_char(&self, c: char) -> bool {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetHasChar, self.as_ptr(), c as u32) };
        res == FcTrue
    }

    /// Check if self is a subset of other `CharSet`.
    pub fn is_subset(&self, other: &Self) -> bool {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetIsSubset, self.as_ptr(), other.as_ptr()) };
        res == FcTrue
    }

    /// Merge self with other `CharSet`.
    pub fn merge(&mut self, other: &Self) {
        let res = unsafe {
            ffi_dispatch!(
                LIB,
                FcCharSetMerge,
                self.as_mut_ptr(),
                other.as_ptr(),
                ptr::null_mut()
            )
        };
        assert_eq!(res, FcTrue);
    }

    /// Intersect self with other `CharSet`.
    pub fn intersect(&self, other: &Self) -> Self {
        let fcset =
            unsafe { ffi_dispatch!(LIB, FcCharSetIntersect, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("intersect failed"),
            _marker: PhantomData,
        }
    }

    /// Subtract other `CharSet` from self.
    pub fn subtract(&self, other: &Self) -> Self {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetSubtract, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("subtract failed"),
            _marker: PhantomData,
        }
    }

    /// Union self with other `CharSet`.
    pub fn union(&self, other: &Self) -> Self {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetUnion, self.as_ptr(), other.as_ptr()) };
        Self {
            fcset: NonNull::new(fcset).expect("union failed"),
            _marker: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const sys::FcCharSet {
        self.fcset.as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcCharSet {
        self.fcset.as_ptr()
    }
}

impl CharSet<'static> {
    /// Create a new, empty `CharSet`.
    pub fn new() -> CharSet<'static> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCreate,) };
        CharSet {
            fcset: NonNull::new(fcset).unwrap(),
            _marker: PhantomData,
        }
    }
    /// Add a character to the `CharSet`.
    pub fn add_char(&mut self, c: char) {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetAddChar, self.as_mut_ptr(), c as u32) };
        assert_eq!(res, FcTrue);
    }

    /// Delete a character from the `CharSet
    pub fn del_char(&mut self, c: char) {
        let res = unsafe { ffi_dispatch!(LIB, FcCharSetDelChar, self.as_mut_ptr(), c as u32) };
        assert_eq!(res, FcTrue);
    }
}

impl<'a> PartialEq for CharSet<'a> {
    fn eq(&self, other: &Self) -> bool {
        let res = unsafe {
            ffi_dispatch!(
                LIB,
                FcCharSetEqual,
                self.fcset.as_ptr(),
                other.fcset.as_ptr()
            )
        };
        res == FcTrue
    }
}

impl<'a> Clone for CharSet<'a> {
    fn clone(&self) -> CharSet<'a> {
        let fcset = unsafe { ffi_dispatch!(LIB, FcCharSetCopy, self.fcset.as_ptr()) };
        CharSet {
            fcset: NonNull::new(fcset).expect("Can't clone CharSet"),
            _marker: PhantomData,
        }
    }
}

impl Drop for CharSet<'_> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcCharSetDestroy, self.as_mut_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        FontConfig::default();
        // assert!(FontConfig::new().is_some());
    }

    #[test]
    fn find_font() {
        let mut config = FontConfig::default();
        config.find("dejavu sans", None).unwrap().print_debug();
        config
            .find("dejavu sans", Some("oblique"))
            .unwrap()
            .print_debug();
    }

    #[test]
    fn iter_and_print() {
        let mut config = FontConfig::default();
        let fontset = config.list_fonts(Pattern::new(), None);
        for pattern in fontset.iter() {
            println!("{:?}", pattern.name());
        }

        // Ensure that the set can be iterated again
        assert!(fontset.iter().count() > 0);
    }

    #[test]
    fn iter_lang_set() {
        let mut config = FontConfig::default();
        let mut pat = Pattern::new();
        let family = CString::new("dejavu sans").unwrap();
        pat.add_string(FC_FAMILY.as_cstr(), &family);
        let mut pattern = pat.font_match(&mut config);
        for lang in pattern.lang_set().unwrap().langs().iter() {
            println!("{:?}", lang);
        }

        // Test find
        assert!(pattern
            .lang_set()
            .unwrap()
            .langs()
            .iter()
            .any(|lang| lang == "za"));

        // Test collect
        let langs = pattern
            .lang_set()
            .unwrap()
            .langs()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        assert!(langs.iter().any(|l| l == "ie"));
    }

    #[test]
    fn iter_font_sort() {
        let mut config = FontConfig::default();
        let mut pat = Pattern::new();
        let family = CString::new("dejavu sans").unwrap();
        pat.add_string(FC_FAMILY.as_cstr(), &family);
        let font_set = pat.font_sort(&mut config, false).unwrap();

        for font in font_set.iter() {
            println!("{:?}", font.name());
        }
        assert!(font_set.iter().count() > 1);
        assert!(font_set.iter().next().unwrap().name().unwrap() == "DejaVu Sans");
    }
}
