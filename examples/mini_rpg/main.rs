use anyhow::Result;

use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::errors::ApparatusError;

struct MiniRpg {}

impl Game for MiniRpg {
    fn on_create(_app: &Apparatus) -> Result<Self, ApparatusError> {
        Ok(Self {})
    }

    fn on_update(&mut self, _app: &mut Apparatus) {}
}

fn main() -> Result<()> {
    let engine = Apparatus::new("Mini RPG", ApparatusSettings::default())?;
    engine.run::<MiniRpg>()?;

    Ok(())
}
