//!
use std::ffi::CStr;
use std::fmt;
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::{CharSet, FcTrue, StringSet};

/// The results of comparing two language strings in [`LangSet`] objects.
#[doc(alias = "FcLangResult")]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LangSetCmp {
    /// The objects match language and territory
    #[doc(alias = "FcLangEqual")]
    Equal, /*= sys::FcLangEqual*/
    /// The objects match in territory but differ in language .
    #[doc(alias = "FcLangDifferentCountry")]
    DifferentCountry, /*= sys::FcLangDifferentCountry,*/
    /// The objects match in language but differ in territory.
    #[doc(alias = "FcLangDifferentTerritory")]
    DifferentTerritory, /*= sys::FcLangDifferentTerritory,*/
    /// The objects differ in language.
    #[doc(alias = "FcLangDifferentLang")]
    DifferentLang, /*= sys::FcLangDifferentLang,*/
}

#[doc(hidden)]
impl From<sys::FcLangResult> for LangSetCmp {
    fn from(value: sys::FcLangResult) -> Self {
        match value {
            sys::FcLangEqual => LangSetCmp::Equal,
            sys::FcLangDifferentTerritory => LangSetCmp::DifferentTerritory,
            #[allow(unreachable_patterns)]
            sys::FcLangDifferentCountry => LangSetCmp::DifferentCountry,
            sys::FcLangDifferentLang => LangSetCmp::DifferentLang,
            _ => unreachable!(),
        }
    }
}

/// An abstract type that holds the set of languages supported by a font.
///
/// Operations to build and compare these sets are provided.   
/// These are computed for a font based on orthographic information built into the fontconfig library.   
/// Fontconfig has orthographies for all of the [ISO 639-1 languages](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes)
/// except for `MS`, `NA`, `PA`, `PS`, `QU`, `RN`, `RW`, `SD`, `SG`, `SN`, `SU` and `ZA`.
#[doc(alias = "FcLangSet")]
pub struct LangSet {
    pub(crate) langset: NonNull<sys::FcLangSet>,
}

impl LangSet {
    /// Create a new langset object
    #[doc(alias = "FcLangSetCreate")]
    pub fn new() -> LangSet {
        let langset = unsafe { ffi_dispatch!(LIB, FcLangSetCreate,) };
        LangSet {
            langset: NonNull::new(langset).unwrap(),
        }
    }

    /// Add a language to a langset
    ///
    /// `lang` should be of the form `Ll-Tt` where `Ll` is a two or three letter language from [ISO 639](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) and
    /// Tt is a territory from [ISO 3166](https://en.wikipedia.org/wiki/ISO_3166-1).
    #[doc(alias = "FcLangSetAdd")]
    pub fn push(&mut self, lang: &CStr) {
        let lang = lang.as_ptr() as *const u8;
        let _ = unsafe { ffi_dispatch!(LIB, FcLangSetAdd, self.as_mut_ptr(), lang) };
    }

    /// Delete a language from a langset
    ///
    /// `lang` is removed from self.   
    /// `lang` should be of the form `Ll-Tt` where `Ll` is a two or three letter language from [ISO 639](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes) and
    /// Tt is a territory from [ISO 3166](https://en.wikipedia.org/wiki/ISO_3166-1).
    pub fn remove(&mut self, _lang: &CStr) {
        unimplemented!("requires version 2.9.0");
        // let lang = lang.as_ptr() as *const u8;
        // let _ = unsafe { ffi_dispatch!(LIB, FcLangSetDel, self.as_mut_ptr(), lang) };
    }

    /// Compare language sets
    ///
    /// Compares language coverage for `self` and `other`.    
    /// If they share any language and territory pair, this function returns [`LangSetCmp::Equal`].   
    /// If they share a language but differ in which territory that language is for,
    ///   this function returns [`LangSetCmp::DifferentTerritory`].    
    /// If they share no languages in common, this function returns [`LangSetCmp::DifferentLang`].
    #[doc(alias = "FcLangSetCompare")]
    #[allow(clippy::should_implement_trait)]
    pub fn cmp(&self, other: &LangSet) -> LangSetCmp {
        let cmp = unsafe { ffi_dispatch!(LIB, FcLangSetCompare, self.as_ptr(), other.as_ptr()) };
        cmp.into()
    }

    /// Check langset subset relation
    ///
    /// Returns true if self contains every language in `rhs`.   
    /// self will 'contain' a language from `rhs` if self has exactly the language,
    /// or either the language or self has no territory.
    pub fn contains(&self, rhs: &LangSet) -> bool {
        let contains =
            unsafe { ffi_dispatch!(LIB, FcLangSetContains, self.as_ptr(), rhs.as_ptr()) };
        contains == FcTrue
    }

    /// Get the list of languages in the langset
    ///
    /// Returns a string set of all languages in langset.
    #[doc(alias = "FcLangSetGetLangs")]
    pub fn langs(&self) -> StringSet {
        let strings = unsafe { ffi_dispatch!(LIB, FcLangSetGetLangs, self.as_ptr()) };
        StringSet {
            set: NonNull::new(strings).unwrap(),
        }
    }

    /// Get character map for a language
    #[doc(alias = "FcLangGetCharSet")]
    pub fn charset(lang: &CStr) -> &CharSet {
        unsafe {
            let charset = ffi_dispatch!(LIB, FcLangGetCharSet, lang.as_ptr() as *const _);
            &*(charset as *const CharSet)
        }
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcLangSet {
        self.langset.as_ptr()
    }

    pub(crate) fn as_ptr(&self) -> *const sys::FcLangSet {
        self.langset.as_ptr()
    }
}

impl Clone for LangSet {
    fn clone(&self) -> LangSet {
        let langset = unsafe { ffi_dispatch!(LIB, FcLangSetCopy, self.as_ptr()) };
        LangSet {
            langset: NonNull::new(langset).unwrap(),
        }
    }
}

impl Drop for LangSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcLangSetDestroy, self.as_mut_ptr()) };
    }
}

impl PartialEq for LangSet {
    fn eq(&self, other: &Self) -> bool {
        let is_eq = unsafe { ffi_dispatch!(LIB, FcLangSetEqual, self.as_ptr(), other.as_ptr()) };
        is_eq == FcTrue
    }
}

impl Default for LangSet {
    /// Returns a string set of the default languages according to the environment variables on the system.   
    /// This function looks for them in order of `FC_LANG`, `LC_ALL`, `LC_CTYPE` and `LANG` then.   
    /// If there are no valid values in those environment variables, "en" will be set as fallback.   
    fn default() -> Self {
        // FIXME: use `FcGetDefaultLangs` which added version 2.9.91.
        // unsafe { ffi_dispatch!(LIB, FcGetDefaultLangs,) }
        LangSet::new()
    }
}

impl fmt::Debug for LangSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let langs = self.langs();
        write!(f, "LangSet {{ langs: {:?} }}", langs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let langset = LangSet::new();
        assert_eq!(langset.langs().iter().count(), 0);
        assert_eq!(langset.langs().iter().next(), None);
    }

    #[test]
    fn push() {
        let mut langset = LangSet::new();
        langset.push(CStr::from_bytes_with_nul(b"en\0").unwrap());
        assert_eq!(langset.langs().iter().count(), 1);
        let langs = langset.langs();
        let mut langs = langs.iter();
        assert_eq!(langs.next(), Some("en"));
        assert_eq!(langs.next(), None);
    }

    #[test]
    fn debug() {
        let mut langset = LangSet::new();
        assert_eq!(format!("{:?}", langset), "LangSet { langs: {} }");
        langset.push(CStr::from_bytes_with_nul(b"en\0").unwrap());
        assert_eq!(format!("{:?}", langset), "LangSet { langs: {\"en\"} }");
    }

    #[test]
    fn contains() {
        let mut langset = LangSet::new();
        langset.push(CStr::from_bytes_with_nul(b"en\0").unwrap());
        langset.push(CStr::from_bytes_with_nul(b"zh\0").unwrap());
        let mut langset2 = LangSet::new();
        langset2.push(CStr::from_bytes_with_nul(b"en\0").unwrap());
        assert!(langset.contains(&langset2));
        assert!(!langset2.contains(&langset));
        langset2.push(CStr::from_bytes_with_nul(b"fr\0").unwrap());
        assert!(!langset.contains(&langset2));
    }

    #[test]
    fn compare() {
        let mut langset = LangSet::new();
        langset.push(CStr::from_bytes_with_nul(b"en-US\0").unwrap());
        let mut langset2 = LangSet::new();
        langset2.push(CStr::from_bytes_with_nul(b"en-UK\0").unwrap());
        assert_eq!(langset.cmp(&langset2), LangSetCmp::DifferentTerritory);
        let mut langset3 = LangSet::new();
        langset3.push(CStr::from_bytes_with_nul(b"en-US\0").unwrap());
        assert_eq!(langset3.cmp(&langset), LangSetCmp::Equal);
        let mut langset4 = LangSet::new();
        langset4.push(CStr::from_bytes_with_nul(b"fr\0").unwrap());
        assert_eq!(langset3.cmp(&langset4), LangSetCmp::DifferentLang);
    }

    #[test]
    fn charset() {
        use crate::OwnedCharSet;

        let charset = LangSet::charset(CStr::from_bytes_with_nul(b"en\0").unwrap());
        let mut cs = OwnedCharSet::new();
        for c in [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
            'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
            'z', 'À', 'Ç', 'È', 'É', 'Ê', 'Ë', 'Ï', 'Ñ', 'Ô', 'Ö', 'à', 'ç', 'è', 'é', 'ê', 'ë',
            'ï', 'ñ', 'ô', 'ö',
        ] {
            cs.add_char(c);
        }
        assert_eq!(
            charset.iter().count(),
            72,
            "{:?}",
            charset.iter().collect::<Vec<_>>()
        );
        assert!(charset.is_subset(&cs));
        assert!(cs.is_subset(charset));
        assert_eq!(charset, cs.as_ref());
    }
}
