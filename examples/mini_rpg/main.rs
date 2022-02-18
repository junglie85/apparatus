use anyhow::Result;

use apparatus::errors::ApparatusError;
use apparatus::{Apparatus, ApparatusSettings, Game};

struct MiniRpg {}

impl Game for MiniRpg {
    fn on_create(_screen_width: usize, _screen_height: usize) -> Result<Self, ApparatusError> {
        Ok(Self {})
    }

    fn on_update(&mut self, _app: &mut Apparatus) {}
}

fn main() -> Result<()> {
    let engine = Apparatus::new("Mini RPG", ApparatusSettings::default())?;
    engine.run::<MiniRpg>()?;

    Ok(())
}
