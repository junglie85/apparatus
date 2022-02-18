use anyhow::Result;
use apparatus::errors::ApparatusError;
use apparatus::{Engine, Game, Input, Renderer, Settings};
use std::time::Duration;

struct MiniRpg {}

impl Game for MiniRpg {
    fn on_create(_screen_width: usize, _screen_height: usize) -> Result<Self, ApparatusError> {
        Ok(Self {})
    }

    fn on_update(&mut self, _input: &impl Input, _dt: Duration) {}

    fn on_render(
        &self,
        _screen_width: usize,
        _screen_height: usize,
        _renderer: &mut impl Renderer,
    ) {
    }
}

fn main() -> Result<()> {
    let engine = Engine::new("Mini RPG", Settings::default());
    engine.run::<MiniRpg>()?;

    Ok(())
}
