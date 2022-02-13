# Apparatus Game Engine

Apparatus is a game engine that takes its inspiration from a number of sources including [Dragonfly][1], [olcPixelGameEngine][2] and [Arcade][3], amongst others.

## Getting started

Implement the `Game` trait and run the `GameEngine`:

```rust
use anyhow::Result;
use log::info;
use std::time::Duration;
use firefly::{Game, GameEngine, GameEngineSettings, GameError, Gfx, Input};

struct Example {}

impl Game for Example {
    fn on_create() -> Result<Self, GameError> {
        let game = Example {};

        Ok(game)
    }
    fn on_update(&mut self, input: &impl Input, dt: Duration) {
        info!("on_update");
    }
    fn on_render(&self, gfx: &mut impl Gfx) {
        info!("on_render");
    }
}

fn main() -> Result<()> {
    let engine = GameEngine::new("Getting started", GameEngineSettings::default());
    engine.run::<Example>()?;

    Ok(())
}
```

See the [examples](#examples) for more in-depth usage. 

## Examples

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
