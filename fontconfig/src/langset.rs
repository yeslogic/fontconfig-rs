use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::{CharSet, FcTrue, StringSet};

/// The results of comparing two language strings or FcLangSet objects.
#[doc(alias = "FcLangResult")]
#[derive(Debug, Copy, Clone)]
pub enum LangSetCmp {
    /// The objects match language and territory
    Equal, /*= sys::FcLangEqual*/
    /// The objects match in territory but differ in language .
    DifferentCountry, /*= sys::FcLangDifferentCountry,*/
    /// The objects match in language but differ in territory.
    DifferentTerritory, /*= sys::FcLangDifferentTerritory,*/
    /// The objects differ in language.
    DifferentLang, /*= sys::FcLangDifferentLang,*/
}

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
/// Fontconfig has orthographies for all of the ISO 639-1 languages
/// except for MS, NA, PA, PS, QU, RN, RW, SD, SG, SN, SU and ZA.
/// If you have orthographic information for any of these languages, please submit them.
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
    /// lang should be of the form Ll-Tt where Ll is a two or three letter language from ISO 639 and
    /// Tt is a territory from ISO 3166.
    #[doc(alias = "FcLangSetAdd")]
    pub fn push(&mut self, lang: &CStr) {
        let lang = lang.as_ptr() as *const u8;
        let _ = unsafe { ffi_dispatch!(LIB, FcLangSetAdd, self.as_mut_ptr(), lang) };
    }

    /// Delete a language from a langset
    ///
    /// lang is removed from self.
    /// lang should be of the form Ll-Tt where Ll is a two or three letter language from ISO 639 and
    /// Tt is a territory from ISO 3166.
    pub fn remove(&mut self, _lang: &CStr) {
        unimplemented!("requires version 2.9.0");
        // let lang = lang.as_ptr() as *const u8;
        // let _ = unsafe { ffi_dispatch!(LIB, FcLangSetDel, self.as_mut_ptr(), lang) };
    }

    /// Compare language sets
    ///
    /// Compares language coverage for ls_a and ls_b.
    /// If they share any language and territory pair, this function returns FcLangEqual.
    /// If they share a language but differ in which territory that language is for,
    ///   this function returns FcLangDifferentTerritory.
    /// If they share no languages in common, this function returns FcLangDifferentLang.
    #[doc(alias = "FcLangSetCompare")]
    pub fn cmp(&self, rhs: &LangSet) -> LangSetCmp {
        let cmp = unsafe { ffi_dispatch!(LIB, FcLangSetCompare, self.as_ptr(), rhs.as_ptr()) };
        cmp.into()
    }

    /// Check langset subset relation
    ///
    /// Returns true if self contains every language in rhs.
    /// self will 'contain' a language from rhs if self has exactly the language,
    /// or either the language or self has no territory.
    pub fn contains(&self, rhs: &LangSet) -> bool {
        let contains =
            unsafe { ffi_dispatch!(LIB, FcLangSetContains, self.as_ptr(), rhs.as_ptr()) };
        contains == FcTrue
    }

    /// Get list of languages
    ///
    /// Returns a string set of all known languages.
    // pub fn langs() -> StringSet {
    //     let langset = unsafe { ffi_dispatch!(LIB, FcGetLangs,) };
    //     StringSet {
    //         langset: NonNull::new(langset).unwrap(),
    //     }
    // }

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
    pub fn charset<'a>(lang: &'a CStr) -> CharSet<'a> {
        let charset = unsafe { ffi_dispatch!(LIB, FcLangGetCharSet, lang.as_ptr() as *const _) };
        CharSet {
            fcset: NonNull::new(charset).unwrap(),
            _marker: PhantomData,
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
