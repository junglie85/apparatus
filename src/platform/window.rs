use crate::platform::FrameBuffer;
use crate::ApparatusError;

pub struct Window {
    width: f32,
    height: f32,
    native_window: minifb::Window,
}

impl Window {
    pub(crate) fn new(name: &str, width: f32, height: f32) -> Result<Self, ApparatusError> {
        let native_window = minifb::Window::new(
            name,
            width as usize,
            height as usize,
            minifb::WindowOptions::default(),
        )
        .map_err(|e| ApparatusError::Window(e.into()))?;

        let window = Self {
            width,
            height,
            native_window,
        };

        Ok(window)
    }

    pub(crate) fn native_window(&self) -> &minifb::Window {
        &self.native_window
    }

    pub(crate) fn display(&mut self, buffer: &FrameBuffer) -> Result<(), ApparatusError> {
        self.native_window
            .update_with_buffer(&buffer.data, self.width as usize, self.height as usize)
            .map_err(|e| ApparatusError::Window(e.into()))
    }

    pub(crate) fn should_close(&self) -> bool {
        !self.native_window.is_open()
    }
}
