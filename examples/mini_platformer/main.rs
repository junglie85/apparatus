use anyhow::Result;

use apparatus::errors::ApparatusError;
use apparatus::{Apparatus, ApparatusSettings, Game};

struct MiniPlatformer {}

impl Game for MiniPlatformer {
    fn on_create(
        _screen_width: usize,
        _screen_height: usize,
    ) -> std::result::Result<Self, ApparatusError> {
        Ok(Self {})
    }

    fn on_update(&mut self, _app: &mut Apparatus) {}
}

fn main() -> Result<()> {
    let app = Apparatus::new("Mini Platformer", ApparatusSettings::default())?;
    app.run::<MiniPlatformer>()?;

    Ok(())
}
