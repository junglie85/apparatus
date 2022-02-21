use anyhow::Result;
use apparatus::color;
use apparatus::color::Color;

use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::engine::key::Key;
use apparatus::errors::ApparatusError;

use apparatus::maths::clamp;
use core::iter::Iterator;

struct Geometry {
    option: u32,
}

impl Game for Geometry {
    fn on_create(_app: &Apparatus) -> std::result::Result<Self, ApparatusError> {
        Ok(Self { option: 1 })
    }

    fn on_update(&mut self, app: &mut Apparatus) {
        if app.was_key_released(Key::Num1) {
            self.option = 1;
        }

        if app.was_key_released(Key::Num2) {
            self.option = 2;
        }

        if app.was_key_released(Key::Num3) {
            self.option = 3;
        }

        if app.was_key_released(Key::Num4) {
            self.option = 4;
        }

        if app.was_key_released(Key::Num5) {
            self.option = 5;
        }

        if app.was_key_released(Key::Num6) {
            self.option = 6;
        }

        if app.was_key_released(Key::Num7) {
            self.option = 7;
        }

        app.clear(color::css::BLACK);

        match self.option {
            1 => draw_lines(app),
            2 => draw_wireframe_triangles(app),
            3 => draw_filled_triangles(app),
            4 => draw_wireframe_rectangles(app),
            5 => draw_filled_rectangles(app),
            6 => draw_wireframe_circles(app),
            7 => draw_filled_circles(app),
            _ => unreachable!("Invalid option number chosen"),
        }
    }
}

fn main() -> Result<()> {
    let app = Apparatus::new("Geometry", ApparatusSettings::default())?;
    app.run::<Geometry>()?;

    Ok(())
}

fn draw_lines(app: &mut Apparatus) {
    draw_line(
        &mut app.renderer,
        20.0,
        20.0,
        1260.0,
        360.0,
        color::css::WHITE,
    );
    draw_line(
        &mut app.renderer,
        20.0,
        20.0,
        640.0,
        700.0,
        color::css::GREEN,
    );

    draw_line(
        &mut app.renderer,
        1260.0,
        20.0,
        20.0,
        360.0,
        color::css::RED,
    );
    draw_line(
        &mut app.renderer,
        1260.0,
        20.0,
        640.0,
        700.0,
        color::css::YELLOW,
    );

    draw_line(
        &mut app.renderer,
        20.0,
        700.0,
        1260.0,
        360.0,
        color::css::PINK,
    );
    draw_line(
        &mut app.renderer,
        20.0,
        700.0,
        640.0,
        20.0,
        color::css::CYAN,
    );

    draw_line(
        &mut app.renderer,
        1260.0,
        700.0,
        20.0,
        360.0,
        color::css::BLUE,
    );
    draw_line(
        &mut app.renderer,
        1260.0,
        700.0,
        640.0,
        20.0,
        color::css::GRAY,
    );
}

fn draw_wireframe_triangles(app: &mut Apparatus) {
    draw_wireframe_triangle(
        &mut app.renderer,
        50.0,
        50.0,
        200.0,
        650.0,
        1100.0,
        200.0,
        color::css::LIGHTSKYBLUE,
    );
}

fn draw_filled_triangles(app: &mut Apparatus) {
    // Sort vertices by y so that y0 <= y1 <= y2.
    // Split into a flat top and flat bottom triangle - see http://www.sunshine2k.de/coding/java/TriangleRasterization/TriangleRasterization.html.
    // Draw them both - see https://mcejp.github.io/2020/11/06/bresenham.html.
    draw_filled_triangle(
        &mut app.renderer,
        50.0,
        50.0,
        200.0,
        650.0,
        1100.0,
        200.0,
        color::css::HONEYDEW,
    );
}

fn draw_wireframe_rectangles(app: &mut Apparatus) {
    draw_wireframe_rectangle(&mut app.renderer, 20.0, 20.0, 100.0, 100.0, color::css::RED);
    draw_wireframe_rectangle(
        &mut app.renderer,
        200.0,
        300.0,
        300.0,
        100.0,
        color::css::DEEPPINK,
    );
}

fn draw_filled_rectangles(app: &mut Apparatus) {
    draw_filled_rectangle(&mut app.renderer, 20.0, 20.0, 100.0, 100.0, color::css::RED);
    draw_filled_rectangle(
        &mut app.renderer,
        200.0,
        300.0,
        300.0,
        100.0,
        color::css::DEEPPINK,
    );
}

fn draw_wireframe_circles(app: &mut Apparatus) {
    draw_wireframe_circle(&mut app.renderer, 640.0, 360.0, 250.0, color::css::GREEN);
}

fn draw_filled_circles(app: &mut Apparatus) {
    draw_filled_circle(&mut app.renderer, 640.0, 360.0, 250.0, color::css::GREEN);
}

use apparatus::renderer::software_2d::Renderer;

struct Bresenham {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    error: i32,
    line_complete: bool,
}

impl Bresenham {
    fn new(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        let (x0, y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();

        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        let error = dx + dy;

        Self {
            x0,
            y0,
            x1,
            y1,
            dx,
            dy,
            sx,
            sy,
            error,
            line_complete: false,
        }
    }
}

/// Compute the next (x, y) coordinate pair for the given line.
/// See https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm for details.
impl Iterator for Bresenham {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        let point = (self.x0 as u32, self.y0 as u32);

        if self.line_complete {
            return None;
        }

        if self.x0 == self.x1 && self.y0 == self.y1 {
            self.line_complete = true;
        }

        let error_2 = 2 * self.error;
        if error_2 >= self.dy {
            if self.x0 == self.x1 {
                self.line_complete = true;
            } else {
                self.error += self.dy;
                self.x0 += self.sx;
            }
        }
        if error_2 < self.dx {
            if self.y0 == self.y1 {
                self.line_complete = true;
            } else {
                self.error += self.dx;
                self.y0 += self.sy;
            }
        }

        Some(point)
    }
}

/// Draw a line from (x0, y0) to (x1, y1) using Bresenham's line algorithm.
fn draw_line(renderer: &mut Renderer, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
    let x0 = (clamp(0.0, x0, renderer.width) + 0.5) as u32;
    let y0 = (clamp(0.0, y0, renderer.height) + 0.5) as u32;
    let x1 = (clamp(0.0, x1, renderer.width) + 0.5) as u32;
    let y1 = (clamp(0.0, y1, renderer.height) + 0.5) as u32;

    let bresenham = Bresenham::new(x0, y0, x1, y1);
    for (x, y) in bresenham {
        renderer.put_pixel(x as f32, y as f32, color);
    }
}

fn draw_wireframe_triangle(
    renderer: &mut Renderer,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: Color,
) {
    draw_line(renderer, x0, y0, x1, y1, color);
    draw_line(renderer, x1, y1, x2, y2, color);
    draw_line(renderer, x2, y2, x0, y0, color);
}

fn draw_filled_triangle(
    renderer: &mut Renderer,
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
        let mut left = Bresenham::new(x0, y0, x1, y1);
        let mut right = Bresenham::new(x0, y0, x2, y2);

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
        let mut left = Bresenham::new(x0, y0, x2, y2);
        let mut right = Bresenham::new(x1, y1, x2, y2);

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
        left: &mut Bresenham,
        right: &mut Bresenham,
        current_left_x: &mut u32,
        current_left_y: &mut u32,
        current_right_x: &mut u32,
        current_right_y: &mut u32,
    ) {
        draw_line(
            renderer,
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
    fill_flat_bottom_triangle(renderer, x1, y1, x3, y1, x2, y2, color);
    fill_flat_top_triangle(renderer, x0, y0, x1, y1, x3, y1, color);
}

fn draw_wireframe_rectangle(
    renderer: &mut Renderer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
) {
    let x1 = x + width;
    let y1 = y + height;
    draw_line(renderer, x, y, x1, y, color);
    draw_line(renderer, x, y, x, y1, color);
    draw_line(renderer, x1, y, x1, y1, color);
    draw_line(renderer, x, y1, x1, y1, color);
}

fn draw_filled_rectangle(
    renderer: &mut Renderer,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: Color,
) {
    let x1 = x + width;
    let y1 = y + height;

    let mut x0 = clamp(0.0, x, renderer.width);
    let mut y0 = clamp(0.0, y, renderer.height);

    let mut x1 = clamp(0.0, x1, renderer.width);
    let mut y1 = clamp(0.0, y1, renderer.height);

    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }

    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }

    for y in y0 as u32..=y1 as u32 {
        for x in x0 as u32..=x1 as u32 {
            renderer.put_pixel(x as f32, y as f32, color);
        }
    }
}

/// Draw a wireframe circle centered on (x, y) with radius using Bresenham's algorithm.
/// See https://www.geeksforgeeks.org/bresenhams-circle-drawing-algorithm/?ref=lbp
fn draw_wireframe_circle(renderer: &mut Renderer, x: f32, y: f32, radius: f32, color: Color) {
    let (x, y) = (x as i32, y as i32);
    let radius = radius as i32;

    let mut x0 = 0;
    let mut y0 = radius;
    let mut d = 3 - 2 * radius;

    while y0 >= x0 {
        renderer.put_pixel((x + x0) as f32, (y + y0) as f32, color);
        renderer.put_pixel((x - x0) as f32, (y + y0) as f32, color);
        renderer.put_pixel((x + x0) as f32, (y - y0) as f32, color);
        renderer.put_pixel((x - x0) as f32, (y - y0) as f32, color);
        renderer.put_pixel((x + y0) as f32, (y + x0) as f32, color);
        renderer.put_pixel((x - y0) as f32, (y + x0) as f32, color);
        renderer.put_pixel((x + y0) as f32, (y - x0) as f32, color);
        renderer.put_pixel((x - y0) as f32, (y - x0) as f32, color);

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
fn draw_filled_circle(renderer: &mut Renderer, x: f32, y: f32, radius: f32, color: Color) {
    let (x, y) = (x as i32, y as i32);
    let radius = radius as i32;

    let mut x0 = 0;
    let mut y0 = radius;
    let mut d = 3 - 2 * radius;

    while y0 >= x0 {
        draw_line(
            renderer,
            (x - x0) as f32,
            (y - y0) as f32,
            (x + x0) as f32,
            (y - y0) as f32,
            color,
        );
        draw_line(
            renderer,
            (x - y0) as f32,
            (y - x0) as f32,
            (x + y0) as f32,
            (y - x0) as f32,
            color,
        );
        draw_line(
            renderer,
            (x - y0) as f32,
            (y + x0) as f32,
            (x + y0) as f32,
            (y + x0) as f32,
            color,
        );
        draw_line(
            renderer,
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
