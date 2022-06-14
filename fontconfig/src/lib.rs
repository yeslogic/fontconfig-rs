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
//! let mut config = FontConfig::default();
//! // `FontConfig::find()` returns `Option` (will rarely be `None` but still could be)
//! let font = config.find("freeserif", None).unwrap();
//! // `name` is a `String`, `path` is a `Path`
//! println!("Name: {}\nPath: {}", font.name, font.path.display());
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
use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::{LIB, LIB_RESULT};
#[cfg(not(feature = "dlopen"))]
use sys::*;

use std::ffi::{CString};
use std::mem;

use std::path::PathBuf;
use std::ptr::{self, NonNull};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub use sys::constants::*;
use sys::FcBool;

use thiserror::Error;

pub mod blanks;
pub mod charset;
pub mod fontset;
pub mod langset;
pub mod matrix;
pub mod objectset;
pub mod pattern;
pub mod strings;
pub mod stringset;

pub use blanks::Blanks;
pub use charset::CharSet;
pub use fontset::FontSet;
pub use langset::{LangSet, LangSetCmp};
pub use matrix::Matrix;
pub use objectset::ObjectSet;
pub use pattern::Pattern;
pub use strings::FcStr;
pub use stringset::StringSet;

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
        // pat.default_substitute();
        // self.substitute(pat, MatchKind::Font);
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
        if guard.checked_sub(1).unwrap_or_default() == 0 {
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

/// Returns the version number of the library.
pub fn version() -> i32 {
    unsafe { ffi_dispatch!(LIB, FcGetVersion,) }
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
