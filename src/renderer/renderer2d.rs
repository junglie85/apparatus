use crate::color::Color;
use crate::engine::Renderer;
use crate::font::{self, Font};
use crate::maths::{clamp, Vec2};
use crate::platform::FrameBuffer;
use crate::Sprite;

pub struct Renderer2d {
    width: f32,
    height: f32,
    pixel_width: usize,
    pixel_height: usize,
    buffer: FrameBuffer,
    default_font: Font,
}

impl Renderer2d {
    pub fn new(
        window_dimensions: Vec2,
        pixel_width: usize,
        pixel_height: usize,
        buffer: FrameBuffer,
    ) -> Self {
        let default_font = font::load_default_font();

        Self {
            width: window_dimensions.x,
            height: window_dimensions.y,
            pixel_width,
            pixel_height,
            buffer,
            default_font,
        }
    }

    pub fn buffer(&self) -> &FrameBuffer {
        &self.buffer
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
}

impl Renderer for Renderer2d {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn clear(&mut self, color: Color) {
        self.buffer.data = vec![color.into(); self.width as usize * self.height as usize];
    }

    fn draw(&mut self, position: Vec2, color: Color) {
        let x = position.x * self.pixel_width as f32;
        let y = position.y * self.pixel_height as f32;
        for pixel_y in 0..self.pixel_height {
            for pixel_x in 0..self.pixel_width {
                let position = Vec2::new(x + pixel_x as f32, y + pixel_y as f32);
                self.put_pixel(position, color);
            }
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
            let rasterized = font::rasterize(c, &self.default_font, size);

            for y in 0..rasterized.height {
                for x in 0..rasterized.width {
                    let font_color = Color::rgba(
                        color.r(),
                        color.g(),
                        color.b(),
                        rasterized.data[y * rasterized.width + x],
                    );
                    self.put_pixel(
                        Vec2::new(
                            origin.x + character_offset_x + rasterized.xmin as f32 + x as f32,
                            origin.y + rasterized.ymin as f32 + (rasterized.height - y) as f32,
                        ),
                        font_color,
                    );
                }
            }

            character_offset_x += rasterized.advance_width;
        }
    }

    fn draw_sprite(&mut self, sprite: &Sprite, pos: Vec2) {
        for sprite_y in 0..sprite.height() as usize {
            for sprite_x in 0..sprite.width() as usize {
                let x = pos.x + sprite_x as f32;
                let y = pos.y + (sprite.height() as usize - sprite_y) as f32;
                let position = Vec2::new(x, y);

                let offset = (sprite_y * sprite.width() as usize + sprite_x) * 4;
                let sprite_data = sprite.data();
                let r = sprite_data[offset];
                let g = sprite_data[offset + 1];
                let b = sprite_data[offset + 2];
                let a = sprite_data[offset + 3];
                let color = Color::rgba(r, g, b, a);

                self.draw(position, color);
            }
        }
    }
}
