//!
use core::fmt;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::NonNull;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

/// FcStrSet holds a list of strings that can be appended to and enumerated.
/// Its unique characteristic is that the enumeration works even while strings are appended during enumeration.
#[doc(alias = "FcStrSet")]
pub struct StringSet {
    pub(crate) set: NonNull<sys::FcStrSet>,
}

impl StringSet {
    /// Create a new, empty `StringSet`.
    #[doc(alias = "FcStrSetCreate")]
    pub fn new() -> StringSet {
        let set = unsafe { ffi_dispatch!(LIB, FcStrSetCreate,) };
        StringSet {
            set: NonNull::new(set).unwrap(),
        }
    }

    /// Creates an iterator to list the strings in set.
    pub fn iter(&self) -> StringSetIter<'_> {
        StringSetIter::new(self)
    }

    fn as_mut_ptr(&mut self) -> *mut sys::FcStrSet {
        self.set.as_ptr()
    }

    fn as_ptr(&self) -> *const sys::FcStrSet {
        self.set.as_ptr()
    }
}

impl Drop for StringSet {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrSetDestroy, self.as_mut_ptr()) };
    }
}

/// Wrapper around `FcStrList`
///
/// The wrapper implements `Iterator` so it can be iterated directly, filtered etc.
/// **Note:** Any entries in the `StringSetIter` that are not valid UTF-8 will be skipped.
///
/// ```
/// use fontconfig::{FontConfig, OwnedPattern};
///
/// let mut config = FontConfig::default(); //.expect("unable to init FontConfig");
///
/// // Find fonts that support japanese
/// let mut pat = OwnedPattern::new();
/// let mut fonts = pat.font_list(&mut config, None);
/// let ja_fonts: Vec<_> = fonts
///         .iter_mut()
///         .filter(|p| {
///             p.lang_set().map_or(false, |langset|
///                 langset.langs().iter().any(|l| l == "ja"))
///         })
///         .collect();
/// println!("{:?}", ja_fonts);
/// println!("{}", ja_fonts.len());
/// ```
#[doc(alias = "FcStrList")]
pub struct StringSetIter<'a> {
    handle: NonNull<sys::FcStrList>,
    _marker: PhantomData<&'a StringSet>,
}

impl<'a> StringSetIter<'a> {
    fn new(set: &'a StringSet) -> StringSetIter<'a> {
        let iter = unsafe {
            let iter = ffi_dispatch!(LIB, FcStrListCreate, set.as_ptr() as *mut _);
            iter
        };
        StringSetIter {
            _marker: PhantomData,
            handle: NonNull::new(iter).unwrap(),
        }
    }
    fn as_mut_ptr(&mut self) -> *mut sys::FcStrList {
        self.handle.as_ptr()
    }
}

impl Drop for StringSetIter<'_> {
    fn drop(&mut self) {
        unsafe { ffi_dispatch!(LIB, FcStrListDone, self.as_mut_ptr()) };
    }
}

impl<'a> Iterator for StringSetIter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let s = ffi_dispatch!(LIB, FcStrListNext, self.as_mut_ptr());
            if s.is_null() {
                return None;
            }
            CStr::from_ptr(s.cast())
                .to_str()
                .ok()
                .or_else(|| self.next())
        }
    }
}

impl Default for StringSet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for StringSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{FontConfig, OwnedPattern};

    #[test]
    fn list_fonts_of_ja() {
        let mut config = FontConfig::default();
        let pat = OwnedPattern::new();
        let fonts = pat.font_list(&mut config, None);
        let ja_fonts = fonts.iter().filter(|p| {
            p.lang_set()
                .map_or(false, |langs| langs.langs().iter().any(|l| l == "ja"))
        });
        assert!(ja_fonts.count() == 0);
        let en_fonts = fonts.iter().filter(|p| {
            p.lang_set()
                .map_or(false, |langs| langs.langs().iter().any(|l| l == "en"))
        });
        assert_ne!(en_fonts.count(), 0);
    }
}
