use fontdue::{Font, FontSettings};
use std::ops::Add;
use std::time::{Duration, Instant};
use thiserror::Error;

pub trait Game {
    fn on_update(&mut self, input: &impl Input, dt: Duration);

    fn on_render(&self, gfx: &mut impl Gfx);
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("no game was provided")]
    NoGameProvided,
    #[error("window error")]
    Window(#[from] WindowError),
}

pub struct GameEngineBuilder<'a, G>
where
    G: Game,
{
    game: Option<G>,
    name: &'a str,
    window_dimensions: Vec2,
}

impl<'a, G> Default for GameEngineBuilder<'a, G>
where
    G: Game,
{
    fn default() -> Self {
        Self {
            game: None,
            name: "Firefly Game Engine",
            window_dimensions: Vec2::new(640.0, 480.0),
        }
    }
}

impl<'a, G> GameEngineBuilder<'a, G>
where
    G: Game,
{
    pub fn build(self) -> GameEngine<'a, G> {
        GameEngine {
            game: self.game,
            name: self.name,
            window_dimensions: self.window_dimensions,
        }
    }

    pub fn with_game(mut self, game: G) -> Self {
        self.game = Some(game);
        self
    }

    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }

    pub fn with_window_dimensions(mut self, width: usize, height: usize) -> Self {
        self.window_dimensions = Vec2::new(width as f32, height as f32);
        self
    }
}

pub struct GameEngine<'a, G>
where
    G: Game,
{
    game: Option<G>,
    name: &'a str,
    window_dimensions: Vec2,
}

impl<'a, G> GameEngine<'a, G>
where
    G: Game,
{
    pub fn builder() -> GameEngineBuilder<'a, G> {
        GameEngineBuilder::default()
    }

    pub fn run(self) -> Result<(), EngineError> {
        let mut game = match self.game {
            Some(game) => game,
            _ => return Err(EngineError::NoGameProvided),
        };

        let mut window = Window::new(self.name, self.window_dimensions)?;
        let frame_buffer = FrameBuffer::new(self.window_dimensions);
        let mut gfx = EngineGfx::new(self.window_dimensions, frame_buffer);

        init_default_font();

        let target_frame_duration = Duration::from_secs_f32(1.0 / 60.0);

        let mut clock = Clock::default();
        clock.tick();

        let mut running = true;
        while running {
            if window.should_close() {
                running = false;
            }

            let input = process_input(&window);

            game.on_update(&input, target_frame_duration);
            game.on_render(&mut gfx);

            let elapsed = clock.elapsed();
            if elapsed < target_frame_duration {
                if let Err(e) = sleep(target_frame_duration - elapsed) {
                    eprintln!("{}", e);
                }
            }

            clock.tick();

            window.display(gfx.buffer())?;

            // Stats.
            #[cfg(debug_assertions)]
            {
                let fps = 1.0 / clock.delta().as_secs_f32();
                println!(
                    "ms/F: {:.2} | FPS: {:.2} | Sleep tolerance (ms): {}",
                    clock.delta().as_secs_f32() * 1_000.0,
                    fps,
                    unsafe { SLEEP_TOLERANCE }.as_micros() as f32 / 1_000_f32
                );
            }
        }

        Ok(())
    }
}

//------------------------------------------- Window -----------------------------------------------

#[derive(Error, Debug)]
pub enum WindowError {
    #[error("could not create a window")]
    Create(#[source] minifb::Error),
    #[error("could not create a window")]
    Display(#[source] minifb::Error),
}

struct Window {
    width: f32,
    height: f32,
    native_window: minifb::Window,
}

impl Window {
    fn new(name: &str, dimensions: Vec2) -> Result<Self, WindowError> {
        let width = dimensions.x;
        let height = dimensions.y;

        let native_window = minifb::Window::new(
            name,
            width as usize,
            height as usize,
            minifb::WindowOptions::default(),
        )
        .map_err(WindowError::Create)?;

        let window = Self {
            width,
            height,
            native_window,
        };

        Ok(window)
    }

    fn display(&mut self, buffer: &FrameBuffer) -> Result<(), WindowError> {
        self.native_window
            .update_with_buffer(&buffer.data, self.width as usize, self.height as usize)
            .map_err(WindowError::Display)
    }

    fn should_close(&self) -> bool {
        !self.native_window.is_open()
    }
}

struct FrameBuffer {
    data: Vec<u32>,
}

impl FrameBuffer {
    fn new(dimensions: Vec2) -> Self {
        Self {
            data: vec![0; (dimensions.x * dimensions.y) as usize],
        }
    }
}

//------------------------------------------- Input ------------------------------------------------

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
}

pub trait Input {
    fn is_key_held(&self, key: Key) -> bool;
}

struct EngineInput {
    keys: Vec<Key>,
}

impl EngineInput {
    fn new() -> Self {
        let keys = Vec::new();

        Self { keys }
    }
}

impl Input for EngineInput {
    fn is_key_held(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;

    #[test]
    fn key_not_pressed_is_not_held() {
        let input = EngineInput::new();

        assert!(!input.is_key_held(Key::Up));
    }

    #[test]
    fn key_pressed_is_held() {
        let mut input = EngineInput::new();
        input.keys.push(Key::Up);

        assert!(input.is_key_held(Key::Up));
    }
}

fn process_input(window: &Window) -> EngineInput {
    let mut input = EngineInput::new();

    window
        .native_window
        .get_keys()
        .iter()
        .for_each(|key| match key {
            minifb::Key::Up => input.keys.push(Key::Up),
            minifb::Key::Down => input.keys.push(Key::Down),
            minifb::Key::Left => input.keys.push(Key::Left),
            minifb::Key::Right => input.keys.push(Key::Right),
            _ => (),
        });

    input
}

//------------------------------------------ Graphics ----------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(f32, f32, f32, f32); // (r, g, b, a)

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(r, g, b, a)
    }

    pub const fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let r = (r / 255) as f32;
        let g = (g / 255) as f32;
        let b = (b / 255) as f32;
        let a = (a / 255) as f32;

        Self::rgba(r, g, b, a)
    }

    pub fn lerp(&self, color: Self) -> Self {
        let r = lerp(self.0, color.0, self.3);
        let g = lerp(self.1, color.1, self.3);
        let b = lerp(self.2, color.2, self.3);

        Self::rgba(r, g, b, 0.0)
    }

    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 0.0);
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        let a = ((color.3 * 255.0) as u32) << 24;
        let r = ((color.0 * 255.0) as u32) << 16;
        let g = ((color.1 * 255.0) as u32) << 8;
        let b = (color.2 * 255.0) as u32;

        a | r | g | b
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn color_can_be_represented_in_argb_by_u32() {
        let color = Color::rgba(0.25, 0.5, 0.75, 1.0);
        let expected = (255 << 24) | ((63.75 as u32) << 16) | ((127.5 as u32) << 8) | 191.25 as u32;

        assert_eq!(expected, Into::<u32>::into(color));
    }

    #[test]
    fn color_can_be_created_from_u8_components() {
        let color = Color::rgba_u8(255, 255, 255, 0);
        let expected = Color::rgba(1.0, 1.0, 1.0, 0.0);

        assert_eq!(expected, color);
    }

    #[test]
    fn linear_blend_red_color_no_alpha_with_blue_color_is_red() {
        let red = Color::rgba(1.0, 0.0, 0.0, 0.0);
        let blue = Color::rgba(0.0, 0.0, 1.0, 0.0);

        assert_eq!(red.lerp(blue), red);
    }

    #[test]
    fn linear_blend_red_color_full_alpha_with_blue_color_is_blue() {
        let red = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let blue = Color::rgba(0.0, 0.0, 1.0, 0.0);

        assert_eq!(red.lerp(blue), blue);
    }
}

pub trait Gfx {
    fn clear(&mut self, color: Color);

    fn draw(&mut self, position: Vec2, color: Color);

    fn fill(&mut self, from: Vec2, to: Vec2, color: Color);

    fn text(&mut self, value: impl AsRef<str>, origin: Vec2);
}

struct EngineGfx {
    width: f32,
    height: f32,
    buffer: FrameBuffer,
}

impl EngineGfx {
    fn new(window_dimensions: Vec2, buffer: FrameBuffer) -> Self {
        Self {
            width: window_dimensions.x,
            height: window_dimensions.y,
            buffer,
        }
    }

    fn buffer(&self) -> &FrameBuffer {
        &self.buffer
    }
}

impl Gfx for EngineGfx {
    fn clear(&mut self, color: Color) {
        self.buffer.data = vec![color.into(); self.width as usize * self.height as usize];
    }

    fn draw(&mut self, position: Vec2, color: Color) {
        let x = position.x;
        let y = self.height - position.y;

        let x = clamp(0.0, x, self.width - 1.0).round();
        let y = clamp(0.0, y, self.height - 1.0).round();

        let dst = self.buffer.data[(y * self.width + x) as usize];
        let dst_a = ((dst >> 24) & 255) as f32 / 255.0;
        let dst_r = ((dst >> 16) & 255) as f32 / 255.0;
        let dst_g = ((dst >> 8) & 255) as f32 / 255.0;
        let dst_b = (dst & 255) as f32 / 255.0;
        let dst = Color::rgba(dst_r, dst_g, dst_b, dst_a);

        self.buffer.data[(y * self.width + x) as usize] = color.lerp(dst).into();
    }

    fn fill(&mut self, from: Vec2, to: Vec2, color: Color) {
        for y in from.y as i32..to.y as i32 {
            for x in from.x as i32..to.x as i32 {
                self.draw(Vec2::new(x as f32, y as f32), color);
            }
        }
    }

    fn text(&mut self, value: impl AsRef<str>, origin: Vec2) {
        unsafe {
            let mut character_offset_x = 0.0;
            for c in value.as_ref().chars() {
                if let Some(font) = &DEFAULT_FONT {
                    let (metrics, bitmap) = font.rasterize(c, DEFAULT_FONT_SIZE);

                    for y in 0..metrics.height {
                        for x in 0..metrics.width {
                            let font_pixel = Color::rgba(
                                0.0,
                                0.0,
                                0.0,
                                1.0 - (bitmap[y * metrics.width + x] as f32 / 255.0),
                            );
                            self.draw(
                                Vec2::new(
                                    origin.x + character_offset_x + metrics.xmin as f32 + x as f32,
                                    origin.y + metrics.ymin as f32 + (metrics.height - y) as f32,
                                ),
                                font_pixel,
                            );
                        }
                    }

                    character_offset_x += metrics.advance_width;
                }
            }
        }
    }
}

//------------------------------------------- Fonts ------------------------------------------------

static mut DEFAULT_FONT: Option<Font> = None;
static DEFAULT_FONT_SIZE: f32 = 24.0;

fn init_default_font() {
    let font_bytes: &[u8] = include_bytes!("../assets/fonts/Orbitron Medium.otf") as &[u8];
    let font_settings: FontSettings = FontSettings {
        scale: DEFAULT_FONT_SIZE,
        ..FontSettings::default()
    };
    unsafe {
        DEFAULT_FONT = Some(Font::from_bytes(font_bytes, font_settings).unwrap());
    }
}

//------------------------------------------- Maths ------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add<f32> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.x + rhs, self.y + rhs)
    }
}

pub fn clamp(min: f32, value: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn lerp(src: f32, dst: f32, t: f32) -> f32 {
    src + (dst - src) * t
}

#[cfg(test)]
mod maths_tests {
    use super::*;

    #[test]
    fn clamp_value_between_min_and_max_is_value() {
        assert_eq!(10.0, clamp(0.0, 10.0, 20.0));
    }

    #[test]
    fn clamp_value_less_than_min_is_min() {
        assert_eq!(0.0, clamp(0.0, -10.0, 20.0));
    }

    #[test]
    fn clamp_value_greater_than_max_is_max() {
        assert_eq!(20.0, clamp(0.0, 30.0, 20.0));
    }

    #[test]
    fn scale_vec2_scales_all_components() {
        let vec = Vec2::new(3.0, 5.0);

        assert_eq!(Vec2::new(7.0, 9.0), vec + 4.0);
    }

    #[test]
    fn lerp_between_two_values() {
        let a = 10.0;
        let b = 50.0;
        let t = 0.75;

        assert_eq!(lerp(a, b, t), 40.0);
    }
}

//------------------------------------------- Clock ------------------------------------------------
#[derive(Default)]
pub struct Clock {
    delta: Duration,
    start: Option<Instant>,
}

impl Clock {
    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn elapsed(&self) -> Duration {
        match self.start {
            Some(start) => start.elapsed(),
            None => Duration::from_secs_f32(0.0),
        }
    }

    pub fn tick(&mut self) {
        let end = Instant::now();
        if let Some(start) = self.start {
            self.delta = end - start;
        }
        self.start = Some(end);
    }
}

//------------------------------------------- Utils ------------------------------------------------
static mut SLEEP_TOLERANCE: Duration = Duration::from_micros(0);

#[derive(Debug, Error)]
pub enum SleepError {
    #[error("sleep exceeded target duration by {:?}", .0)]
    TargetDurationExceeded(Duration),
}

fn sleep(duration: Duration) -> Result<(), SleepError> {
    let mut clock = Clock::default();
    clock.tick();

    let tolerance = unsafe { SLEEP_TOLERANCE };

    if tolerance < duration {
        if duration - tolerance > Duration::from_secs_f32(0.0) {
            std::thread::sleep(duration - tolerance);
        }

        let elapsed = clock.elapsed();
        if elapsed > duration {
            unsafe {
                SLEEP_TOLERANCE += Duration::from_micros(100);
            }
            return Err(SleepError::TargetDurationExceeded(elapsed - duration));
        }
    }

    while clock.elapsed() < duration {
        // Eat CPU cycles.
    }

    Ok(())
}
