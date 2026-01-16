//! C-compatible geometry types for FFI
//!
//! Provides C struct wrappers for Rust geometry types that C# can marshal directly.

use crate::geometry::{Point as RustPoint, Rect as RustRect};

/// C-compatible point type (blittable struct)
///
/// Directly marshals to C# Point struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CPoint {
    pub x: f32,
    pub y: f32,
}

impl CPoint {
    /// Convert from Rust Point
    pub fn from_rust(p: &RustPoint) -> Self {
        CPoint { x: p.x, y: p.y }
    }

    /// Convert to Rust Point
    pub fn to_rust(&self) -> RustPoint {
        RustPoint::new(self.x, self.y)
    }
}

/// C-compatible rectangle type (blittable struct)
///
/// Directly marshals to C# Rect struct
/// Represents a rectangle with coordinates (x, y) and size (width, height)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl CRect {
    /// Convert from Rust Rect
    pub fn from_rust(r: &RustRect) -> Self {
        CRect {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }

    /// Convert to Rust Rect
    pub fn to_rust(&self) -> RustRect {
        RustRect::new(self.x, self.y, self.width, self.height)
    }
}

/// C-compatible color type (blittable struct)
///
/// Represents RGB or CMYK colors as normalized values (0.0-1.0)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl CColor {
    /// Create an RGB color
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        CColor { r, g, b, a: 1.0 }
    }

    /// Create an RGBA color
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        CColor { r, g, b, a }
    }

    /// Create black
    pub fn black() -> Self {
        CColor::rgb(0.0, 0.0, 0.0)
    }

    /// Create white
    pub fn white() -> Self {
        CColor::rgb(1.0, 1.0, 1.0)
    }
}

/// C-compatible matrix for transformations
///
/// Represents a 2x3 affine transformation matrix:
/// | a  b  0 |
/// | c  d  0 |
/// | e  f  1 |
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CMatrix {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl CMatrix {
    /// Identity matrix
    pub fn identity() -> Self {
        CMatrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Create a translation matrix
    pub fn translation(x: f32, y: f32) -> Self {
        CMatrix {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: x,
            f: y,
        }
    }

    /// Create a scaling matrix
    pub fn scale(sx: f32, sy: f32) -> Self {
        CMatrix {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }
}

/// C-compatible dimensions
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CDimensions {
    pub width: f32,
    pub height: f32,
}

impl CDimensions {
    pub fn new(width: f32, height: f32) -> Self {
        CDimensions { width, height }
    }
}

/// C-compatible margin
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CMargin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl CMargin {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        CMargin {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(size: f32) -> Self {
        CMargin {
            top: size,
            right: size,
            bottom: size,
            left: size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_conversion() {
        let rust_point = RustPoint::new(10.5, 20.5);
        let c_point = CPoint::from_rust(&rust_point);
        assert_eq!(c_point.x, 10.5);
        assert_eq!(c_point.y, 20.5);

        let back_to_rust = c_point.to_rust();
        assert_eq!(back_to_rust.x, rust_point.x);
        assert_eq!(back_to_rust.y, rust_point.y);
    }

    #[test]
    fn test_rect_conversion() {
        let rust_rect = RustRect::new(0.0, 0.0, 100.0, 100.0);
        let c_rect = CRect::from_rust(&rust_rect);
        assert_eq!(c_rect.x0, 0.0);
        assert_eq!(c_rect.y0, 0.0);
        assert_eq!(c_rect.x1, 100.0);
        assert_eq!(c_rect.y1, 100.0);
    }

    #[test]
    fn test_matrix_identity() {
        let m = CMatrix::identity();
        assert_eq!(m.a, 1.0);
        assert_eq!(m.d, 1.0);
        assert_eq!(m.b, 0.0);
        assert_eq!(m.c, 0.0);
        assert_eq!(m.e, 0.0);
        assert_eq!(m.f, 0.0);
    }
}
