use fontdue::{Font, FontSettings};
use minifb::{KeyRepeat, Window, WindowOptions};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// TODO: Public API sizes as f32 (use a math library for points?).
// TODO: Load sprite images.
// TODO: Load and display fonts.
// TODO: Split into update and render stages?

pub trait Gfx {
    fn clear(&mut self, pixel: Pixel);

    fn draw(&mut self, x: usize, y: usize, pixel: Pixel);

    fn fill(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, pixel: Pixel);

    fn width(&self) -> usize;

    fn height(&self) -> usize;
}

pub trait Game {
    fn on_create(&mut self); // TODO: Return Result<(), CreateError>?

    fn on_update(&mut self, gfx: &mut impl Gfx, dt: Duration, input: &impl Input);
}

pub struct GameEngine {
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
    name: String,
}

impl GameEngine {
    pub fn new(
        width: usize,
        height: usize,
        tile_width: usize,
        tile_height: usize,
        name: impl AsRef<str>,
    ) -> Self {
        Self {
            width,
            height,
            tile_width,
            tile_height,
            name: name.as_ref().to_string(),
        }
    }

    pub fn start(&mut self, game: &mut impl Game) {
        let framebuffer = FrameBuffer {
            width: self.width,
            height: self.height,
            data: vec![0; self.width * self.height],
        };

        let mut gfx = Gfx2d::new(framebuffer, self.tile_width, self.tile_height);
        let mut window = Window::new(
            self.name.as_str(),
            self.width,
            self.height,
            WindowOptions::default(),
        )
        .unwrap_or_else(|e| panic!("{}", e));

        window.limit_update_rate(Some(Duration::from_micros(4_000)));

        // Load fonts up front.
        let font = include_bytes!("../assets/fonts/Orbitron Medium.otf") as &[u8];
        let settings = FontSettings {
            scale: 12.0,
            ..FontSettings::default()
        };
        let font = Font::from_bytes(font, settings).unwrap();
        let (metrics, bitmap) = font.rasterize_subpixel('a', 12.0);
        println!("P6\n{} {}\n255\n", metrics.width, metrics.height);
        for y in 0..metrics.height {
            for x in 0..metrics.width {
                print!("{}, ", bitmap[y * metrics.width + x]);
            }
            println!();
        }

        game.on_create();

        let target_frame_duration = Duration::from_micros((1_000_000.0 / 30.0) as u64);

        let mut start = Instant::now();
        let mut sleep_tolerance = Duration::from_micros(0);
        while window.is_open() {
            // Input.
            let mut input = InputState::new();
            if window.is_key_down(minifb::Key::Up) {
                input.keys.insert(Key::Up, KeyState { is_held: true });
            }
            if window.is_key_down(minifb::Key::Left) {
                input.keys.insert(Key::Left, KeyState { is_held: true });
            }
            if window.is_key_down(minifb::Key::Right) {
                input.keys.insert(Key::Right, KeyState { is_held: true });
            }
            let input = input;

            game.on_update(&mut gfx, target_frame_duration, &input);

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    gfx.draw(
                        (gfx.width / 2) + x,
                        ((gfx.height / 2) / 2) + y,
                        Pixel::from_rgba(
                            0.0,
                            0.0,
                            0.0,
                            bitmap[y * metrics.width + x] as f32 / 255.0,
                        ),
                    );
                    print!(
                        "{} ({}, {}), ",
                        bitmap[y * metrics.width + x],
                        (gfx.width / 2) + x,
                        ((gfx.height / 2) / 2) + y
                    );
                }
                println!();
            }
            println!();

            // Audio.

            // Sleep.
            let elapsed = start.elapsed();
            if elapsed < target_frame_duration {
                if elapsed + sleep_tolerance < target_frame_duration {
                    let sleep = target_frame_duration - (elapsed + sleep_tolerance);
                    if sleep > Duration::from_micros(0) {
                        std::thread::sleep(sleep);
                    }

                    let elapsed = start.elapsed();
                    if elapsed > target_frame_duration {
                        sleep_tolerance += Duration::from_micros(100);
                        eprintln!("Sleep caused frame to exceed target duration");
                    }
                }

                let mut elapsed = start.elapsed();
                while elapsed < target_frame_duration {
                    // Eat CPU cycles.
                    elapsed = start.elapsed();
                }
            } else {
                eprintln!(
                    "Missed target frame duration (got {:?}ms)",
                    elapsed.as_millis()
                );
            }

            let end = Instant::now();
            #[cfg(debug_assertions)]
            let frame_duration = end - start;
            start = end;

            // Display.
            let framebuffer = &mut gfx.buffer;
            window
                .update_with_buffer(&framebuffer.data, framebuffer.width(), framebuffer.height())
                .unwrap();

            // Stats.
            #[cfg(debug_assertions)]
            {
                let fps = 1_000 / frame_duration.as_millis();
                // println!(
                //     "ms/F: {} | FPS: {} | Sleep tolerance (ms): {}",
                //     frame_duration.as_millis() as f32,
                //     fps,
                //     sleep_tolerance.as_micros() as f32 / 1_000_f32
                // );
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Pixel {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Pixel> for u32 {
    fn from(pixel: Pixel) -> Self {
        let r = (pixel.r * 255.0).round() as u32;
        let g = (pixel.g * 255.0).round() as u32;
        let b = (pixel.b * 255.0).round() as u32;
        let a = (pixel.a * 255.0).round() as u32;

        (a << 24) | (r << 16) | (g << 8) | b
    }
}

impl Copy for Pixel {}

struct FrameBuffer {
    width: usize,
    height: usize,
    data: Vec<u32>,
}

impl FrameBuffer {
    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

struct Gfx2d {
    buffer: FrameBuffer,
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
}

impl Gfx2d {
    fn new(buffer: FrameBuffer, tile_width: usize, tile_height: usize) -> Self {
        let width = buffer.width() / tile_width;
        let height = buffer.height() / tile_height;

        Self {
            buffer,
            width,
            height,
            tile_width,
            tile_height,
        }
    }
}

impl Gfx for Gfx2d {
    fn clear(&mut self, pixel: Pixel) {
        self.buffer.data = vec![Into::<u32>::into(pixel); self.buffer.width * self.buffer.height];
    }

    fn draw(&mut self, x: usize, y: usize, pixel: Pixel) {
        //TODO: lerp!
        let x1 = x * self.tile_width;
        let y1 = self.buffer.height - y * self.tile_height;
        let x2 = x1 + self.tile_width;
        let y2 = y1 - self.tile_height;

        for y in y2..y1 {
            for x in x1..x2 {
                self.buffer.data[y * self.buffer.width + x] = pixel.into();
            }
        }
    }

    fn fill(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, pixel: Pixel) {
        for y in y1..y2 {
            for x in x1..x2 {
                self.draw(x, y, pixel);
            }
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

pub trait Input {
    fn is_key_held(&self, key: Key) -> bool;
}

pub struct KeyState {
    is_held: bool,
}

#[derive(Eq, PartialEq, Hash)]
pub enum Key {
    Up,
    Left,
    Right,
}

pub struct InputState {
    keys: HashMap<Key, KeyState>,
}

impl InputState {
    fn new() -> Self {
        let keys = HashMap::new();

        Self { keys }
    }
}

impl Input for InputState {
    fn is_key_held(&self, key: Key) -> bool {
        if let Some(key) = self.keys.get(&key) {
            return key.is_held;
        }

        false
    }
}

pub fn clamp<T>(min: T, value: T, max: T) -> T
where
    T: PartialOrd,
{
    if value < min {
        return min;
    } else if value > max {
        return max;
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_has_32bit_argb_representation() {
        let pixel = Pixel::from_rgba(0.8, 0.2, 0.2, 0.5);

        let expected: u32 = (128 << 24) | (204 << 16) | (51 << 8) | 51;

        assert_eq!(Into::<u32>::into(pixel), expected);
    }

    #[test]
    fn input_key_is_not_held() {
        let input = InputState::new();

        assert_eq!(input.is_key_held(Key::Up), false);
    }

    #[test]
    fn input_key_is_held() {
        let mut input = InputState::new();
        input.keys.insert(Key::Up, KeyState { is_held: true });

        assert_eq!(input.is_key_held(Key::Up), true);
    }

    #[test]
    fn clamp_value_between_min_max_is_value() {
        let value = clamp(1.0, 2.0, 3.0);

        assert_eq!(value, 2.0);
    }

    #[test]
    fn clamp_value_less_than_min_is_min() {
        let value = clamp(-1.0, -2.0, 3.0);

        assert_eq!(value, -1.0);
    }

    #[test]
    fn clamp_value_more_than_max_is_max() {
        let value = clamp(1.0, 5.0, 3.0);

        assert_eq!(value, 3.0);
    }
}
