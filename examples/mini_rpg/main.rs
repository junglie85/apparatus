use anyhow::Result;
use apparatus::{Game, GameEngineSettings, GameError, Gfx, Input};
use std::time::Duration;

struct MiniRpg {}

impl Game for MiniRpg {
    fn on_create() -> std::result::Result<Self, GameError> {
        Ok(Self {})
    }

    fn on_update(&mut self, input: &impl Input, dt: Duration) {}

    fn on_render(&self, gfx: &mut impl Gfx) {}
}

fn main() -> Result<()> {
    apparatus::run::<MiniRpg>("Mini RPG", GameEngineSettings::default())?;

    Ok(())
}
