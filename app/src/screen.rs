//! Screen coordinate spaces.
//!
//! What it is: type-safe wrappers over the pixel values macroquad hands us in
//! three easily-confused scales:
//! - [`Physical`] — raw device pixels, what `touches()[i].position` reports;
//! - [`Logical`] — DPI-independent pixels, what `screen_width()` and
//!   `mouse_position()` report (macroquad divides these by the DPI scale);
//! - [`Canvas`] — the fixed playfield the game draws in (`0..LOGICAL_W`,
//!   `0..LOGICAL_H`), produced by [`crate::canvas::Canvas::to_canvas`].
//!
//! The space is a type parameter on [`Point`] / [`Vector`] / [`Size`], so the
//! compiler rejects comparing or combining values from different spaces — the
//! very mistake that once shrank the touch steer zone by the DPI factor. The
//! only way across is an explicit conversion, which forces the DPI scale (or the
//! canvas layout) to be named at the crossing.
//!
//! What it is not: a maths library. It carries exactly the operations the input
//! and canvas layers need. Every wrapper is `#[repr(transparent)]` over a `Vec2`
//! plus a zero-sized [`PhantomData`], so the safety costs nothing at runtime.
//!
//! The physical/logical bridge is deliberately symmetric even where only one
//! direction is used today, so the conversions read as a complete pair.
#![allow(dead_code)]

use std::marker::PhantomData;

use macroquad::prelude::*;

/// Raw device pixels — what `touches()[i].position` reports.
pub enum Physical {}
/// DPI-independent pixels — what `screen_width()` / `mouse_position()` report.
pub enum Logical {}
/// The fixed playfield the game draws into (`0..LOGICAL_W`, `0..LOGICAL_H`).
pub enum Canvas {}

/// A position in screen space `S`.
#[repr(transparent)]
pub struct Point<S> {
    v: Vec2,
    _space: PhantomData<S>,
}

/// A displacement in screen space `S` (the difference of two [`Point`]s).
#[repr(transparent)]
pub struct Vector<S> {
    v: Vec2,
    _space: PhantomData<S>,
}

/// An extent (width, height) in screen space `S`.
#[repr(transparent)]
pub struct Size<S> {
    v: Vec2,
    _space: PhantomData<S>,
}

// The derives can't be used: they would add an `S: Clone`/`Copy` bound, but the
// space markers are uninhabited. The wrappers are always `Copy` regardless of
// `S`, since `Vec2` and `PhantomData` are.
macro_rules! copy_wrapper {
    ($t:ident) => {
        impl<S> Clone for $t<S> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<S> Copy for $t<S> {}
        impl<S> $t<S> {
            const fn wrap(v: Vec2) -> Self {
                Self {
                    v,
                    _space: PhantomData,
                }
            }
        }
    };
}
copy_wrapper!(Point);
copy_wrapper!(Vector);
copy_wrapper!(Size);

impl<S> Point<S> {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self::wrap(vec2(x, y))
    }
    #[must_use]
    pub fn x(self) -> f32 {
        self.v.x
    }
    #[must_use]
    pub fn y(self) -> f32 {
        self.v.y
    }
}

impl<S> Vector<S> {
    #[must_use]
    pub fn x(self) -> f32 {
        self.v.x
    }
    #[must_use]
    pub fn y(self) -> f32 {
        self.v.y
    }
}

impl<S> Size<S> {
    #[must_use]
    pub fn new(w: f32, h: f32) -> Self {
        Self::wrap(vec2(w, h))
    }
    #[must_use]
    pub fn w(self) -> f32 {
        self.v.x
    }
    #[must_use]
    pub fn h(self) -> f32 {
        self.v.y
    }
}

/// `Point - Point` is a same-space displacement.
impl<S> std::ops::Sub for Point<S> {
    type Output = Vector<S>;
    fn sub(self, rhs: Self) -> Vector<S> {
        Vector::wrap(self.v - rhs.v)
    }
}

/// `Point + Vector` moves the point within its space.
impl<S> std::ops::Add<Vector<S>> for Point<S> {
    type Output = Point<S>;
    fn add(self, rhs: Vector<S>) -> Point<S> {
        Point::wrap(self.v + rhs.v)
    }
}

// --- the one bridge between the physical and logical scales: the DPI factor ---

impl Point<Physical> {
    /// Convert to logical pixels using the current DPI scale.
    #[must_use]
    pub fn to_logical(self, dpi: f32) -> Point<Logical> {
        Point::wrap(self.v / dpi)
    }
}

impl Point<Logical> {
    /// Convert to physical pixels using the current DPI scale.
    #[must_use]
    pub fn to_physical(self, dpi: f32) -> Point<Physical> {
        Point::wrap(self.v * dpi)
    }
}

impl Size<Logical> {
    /// Convert to physical pixels using the current DPI scale.
    #[must_use]
    pub fn to_physical(self, dpi: f32) -> Size<Physical> {
        Size::wrap(self.v * dpi)
    }
}

// --- the boundary: the ONLY place the raw macroquad pixel APIs are read --------

/// The window's DPI scale (physical pixels per logical pixel).
#[must_use]
pub fn dpi() -> f32 {
    screen_dpi_scale()
}

/// The window size, in logical pixels.
#[must_use]
pub fn window() -> Size<Logical> {
    Size::new(screen_width(), screen_height())
}

/// The pointer position, in logical pixels.
#[must_use]
pub fn mouse() -> Point<Logical> {
    let (x, y) = mouse_position();
    Point::new(x, y)
}

/// A live touch, position tagged as physical so it can't be compared against a
/// logical width by accident.
pub struct Finger {
    pub id: u64,
    pub phase: TouchPhase,
    pub at: Point<Physical>,
}

/// The live touches this frame, positions in physical pixels.
pub fn fingers() -> impl Iterator<Item = Finger> {
    touches().into_iter().map(|t| Finger {
        id: t.id,
        phase: t.phase,
        at: Point::new(t.position.x, t.position.y),
    })
}

/// Whether any touch began this frame (a tap) — used to confirm menu screens.
#[must_use]
pub fn any_tap() -> bool {
    fingers().any(|f| matches!(f.phase, TouchPhase::Started))
}
