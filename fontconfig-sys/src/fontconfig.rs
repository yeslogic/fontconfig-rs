// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[cfg(feature = "dlopen")]
static SONAME: &str = if cfg!(windows) {
    "libfontconfig.dll"
} else if cfg!(target_vendor = "apple") {
    "libfontconfig.dylib.1"
} else {
    "libfontconfig.so.1"
};

#[cfg(feature = "dlopen")]
lazy_static::lazy_static! {
    pub static ref LIB: &'static Fc = LIB_RESULT.as_ref().unwrap();

    pub static ref LIB_RESULT: Result<Fc, dlib::DlError> =
        unsafe { Fc::open(SONAME) };
}

use std::os::raw::{c_char, c_double, c_int, c_uchar, c_uint, c_ushort, c_void};

pub type FcChar8 = c_uchar;
pub type FcChar16 = c_ushort;
pub type FcChar32 = c_uint;
pub type FcBool = c_int;

pub type enum__FcType = c_uint;
pub const FcTypeVoid: u32 = 0_u32;
pub const FcTypeInteger: u32 = 1_u32;
pub const FcTypeDouble: u32 = 2_u32;
pub const FcTypeString: u32 = 3_u32;
pub const FcTypeBool: u32 = 4_u32;
pub const FcTypeMatrix: u32 = 5_u32;
pub const FcTypeCharSet: u32 = 6_u32;
pub const FcTypeFTFace: u32 = 7_u32;
pub const FcTypeLangSet: u32 = 8_u32;

pub type FcType = enum__FcType;

pub mod constants {
    use super::c_int;

    pub const FC_WEIGHT_THIN: c_int = 0;
    pub const FC_WEIGHT_EXTRALIGHT: c_int = 40;
    pub const FC_WEIGHT_ULTRALIGHT: c_int = FC_WEIGHT_EXTRALIGHT;
    pub const FC_WEIGHT_LIGHT: c_int = 50;
    pub const FC_WEIGHT_BOOK: c_int = 75;
    pub const FC_WEIGHT_REGULAR: c_int = 80;
    pub const FC_WEIGHT_NORMAL: c_int = FC_WEIGHT_REGULAR;
    pub const FC_WEIGHT_MEDIUM: c_int = 100;
    pub const FC_WEIGHT_DEMIBOLD: c_int = 180;
    pub const FC_WEIGHT_SEMIBOLD: c_int = FC_WEIGHT_DEMIBOLD;
    pub const FC_WEIGHT_BOLD: c_int = 200;
    pub const FC_WEIGHT_EXTRABOLD: c_int = 205;
    pub const FC_WEIGHT_ULTRABOLD: c_int = FC_WEIGHT_EXTRABOLD;
    pub const FC_WEIGHT_BLACK: c_int = 210;
    pub const FC_WEIGHT_HEAVY: c_int = FC_WEIGHT_BLACK;
    pub const FC_WEIGHT_EXTRABLACK: c_int = 215;
    pub const FC_WEIGHT_ULTRABLACK: c_int = FC_WEIGHT_EXTRABLACK;

    pub const FC_SLANT_ROMAN: c_int = 0;
    pub const FC_SLANT_ITALIC: c_int = 100;
    pub const FC_SLANT_OBLIQUE: c_int = 110;

    pub const FC_WIDTH_ULTRACONDENSED: c_int = 50;
    pub const FC_WIDTH_EXTRACONDENSED: c_int = 63;
    pub const FC_WIDTH_CONDENSED: c_int = 75;
    pub const FC_WIDTH_SEMICONDENSED: c_int = 87;
    pub const FC_WIDTH_NORMAL: c_int = 100;
    pub const FC_WIDTH_SEMIEXPANDED: c_int = 113;
    pub const FC_WIDTH_EXPANDED: c_int = 125;
    pub const FC_WIDTH_EXTRAEXPANDED: c_int = 150;
    pub const FC_WIDTH_ULTRAEXPANDED: c_int = 200;

    pub const FC_PROPORTIONAL: c_int = 0;
    pub const FC_DUAL: c_int = 90;
    pub const FC_MONO: c_int = 100;
    pub const FC_CHARCELL: c_int = 110;

    pub const FC_RGBA_UNKNOWN: c_int = 0;
    pub const FC_RGBA_RGB: c_int = 1;
    pub const FC_RGBA_BGR: c_int = 2;
    pub const FC_RGBA_VRGB: c_int = 3;
    pub const FC_RGBA_VBGR: c_int = 4;
    pub const FC_RGBA_NONE: c_int = 5;

    pub const FC_HINT_NONE: c_int = 0;
    pub const FC_HINT_SLIGHT: c_int = 1;
    pub const FC_HINT_MEDIUM: c_int = 2;
    pub const FC_HINT_FULL: c_int = 3;

    pub const FC_LCD_NONE: c_int = 0;
    pub const FC_LCD_DEFAULT: c_int = 1;
    pub const FC_LCD_LIGHT: c_int = 2;
    pub const FC_LCD_LEGACY: c_int = 3;

    pub const FC_CHARSET_MAP_SIZE: c_int = 8;
    pub const FC_UTF8_MAX_LEN: c_int = 6;

    const_cstr! {
        pub FC_FAMILY = "family";
        pub FC_STYLE = "style";
        pub FC_SLANT = "slant";
        pub FC_WEIGHT = "weight";
        pub FC_SIZE = "size";
        pub FC_ASPECT = "aspect";
        pub FC_PIXEL_SIZE = "pixelsize";
        pub FC_SPACING = "spacing";
        pub FC_FOUNDRY = "foundry";
        pub FC_ANTIALIAS = "antialias";
        pub FC_HINTING = "hinting";
        pub FC_HINT_STYLE = "hintstyle";
        pub FC_VERTICAL_LAYOUT = "verticallayout";
        pub FC_AUTOHINT = "autohint";
        pub FC_GLOBAL_ADVANCE = "globaladvance";
        pub FC_WIDTH = "width";
        pub FC_FILE = "file";
        pub FC_INDEX = "index";
        pub FC_FT_FACE = "ftface";
        pub FC_RASTERIZER = "rasterizer";
        pub FC_OUTLINE = "outline";
        pub FC_SCALABLE = "scalable";
        pub FC_COLOR = "color";
        pub FC_VARIABLE = "variable";
        pub FC_SCALE = "scale";
        pub FC_SYMBOL = "symbol";
        pub FC_DPI = "dpi";
        pub FC_RGBA = "rgba";
        pub FC_MINSPACE = "minspace";
        pub FC_SOURCE = "source";
        pub FC_CHARSET = "charset";
        pub FC_LANG = "lang";
        pub FC_FONTVERSION = "fontversion";
        pub FC_FULLNAME = "fullname";
        pub FC_FAMILYLANG = "familylang";
        pub FC_STYLELANG = "stylelang";
        pub FC_FULLNAMELANG = "fullnamelang";
        pub FC_CAPABILITY = "capability";
        pub FC_FONTFORMAT = "fontformat";
        pub FC_EMBOLDEN = "embolden";
        pub FC_EMBEDDED_BITMAP = "embeddedbitmap";
        pub FC_DECORATIVE = "decorative";
        pub FC_LCD_FILTER = "lcdfilter";
        pub FC_FONT_FEATURES = "fontfeatures";
        pub FC_FONT_VARIATIONS = "fontvariations";
        pub FC_NAMELANG = "namelang";
        pub FC_PRGNAME = "prgname";
        pub FC_HASH = "hash";
        pub FC_POSTSCRIPT_NAME = "postscriptname";
        pub FC_FONT_HAS_HINT = "fonthashint";
        pub FC_CACHE_SUFFIX = ".cache-";
        pub FC_DIR_CACHE_FILE = "fonts.cache-";
        pub FC_USER_CACHE_FILE = ".fonts.cache-";
        pub FC_CHARWIDTH = "charwidth";
        pub FC_CHAR_WIDTH = "charwidth";
        pub FC_CHAR_HEIGHT = "charheight";
        pub FC_MATRIX = "matrix";
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct struct__FcMatrix {
    pub xx: c_double,
    pub xy: c_double,
    pub yx: c_double,
    pub yy: c_double,
}

pub type FcMatrix = struct__FcMatrix;

pub type struct__FcCharSet = c_void;

pub type FcCharSet = struct__FcCharSet;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct struct__FcObjectType {
    pub object: *mut c_char,
    pub _type: FcType,
}

pub type FcObjectType = struct__FcObjectType;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct struct__FcConstant {
    pub name: *mut FcChar8,
    pub object: *mut c_char,
    pub value: c_int,
}

pub type FcConstant = struct__FcConstant;

pub type enum__FcResult = c_uint;
pub const FcResultMatch: u32 = 0_u32;
pub const FcResultNoMatch: u32 = 1_u32;
pub const FcResultTypeMismatch: u32 = 2_u32;
pub const FcResultNoId: u32 = 3_u32;
pub const FcResultOutOfMemory: u32 = 4_u32;

pub type FcResult = enum__FcResult;

pub type struct__FcPattern = c_void;

pub type FcPattern = struct__FcPattern;

pub type struct__FcLangSet = c_void;

pub type FcLangSet = struct__FcLangSet;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct struct__FcValue {
    pub _type: FcType,
    pub u: union_unnamed1,
}

pub type FcValue = struct__FcValue;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct struct__FcFontSet {
    pub nfont: c_int,
    pub sfont: c_int,
    pub fonts: *mut *mut FcPattern,
}

pub type FcFontSet = struct__FcFontSet;

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct struct__FcObjectSet {
    pub nobject: c_int,
    pub sobject: c_int,
    pub objects: *mut *mut c_char,
}

pub type FcObjectSet = struct__FcObjectSet;

pub type enum__FcMatchKind = c_uint;
pub const FcMatchPattern: u32 = 0_u32;
pub const FcMatchFont: u32 = 1_u32;
pub const FcMatchScan: u32 = 2_u32;

pub type FcMatchKind = enum__FcMatchKind;

pub type enum__FcLangResult = c_uint;
pub const FcLangEqual: u32 = 0_u32;
pub const FcLangDifferentCountry: u32 = 1_u32;
pub const FcLangDifferentTerritory: u32 = 1_u32;
pub const FcLangDifferentLang: u32 = 2_u32;

pub type FcLangResult = enum__FcLangResult;

pub type enum__FcSetName = c_uint;
pub const FcSetSystem: u32 = 0_u32;
pub const FcSetApplication: u32 = 1_u32;

pub type FcSetName = enum__FcSetName;

pub type struct__FcAtomic = c_void;

pub type FcAtomic = struct__FcAtomic;

pub type FcEndian = c_uint;
pub const FcEndianBig: u32 = 0_u32;
pub const FcEndianLittle: u32 = 1_u32;

pub type struct__FcConfig = c_void;

pub type FcConfig = struct__FcConfig;

pub type struct__FcGlobalCache = c_void;

pub type FcFileCache = struct__FcGlobalCache;

pub type struct__FcBlanks = c_void;

pub type FcBlanks = struct__FcBlanks;

pub type struct__FcStrList = c_void;

pub type FcStrList = struct__FcStrList;

pub type struct__FcStrSet = c_void;

pub type FcStrSet = struct__FcStrSet;

pub type struct__FcCache = c_void;

pub type FcCache = struct__FcCache;

pub type union_unnamed1 = c_void;

dlib::external_library!(Fc, "fontconfig",
    functions:
        fn FcBlanksCreate() -> *mut FcBlanks,

        fn FcBlanksDestroy(*mut FcBlanks) -> (),

        fn FcBlanksAdd(*mut FcBlanks, FcChar32) -> FcBool,

        fn FcBlanksIsMember(*mut FcBlanks, FcChar32) -> FcBool,

        fn FcCacheDir(*mut FcCache) -> *const FcChar8,

        fn FcCacheCopySet(*const FcCache) -> *mut FcFontSet,

        fn FcCacheSubdir(*const FcCache, c_int) -> *const FcChar8,

        fn FcCacheNumSubdir(*const FcCache) -> c_int,

        fn FcCacheNumFont(*const FcCache) -> c_int,

        fn FcDirCacheUnlink(*const FcChar8, *mut FcConfig) -> FcBool,

        fn FcDirCacheValid(*const FcChar8) -> FcBool,

        fn FcConfigHome() -> *mut FcChar8,

        fn FcConfigEnableHome(FcBool) -> FcBool,

        fn FcConfigFilename(*const FcChar8) -> *mut FcChar8,

        fn FcConfigCreate() -> *mut FcConfig,

        fn FcConfigReference(*mut FcConfig) -> *mut FcConfig,

        fn FcConfigDestroy(*mut FcConfig) -> (),

        fn FcConfigSetCurrent(*mut FcConfig) -> FcBool,

        fn FcConfigGetCurrent() -> *mut FcConfig,

        fn FcConfigUptoDate(*mut FcConfig) -> FcBool,

        fn FcConfigBuildFonts(*mut FcConfig) -> FcBool,

        fn FcConfigGetFontDirs(*mut FcConfig) -> *mut FcStrList,

        fn FcConfigGetConfigDirs(*mut FcConfig) -> *mut FcStrList,

        fn FcConfigGetConfigFiles(*mut FcConfig) -> *mut FcStrList,

        fn FcConfigGetCache(*mut FcConfig) -> *mut FcChar8,

        fn FcConfigGetBlanks(*mut FcConfig) -> *mut FcBlanks,

        fn FcConfigGetCacheDirs(*const FcConfig) -> *mut FcStrList,

        fn FcConfigGetRescanInterval(*mut FcConfig) -> c_int,

        fn FcConfigSetRescanInterval(*mut FcConfig, c_int) -> FcBool,

        fn FcConfigGetFonts(*mut FcConfig, FcSetName) -> *mut FcFontSet,

        fn FcConfigAppFontAddFile(*mut FcConfig, *const FcChar8) -> FcBool,

        fn FcConfigAppFontAddDir(*mut FcConfig, *const FcChar8) -> FcBool,

        fn FcConfigAppFontClear(*mut FcConfig) -> (),

        fn FcConfigSubstituteWithPat(
            *mut FcConfig,
            *mut FcPattern,
            *mut FcPattern,
            FcMatchKind
        ) -> FcBool,

        fn FcConfigSubstitute(
            *mut FcConfig,
            *mut FcPattern,
            FcMatchKind
        ) -> FcBool,

        fn FcCharSetCreate() -> *mut FcCharSet,

        fn FcCharSetNew() -> *mut FcCharSet,

        fn FcCharSetDestroy(*mut FcCharSet) -> (),

        fn FcCharSetAddChar(*mut FcCharSet, FcChar32) -> FcBool,

        fn FcCharSetCopy(*mut FcCharSet) -> *mut FcCharSet,

        fn FcCharSetEqual(*const FcCharSet, *const FcCharSet) -> FcBool,

        fn FcCharSetIntersect(*const FcCharSet, *const FcCharSet) -> *mut FcCharSet,

        fn FcCharSetUnion(*const FcCharSet, *const FcCharSet) -> *mut FcCharSet,

        fn FcCharSetSubtract(*const FcCharSet, *const FcCharSet) -> *mut FcCharSet,

        fn FcCharSetMerge(*mut FcCharSet, *const FcCharSet, *mut FcBool) -> FcBool,

        fn FcCharSetHasChar(*const FcCharSet, FcChar32) -> FcBool,

        fn FcCharSetCount(*const FcCharSet) -> FcChar32,

        fn FcCharSetIntersectCount(*const FcCharSet, *const FcCharSet) -> FcChar32,

        fn FcCharSetSubtractCount(*const FcCharSet, *const FcCharSet) -> FcChar32,

        fn FcCharSetIsSubset(*const FcCharSet, *const FcCharSet) -> FcBool,

        fn FcCharSetFirstPage(
            *const FcCharSet,
            *mut FcChar32,
            *mut FcChar32
        ) -> FcChar32,

        fn FcCharSetNextPage(
            *const FcCharSet,
            *mut FcChar32,
            *mut FcChar32
        ) -> FcChar32,

        fn FcCharSetCoverage(
            *const FcCharSet,
            FcChar32,
            *mut FcChar32
        ) -> FcChar32,

        fn FcValuePrint(FcValue) -> (),

        fn FcPatternPrint(*const FcPattern) -> (),

        fn FcFontSetPrint(*mut FcFontSet) -> (),

        fn FcDefaultSubstitute(*mut FcPattern) -> (),

        fn FcFileIsDir(*const FcChar8) -> FcBool,

        fn FcFileScan(
            *mut FcFontSet,
            *mut FcStrSet,
            *mut FcFileCache,
            *mut FcBlanks,
            *const FcChar8,
            FcBool
        ) -> FcBool,

        fn FcDirScan(
            *mut FcFontSet,
            *mut FcStrSet,
            *mut FcFileCache,
            *mut FcBlanks,
            *const FcChar8,
            FcBool
        ) -> FcBool,

        fn FcDirSave(*mut FcFontSet, *const FcStrSet, *mut FcChar8) -> FcBool,

        fn FcDirCacheLoad(
            *const FcChar8,
            *mut FcConfig,
            *mut *mut FcChar8
        ) -> *mut FcCache,

        fn FcDirCacheRead(
            *const FcChar8,
            FcBool,
            *mut FcConfig
        ) -> *mut FcCache,

        // fn FcDirCacheLoadFile(*mut FcChar8, *mut struct_stat) -> *mut FcCache,

        fn FcDirCacheUnload(*mut FcCache) -> (),

        fn FcFreeTypeQuery(
            *const FcChar8,
            c_int,
            *mut FcBlanks,
            *mut c_int
        ) -> *mut FcPattern,

        fn FcFontSetCreate() -> *mut FcFontSet,

        fn FcFontSetDestroy(*mut FcFontSet) -> (),

        fn FcFontSetAdd(*mut FcFontSet, *mut FcPattern) -> FcBool,

        fn FcInitLoadConfig() -> *mut FcConfig,

        fn FcInitLoadConfigAndFonts() -> *mut FcConfig,

        fn FcInit() -> FcBool,

        fn FcFini() -> (),

        fn FcGetVersion() -> c_int,

        fn FcInitReinitialize() -> FcBool,

        fn FcInitBringUptoDate() -> FcBool,

        fn FcGetLangs() -> *mut FcStrSet,

        fn FcLangGetCharSet(*const FcChar8) -> *mut FcCharSet,

        fn FcLangSetCreate() -> *mut FcLangSet,

        fn FcLangSetDestroy(*mut FcLangSet) -> (),

        fn FcLangSetCopy(*const FcLangSet) -> *mut FcLangSet,

        fn FcLangSetAdd(*mut FcLangSet, *const FcChar8) -> FcBool,

        fn FcLangSetHasLang(*const FcLangSet, *const FcChar8) -> FcLangResult,

        fn FcLangSetCompare(*const FcLangSet, *const FcLangSet) -> FcLangResult,

        fn FcLangSetContains(*const FcLangSet, *const FcLangSet) -> FcBool,

        fn FcLangSetEqual(*const FcLangSet, *const FcLangSet) -> FcBool,

        fn FcLangSetHash(*const FcLangSet) -> FcChar32,

        fn FcLangSetGetLangs(*const FcLangSet) -> *mut FcStrSet,

        fn FcObjectSetCreate() -> *mut FcObjectSet,

        fn FcObjectSetAdd(*mut FcObjectSet, *const c_char) -> FcBool,

        fn FcObjectSetDestroy(*mut FcObjectSet) -> (),

        // fn FcObjectSetVaBuild(*mut c_char, *mut __va_list_tag) -> *mut FcObjectSet,

        fn FcFontSetList(
            *mut FcConfig,
            *mut *mut FcFontSet,
            c_int,
            *mut FcPattern,
            *mut FcObjectSet
        ) -> *mut FcFontSet,

        fn FcFontList(
            *mut FcConfig,
            *mut FcPattern,
            *mut FcObjectSet
        ) -> *mut FcFontSet,

        fn FcAtomicCreate(*const FcChar8) -> *mut FcAtomic,

        fn FcAtomicLock(*mut FcAtomic) -> FcBool,

        fn FcAtomicNewFile(*mut FcAtomic) -> *mut FcChar8,

        fn FcAtomicOrigFile(*mut FcAtomic) -> *mut FcChar8,

        fn FcAtomicReplaceOrig(*mut FcAtomic) -> FcBool,

        fn FcAtomicDeleteNew(*mut FcAtomic) -> (),

        fn FcAtomicUnlock(*mut FcAtomic) -> (),

        fn FcAtomicDestroy(*mut FcAtomic) -> (),

        fn FcFontSetMatch(
            *mut FcConfig,
            *mut *mut FcFontSet,
            c_int,
            *mut FcPattern,
            *mut FcResult
        ) -> *mut FcPattern,

        fn FcFontMatch(
            *mut FcConfig,
            *mut FcPattern,
            *mut FcResult
        ) -> *mut FcPattern,

        fn FcFontRenderPrepare(
            *mut FcConfig,
            *mut FcPattern,
            *mut FcPattern
        ) -> *mut FcPattern,

        fn FcFontSetSort(
            *mut FcConfig,
            *mut *mut FcFontSet,
            c_int,
            *mut FcPattern,
            FcBool,
            *mut *mut FcCharSet,
            *mut FcResult
        ) -> *mut FcFontSet,

        fn FcFontSort(
            *mut FcConfig,
            *mut FcPattern,
            FcBool,
            *mut *mut FcCharSet,
            *mut FcResult
        ) -> *mut FcFontSet,

        fn FcFontSetSortDestroy(*mut FcFontSet) -> (),

        fn FcMatrixCopy(*const FcMatrix) -> *mut FcMatrix,

        fn FcMatrixEqual(*const FcMatrix, *const FcMatrix) -> FcBool,

        fn FcMatrixMultiply(*mut FcMatrix, *const FcMatrix, *const FcMatrix) -> (),

        fn FcMatrixRotate(*mut FcMatrix, c_double, c_double) -> (),

        fn FcMatrixScale(*mut FcMatrix, c_double, c_double) -> (),

        fn FcMatrixShear(*mut FcMatrix, c_double, c_double) -> (),

        fn FcNameRegisterObjectTypes(*const FcObjectType, c_int) -> FcBool,

        fn FcNameUnregisterObjectTypes(*const FcObjectType, c_int) -> FcBool,

        fn FcNameGetObjectType(*const c_char) -> *const FcObjectType,

        fn FcNameRegisterConstants(*const FcConstant, c_int) -> FcBool,

        fn FcNameUnregisterConstants(*const FcConstant, c_int) -> FcBool,

        fn FcNameGetConstant(*mut FcChar8) -> *const FcConstant,

        fn FcNameConstant(*mut FcChar8, *mut c_int) -> FcBool,

        fn FcNameParse(*const FcChar8) -> *mut FcPattern,

        fn FcNameUnparse(*mut FcPattern) -> *mut FcChar8,

        fn FcPatternCreate() -> *mut FcPattern,

        fn FcPatternDuplicate(*const FcPattern) -> *mut FcPattern,

        fn FcPatternReference(*mut FcPattern) -> (),

        fn FcPatternFilter(*mut FcPattern, *const FcObjectSet) -> *mut FcPattern,

        fn FcValueDestroy(FcValue) -> (),

        fn FcValueEqual(FcValue, FcValue) -> FcBool,

        fn FcValueSave(FcValue) -> FcValue,

        fn FcPatternDestroy(*mut FcPattern) -> (),

        fn FcPatternEqual(*const FcPattern, *const FcPattern) -> FcBool,

        fn FcPatternEqualSubset(
            *const FcPattern,
            *const FcPattern,
            *const FcObjectSet
        ) -> FcBool,

        fn FcPatternHash(*const FcPattern) -> FcChar32,

        fn FcPatternAdd(
            *mut FcPattern,
            *const c_char,
            FcValue,
            FcBool
        ) -> FcBool,

        fn FcPatternAddWeak(
            *mut FcPattern,
            *const c_char,
            FcValue,
            FcBool
        ) -> FcBool,

        fn FcPatternGet(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut FcValue
        ) -> FcResult,

        fn FcPatternDel(*mut FcPattern, *const c_char) -> FcBool,

        fn FcPatternRemove(*mut FcPattern, *const c_char, c_int) -> FcBool,

        fn FcPatternAddInteger(*mut FcPattern, *const c_char, c_int) -> FcBool,

        fn FcPatternAddDouble(*mut FcPattern, *const c_char, c_double) -> FcBool,

        fn FcPatternAddString(
            *mut FcPattern,
            *const c_char,
            *const FcChar8
        ) -> FcBool,

        fn FcPatternAddMatrix(
            *mut FcPattern,
            *const c_char,
            *const FcMatrix
        ) -> FcBool,

        fn FcPatternAddCharSet(
            *mut FcPattern,
            *const c_char,
            *const FcCharSet
        ) -> FcBool,

        fn FcPatternAddBool(*mut FcPattern, *const c_char, FcBool) -> FcBool,

        fn FcPatternAddLangSet(
            *mut FcPattern,
            *const c_char,
            *const FcLangSet
        ) -> FcBool,

        fn FcPatternGetInteger(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut c_int
        ) -> FcResult,

        fn FcPatternGetDouble(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut c_double
        ) -> FcResult,

        fn FcPatternGetString(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut *mut FcChar8
        ) -> FcResult,

        fn FcPatternGetMatrix(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut *mut FcMatrix
        ) -> FcResult,

        fn FcPatternGetCharSet(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut *mut FcCharSet
        ) -> FcResult,

        fn FcPatternGetBool(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut FcBool
        ) -> FcResult,

        fn FcPatternGetLangSet(
            *mut FcPattern,
            *const c_char,
            c_int,
            *mut *mut FcLangSet
        ) -> FcResult,

        // fn FcPatternVaBuild(*mut FcPattern, *mut __va_list_tag) -> *mut FcPattern,

        fn FcPatternFormat(*mut FcPattern, *const FcChar8) -> *mut FcChar8,

        fn FcStrCopy(*const FcChar8) -> *mut FcChar8,

        fn FcStrCopyFilename(*const FcChar8) -> *mut FcChar8,

        fn FcStrPlus(*const FcChar8, *const FcChar8) -> *mut FcChar8,

        fn FcStrFree(*mut FcChar8) -> (),

        fn FcStrDowncase(*const FcChar8) -> *mut FcChar8,

        fn FcStrCmpIgnoreCase(*const FcChar8, *const FcChar8) -> c_int,

        fn FcStrCmp(*const FcChar8, *const FcChar8) -> c_int,

        fn FcStrStrIgnoreCase(*const FcChar8, *const FcChar8) -> *mut FcChar8,

        fn FcStrStr(*const FcChar8, *const FcChar8) -> *mut FcChar8,

        fn FcUtf8ToUcs4(*mut FcChar8, *mut FcChar32, c_int) -> c_int,

        fn FcUtf8Len(
            *mut FcChar8,
            c_int,
            *mut c_int,
            *mut c_int
        ) -> FcBool,

        fn FcUcs4ToUtf8(FcChar32, *mut FcChar8) -> c_int,

        fn FcUtf16ToUcs4(
            *mut FcChar8,
            FcEndian,
            *mut FcChar32,
            c_int
        ) -> c_int,

        fn FcUtf16Len(
            *mut FcChar8,
            FcEndian,
            c_int,
            *mut c_int,
            *mut c_int
        ) -> FcBool,

        fn FcStrDirname(*const FcChar8) -> *mut FcChar8,

        fn FcStrBasename(*const FcChar8) -> *mut FcChar8,

        fn FcStrSetCreate() -> *mut FcStrSet,

        fn FcStrSetMember(*mut FcStrSet, *const FcChar8) -> FcBool,

        fn FcStrSetEqual(*mut FcStrSet, *mut FcStrSet) -> FcBool,

        fn FcStrSetAdd(*mut FcStrSet, *const FcChar8) -> FcBool,

        fn FcStrSetAddFilename(*mut FcStrSet, *const FcChar8) -> FcBool,

        fn FcStrSetDel(*mut FcStrSet, *const FcChar8) -> FcBool,

        fn FcStrSetDestroy(*mut FcStrSet) -> (),

        fn FcStrListCreate(*mut FcStrSet) -> *mut FcStrList,

        fn FcStrListNext(*mut FcStrList) -> *mut FcChar8,

        fn FcStrListDone(*mut FcStrList) -> (),

        fn FcConfigParseAndLoad(
            *mut FcConfig,
            *const FcChar8,
            FcBool
        ) -> FcBool,

    varargs:
        fn FcPatternBuild(*mut FcPattern) -> *mut FcPattern,
        fn FcObjectSetBuild(*mut c_char) -> *mut FcObjectSet,
);
