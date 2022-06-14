//!
use std::ops::Mul;

use fontconfig_sys as sys;

use sys::ffi_dispatch;

#[cfg(feature = "dlopen")]
use sys::statics::LIB;
#[cfg(not(feature = "dlopen"))]
use sys::*;

use crate::FcTrue;

/// An Matrix holds an affine transformation, usually used to reshape glyphs.
/// A small set of matrix operations are provided to manipulate these.
#[doc(alias = "FcMatrix")]
#[derive(Clone)]
pub struct Matrix {
    matrix: Box<sys::FcMatrix>,
}

impl Matrix {
    /// Initialize an Matrix to the identity matrix.
    pub fn new() -> Matrix {
        let matrix = Box::new(sys::FcMatrix {
            xx: 1.,
            xy: 0.,
            yx: 0.,
            yy: 1.,
        });
        Matrix { matrix }
    }

    /// Rotate a matrix
    ///
    /// Rotates matrix by the angle who's sine is sin and cosine is cos.
    /// This is done by multiplying by the matrix:
    pub fn rotate(&mut self, cos: f64, sin: f64) {
        unsafe { ffi_dispatch!(LIB, FcMatrixRotate, self.as_mut_ptr(), cos, sin) };
    }

    /// Scale a matrix
    ///
    /// Multiplies matrix x values by sx and y values by dy.
    /// This is done by multiplying by the matrix:
    pub fn scale(&mut self, sx: f64, dy: f64) {
        unsafe {
            ffi_dispatch!(LIB, FcMatrixScale, self.as_mut_ptr(), sx, dy);
        }
    }

    /// Shear a matrix
    ///
    /// Shears matrix horizontally by sh and vertically by sv.
    /// This is done by multiplying by the matrix:
    pub fn shear(&mut self, sh: f64, sv: f64) {
        unsafe { ffi_dispatch!(LIB, FcMatrixShear, self.as_mut_ptr(), sh, sv) };
    }

    pub(crate) fn as_ptr(&self) -> *const sys::FcMatrix {
        &*self.matrix
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::FcMatrix {
        &mut *self.matrix
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        FcTrue == unsafe { ffi_dispatch!(LIB, FcMatrixEqual, self.as_ptr(), other.as_ptr()) }
    }
}

impl Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let mut matrix = Matrix::new();
        unsafe {
            ffi_dispatch!(
                LIB,
                FcMatrixMultiply,
                matrix.as_mut_ptr(),
                self.as_ptr(),
                other.as_ptr()
            )
        };
        matrix
    }
}

impl std::fmt::Debug for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix")
            .field("xx", &self.matrix.xx)
            .field("xy", &self.matrix.xy)
            .field("yx", &self.matrix.yx)
            .field("yy", &self.matrix.yy)
            .finish()
    }
}

impl Default for Matrix {
    fn default() -> Self {
        Matrix::new()
    }
}
