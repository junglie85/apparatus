use anyhow::Result;
use firefly::{clamp, colors, Color, Game, GameEngine, Gfx, Input, Key, Sprite, Vec2};
use std::time::Duration;

struct TrackSegment {
    curvature: f32,
    distance: f32,
}

impl From<(f32, f32)> for TrackSegment {
    fn from((curvature, distance): (f32, f32)) -> Self {
        Self {
            curvature,
            distance,
        }
    }
}

struct RetroRacer {
    width: usize,
    height: usize,
    sprites: Vec<Sprite>,
    car_pos: f32,
    distance: f32,
    speed: f32,
    curvature: f32,
    track_curvature: f32,
    player_curvature: f32,
    track_distance: f32,
    current_lap_time: Duration,
    track: Vec<TrackSegment>,
    track_segment: usize,
}

impl RetroRacer {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            sprites: Vec::new(),
            car_pos: 0.0,
            distance: 0.0,
            speed: 0.0,
            curvature: 0.0,
            track_curvature: 0.0,
            player_curvature: 0.0,
            track_distance: 0.0,
            current_lap_time: Duration::from_millis(0),
            track: Vec::new(),
            track_segment: 0,
        }
    }
}

impl Game for RetroRacer {
    fn on_create(&mut self) {
        let sprite_bytes = include_bytes!("assets/red_racer_32x32.png");
        let sprite = Sprite::from_bytes(sprite_bytes);
        self.sprites.push(sprite);

        self.track = [
            (0.0, 10.0),
            (0.0, 200.0),
            (1.0, 200.0),
            (0.0, 400.0),
            (-1.0, 100.0),
            (0.0, 200.0),
            (-1.0, 200.0),
            (1.0, 200.0),
            (0.0, 200.0),
            (0.2, 500.0),
            (0.0, 200.0),
        ]
        .into_iter()
        .map(|t| t.into())
        .collect();

        self.track_distance = self
            .track
            .iter()
            .map(|TrackSegment { distance, .. }| distance)
            .sum();
    }

    fn on_update(&mut self, input: &impl Input, dt: Duration) {
        if input.is_key_held(Key::Up) {
            self.speed += 2.0 * dt.as_secs_f32();
            self.distance += 100.0 * dt.as_secs_f32();
        } else {
            self.speed -= 1.0;
        }

        if input.is_key_held(Key::Left) {
            self.player_curvature -= 0.7 * dt.as_secs_f32();
        }

        if input.is_key_held(Key::Right) {
            self.player_curvature += 0.7 * dt.as_secs_f32();
        }

        if (self.player_curvature - self.track_curvature).abs() >= 0.8 {
            self.speed -= 5.0 * dt.as_secs_f32();
        }

        self.speed = clamp(0.0, self.speed, 1.0);
        self.distance += 70.0 * self.speed * dt.as_secs_f32();

        let mut offset = 0.0;
        let mut track_segment = 0;

        if self.distance > self.track_distance {
            self.distance -= self.track_distance;
        }

        // TODO: Should keep track of these in the game state.
        while track_segment < self.track.len() && offset < self.distance {
            if let Some(segment) = self.track.get(track_segment) {
                offset += segment.distance;
                track_segment += 1;
            }
        }

        self.track_segment = track_segment;

        let target_curvature = match self.track.get(track_segment) {
            Some(&TrackSegment { curvature, .. }) => curvature,
            _ => 0.0,
        };
        let track_curvature_difference =
            (target_curvature - self.curvature) * dt.as_secs_f32() * self.speed;
        self.curvature += track_curvature_difference;
        self.track_curvature += self.curvature * dt.as_secs_f32() * self.speed;

        self.car_pos = self.player_curvature - self.track_curvature;
    }

    fn on_render(&self, gfx: &mut impl Gfx) {
        gfx.clear(Color::rgba(204, 51, 204, 0));

        let scale = 4_f32;
        let screen_height = self.height / scale as usize;
        let screen_width = self.width / scale as usize;

        let track_segment = self.track_segment;
        let distance = self.distance;
        let curvature = self.curvature;

        let road = if track_segment == 0 || track_segment == 1 {
            colors::css::WHITE
        } else {
            colors::css::LIGHTGREY
        };

        for y in 0..(screen_height / 2) {
            let perspective =
                ((screen_height as f32 / 2.0) - y as f32) / (screen_height as f32 / 2.0);

            let grass = if (20.0 * (1.0 - perspective).powf(3.0) + distance * 0.1).sin() > 0.0 {
                colors::css::LAWNGREEN
            } else {
                colors::css::DARKGREEN
            };

            let clipboard = if (80.0 * (1.0 - perspective).powf(3.0) + distance * 0.1).sin() > 0.0 {
                colors::css::DARKRED
            } else {
                colors::css::WHITE
            };

            for x in 0..screen_width {
                let middle_point = 0.5 + curvature * (1.0 - perspective).powf(3.0);
                let road_width = 0.1 + perspective * 0.8;
                let clipboard_width = road_width * 0.15;

                let half_road_width = road_width * 0.5;
                let left_grass =
                    (middle_point - half_road_width - clipboard_width) * screen_width as f32;
                let left_clipboard = (middle_point - half_road_width) * screen_width as f32;
                let right_clipboard = (middle_point + half_road_width) * screen_width as f32;
                let right_grass =
                    (middle_point + half_road_width + clipboard_width) * screen_width as f32;

                let x = x as f32;
                let y = y as f32;
                let x1 = x * scale;
                let y1 = y * scale;
                let x2 = x1 + scale;
                let y2 = y1 + scale;
                if x >= 0.0 && x < left_grass {
                    gfx.fill_rect(Vec2::new(x1, y1), Vec2::new(x2, y2), grass);
                }
                if x >= left_grass && x < left_clipboard {
                    gfx.fill_rect(Vec2::new(x1, y1), Vec2::new(x2, y2), clipboard);
                }
                if x >= left_clipboard && x < right_clipboard {
                    gfx.fill_rect(Vec2::new(x1, y1), Vec2::new(x2, y2), road);
                }
                if x >= right_clipboard && x < right_grass {
                    gfx.fill_rect(Vec2::new(x1, y1), Vec2::new(x2, y2), clipboard);
                }
                if x >= right_grass && x < screen_width as f32 {
                    gfx.fill_rect(Vec2::new(x1, y1), Vec2::new(x2, y2), grass);
                }
            }
        }

        // Draw car.
        if let Some(car_sprite) = self.sprites.get(0) {
            let car_x = (((screen_width as f32 / 2.0)
                + ((screen_width as f32 * self.car_pos) / 2.0))
                * scale)
                - (car_sprite.width() as f32 / 2.0);
            let car_y = 20.0 * scale;

            gfx.draw_sprite(car_sprite, Vec2::new(car_x as f32, car_y as f32));
        }
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
