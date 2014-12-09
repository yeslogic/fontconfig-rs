extern crate libc;

use ffi::{FcPattern, FcObjectSet, FcFontSet};

use std::c_str::CString;
use std::mem;

mod ffi;

struct Pattern {
    pat: *mut FcPattern,
    strings: Vec<CString>,    
}

impl Pattern {
    fn add_string(&mut self, name: &str, val: &str) {
        let c_name = name.to_c_str();
        let c_val = val.to_c_str();

        unsafe { ffi::FcPatternAddString(self.pat, c_name.as_ptr(), c_val.as_ptr() as *const u8); }

        self.strings.push_all(&[c_name, c_val]);
    }
    
    unsafe fn to_pattern(&self) -> *mut FcPattern {
        self.pat        
    }  
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe { ffi::FcPatternDestroy(self.pat); }
    }    
}

struct Properties {
    vec: Vec<CString>,
}

impl Properties {
    pub fn new() -> Properties {
        Properties { vec: Vec::new() }    
    }

    fn add(&mut self, prop: &str) {
        self.vec.push(prop.to_c_str());
    }
    
    unsafe fn to_object_set(&self) -> *mut FcObjectSet { 
        let object_set = ffi::FcObjectSetCreate();
        
        for c_str in self.vec.iter() {
            assert_eq!(ffi::FcObjectSetAdd(object_set, c_str.as_ptr()), 1);
        }
      
        object_set
    } 
}

struct FontSet {
    fonts: *mut FcFontSet,
}

impl FontSet {

    unsafe fn with_pattern_args(pat: *mut FcPattern, args: *mut FcObjectSet) -> FontSet {
        FontSet { fonts: ffi::FcFontList(mem::transmute(0u), pat, args) }  
    }
    
}

impl Drop for FcFontSet {
    fn drop(&mut self) {
        // Transmute shouldn't be necessary but somehow is, otherwise:
        // error: mismatched types: expected `*mut ffi::Struct__FcFontSet`, 
        // found `*mut *mut ffi::Struct__FcPattern` 
        // (expected struct ffi::Struct__FcFontSet, found *-ptr)
        unsafe { ffi::FcFontSetDestroy(mem::transmute(self.fonts)); }  
    }    
}

#[test]
fn it_works() {
    let result = unsafe { ffi::FcInit() };

    assert_eq!(result, 1);
}

#[test]
fn test_props() {
    let mut props = Properties::new();
    props.add("family");
    props.add("file");

}

