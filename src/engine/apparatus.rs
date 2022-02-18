use crate::engine::clock::Clock;
use crate::engine::logger::Logger;
use crate::maths::Vec2;
use crate::platform::input::PlatformInput;
use crate::platform::window::Window;
use crate::platform::FrameBuffer;
use crate::renderer::renderer2d::Renderer2d;
use crate::util::{get_sleep_tolerance, sleep};
use crate::{color, ApparatusError, Game, Renderer};
use log::error;
use std::time::Duration;

pub struct Settings {
    width: usize,
    height: usize,
    pixel_width: usize,
    pixel_height: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            pixel_width: 1,
            pixel_height: 1,
        }
    }
}

impl Settings {
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

pub struct Engine<'a> {
    name: &'a str,
    pixel_width: usize,
    pixel_height: usize,
    screen_width: usize,
    screen_height: usize,
    window_dimensions: Vec2,
}

impl<'a> Engine<'a> {
    pub fn new(name: &'a str, settings: Settings) -> Engine<'a> {
        let pixel_width = settings.pixel_width;
        let pixel_height = settings.pixel_height;
        let screen_width = settings.width;
        let screen_height = settings.height;
        let window_dimensions = Vec2::new(
            (screen_width * pixel_width) as f32,
            (screen_height * pixel_height) as f32,
        );

        Self {
            name,
            pixel_width,
            pixel_height,
            screen_width,
            screen_height,
            window_dimensions,
        }
    }

    pub fn run<G>(self) -> Result<(), ApparatusError>
    where
        G: Game,
    {
        let _logger = Logger::init()?;

        let mut clock = Clock::default();
        clock.tick();

        let mut window = Window::new(self.name, self.window_dimensions)?;
        let frame_buffer = FrameBuffer::new(self.window_dimensions);
        let mut renderer = Renderer2d::new(
            self.window_dimensions,
            self.pixel_width,
            self.pixel_height,
            frame_buffer,
        );
        let mut input = PlatformInput::new();

        let mut game = G::on_create(self.screen_width, self.screen_height)?;

        let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0);

        let mut running = true;
        while running {
            if window.should_close() {
                running = false;
            }

            input.process_input(&window);

            game.on_update(&input, target_frame_duration);
            game.on_render(self.screen_width, self.screen_height, &mut renderer);

            let elapsed = clock.elapsed();
            if elapsed < target_frame_duration {
                if let Err(e) = sleep(target_frame_duration - elapsed) {
                    error!("{}", e);
                }
            }

            clock.tick();

            // Stats.
            #[cfg(debug_assertions)]
            {
                let fps = 1.0 / clock.delta().as_secs_f32();
                let Vec2 {
                    x: width,
                    y: height,
                } = self.window_dimensions;
                let debug_box_left = width - 190.0;
                renderer.fill_rect(
                    Vec2::new(debug_box_left, height),
                    Vec2::new(width, height - 50.0),
                    color::css::SILVER,
                );
                renderer.draw_string(
                    format!("ms/F: {:.2}", clock.delta().as_secs_f32() * 1_000.0),
                    Vec2::new(width - 180.0, height - 20.0),
                    color::css::BLACK,
                    12.0,
                );
                renderer.draw_string(
                    format!("FPS: {:.2}", fps),
                    Vec2::new(width - 180.0, height - 30.0),
                    color::css::BLACK,
                    12.0,
                );
                renderer.draw_string(
                    format!(
                        "Sleep tolerance (ms): {}",
                        get_sleep_tolerance().as_micros() as f32 / 1_000.0
                    ),
                    Vec2::new(width - 180.0, height - 40.0),
                    color::css::BLACK,
                    12.0,
                );
            }

            window.display(renderer.buffer())?;
        }

        Ok(())
    }
}
