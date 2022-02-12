use flexi_logger::{FileSpec, FlexiLoggerError, Logger, WriteMode};
use fontdue::{Font as NativeFont, FontSettings};
use image::io::Reader;
use image::{ColorType, GenericImageView, Pixel};
use log::error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Cursor;
use std::ops::Add;
use std::time::{Duration, Instant};
use thiserror::Error;

pub trait Game {
    fn on_create(&mut self);

    fn on_update(&mut self, input: &impl Input, dt: Duration);

    fn on_render(&self, gfx: &mut impl Gfx);
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("error initialising engine")]
    Initialisation(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("no game was provided")]
    NoGameProvided,
    #[error("window error")]
    Window(#[from] WindowError),
}

impl From<LoggerError> for EngineError {
    fn from(e: LoggerError) -> Self {
        Self::Initialisation(e.into())
    }
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
        let _logger = EngineLogger::init()?;

        let mut game = match self.game {
            Some(game) => game,
            _ => return Err(EngineError::NoGameProvided),
        };

        let mut window = Window::new(self.name, self.window_dimensions)?;
        let frame_buffer = FrameBuffer::new(self.window_dimensions);
        let mut gfx = EngineGfx::new(self.window_dimensions, frame_buffer);

        game.on_create();

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
                    error!("{}", e);
                }
            }

            clock.tick();

            // Stats.
            #[cfg(debug_assertions)]
            {
                let fps = 1.0 / clock.delta().as_secs_f32();
                let Vec2 {
                    x: width,
                    y: height,
                } = self.window_dimensions;
                let debug_box_left = width - 190.0;
                gfx.fill_rect(
                    Vec2::new(debug_box_left, height),
                    Vec2::new(width, height - 50.0),
                    colors::css::SILVER,
                );
                gfx.draw_string(
                    format!("ms/F: {:.2}", clock.delta().as_secs_f32() * 1_000.0),
                    Vec2::new(width - 180.0, height - 20.0),
                    colors::css::BLACK,
                    12.0,
                );
                gfx.draw_string(
                    format!("FPS: {:.2}", fps),
                    Vec2::new(width - 180.0, height - 30.0),
                    colors::css::BLACK,
                    12.0,
                );
                gfx.draw_string(
                    format!(
                        "Sleep tolerance (ms): {}",
                        unsafe { SLEEP_TOLERANCE }.as_micros() as f32 / 1_000.0
                    ),
                    Vec2::new(width - 180.0, height - 40.0),
                    colors::css::BLACK,
                    12.0,
                );
            }

            window.display(gfx.buffer())?;
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

#[derive(Clone, Copy, PartialEq)]
pub struct Color([u8; 4]); // [a, r, g, b]

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([a, r, g, b])
    }

    pub const fn r(&self) -> u8 {
        self.0[1]
    }

    pub const fn g(&self) -> u8 {
        self.0[2]
    }

    pub const fn b(&self) -> u8 {
        self.0[3]
    }

    pub const fn a(&self) -> u8 {
        self.0[0]
    }

    pub fn linear_blend(src: Self, dst: Self) -> Self {
        let t = src.a() as f32 / 255.0;
        let r = (Color::interpolate_scalar(src.r() as f32 / 255.0, dst.r() as f32 / 255.0, t)
            * 255.0) as u8;
        let g = (Color::interpolate_scalar(src.g() as f32 / 255.0, dst.g() as f32 / 255.0, t)
            * 255.0) as u8;
        let b = (Color::interpolate_scalar(src.b() as f32 / 255.0, dst.b() as f32 / 255.0, t)
            * 255.0) as u8;

        Self::rgba(r, g, b, 255)
    }

    pub fn interpolate_scalar(src: f32, dst: f32, t: f32) -> f32 {
        dst * (1.0 - t) + src * t
        // Or: `dst + (src - dst) * t`.
    }
}

impl From<Color> for u32 {
    fn from(color: Color) -> Self {
        u32::from_be_bytes(color.0)
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color{{r={:<3} g={:<3} b={:<3} a={:<3}}}",
            self.r(),
            self.g(),
            self.b(),
            self.a()
        )
    }
}

pub mod colors {
    pub mod css {
        use crate::Color;

        pub const ALICEBLUE: Color = Color::rgba(240, 248, 255, 255);
        pub const ANTIQUEWHITE: Color = Color::rgba(250, 235, 215, 255);
        pub const AQUA: Color = Color::rgba(0, 255, 255, 255);
        pub const AQUAMARINE: Color = Color::rgba(127, 255, 212, 255);
        pub const AZURE: Color = Color::rgba(240, 255, 255, 255);
        pub const BEIGE: Color = Color::rgba(245, 245, 220, 255);
        pub const BISQUE: Color = Color::rgba(255, 228, 196, 255);
        pub const BLACK: Color = Color::rgba(0, 0, 0, 255);
        pub const BLANCHEDALMOND: Color = Color::rgba(255, 235, 205, 255);
        pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
        pub const BLUEVIOLET: Color = Color::rgba(138, 43, 226, 255);
        pub const BROWN: Color = Color::rgba(165, 42, 42, 255);
        pub const BURLYWOOD: Color = Color::rgba(222, 184, 135, 255);
        pub const CADETBLUE: Color = Color::rgba(95, 158, 160, 255);
        pub const CHARTREUSE: Color = Color::rgba(127, 255, 0, 255);
        pub const CHOCOLATE: Color = Color::rgba(210, 105, 30, 255);
        pub const CORAL: Color = Color::rgba(255, 127, 80, 255);
        pub const CORNFLOWERBLUE: Color = Color::rgba(100, 149, 237, 255);
        pub const CORNSILK: Color = Color::rgba(255, 248, 220, 255);
        pub const CRIMSON: Color = Color::rgba(220, 20, 60, 255);
        pub const CYAN: Color = Color::rgba(0, 255, 255, 255);
        pub const DARKBLUE: Color = Color::rgba(0, 0, 139, 255);
        pub const DARKCYAN: Color = Color::rgba(0, 139, 139, 255);
        pub const DARKGOLDENROD: Color = Color::rgba(184, 134, 11, 255);
        pub const DARKGRAY: Color = Color::rgba(169, 169, 169, 255);
        pub const DARKGREEN: Color = Color::rgba(0, 100, 0, 255);
        pub const DARKGREY: Color = Color::rgba(169, 169, 169, 255);
        pub const DARKKHAKI: Color = Color::rgba(189, 183, 107, 255);
        pub const DARKMAGENTA: Color = Color::rgba(139, 0, 139, 255);
        pub const DARKOLIVEGREEN: Color = Color::rgba(85, 107, 47, 255);
        pub const DARKORANGE: Color = Color::rgba(255, 140, 0, 255);
        pub const DARKORCHID: Color = Color::rgba(153, 50, 204, 255);
        pub const DARKRED: Color = Color::rgba(139, 0, 0, 255);
        pub const DARKSALMON: Color = Color::rgba(233, 150, 122, 255);
        pub const DARKSEAGREEN: Color = Color::rgba(143, 188, 143, 255);
        pub const DARKSLATEBLUE: Color = Color::rgba(72, 61, 139, 255);
        pub const DARKSLATEGRAY: Color = Color::rgba(47, 79, 79, 255);
        pub const DARKTURQUOISE: Color = Color::rgba(0, 206, 209, 255);
        pub const DARKVIOLET: Color = Color::rgba(148, 0, 211, 255);
        pub const DEEPPINK: Color = Color::rgba(255, 20, 147, 255);
        pub const DEEPSKYBLUE: Color = Color::rgba(0, 191, 255, 255);
        pub const DIMGRAY: Color = Color::rgba(105, 105, 105, 255);
        pub const DODGERBLUE: Color = Color::rgba(30, 144, 255, 255);
        pub const FIREBRICK: Color = Color::rgba(178, 34, 34, 255);
        pub const FLORALWHITE: Color = Color::rgba(255, 250, 240, 255);
        pub const FORESTGREEN: Color = Color::rgba(34, 139, 34, 255);
        pub const FUCHSIA: Color = Color::rgba(255, 0, 255, 255);
        pub const GAINSBORO: Color = Color::rgba(220, 220, 220, 255);
        pub const GHOSTWHITE: Color = Color::rgba(248, 248, 255, 255);
        pub const GOLD: Color = Color::rgba(255, 215, 0, 255);
        pub const GOLDENROD: Color = Color::rgba(218, 165, 32, 255);
        pub const GRAY: Color = Color::rgba(128, 128, 128, 255);
        pub const GREEN: Color = Color::rgba(0, 128, 0, 255);
        pub const GREENYELLOW: Color = Color::rgba(173, 255, 47, 255);
        pub const GREY: Color = Color::rgba(128, 128, 128, 255);
        pub const HONEYDEW: Color = Color::rgba(240, 255, 240, 255);
        pub const HOTPINK: Color = Color::rgba(255, 105, 180, 255);
        pub const INDIANRED: Color = Color::rgba(205, 92, 92, 255);
        pub const INDIGO: Color = Color::rgba(75, 0, 130, 255);
        pub const IVORY: Color = Color::rgba(255, 255, 240, 255);
        pub const KHAKI: Color = Color::rgba(240, 230, 140, 255);
        pub const LAVENDER: Color = Color::rgba(230, 230, 250, 255);
        pub const LAVENDERBLUSH: Color = Color::rgba(255, 240, 245, 255);
        pub const LAWNGREEN: Color = Color::rgba(124, 252, 0, 255);
        pub const LEMONCHIFFON: Color = Color::rgba(255, 250, 205, 255);
        pub const LIGHTBLUE: Color = Color::rgba(173, 216, 230, 255);
        pub const LIGHTCORAL: Color = Color::rgba(240, 128, 128, 255);
        pub const LIGHTCYAN: Color = Color::rgba(224, 255, 255, 255);
        pub const LIGHTGOLDENRODYELLOW: Color = Color::rgba(250, 250, 210, 255);
        pub const LIGHTGRAY: Color = Color::rgba(211, 211, 211, 255);
        pub const LIGHTGREEN: Color = Color::rgba(144, 238, 144, 255);
        pub const LIGHTGREY: Color = Color::rgba(211, 211, 211, 255);
        pub const LIGHTPINK: Color = Color::rgba(255, 182, 193, 255);
        pub const LIGHTSALMON: Color = Color::rgba(255, 160, 122, 255);
        pub const LIGHTSEAGREEN: Color = Color::rgba(32, 178, 170, 255);
        pub const LIGHTSKYBLUE: Color = Color::rgba(135, 206, 250, 255);
        pub const LIGHTSLATEGRAY: Color = Color::rgba(119, 136, 153, 255);
        pub const LIGHTSTEELBLUE: Color = Color::rgba(176, 196, 222, 255);
        pub const LIGHTYELLOW: Color = Color::rgba(255, 255, 224, 255);
        pub const LIME: Color = Color::rgba(0, 255, 0, 255);
        pub const LIMEGREEN: Color = Color::rgba(50, 205, 50, 255);
        pub const LINEN: Color = Color::rgba(250, 240, 230, 255);
        pub const MAGENTA: Color = Color::rgba(255, 0, 255, 255);
        pub const MAROON: Color = Color::rgba(128, 0, 0, 255);
        pub const MEDIUMAQUAMARINE: Color = Color::rgba(102, 205, 170, 255);
        pub const MEDIUMBLUE: Color = Color::rgba(0, 0, 205, 255);
        pub const MEDIUMORCHID: Color = Color::rgba(186, 85, 211, 255);
        pub const MEDIUMPURPLE: Color = Color::rgba(147, 112, 219, 255);
        pub const MEDIUMSEAGREEN: Color = Color::rgba(60, 179, 113, 255);
        pub const MEDIUMSLATEBLUE: Color = Color::rgba(123, 104, 238, 255);
        pub const MEDIUMSPRINGGREEN: Color = Color::rgba(0, 250, 154, 255);
        pub const MEDIUMTURQUOISE: Color = Color::rgba(72, 209, 204, 255);
        pub const MEDIUMVIOLETRED: Color = Color::rgba(199, 21, 133, 255);
        pub const MIDNIGHTBLUE: Color = Color::rgba(25, 25, 112, 255);
        pub const MINTCREAM: Color = Color::rgba(245, 255, 250, 255);
        pub const MISTYROSE: Color = Color::rgba(255, 228, 225, 255);
        pub const MOCCASIN: Color = Color::rgba(255, 228, 181, 255);
        pub const NAVAJOWHITE: Color = Color::rgba(255, 222, 173, 255);
        pub const NAVY: Color = Color::rgba(0, 0, 128, 255);
        pub const OLDLACE: Color = Color::rgba(253, 245, 230, 255);
        pub const OLIVE: Color = Color::rgba(128, 128, 0, 255);
        pub const OLIVEDRAB: Color = Color::rgba(107, 142, 35, 255);
        pub const ORANGE: Color = Color::rgba(255, 165, 0, 255);
        pub const ORANGERED: Color = Color::rgba(255, 69, 0, 255);
        pub const ORCHID: Color = Color::rgba(218, 112, 214, 255);
        pub const PALEGOLDENROD: Color = Color::rgba(238, 232, 170, 255);
        pub const PALEGREEN: Color = Color::rgba(152, 251, 152, 255);
        pub const PALETURQUOISE: Color = Color::rgba(175, 238, 238, 255);
        pub const PALEVIOLETRED: Color = Color::rgba(219, 112, 147, 255);
        pub const PAPAYAWHIP: Color = Color::rgba(255, 239, 213, 255);
        pub const PEACHPUFF: Color = Color::rgba(255, 218, 185, 255);
        pub const PERU: Color = Color::rgba(205, 133, 63, 255);
        pub const PINK: Color = Color::rgba(255, 192, 203, 255);
        pub const PLUM: Color = Color::rgba(221, 160, 221, 255);
        pub const POWDERBLUE: Color = Color::rgba(176, 224, 230, 255);
        pub const PURPLE: Color = Color::rgba(128, 0, 128, 255);
        pub const RED: Color = Color::rgba(255, 0, 0, 255);
        pub const ROSYBROWN: Color = Color::rgba(188, 143, 143, 255);
        pub const ROYALBLUE: Color = Color::rgba(65, 105, 225, 255);
        pub const SADDLEBROWN: Color = Color::rgba(139, 69, 19, 255);
        pub const SALMON: Color = Color::rgba(250, 128, 114, 255);
        pub const SANDYBROWN: Color = Color::rgba(244, 164, 96, 255);
        pub const SEAGREEN: Color = Color::rgba(46, 139, 87, 255);
        pub const SEASHELL: Color = Color::rgba(255, 245, 238, 255);
        pub const SIENNA: Color = Color::rgba(160, 82, 45, 255);
        pub const SILVER: Color = Color::rgba(192, 192, 192, 255);
        pub const SKYBLUE: Color = Color::rgba(135, 206, 235, 255);
        pub const SLATEBLUE: Color = Color::rgba(106, 90, 205, 255);
        pub const SLATEGRAY: Color = Color::rgba(112, 128, 144, 255);
        pub const SNOW: Color = Color::rgba(255, 250, 250, 255);
        pub const SPRINGGREEN: Color = Color::rgba(0, 255, 127, 255);
        pub const STEELBLUE: Color = Color::rgba(70, 130, 180, 255);
        pub const TAN: Color = Color::rgba(210, 180, 140, 255);
        pub const TEAL: Color = Color::rgba(0, 128, 128, 255);
        pub const THISTLE: Color = Color::rgba(216, 191, 216, 255);
        pub const TOMATO: Color = Color::rgba(255, 99, 71, 255);
        pub const TURQUOISE: Color = Color::rgba(64, 224, 208, 255);
        pub const VIOLET: Color = Color::rgba(238, 130, 238, 255);
        pub const WHEAT: Color = Color::rgba(245, 222, 179, 255);
        pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
        pub const WHITESMOKE: Color = Color::rgba(245, 245, 245, 255);
        pub const YELLOW: Color = Color::rgba(255, 255, 0, 255);
        pub const YELLOWGREEN: Color = Color::rgba(154, 205, 50, 255);
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    #[test]
    fn color_has_rgba_components() {
        let color = Color::rgba(50, 100, 150, 200);

        assert_eq!(50, color.r());
        assert_eq!(100, color.g());
        assert_eq!(150, color.b());
        assert_eq!(200, color.a());
    }

    #[test]
    fn color_can_be_represented_in_argb_by_u32() {
        let color = Color::rgba(64, 128, 192, 255);
        let expected = (255 << 24) | (64 << 16) | (128 << 8) | 192;

        assert_eq!(expected, Into::<u32>::into(color));
    }

    #[test]
    fn interpolate_between_two_values() {
        let a = 10.0;
        let b = 50.0;
        let t = 0.75;

        assert_eq!(Color::interpolate_scalar(a, b, t), 20.0);
    }

    #[test]
    fn linear_blend_red_color_full_opacity_onto_blue_color_is_red() {
        let red = colors::css::RED;
        let blue = colors::css::BLUE;

        assert_eq!(Color::linear_blend(red, blue), red);
    }

    #[test]
    fn linear_blend_red_color_full_transparency_onto_blue_color_is_blue() {
        let red = Color::rgba(255, 0, 0, 0);
        let blue = colors::css::BLUE;

        assert_eq!(Color::linear_blend(red, blue), blue);
    }
}

pub struct Sprite {
    width: u32,
    height: u32,
    // data: Vec<u8>,
    pixels: Vec<Color>,
}

impl Sprite {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let cursor = Cursor::new(bytes);
        let reader = Reader::new(cursor)
            .with_guessed_format()
            .expect("Cursor io never fails");
        let image = reader.decode().unwrap(); // TODO: remove unwraps.

        let (width, height) = image.dimensions();

        let mut pixels = Vec::with_capacity(width as usize * height as usize);
        if image.color() == ColorType::Rgba8 {
            for (_, _, pixel) in image.pixels() {
                let channels = pixel.channels();
                let r = channels[0];
                let g = channels[1];
                let b = channels[2];
                let a = channels[3];
                pixels.push(Color::rgba(r, g, b, a));
            }
        }

        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

pub trait Gfx {
    fn clear(&mut self, color: Color);

    fn put_pixel(&mut self, position: Vec2, color: Color);

    fn fill_rect(&mut self, from: Vec2, to: Vec2, color: Color);

    fn draw_string(&mut self, value: impl AsRef<str>, origin: Vec2, color: Color, size: f32);

    fn draw_sprite(&mut self, sprite: &Sprite, pos: Vec2);
}

struct EngineGfx {
    width: f32,
    height: f32,
    buffer: FrameBuffer,
    default_font: Font,
    _default_font_size: f32,
}

impl EngineGfx {
    fn new(window_dimensions: Vec2, buffer: FrameBuffer) -> Self {
        let default_font_size = 24.0;
        let default_font_settings = FontSettings {
            scale: default_font_size,
            ..FontSettings::default()
        };
        let default_font_bytes = include_bytes!("../assets/fonts/Orbitron Medium.otf") as &[u8];
        let default_font =
            Font(NativeFont::from_bytes(default_font_bytes, default_font_settings).unwrap());

        Self {
            width: window_dimensions.x,
            height: window_dimensions.y,
            buffer,
            default_font,
            _default_font_size: default_font_size,
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

    fn put_pixel(&mut self, position: Vec2, color: Color) {
        let x = position.x;
        let y = self.height - position.y;

        // TODO: transmute?
        if x >= 0.0 && x < self.width && y >= 0.0 && y < self.height {
            let dst = self.buffer.data[(y * self.width + x) as usize];
            let dst_a = ((dst >> 24) & 255) as u8;
            let dst_r = ((dst >> 16) & 255) as u8;
            let dst_g = ((dst >> 8) & 255) as u8;
            let dst_b = (dst & 255) as u8;
            let dst = Color::rgba(dst_r, dst_g, dst_b, dst_a);

            self.buffer.data[(y * self.width + x) as usize] =
                Color::linear_blend(color, dst).into();
        }
    }

    fn fill_rect(&mut self, from: Vec2, to: Vec2, color: Color) {
        let mut x1 = clamp(0.0, from.x, self.width);
        let mut x2 = clamp(0.0, to.x, self.width);
        let mut y1 = clamp(0.0, from.y, self.height);
        let mut y2 = clamp(0.0, to.y, self.height);

        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }

        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }

        for y in y1 as u32..=y2 as u32 {
            for x in x1 as u32..=x2 as u32 {
                self.put_pixel(Vec2::new(x as f32, y as f32), color);
            }
        }
    }

    fn draw_string(&mut self, value: impl AsRef<str>, origin: Vec2, color: Color, size: f32) {
        let mut character_offset_x = 0.0;
        for c in value.as_ref().chars() {
            let (metrics, bitmap) = self.default_font.0.rasterize(c, size);

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    let font_color = Color::rgba(
                        color.r(),
                        color.g(),
                        color.b(),
                        bitmap[y * metrics.width + x],
                    );
                    self.put_pixel(
                        Vec2::new(
                            origin.x + character_offset_x + metrics.xmin as f32 + x as f32,
                            origin.y + metrics.ymin as f32 + (metrics.height - y) as f32,
                        ),
                        font_color,
                    );
                }
            }

            character_offset_x += metrics.advance_width;
        }
    }

    fn draw_sprite(&mut self, sprite: &Sprite, pos: Vec2) {
        for sprite_y in 0..sprite.height as usize {
            for sprite_x in 0..sprite.width as usize {
                let x = pos.x + sprite_x as f32;
                let y = pos.y + (sprite.height as usize - sprite_y) as f32;
                let color = sprite.pixels[sprite_y * sprite.width as usize + sprite_x];
                self.put_pixel(Vec2::new(x, y), color);
            }
        }
    }
}

//------------------------------------------- Fonts ------------------------------------------------

pub struct Font(NativeFont);

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

//------------------------------------------ Logging -----------------------------------------------
#[derive(Debug, Error)]
pub enum LoggerError {
    #[error("could not initialise logger")]
    Initialisation(#[from] FlexiLoggerError),
}

struct EngineLogger {
    _handle: flexi_logger::LoggerHandle,
}

impl EngineLogger {
    fn init() -> Result<Self, LoggerError> {
        let handle = Logger::try_with_str("debug")?
            .log_to_file(FileSpec::default().suppress_timestamp())
            .write_mode(WriteMode::Async)
            .start()?;

        let logger = Self { _handle: handle };

        Ok(logger)
    }
}
