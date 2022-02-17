use anyhow::Result;
use apparatus::color::Color;
use apparatus::errors::ApparatusError;
use apparatus::{color, lerp, Game, Input, Key, Renderer, Settings, Vec2};
use rand::prelude::*;
use rand::Rng;
use std::time::Duration;

/// Example showing how to generate 1D and 2D Perlin Noise.
/// Controls:
/// - Space bar => Cycle through octaves 1 - 8.
/// - Q => Increase the scaling bias (smooths out the noise).
/// - A => Decrease the scaling bias (make the noise more prominent).
/// - Z => Re-generate the noise seed data.
/// - 1 => 1D Perlin noise.
/// - 2 => 2D Perlin noise.

fn generate_noise_seed(output_size: usize, noise_seed: &mut Vec<f32>, rng: &mut ThreadRng) {
    unsafe { noise_seed.set_len(output_size) };
    for i in noise_seed.iter_mut() {
        *i = rng.gen_range(0.0..=1.0);
    }
}

fn generate_perlin_noise_1d(
    count: usize,
    octaves: usize,
    bias: f32,
    seed: &[f32],
    output: &mut Vec<f32>,
) {
    let bias = 1.0 / bias;

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
            scale *= bias;
        }

        *x = noise / scale_accumulator;
    }
}

fn generate_perlin_noise_2d(
    width: usize,
    height: usize,
    octaves: usize,
    bias: f32,
    seed: &[f32],
    output: &mut Vec<f32>,
) {
    let count = width;
    let bias = 1.0 / bias;

    for x in 0..width {
        for y in 0..height {
            let mut noise = 0.0;
            let mut scale = 1.0;
            let mut scale_accumulator = 0.0;

            for octave in 0..octaves {
                let pitch = count >> octave;

                let sample_x1 = (x / pitch) * pitch;
                let sample_y1 = (y / pitch) * pitch;

                let sample_x2 = (sample_x1 + pitch) % width;
                let sample_y2 = (sample_y1 + pitch) % height;

                let blend_x = (x - sample_x1) as f32 / pitch as f32;
                let blend_y = (y - sample_y1) as f32 / pitch as f32;

                let sample_t = lerp(
                    seed[sample_y1 * width + sample_x2],
                    seed[sample_y1 * width + sample_x1],
                    blend_x,
                );
                let sample_b = lerp(
                    seed[sample_y2 * width + sample_x2],
                    seed[sample_y2 * width + sample_x1],
                    blend_x,
                );

                noise += (blend_y * (sample_b - sample_t) + sample_t) * scale;
                scale_accumulator += scale;
                scale *= bias;
            }

            output[y * width + x] = noise / scale_accumulator;
        }
    }
}

enum Mode {
    OneDimension,
    TwoDimensionsGreyscale,
}

struct PerlinNoise {
    noise_seed_2d: Vec<f32>,
    perlin_noise_2d: Vec<f32>,
    output_width: usize,
    output_height: usize,
    noise_seed_1d: Vec<f32>,
    perlin_noise_1d: Vec<f32>,
    output_size: usize,
    octave_count: usize,
    scaling_bias: f32,
    mode: Mode,
    rng: ThreadRng,
}

impl PerlinNoise {
    fn new(output_size: usize, output_width: usize, output_height: usize, rng: ThreadRng) -> Self {
        let noise_seed_2d = vec![0.0; output_width * output_height];
        let perlin_noise_2d = vec![0.0; output_width * output_height];
        let noise_seed_1d = vec![0.0; output_size];
        let perlin_noise_1d = vec![0.0; output_size];
        let octave_count = 1;
        let scaling_bias = 2.0;
        let mode = Mode::OneDimension;

        Self {
            noise_seed_2d,
            perlin_noise_2d,
            output_width,
            output_height,
            noise_seed_1d,
            perlin_noise_1d,
            output_size,
            octave_count,
            scaling_bias,
            mode,
            rng,
        }
    }
}

impl Game for PerlinNoise {
    fn on_create(
        screen_width: usize,
        screen_height: usize,
    ) -> std::result::Result<Self, ApparatusError> {
        let output_size = screen_width;
        let output_width = screen_width;
        let output_height = screen_height;
        let rng = rand::thread_rng();
        let mut perlin_noise = PerlinNoise::new(output_size, output_width, output_height, rng);

        generate_noise_seed(
            output_size,
            &mut perlin_noise.noise_seed_1d,
            &mut perlin_noise.rng,
        );
        generate_noise_seed(
            output_width * output_height,
            &mut perlin_noise.noise_seed_2d,
            &mut perlin_noise.rng,
        );

        Ok(perlin_noise)
    }

    fn on_update(&mut self, input: &impl Input, _dt: Duration) {
        if input.was_key_released(Key::Space) {
            self.octave_count += 1;
        }

        if input.was_key_released(Key::Q) {
            self.scaling_bias += 0.2;
        }

        if input.was_key_released(Key::A) {
            self.scaling_bias -= 0.2;
        }

        if self.scaling_bias <= 0.0 {
            self.scaling_bias = 0.2;
        }

        if input.was_key_released(Key::Num1) {
            self.mode = Mode::OneDimension;
        }

        if input.was_key_released(Key::Num2) {
            self.mode = Mode::TwoDimensionsGreyscale;
        }

        if self.octave_count > 8 {
            self.octave_count = 1;
        }

        match self.mode {
            Mode::OneDimension => {
                if input.was_key_released(Key::Z) {
                    generate_noise_seed(self.output_size, &mut self.noise_seed_1d, &mut self.rng);
                }

                generate_perlin_noise_1d(
                    self.output_size,
                    self.octave_count,
                    self.scaling_bias,
                    &self.noise_seed_1d,
                    &mut self.perlin_noise_1d,
                );
            }
            Mode::TwoDimensionsGreyscale => {
                if input.was_key_released(Key::Z) {
                    generate_noise_seed(
                        self.output_width * self.output_height,
                        &mut self.noise_seed_2d,
                        &mut self.rng,
                    );
                }

                generate_perlin_noise_2d(
                    self.output_width,
                    self.output_height,
                    self.octave_count,
                    self.scaling_bias,
                    &self.noise_seed_2d,
                    &mut self.perlin_noise_2d,
                );
            }
        };
    }

    fn on_render(&self, screen_width: usize, screen_height: usize, renderer: &mut impl Renderer) {
        renderer.clear(color::css::BLACK);

        match self.mode {
            Mode::OneDimension => {
                for x in 0..screen_width as usize {
                    let y = (screen_height as f32 / 2.0)
                        + (screen_height as f32 / 2.0 * self.perlin_noise_1d[x]);
                    for f in (screen_height as f32 / 2.0) as usize..y as usize {
                        renderer.draw(Vec2::new(x as f32, f as f32), color::css::GREEN);
                    }
                }
            }
            Mode::TwoDimensionsGreyscale => {
                for x in 0..self.output_width {
                    for y in 0..self.output_height {
                        let noise = self.perlin_noise_2d[y * self.output_width + x];
                        let channel = (noise * 255.0) as u8;
                        let color = Color::rgba(channel, channel, channel, 255);
                        renderer.draw(Vec2::new(x as f32, y as f32), color);
                    }
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let settings = Settings::default()
        .with_screen_size(250, 180)
        .with_pixel_size(2, 2);
    apparatus::run::<PerlinNoise>("Perlin Noise", settings)?;

    Ok(())
}
