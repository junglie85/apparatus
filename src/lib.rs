use fontdue::{Font as NativeFont, FontSettings};
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
                    Color::SILVER,
                );
                gfx.draw_string(
                    format!("ms/F: {:.2}", clock.delta().as_secs_f32() * 1_000.0),
                    Vec2::new(width - 180.0, height - 20.0),
                    Color::BLACK,
                    12.0,
                );
                gfx.draw_string(
                    format!("FPS: {:.2}", fps),
                    Vec2::new(width - 180.0, height - 30.0),
                    Color::BLACK,
                    12.0,
                );
                gfx.draw_string(
                    format!(
                        "Sleep tolerance (ms): {}",
                        unsafe { SLEEP_TOLERANCE }.as_micros() as f32 / 1_000.0
                    ),
                    Vec2::new(width - 180.0, height - 40.0),
                    Color::BLACK,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(f32, f32, f32, f32); // (r, g, b, a)

impl Color {
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(r, g, b, a)
    }

    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let r = r as f32 / 255.0;
        let g = g as f32 / 255.0;
        let b = b as f32 / 255.0;
        let a = a as f32 / 255.0;

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
    pub const SILVER: Self = Self::rgba(0.7529411765, 0.7529411765, 0.7529411765, 0.0);
    pub const YELLOW: Self = Self::rgba(1.0, 1.0, 0.0, 0.0);
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

pub mod colors {
    pub mod css {
        // aliceblue	#f0f8ff	240,248,255
        //     antiquewhite	#faebd7	250,235,215
        //     aqua	#00ffff	0,255,255
        //     aquamarine	#7fffd4	127,255,212
        //     azure	#f0ffff	240,255,255
        //     beige	#f5f5dc	245,245,220
        //     bisque	#ffe4c4	255,228,196
        //     black	#000000	0,0,0
        //     blanchedalmond	#ffebcd	255,235,205
        //     blue	#0000ff	0,0,255
        //     blueviolet	#8a2be2	138,43,226
        //     brown	#a52a2a	165,42,42
        //     burlywood	#deb887	222,184,135
        //     cadetblue	#5f9ea0	95,158,160
        //     chartreuse	#7fff00	127,255,0
        //     chocolate	#d2691e	210,105,30
        //     coral	#ff7f50	255,127,80
        //     cornflowerblue	#6495ed	100,149,237
        //     cornsilk	#fff8dc	255,248,220
        //     crimson	#dc143c	220,20,60
        //     cyan	#00ffff	0,255,255
        //     darkblue	#00008b	0,0,139
        //     darkcyan	#008b8b	0,139,139
        //     darkgoldenrod	#b8860b	184,134,11
        //     darkgray	#a9a9a9	169,169,169
        //     darkgreen	#006400	0,100,0
        //     darkgrey	#a9a9a9	169,169,169
        //     darkkhaki	#bdb76b	189,183,107
        //     darkmagenta	#8b008b	139,0,139
        //     darkolivegreen	#556b2f	85,107,47
        //     darkorange	#ff8c00	255,140,0
        //     darkorchid	#9932cc	153,50,204
        //     darkred	#8b0000	139,0,0
        //     darksalmon	#e9967a	233,150,122
        //     darkseagreen	#8fbc8f	143,188,143
        //     darkslateblue	#483d8b	72,61,139
        //     darkslategray	#2f4f4f	47,79,79
        //     darkslategrey	#2f4f4f	47,79,79
        //     darkturquoise	#00ced1	0,206,209
        //     darkviolet	#9400d3	148,0,211
        //     deeppink	#ff1493	255,20,147
        //     deepskyblue	#00bfff	0,191,255
        //     dimgray	#696969	105,105,105
        //     dimgrey	#696969	105,105,105
        //     dodgerblue	#1e90ff	30,144,255
        //     firebrick	#b22222	178,34,34
        //     floralwhite	#fffaf0	255,250,240
        //     forestgreen	#228b22	34,139,34
        //     fuchsia	#ff00ff	255,0,255
        //     gainsboro	#dcdcdc	220,220,220
        //     ghostwhite	#f8f8ff	248,248,255
        //     gold	#ffd700	255,215,0
        //     goldenrod	#daa520	218,165,32
        //     gray	#808080	128,128,128
        //     green	#008000	0,128,0
        //     greenyellow	#adff2f	173,255,47
        //     grey	#808080	128,128,128
        //     honeydew	#f0fff0	240,255,240
        //     hotpink	#ff69b4	255,105,180
        //     indianred	#cd5c5c	205,92,92
        //     indigo	#4b0082	75,0,130
        //     ivory	#fffff0	255,255,240
        //     khaki	#f0e68c	240,230,140
        //     lavender	#e6e6fa	230,230,250
        //     lavenderblush	#fff0f5	255,240,245
        //     lawngreen	#7cfc00	124,252,0
        //     lemonchiffon	#fffacd	255,250,205
        //     lightblue	#add8e6	173,216,230
        //     lightcoral	#f08080	240,128,128
        //     lightcyan	#e0ffff	224,255,255
        //     lightgoldenrodyellow	#fafad2	250,250,210
        //     lightgray	#d3d3d3	211,211,211
        //     lightgreen	#90ee90	144,238,144
        //     lightgrey	#d3d3d3	211,211,211
        //     lightpink	#ffb6c1	255,182,193
        //     lightsalmon	#ffa07a	255,160,122
        //     lightseagreen	#20b2aa	32,178,170
        //     lightskyblue	#87cefa	135,206,250
        //     lightslategray	#778899	119,136,153
        //     lightslategrey	#778899	119,136,153
        //     lightsteelblue	#b0c4de	176,196,222
        //     lightyellow	#ffffe0	255,255,224
        //     lime	#00ff00	0,255,0
        //     limegreen	#32cd32	50,205,50
        //     linen	#faf0e6	250,240,230
        //     magenta	#ff00ff	255,0,255
        //     maroon	#800000	128,0,0
        //     mediumaquamarine	#66cdaa	102,205,170
        //     mediumblue	#0000cd	0,0,205
        //     mediumorchid	#ba55d3	186,85,211
        //     mediumpurple	#9370db	147,112,219
        //     mediumseagreen	#3cb371	60,179,113
        //     mediumslateblue	#7b68ee	123,104,238
        //     mediumspringgreen	#00fa9a	0,250,154
        //     mediumturquoise	#48d1cc	72,209,204
        //     mediumvioletred	#c71585	199,21,133
        //     midnightblue	#191970	25,25,112
        //     mintcream	#f5fffa	245,255,250
        //     mistyrose	#ffe4e1	255,228,225
        //     moccasin	#ffe4b5	255,228,181
        //     navajowhite	#ffdead	255,222,173
        //     navy	#000080	0,0,128
        //     oldlace	#fdf5e6	253,245,230
        //     olive	#808000	128,128,0
        //     olivedrab	#6b8e23	107,142,35
        //     orange	#ffa500	255,165,0
        //     orangered	#ff4500	255,69,0
        //     orchid	#da70d6	218,112,214
        //     palegoldenrod	#eee8aa	238,232,170
        //     palegreen	#98fb98	152,251,152
        //     paleturquoise	#afeeee	175,238,238
        //     palevioletred	#db7093	219,112,147
        //     papayawhip	#ffefd5	255,239,213
        //     peachpuff	#ffdab9	255,218,185
        //     peru	#cd853f	205,133,63
        //     pink	#ffc0cb	255,192,203
        //     plum	#dda0dd	221,160,221
        //     powderblue	#b0e0e6	176,224,230
        //     purple	#800080	128,0,128
        //     red	#ff0000	255,0,0
        //     rosybrown	#bc8f8f	188,143,143
        //     royalblue	#4169e1	65,105,225
        //     saddlebrown	#8b4513	139,69,19
        //     salmon	#fa8072	250,128,114
        //     sandybrown	#f4a460	244,164,96
        //     seagreen	#2e8b57	46,139,87
        //     seashell	#fff5ee	255,245,238
        //     sienna	#a0522d	160,82,45
        //     silver	#c0c0c0	192,192,192
        //     skyblue	#87ceeb	135,206,235
        //     slateblue	#6a5acd	106,90,205
        //     slategray	#708090	112,128,144
        //     slategrey	#708090	112,128,144
        //     snow	#fffafa	255,250,250
        //     springgreen	#00ff7f	0,255,127
        //     steelblue	#4682b4	70,130,180
        //     tan	#d2b48c	210,180,140
        //     teal	#008080	0,128,128
        //     thistle	#d8bfd8	216,191,216
        //     tomato	#ff6347	255,99,71
        //     turquoise	#40e0d0	64,224,208
        //     violet	#ee82ee	238,130,238
        //     wheat	#f5deb3	245,222,179
        //     white	#ffffff	255,255,255
        //     whitesmoke	#f5f5f5	245,245,245
        //     yellow	#ffff00	255,255,0
        //     yellowgreen	#9acd32	154,205,50
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

    fn put_pixel(&mut self, position: Vec2, color: Color);

    fn fill_rect(&mut self, from: Vec2, to: Vec2, color: Color);

    fn draw_string(&mut self, value: impl AsRef<str>, origin: Vec2, color: Color, size: f32);
}

struct EngineGfx {
    width: f32,
    height: f32,
    buffer: FrameBuffer,
    default_font: FontWrapper,
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
            FontWrapper(NativeFont::from_bytes(default_font_bytes, default_font_settings).unwrap());

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

        if x >= 0.0 && x < self.width && y >= 0.0 && y < self.height - 1.0 {
            let dst = self.buffer.data[(y * self.width + x) as usize];
            let dst_a = ((dst >> 24) & 255) as f32 / 255.0;
            let dst_r = ((dst >> 16) & 255) as f32 / 255.0;
            let dst_g = ((dst >> 8) & 255) as f32 / 255.0;
            let dst_b = (dst & 255) as f32 / 255.0;
            let dst = Color::rgba(dst_r, dst_g, dst_b, dst_a);

            self.buffer.data[(y * self.width + x) as usize] = color.lerp(dst).into();
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
                        color.0,
                        color.1,
                        color.2,
                        1.0 - (bitmap[y * metrics.width + x] as f32 / 255.0),
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
}

//------------------------------------------- Fonts ------------------------------------------------

pub struct FontWrapper(NativeFont);

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
