use crate::Vec2;

pub mod input;
pub mod window;

pub struct FrameBuffer {
    pub(crate) data: Vec<u32>,
}

impl FrameBuffer {
    pub(crate) fn new(dimensions: Vec2) -> Self {
        Self {
            data: vec![0; (dimensions.x * dimensions.y) as usize],
        }
    }
}
