use crate::color::Color;
use crate::engine::sprite::Sprite;
use crate::engine::Point;
use crate::font;
use crate::font::Font;
use crate::maths::clamp;
use crate::platform::framebuffer::FrameBuffer;
use crate::renderer::bresenham::BresenhamLine;

pub struct Renderer {
    width: f32,
    height: f32,
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

    fn put_pixel(&mut self, x: f32, y: f32, color: Color) {
        let y = self.height - y;

        // TODO: transmute?
        if x >= 0.0 && x < self.width && y >= 0.0 && y < self.height {
            let buffer_idx = y as usize * self.width as usize + x as usize;

            let dst = self.buffer.data[buffer_idx];
            let dst_a = ((dst >> 24) & 255) as u8;
            let dst_r = ((dst >> 16) & 255) as u8;
            let dst_g = ((dst >> 8) & 255) as u8;
            let dst_b = (dst & 255) as u8;
            let dst = Color::rgba(dst_r, dst_g, dst_b, dst_a);

            self.buffer.data[buffer_idx] = Color::linear_blend(color, dst).into();
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
                let x = clamp(0.0, x, self.width);
                let y = clamp(0.0, y, self.height);

                self.put_pixel(x, y, color);
            }
        }
    }

    /// Draw a line from (x0, y0) to (x1, y1) using Bresenham's line algorithm.
    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        let x0 = (clamp(0.0, x0.floor(), self.width) + 0.5) as u32;
        let y0 = (clamp(0.0, y0.floor(), self.height) + 0.5) as u32;
        let x1 = (clamp(0.0, x1.floor(), self.width) + 0.5) as u32;
        let y1 = (clamp(0.0, y1.floor(), self.height) + 0.5) as u32;

        let line = BresenhamLine::new(x0, y0, x1, y1);
        for (x, y) in line {
            self.draw(x as f32, y as f32, color);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_wireframe_triangle(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) {
        self.draw_line(x0, y0, x1, y1, color);
        self.draw_line(x1, y1, x2, y2, color);
        self.draw_line(x2, y2, x0, y0, color);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_filled_triangle(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Color,
    ) {
        let (mut x0, mut y0, mut x1, mut y1, mut x2, mut y2) = (x0, y0, x1, y1, x2, y2);
        // Sort vertices by y so that y0 <= y1 <= y2.
        if y1 < y0 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        if y2 < y0 {
            std::mem::swap(&mut x0, &mut x2);
            std::mem::swap(&mut y0, &mut y2);
        }

        if y2 < y1 {
            std::mem::swap(&mut x2, &mut x1);
            std::mem::swap(&mut y2, &mut y1);
        }

        // Split into a flat top and flat bottom triangle - see http://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html.
        let x3 = x0 + ((y1 - y0) / (y2 - y0)) * (x2 - x0);

        // For each top and bottom triangle, draw each side, when y increases, we have a straight horizontal line, draw it and repeat.
        fn fill_flat_top_triangle(
            renderer: &mut Renderer,
            x0: u32,
            y0: u32,
            x1: u32,
            y1: u32,
            x2: u32,
            y2: u32,
            color: Color,
        ) {
            let mut left = BresenhamLine::new(x0, y0, x1, y1);
            let mut right = BresenhamLine::new(x0, y0, x2, y2);

            let mut current_left_x = x0;
            let mut current_left_y = y0;
            let mut current_right_x = x0;
            let mut current_right_y = y0;

            while current_left_y < y1 && current_right_y < y2 {
                fill_inner_triangle(
                    renderer,
                    color,
                    &mut left,
                    &mut right,
                    &mut current_left_x,
                    &mut current_left_y,
                    &mut current_right_x,
                    &mut current_right_y,
                );
            }
        }

        fn fill_flat_bottom_triangle(
            renderer: &mut Renderer,
            x0: u32,
            y0: u32,
            x1: u32,
            y1: u32,
            x2: u32,
            y2: u32,
            color: Color,
        ) {
            let mut left = BresenhamLine::new(x0, y0, x2, y2);
            let mut right = BresenhamLine::new(x1, y1, x2, y2);

            let mut current_left_x = x0;
            let mut current_left_y = y0;
            let mut current_right_x = x1;
            let mut current_right_y = y1;

            while current_left_y < y2 && current_right_y < y2 {
                fill_inner_triangle(
                    renderer,
                    color,
                    &mut left,
                    &mut right,
                    &mut current_left_x,
                    &mut current_left_y,
                    &mut current_right_x,
                    &mut current_right_y,
                );
            }
        }

        fn fill_inner_triangle(
            renderer: &mut Renderer,
            color: Color,
            left: &mut BresenhamLine,
            right: &mut BresenhamLine,
            current_left_x: &mut u32,
            current_left_y: &mut u32,
            current_right_x: &mut u32,
            current_right_y: &mut u32,
        ) {
            renderer.draw_line(
                *current_left_x as f32,
                *current_left_y as f32,
                *current_right_x as f32,
                *current_right_y as f32,
                color,
            );

            for (x, y) in left.by_ref() {
                if y > *current_left_y {
                    *current_left_x = x;
                    *current_left_y = y;
                    break;
                }
            }

            for (x, y) in right.by_ref() {
                if y > *current_right_y {
                    *current_right_x = x;
                    *current_right_y = y;
                    break;
                }
            }
        }

        let (x0, y0, x1, y1, x2, y2, x3) = (
            x0 as u32, y0 as u32, x1 as u32, y1 as u32, x2 as u32, y2 as u32, x3 as u32,
        );
        fill_flat_bottom_triangle(self, x1, y1, x3, y1, x2, y2, color);
        fill_flat_top_triangle(self, x0, y0, x1, y1, x3, y1, color);
    }

    pub fn draw_wireframe_rectangle(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        let x1 = x + width;
        let y1 = y + height;
        self.draw_line(x, y, x1, y, color);
        self.draw_line(x, y, x, y1, color);
        self.draw_line(x1, y, x1, y1, color);
        self.draw_line(x, y1, x1, y1, color);
    }

    pub fn draw_filled_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let x1 = x + width;
        let y1 = y + height;

        let mut x0 = clamp(0.0, x.floor(), self.width);
        let mut y0 = clamp(0.0, y.floor(), self.height);

        let mut x1 = clamp(0.0, x1.floor(), self.width);
        let mut y1 = clamp(0.0, y1.floor(), self.height);

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
        }

        if y0 > y1 {
            std::mem::swap(&mut y0, &mut y1);
        }

        for y in y0 as u32..=y1 as u32 {
            for x in x0 as u32..=x1 as u32 {
                self.draw(x as f32, y as f32, color);
            }
        }
    }

    /// Draw a wireframe circle centered on (x, y) with radius using Bresenham's algorithm.
    /// See https://www.geeksforgeeks.org/bresenhams-circle-drawing-algorithm/?ref=lbp
    pub fn draw_wireframe_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        let (x, y) = (x as i32, y as i32);
        let radius = radius as i32;

        let mut x0 = 0;
        let mut y0 = radius;
        let mut d = 3 - 2 * radius;

        while y0 >= x0 {
            self.draw((x + x0) as f32, (y + y0) as f32, color);
            self.draw((x - x0) as f32, (y + y0) as f32, color);
            self.draw((x + x0) as f32, (y - y0) as f32, color);
            self.draw((x - x0) as f32, (y - y0) as f32, color);
            self.draw((x + y0) as f32, (y + x0) as f32, color);
            self.draw((x - y0) as f32, (y + x0) as f32, color);
            self.draw((x + y0) as f32, (y - x0) as f32, color);
            self.draw((x - y0) as f32, (y - x0) as f32, color);

            x0 += 1;
            if d > 0 {
                y0 -= 1;
                d += 4 * (x0 - y0) + 10;
            } else {
                d += 4 * x0 + 6;
            }
        }
    }

    /// Draw a filled circle centered on (x, y) with radius using Bresenham's algorithm.
    pub fn draw_filled_circle(&mut self, x: f32, y: f32, radius: f32, color: Color) {
        let (x, y) = (x as i32, y as i32);
        let radius = radius as i32;

        let mut x0 = 0;
        let mut y0 = radius;
        let mut d = 3 - 2 * radius;

        while y0 >= x0 {
            self.draw_line(
                (x - x0) as f32,
                (y - y0) as f32,
                (x + x0) as f32,
                (y - y0) as f32,
                color,
            );
            self.draw_line(
                (x - y0) as f32,
                (y - x0) as f32,
                (x + y0) as f32,
                (y - x0) as f32,
                color,
            );
            self.draw_line(
                (x - y0) as f32,
                (y + x0) as f32,
                (x + y0) as f32,
                (y + x0) as f32,
                color,
            );
            self.draw_line(
                (x - x0) as f32,
                (y + y0) as f32,
                (x + x0) as f32,
                (y + y0) as f32,
                color,
            );

            x0 += 1;
            if d > 0 {
                y0 -= 1;
                d += 4 * (x0 - y0) + 10;
            } else {
                d += 4 * x0 + 6;
            }
        }
    }

    /// Draw a wireframe outline of a model at a given position (translation), rotation (radians) and scale.
    pub fn draw_wireframe_model(
        &mut self,
        position: Point,
        rotation: f32,
        scale: f32,
        model: &[Point],
        color: Color,
    ) {
        let vertices: Vec<Point> = model
            .iter()
            .map(|vertex| {
                let (x, y) = (vertex.x(), vertex.y());

                let (x, y) = (x * scale, y * scale); // Scale.

                // y-axis is up, but we draw as if it is down, which means the rotation is in the wrong direction, so flip it.
                let rotation = -rotation;
                let (x, y) = (
                    x * rotation.cos() - y * rotation.sin(),
                    y * rotation.cos() + x * rotation.sin(),
                ); // Rotate.

                let (x, y) = (x + position.x(), y + position.y()); // Translate

                (x, y).into()
            })
            .collect();

        let count = vertices.len();
        for i in 0..count {
            let a = &vertices[i];
            let b = &vertices[(i + 1) % count];
            self.draw_line(a.x(), a.y(), b.x(), b.y(), color);
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

    pub fn draw_filled_rectangle_unscaled(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        let x1 = x + width;
        let y1 = y + height;

        let mut x0 = clamp(0.0, x, self.width);
        let mut y0 = clamp(0.0, y, self.height);

        let mut x1 = clamp(0.0, x1, self.width);
        let mut y1 = clamp(0.0, y1, self.height);

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
        }

        if y0 > y1 {
            std::mem::swap(&mut y0, &mut y1);
        }

        for y in y0 as u32..=y1 as u32 {
            for x in x0 as u32..=x1 as u32 {
                self.put_pixel(x as f32, y as f32, color);
            }
        }
    }
}
