use std::ops::Add;

// TODO: Use a maths library and re-export it; or, these are probably good candidates for macros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x + rhs, self.y + rhs)
    }
}

pub fn clamp(min: f32, value: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod maths_tests {
    use super::*;

    #[test]
    fn clamp_value_between_min_and_max_is_value() {
        assert_eq!(10.0, clamp(0.0, 10.0, 20.0));
    }

    #[test]
    fn clamp_value_less_than_min_is_min() {
        assert_eq!(0.0, clamp(0.0, -10.0, 20.0));
    }

    #[test]
    fn clamp_value_greater_than_max_is_max() {
        assert_eq!(20.0, clamp(0.0, 30.0, 20.0));
    }

    #[test]
    fn scalar_addition_vec2_adds_to_all_components() {
        let vec = Vec2::new(3.0, 5.0);

        assert_eq!(Vec2::new(7.0, 9.0), vec + 4.0);
    }
}
