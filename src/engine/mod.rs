use std::fmt::{Display, Formatter};

pub mod apparatus;
pub mod clock;
pub mod game;
pub mod key;
pub mod logger;
pub mod mouse;
pub mod sprite;

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Point(f32, f32);

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self(x, y)
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}
