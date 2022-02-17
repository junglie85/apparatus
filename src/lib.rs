pub use crate::engine::input::*;
pub use crate::engine::sprite::*;
use crate::engine::Engine;
pub use crate::engine::Renderer;
use crate::errors::ApparatusError;
pub use crate::maths::*;
use std::time::Duration;

pub mod color;
mod engine;
pub mod errors;
pub mod font;
mod maths;
mod platform;
mod renderer;
mod util;

pub trait Game<Game = Self> {
    /// Called once, after the engine has initialised.
    fn on_create(screen_width: usize, screen_height: usize) -> Result<Game, ApparatusError>;

    /// Called once per frame.
    fn on_update(&mut self, input: &impl Input, dt: Duration);

    /// Called once per frame.
    fn on_render(&self, screen_width: usize, screen_height: usize, renderer: &mut impl Renderer);
}

pub fn run<G>(name: &str, settings: Settings) -> Result<(), ApparatusError>
where
    G: Game,
{
    let engine = Engine::new(name, settings);
    engine.run::<G>()
}

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
