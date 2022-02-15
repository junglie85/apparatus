use crate::color::Color;
use crate::engine::input::Key;
use crate::engine::sprite::Sprite;
use crate::maths::Vec2;
use crate::platform::input::PlatformInput;
use crate::platform::window::Window;
use crate::platform::FrameBuffer;
use crate::renderer::renderer2d::Renderer2d;
use crate::util::{get_sleep_tolerance, sleep};
use crate::{color, ApparatusError, Game, Settings};
use clock::Clock;
use log::error;
use logger::Logger;
use std::time::Duration;

pub mod clock;
pub mod input;
pub mod logger;
pub mod sprite;

pub trait Renderer {
    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn clear(&mut self, color: Color);

    fn put_pixel(&mut self, position: Vec2, color: Color);

    fn fill_rect(&mut self, from: Vec2, to: Vec2, color: Color);

    fn draw_string(&mut self, value: impl AsRef<str>, origin: Vec2, color: Color, size: f32);

    fn draw_sprite(&mut self, sprite: &Sprite, pos: Vec2);
}

pub(crate) struct Engine<'a> {
    name: &'a str,
    window_dimensions: Vec2,
}

impl<'a> Engine<'a> {
    pub(crate) fn new(name: &'a str, settings: Settings) -> Engine<'a> {
        let window_dimensions = Vec2::new(settings.width as f32, settings.height as f32);

        Self {
            name,
            window_dimensions,
        }
    }

    pub(crate) fn run<G>(self) -> Result<(), ApparatusError>
    where
        G: Game,
    {
        let _logger = Logger::init()?;

        let mut window = Window::new(self.name, self.window_dimensions)?;
        let frame_buffer = FrameBuffer::new(self.window_dimensions);
        let mut renderer = Renderer2d::new(self.window_dimensions, frame_buffer);

        let mut game = G::on_create()?;

        let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0);

        let mut clock = Clock::default();
        clock.tick();

        let mut running = true;
        while running {
            if window.should_close() {
                running = false;
            }

            let input = process_input(&window);

            game.on_update(&input, target_frame_duration);
            game.on_render(&mut renderer);

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

fn process_input(window: &Window) -> PlatformInput {
    let mut input = PlatformInput::new();

    window
        .native_window()
        .get_keys()
        .iter()
        .for_each(|key| match key {
            minifb::Key::Up => input.keys.push(Key::Up),
            minifb::Key::Down => input.keys.push(Key::Down),
            minifb::Key::Left => input.keys.push(Key::Left),
            minifb::Key::Right => input.keys.push(Key::Right),
            _ => (),
        });

    input
}
