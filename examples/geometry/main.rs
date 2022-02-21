use anyhow::Result;

use apparatus::color;
use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::engine::key::Key;
use apparatus::errors::ApparatusError;

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
    let settings = ApparatusSettings::default()
        .with_pixel_size(4, 4)
        .with_screen_size(320, 180);
    let app = Apparatus::new("Geometry", settings)?;
    app.run::<Geometry>()?;

    Ok(())
}

fn draw_lines(app: &mut Apparatus) {
    app.draw_line(20.0, 20.0, 300.0, 90.0, color::css::WHITE);
    app.draw_line(20.0, 20.0, 160.0, 160.0, color::css::GREEN);

    app.draw_line(300.0, 20.0, 20.0, 90.0, color::css::RED);
    app.draw_line(300.0, 20.0, 160.0, 160.0, color::css::YELLOW);

    app.draw_line(20.0, 160.0, 300.0, 90.0, color::css::PINK);
    app.draw_line(20.0, 160.0, 160.0, 20.0, color::css::CYAN);

    app.draw_line(300.0, 160.0, 20.0, 90.0, color::css::BLUE);
    app.draw_line(300.0, 160.0, 160.0, 20.0, color::css::GRAY);
}

fn draw_wireframe_triangles(app: &mut Apparatus) {
    app.draw_wireframe_triangle(
        50.0,
        50.0,
        200.0,
        30.0,
        260.0,
        130.0,
        color::css::LIGHTSKYBLUE,
    );
}

fn draw_filled_triangles(app: &mut Apparatus) {
    app.draw_filled_triangle(50.0, 50.0, 200.0, 30.0, 260.0, 130.0, color::css::HONEYDEW);
}

fn draw_wireframe_rectangles(app: &mut Apparatus) {
    app.draw_wireframe_rectangle(10.0, 10.0, 50.0, 50.0, color::css::RED);
    app.draw_wireframe_rectangle(70.0, 70.0, 200.0, 100.0, color::css::DEEPPINK);
}

fn draw_filled_rectangles(app: &mut Apparatus) {
    app.draw_filled_rectangle(10.0, 10.0, 50.0, 50.0, color::css::RED);
    app.draw_filled_rectangle(70.0, 70.0, 200.0, 100.0, color::css::DEEPPINK);
}

fn draw_wireframe_circles(app: &mut Apparatus) {
    app.draw_wireframe_circle(160.0, 90.0, 70.0, color::css::GREEN);
}

fn draw_filled_circles(app: &mut Apparatus) {
    app.draw_filled_circle(160.0, 90.0, 70.0, color::css::GREEN);
}
