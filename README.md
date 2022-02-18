# Apparatus Game Engine

Apparatus is a game engine that takes its inspiration from a number of sources including [Dragonfly][1], [olcPixelGameEngine][2] and [Arcade][3], amongst others.

## Getting started

Implement the `Game` trait and tell the `Apparatus` to `run` it:

```rust
use anyhow::Result;
use log::info;

use apparatus::color::Color;
use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::engine::key::Key;
use apparatus::errors::ApparatusError;
use apparatus::maths::clamp;

struct Example {
    color: Color,
}

impl Game for Example {
    fn on_create(_screen_width: usize, _screen_height: usize) -> Result<Self, ApparatusError> {
        let game = Example {
            color: Color::rgba(128, 128, 128, 255),
        };

        Ok(game)
    }

    fn on_update(&mut self, app: &mut Apparatus) {
        info!("updating");

        let dt = app.elapsed_time();

        if app.is_key_held(Key::Up) {
            let r = clamp(
                0.0,
                self.color.r() as f32 + (100.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;
            let g = clamp(
                0.0,
                self.color.g() as f32 + (100.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;
            let b = clamp(
                0.0,
                self.color.b() as f32 + (100.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;

            self.color = Color::rgba(r, g, b, 255);
        }

        if app.is_key_held(Key::Down) {
            let r = clamp(
                0.0,
                self.color.r() as f32 - (50.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;
            let g = clamp(
                0.0,
                self.color.g() as f32 - (50.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;
            let b = clamp(
                0.0,
                self.color.b() as f32 - (50.0 * dt.as_secs_f32()),
                255.0,
            ) as u8;

            self.color = Color::rgba(r, g, b, 255);
        }

        info!("rendering");

        app.clear(self.color);
    }
}

fn main() -> Result<()> {
    let engine = Apparatus::new("Getting Started", ApparatusSettings::default())?;
    engine.run::<Example>()?;

    Ok(())
}

```

See the [examples](#examples) for more in-depth usage. 

## Examples

- [Perlin Noise](examples/perlin_noise)
- [Retro Racer](examples/retro_racer)

## Development

### Running tests

Run tests from the command line with cargo:

```commandline
cargo test
```

## Versioning

Apparatus is in very early development and does not currently follow semver.
Neither does it commit to a minimum supported Rust version.

[1]: https://dragonfly.wpi.edu/ "Dragonfly"
[2]: https://github.com/OneLoneCoder/olcPixelGameEngine "olcPixelGameEngine"
[3]: https://api.arcade.academy/en/latest/ "The Python Arcade Library"
