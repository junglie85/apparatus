use crate::{ApparatusError, Input, Renderer};
use std::time::Duration;

pub trait Game<Game = Self> {
    /// Called once, after the engine has initialised.
    fn on_create(screen_width: usize, screen_height: usize) -> Result<Game, ApparatusError>;

    /// Called once per frame.
    fn on_update(&mut self, input: &impl Input, dt: Duration);

    /// Called once per frame.
    fn on_render(&self, screen_width: usize, screen_height: usize, renderer: &mut impl Renderer);
}
