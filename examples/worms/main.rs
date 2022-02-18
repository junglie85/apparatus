use anyhow::Result;

use apparatus::{Apparatus, ApparatusSettings, Game};
use apparatus::errors::ApparatusError;

// Implementation notes:
// - All units (worms) have circular collision boxes.
// - Pixel level collision.

struct Worms {
    map_width: usize,
    map_height: usize,
    map: Vec<u8>,
}

impl Worms {
    fn new() -> Self {
        let map_width = 1024;
        let map_height = 512;
        let map = vec![0; map_width * map_height];

        Self {
            map_width,
            map_height,
            map,
        }
    }

    fn create_map(&mut self) {}
}

impl Game for Worms {
    fn on_create(
        _screen_width: usize,
        _screen_height: usize,
    ) -> std::result::Result<Self, ApparatusError> {
        let worms = Worms::new();

        Ok(worms)
    }

    fn on_update(&mut self, _app: &mut Apparatus) {}
}

fn main() -> Result<()> {
    let settings = ApparatusSettings::default()
        .with_screen_size(250, 180)
        .with_pixel_size(2, 2);
    let engine = Apparatus::new("Worms", settings)?;
    engine.run::<Worms>()?;

    Ok(())
}
