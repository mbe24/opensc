//! Playfield geometry.
//!
//! What it is: the simulation's own 2D types — a [`Pos`] (a position in playfield
//! pixels) and a [`Vel`] (a velocity or displacement, pixels per tick). The
//! playfield is the only space the simulation knows, so unlike the UI's `screen`
//! module these carry no space tag; they exist to stop x/y swaps and
//! position-vs-velocity confusion, and to let `pos + vel` read as the physics it
//! is.
//!
//! The algebra is deliberate: `Pos - Pos` is a [`Vel`] (a displacement), `Pos +
//! Vel` is a `Pos`, and `Vel + Vel` is a `Vel`; adding two positions is
//! meaningless and simply isn't provided. Both are plain `i32` pairs, so they are
//! zero-cost over the tuples they replace.

/// A position in playfield pixels — origin top-left, y-down.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

/// A velocity or displacement in playfield pixels (per tick, where it's a rate).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Vel {
    pub dh: i32,
    pub dv: i32,
}

impl Pos {
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl Vel {
    #[must_use]
    pub const fn new(dh: i32, dv: i32) -> Self {
        Self { dh, dv }
    }
}

/// `Pos + Vel` moves the position.
impl std::ops::Add<Vel> for Pos {
    type Output = Pos;
    fn add(self, v: Vel) -> Pos {
        Pos::new(self.x + v.dh, self.y + v.dv)
    }
}

impl std::ops::AddAssign<Vel> for Pos {
    fn add_assign(&mut self, v: Vel) {
        self.x += v.dh;
        self.y += v.dv;
    }
}

/// `Pos - Pos` is the displacement between them.
impl std::ops::Sub for Pos {
    type Output = Vel;
    fn sub(self, o: Pos) -> Vel {
        Vel::new(self.x - o.x, self.y - o.y)
    }
}

/// `Vel + Vel` composes displacements.
impl std::ops::Add for Vel {
    type Output = Vel;
    fn add(self, o: Vel) -> Vel {
        Vel::new(self.dh + o.dh, self.dv + o.dv)
    }
}

/// An inclusive range of requested velocities — the corners of the control box
/// the input maps a stick/pointer into. Named fields keep the asymmetric
/// vertical range (`min.dv != -max.dv`) from being transposed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VelRange {
    pub min: Vel,
    pub max: Vel,
}

impl VelRange {
    #[must_use]
    pub const fn new(min: Vel, max: Vel) -> Self {
        Self { min, max }
    }
}

/// An axis-aligned rectangle in playfield pixels — a layout box (`(x, y)`
/// top-left, `w`×`h`). Distinct from the texture-space [`crate::sprite::SrcRect`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
}
