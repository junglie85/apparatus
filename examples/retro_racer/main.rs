use anyhow::Result;
use firefly::{clamp, Color, Game, GameEngine, Gfx, Input, Key, Sprite, Vec2};
use std::time::Duration;

struct RetroRacer {
    width: usize,
    height: usize,
    position: Vec2,
    sprites: Vec<Sprite>,
}

impl RetroRacer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            position: Vec2::new(100.0, 100.0),
            sprites: Vec::new(),
        }
    }
}

impl Game for RetroRacer {
    fn on_create(&mut self) {
        let sprite_bytes = include_bytes!("assets/red_racer_32x32.png");
        let sprite = Sprite::from_bytes(sprite_bytes);
        self.sprites.push(sprite);
    }

    fn on_update(&mut self, input: &impl Input, dt: Duration) {
        let distance = 100.0 * dt.as_secs_f32();
        if input.is_key_held(Key::Up) {
            self.position.y += distance;
        }
        if input.is_key_held(Key::Down) {
            self.position.y -= distance;
        }
        if input.is_key_held(Key::Left) {
            self.position.x -= distance;
        }
        if input.is_key_held(Key::Right) {
            self.position.x += distance;
        }

        self.position.x = clamp(0.0, self.position.x, self.width as f32);
        self.position.y = clamp(0.0, self.position.y, self.height as f32);
    }

    fn on_render(&self, gfx: &mut impl Gfx) {
        gfx.clear(Color::rgba(204, 51, 204, 0));
        gfx.put_pixel(self.position, Color::BLACK);
        gfx.fill_rect(
            self.position + 10.0,
            self.position + 10.0 + 4.0,
            Color::BLACK,
        );
        gfx.draw_string(
            "Retro Racer!",
            Vec2::new(50.0, self.height as f32 - 50.0),
            Color::YELLOW,
            24.0,
        );
        gfx.draw_sprite(self.sprites.get(0).unwrap(), Vec2::new(200.0, 200.0));
    }
}

fn main() -> Result<()> {
    let retro_racer = RetroRacer::new(1280, 720);

    let engine = GameEngine::builder()
        .with_game(retro_racer)
        .with_name("Retro Racer!")
        .with_window_dimensions(1280, 720)
        .build();

    engine.run()?;

    Ok(())
}
