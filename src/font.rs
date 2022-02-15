use fontdue::{Font as NativeFont, FontSettings};

pub struct Font(NativeFont);

pub(crate) fn load_default_font() -> Font {
    let default_font_size = 24.0;
    let default_font_settings = FontSettings {
        scale: default_font_size,
        ..FontSettings::default()
    };
    let default_font_bytes = include_bytes!("../assets/fonts/Orbitron Medium.otf") as &[u8];
    let default_font =
        Font(NativeFont::from_bytes(default_font_bytes, default_font_settings).unwrap());

    default_font
}

pub struct RasterizedFont {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance_width: f32,
    pub data: Vec<u8>,
}

pub(crate) fn rasterize(character: char, font: &Font, size: f32) -> RasterizedFont {
    let (metrics, data) = font.0.rasterize(character, size);

    RasterizedFont {
        width: metrics.width,
        height: metrics.height,
        xmin: metrics.xmin,
        ymin: metrics.ymin,
        advance_width: metrics.advance_width,
        data,
    }
}
