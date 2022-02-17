use anyhow::Result;
use apparatus::color::Color;
use apparatus::errors::ApparatusError;
use apparatus::{color, lerp, Game, Input, Key, Renderer, Settings, Vec2};
use rand::Rng;
use std::time::Duration;

// Implementation notes:
// - All units (worms) have circular collision boxes.
// - Pixel level collision.

struct Worms {
    map_width: usize,
    map_height: usize,
    map: Vec<u8>,

    output_size: usize,
    noise_seed_1d: Vec<f32>,
    perlin_noise_1d: Vec<f32>,

    octave_count: usize,
}

impl Worms {
    fn new(output_size: usize) -> Self {
        let map_width = 1024;
        let map_height = 512;
        let map = vec![0; map_width * map_height];

        let noise_seed_1d = vec![0.0; output_size];
        let perlin_noise_1d = vec![0.0; output_size];

        let octave_count = 1;

        Self {
            map_width,
            map_height,
            map,

            output_size,
            noise_seed_1d,
            perlin_noise_1d,

            octave_count,
        }
    }

    fn create_map(&mut self) {}

    fn generate_perlin_noise_1d(octaves: usize, seed: &[f32], output: &mut Vec<f32>) {
        let count = output.len();

        for (i, x) in output.iter_mut().enumerate() {
            let mut noise = 0.0;
            let mut scale = 1.0;
            let mut scale_accumulator = 0.0;

            for octave in 0..octaves {
                let pitch = count >> octave;
                let sample_1 = (i / pitch) * pitch;
                let sample_2 = (sample_1 + pitch) % count;

                let blend = (i - sample_1) as f32 / pitch as f32;
                let sample = lerp(seed[sample_2], seed[sample_1], blend);
                noise += sample * scale;
                scale_accumulator += scale;
                scale *= 0.5;
            }

            *x = noise / scale_accumulator;
        }
    }
}

impl Game for Worms {
    fn on_create(
        screen_width: usize,
        _screen_height: usize,
    ) -> std::result::Result<Self, ApparatusError> {
        let output_size = screen_width;
        let mut worms = Worms::new(output_size);

        use rand::prelude::*;
        let mut rng = rand::thread_rng();
        for i in &mut worms.noise_seed_1d {
            *i = rng.gen_range(0.0..=1.0);
        }

        Ok(worms)
    }

    fn on_update(&mut self, input: &impl Input, dt: Duration) {
        if input.was_key_released(Key::Space) {
            self.octave_count += 1;
        }

        if input.was_key_released(Key::Z) {
            use rand::prelude::*;
            let mut rng = rand::thread_rng();
            for i in &mut self.noise_seed_1d {
                *i = rng.gen_range(0.0..=1.0);
            }
        }

        if self.octave_count > 8 {
            self.octave_count = 1;
        }

        Self::generate_perlin_noise_1d(
            self.octave_count,
            &self.noise_seed_1d,
            &mut self.perlin_noise_1d,
        );
    }

    fn on_render(&self, screen_width: usize, screen_height: usize, renderer: &mut impl Renderer) {
        renderer.clear(color::css::BLACK);

        for x in 0..screen_width as usize {
            let y = (screen_height as f32 / 2.0)
                + (screen_height as f32 / 2.0 * self.perlin_noise_1d[x]);
            for f in (screen_height as f32 / 2.0) as usize..y as usize {
                renderer.draw(Vec2::new(x as f32, f as f32), color::css::GREEN);
            }
        }
    }
}

fn main() -> Result<()> {
    let settings = Settings::default()
        .with_window_size(250, 180)
        .with_pixel_size(2, 2);
    apparatus::run::<Worms>("Worms", settings)?;

    Ok(())
}
