use std::time::Duration;

use log::error;

use crate::color::Color;
use crate::engine::clock::Clock;
use crate::engine::game::Game;
use crate::engine::key::Key;
use crate::engine::logger::Logger;
use crate::engine::sprite::Sprite;
use crate::errors::ApparatusError;
use crate::platform::framebuffer::FrameBuffer;
use crate::platform::input::Input;
use crate::platform::window::Window;
use crate::renderer::software_2d::Renderer;
use crate::{color, util};

pub struct ApparatusSettings {
    width: usize,
    height: usize,
    pixel_width: usize,
    pixel_height: usize,
}

impl Default for ApparatusSettings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            pixel_width: 1,
            pixel_height: 1,
        }
    }
}

impl ApparatusSettings {
    /// Set the number of pixels in width and height for each "virtual pixel".
    /// Defaults to 1 x 1.
    pub fn with_pixel_size(mut self, width: usize, height: usize) -> Self {
        self.pixel_width = width;
        self.pixel_height = height;
        self
    }

    /// Set the desired initial width and height of the screen in "virtual pixels".
    /// Defaults to 1280 x 720.
    pub fn with_screen_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

pub struct Apparatus {
    pixel_width: usize,
    pixel_height: usize,
    screen_width: usize,
    screen_height: usize,
    window_width: f32,
    window_height: f32,

    _logger: Logger,
    clock: Clock,
    window: Window,
    renderer: Renderer,
    input: Input,
    target_frame_duration: Duration,
    running: bool,
}

impl Apparatus {
    pub fn new(name: &str, settings: ApparatusSettings) -> Result<Apparatus, ApparatusError> {
        let pixel_width = settings.pixel_width;
        let pixel_height = settings.pixel_height;
        let screen_width = settings.width;
        let screen_height = settings.height;
        let window_width = (screen_width * pixel_width) as f32;
        let window_height = (screen_height * pixel_height) as f32;

        let _logger = Logger::init()?;

        let mut clock = Clock::default();
        clock.tick();

        let window = Window::new(name, window_width, window_height)?;
        let frame_buffer = FrameBuffer::new(window_width as usize, window_height as usize);
        let renderer = Renderer::new(
            window_width,
            window_height,
            pixel_width,
            pixel_height,
            frame_buffer,
        );
        let input = Input::new();

        let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0);

        let running = false;

        let app = Self {
            pixel_width,
            pixel_height,
            screen_width,
            screen_height,
            window_width,
            window_height,

            _logger,
            clock,
            window,
            renderer,
            input,
            target_frame_duration,
            running,
        };

        Ok(app)
    }

    pub fn run<G>(mut self) -> Result<(), ApparatusError>
    where
        G: Game,
    {
        let mut game = G::on_create(&self)?;

        self.clock.tick();

        self.running = true;
        while self.running {
            if self.window.should_close() {
                self.running = false;
            }

            self.input.process_input(&self.window);

            game.on_update(&mut self);

            let elapsed = self.clock.elapsed();
            if elapsed < self.target_frame_duration {
                if let Err(e) = util::sleep(self.target_frame_duration - elapsed) {
                    error!("{}", e);
                }
            }

            self.clock.tick();

            // Stats.
            #[cfg(debug_assertions)]
            {
                let fps = 1.0 / self.clock.delta().as_secs_f32();
                let debug_box_width = 190.0;
                let debug_box_left = self.window_width - debug_box_width;
                let debug_box_height = 50.0;
                let debug_box_bottom = self.window_height - debug_box_height;
                self.renderer.draw_filled_rectangle(
                    debug_box_left,
                    debug_box_bottom,
                    debug_box_width,
                    debug_box_height,
                    color::css::SILVER,
                );
                self.renderer.draw_string(
                    format!("ms/F: {:.2}", self.clock.delta().as_secs_f32() * 1_000.0),
                    debug_box_left + 10.0,
                    debug_box_bottom + debug_box_height - 20.0,
                    color::css::BLACK,
                    12.0,
                );
                self.renderer.draw_string(
                    format!("FPS: {:.2}", fps),
                    debug_box_left + 10.0,
                    debug_box_bottom + debug_box_height - 30.0,
                    color::css::BLACK,
                    12.0,
                );
                self.renderer.draw_string(
                    format!(
                        "Sleep tolerance (ms): {}",
                        util::get_sleep_tolerance().as_micros() as f32 / 1_000.0
                    ),
                    debug_box_left + 10.0,
                    debug_box_bottom + debug_box_height - 40.0,
                    color::css::BLACK,
                    12.0,
                );
            }

            self.window.display(self.renderer.buffer())?;
        }

        Ok(())
    }

    // ----- Info -----
    pub fn pixel_width(&self) -> usize {
        self.pixel_width
    }

    pub fn pixel_height(&self) -> usize {
        self.pixel_height
    }

    pub fn screen_width(&self) -> usize {
        self.screen_width
    }

    pub fn screen_height(&self) -> usize {
        self.screen_height
    }

    pub fn window_width(&self) -> f32 {
        self.window_width
    }

    pub fn window_height(&self) -> f32 {
        self.window_height
    }

    // ----- Timing -----
    pub fn elapsed_time(&self) -> Duration {
        self.target_frame_duration
    }

    // ----- Input -----
    pub fn is_key_held(&self, key: Key) -> bool {
        self.input.is_key_held(key)
    }

    pub fn was_key_released(&self, key: Key) -> bool {
        self.input.was_key_released(key)
    }

    // ----- Graphics -----
    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color);
    }

    pub fn draw(&mut self, x: f32, y: f32, color: Color) {
        self.renderer.draw(x, y, color);
    }

    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        self.renderer.draw_line(x0, y0, x1, y1, color);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_wireframe_triangle(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) {
        self.renderer
            .draw_wireframe_triangle(x0, y0, x1, y1, x2, y2, color);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_filled_triangle(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) {
        self.renderer
            .draw_filled_triangle(x0, y0, x1, y1, x2, y2, color);
    }

    pub fn draw_wireframe_rectangle(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        self.renderer
            .draw_wireframe_rectangle(x, y, width, height, color);
    }

    pub fn draw_filled_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.renderer
            .draw_filled_rectangle(x, y, width, height, color);
    }

    pub fn draw_wireframe_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.renderer.draw_wireframe_circle(x, y, radius, color);
    }

    pub fn draw_filled_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        self.renderer.draw_filled_circle(x, y, radius, color);
    }

    pub fn draw_string(&mut self, value: impl AsRef<str>, x: f32, y: f32, color: Color, size: f32) {
        self.renderer.draw_string(value, x, y, color, size);
    }

    pub fn draw_sprite(&mut self, x: f32, y: f32, sprite: &Sprite) {
        self.renderer.draw_sprite(x, y, sprite);
    }
}
