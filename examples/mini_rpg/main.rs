use anyhow::Result;
use apparatus::errors::ApparatusError;
use apparatus::{Game, Input, Renderer, Settings};
use std::time::Duration;

struct MiniRpg {}

impl Game for MiniRpg {
    fn on_create() -> Result<Self, ApparatusError> {
        Ok(Self {})
    }

    fn on_update(&mut self, input: &impl Input, dt: Duration) {}

    fn on_render(&self, renderer: &mut impl Renderer) {}
}

fn main() -> Result<()> {
    apparatus::run::<MiniRpg>("Mini RPG", Settings::default())?;

    Ok(())
}
