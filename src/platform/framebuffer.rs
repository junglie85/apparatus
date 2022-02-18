pub struct FrameBuffer {
    pub(crate) data: Vec<u32>,
}

impl FrameBuffer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0; width * height],
        }
    }
}
