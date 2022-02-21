use crate::color::Color;
use crate::engine::sprite::Sprite;
use crate::font;
use crate::font::Font;
use crate::maths::clamp;
use crate::platform::framebuffer::FrameBuffer;

pub struct Renderer {
    pub width: f32,
    pub height: f32,
    pixel_width: usize,
    pixel_height: usize,
    buffer: FrameBuffer,
    default_font: Font,
}

impl Renderer {
    pub fn new(
        width: f32,
        height: f32,
        pixel_width: usize,
        pixel_height: usize,
        buffer: FrameBuffer,
    ) -> Self {
        let default_font = font::load_default_font();

        Self {
            width,
            height,
            pixel_width,
            pixel_height,
            buffer,
            default_font,
        }
    }

    pub fn buffer(&self) -> &FrameBuffer {
        &self.buffer
    }

    pub fn put_pixel(&mut self, x: f32, y: f32, color: Color) {
        let y = self.height - y;

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

    pub fn clear(&mut self, color: Color) {
        self.buffer.data = vec![color.into(); self.width as usize * self.height as usize];
    }

    pub fn draw(&mut self, x: f32, y: f32, color: Color) {
        let x = x * self.pixel_width as f32;
        let y = y * self.pixel_height as f32;
        for pixel_y in 0..self.pixel_height {
            for pixel_x in 0..self.pixel_width {
                let x = x + pixel_x as f32;
                let y = y + pixel_y as f32;
                self.put_pixel(x, y, color);
            }
        }
    }

    pub fn fill_rect(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color) {
        let mut x1 = clamp(0.0, x1, self.width);
        let mut y1 = clamp(0.0, y1, self.height);

        let mut x2 = clamp(0.0, x2, self.width);
        let mut y2 = clamp(0.0, y2, self.height);

        if x1 > x2 {
            std::mem::swap(&mut x1, &mut x2);
        }

        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }

        for y in y1 as u32..=y2 as u32 {
            for x in x1 as u32..=x2 as u32 {
                self.put_pixel(x as f32, y as f32, color);
            }
        }
    }

    pub fn draw_string(&mut self, value: impl AsRef<str>, x: f32, y: f32, color: Color, size: f32) {
        let mut character_offset_x = 0.0;
        for c in value.as_ref().chars() {
            let rasterized = font::rasterize(c, &self.default_font, size);

            for rasterized_y in 0..rasterized.height {
                for rasterized_x in 0..rasterized.width {
                    let font_color = Color::rgba(
                        color.r(),
                        color.g(),
                        color.b(),
                        rasterized.data[rasterized_y * rasterized.width + rasterized_x],
                    );
                    self.put_pixel(
                        x + character_offset_x + rasterized.xmin as f32 + rasterized_x as f32,
                        y + rasterized.ymin as f32 + (rasterized.height - rasterized_y) as f32,
                        font_color,
                    );
                }
            }

            character_offset_x += rasterized.advance_width;
        }
    }

    pub fn draw_sprite(&mut self, x: f32, y: f32, sprite: &Sprite) {
        for sprite_y in 0..sprite.height() as usize {
            for sprite_x in 0..sprite.width() as usize {
                let x = x + sprite_x as f32;
                let y = y + (sprite.height() as usize - sprite_y) as f32;

                let offset = (sprite_y * sprite.width() as usize + sprite_x) * 4;
                let sprite_data = sprite.data();
                let r = sprite_data[offset];
                let g = sprite_data[offset + 1];
                let b = sprite_data[offset + 2];
                let a = sprite_data[offset + 3];
                let color = Color::rgba(r, g, b, a);

                self.draw(x, y, color);
            }
        }
    }
}
