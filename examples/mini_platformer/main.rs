use anyhow::Result;
use apparatus::errors::ApparatusError;
use apparatus::{Game, Input, Renderer, Settings};
use std::time::Duration;

struct MiniPlatformer {}

impl Game for MiniPlatformer {
    fn on_create(
        _screen_width: usize,
        _screen_height: usize,
    ) -> std::result::Result<Self, ApparatusError> {
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
    apparatus::run::<MiniPlatformer>("Mini Platformer", Settings::default())?;

    Ok(())
}
