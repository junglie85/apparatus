use crate::color::Color;
use crate::engine::sprite::Sprite;
use crate::maths::Vec2;

pub trait Renderer {
    fn width(&self) -> f32;

    fn height(&self) -> f32;

    fn clear(&mut self, color: Color);

    fn draw(&mut self, position: Vec2, color: Color);

    fn fill_rect(&mut self, from: Vec2, to: Vec2, color: Color);

    fn draw_string(&mut self, value: impl AsRef<str>, origin: Vec2, color: Color, size: f32);

    fn draw_sprite(&mut self, sprite: &Sprite, pos: Vec2);
}
