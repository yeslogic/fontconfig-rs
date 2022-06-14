//!
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
/// use fontconfig::{FontConfig, Pattern};
///
/// let mut config = FontConfig::default(); //.expect("unable to init FontConfig");
///
/// // Find fonts that support japanese
/// let mut fonts = config.list_fonts(Pattern::new(), None);
/// let ja_fonts: Vec<_> = fonts
///         .iter_mut()
///         .filter_map(|mut p: Pattern| {
///             let langset = p.lang_set()?;
///             Some(langset.langs().iter().any(|l| l == "ja"))
///         })
///         .collect();
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

#[cfg(test)]
mod tests {
    use crate::{FontConfig, Pattern};

    #[test]
    fn list_fonts_of_ja() {
        let mut config = FontConfig::default();
        let mut fonts = config.list_fonts(Pattern::new(), None);
        let ja_fonts = fonts.iter_mut().filter_map(|mut p: Pattern| {
            let langset = p.lang_set()?;
            // .map_or(false, |mut langs| langs.langs().iter().any(|l| l == "ja"))
            if langset.langs().iter().any(|l| l == "ja") {
                Some(p)
            } else {
                None
            }
        });
        assert!(ja_fonts.count() > 0);
    }
}
