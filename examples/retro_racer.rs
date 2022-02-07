use firefly::{clamp, Game, GameEngine, Gfx, Input, Key, Pixel};
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
    car_pos: f32,
    distance: f32,
    speed: f32,
    curvature: f32,
    track_curvature: f32,
    player_curvature: f32,
    track_distance: f32,
    current_lap_time: Duration,
    track: Vec<TrackSegment>,
}

impl RetroRacer {
    fn new() -> Self {
        Self {
            car_pos: 0.0,
            distance: 0.0,
            speed: 0.0,
            curvature: 0.0,
            track_curvature: 0.0,
            player_curvature: 0.0,
            track_distance: 0.0,
            current_lap_time: Duration::from_millis(0),
            track: Vec::new(),
        }
    }
}

impl Game for RetroRacer {
    fn on_create(&mut self) {
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

    fn on_update(&mut self, gfx: &mut impl Gfx, dt: Duration, input: &impl Input) {
        // Update.
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

        let target_curvature = match self.track.get(track_segment) {
            Some(&TrackSegment { curvature, .. }) => curvature,
            _ => 0.0,
        };
        let track_curvature_difference =
            (target_curvature - self.curvature) * dt.as_secs_f32() * self.speed;
        self.curvature += track_curvature_difference;

        self.track_curvature += self.curvature * dt.as_secs_f32() * self.speed;

        // Render.
        gfx.clear(Pixel::from_rgba(0.0, 0.0, 0.0, 0.0).into());

        let grass_light = Pixel::from_rgba(0.0, 0.5, 0.0, 0.0);
        let grass_dark = Pixel::from_rgba(0.0, 0.39, 0.0, 0.0);
        let clipboard_light = Pixel::from_rgba(1.0, 1.0, 1.0, 0.0);
        let clipboard_dark = Pixel::from_rgba(1.0, 0.0, 0.0, 0.0);
        let road_gray = Pixel::from_rgba(0.7, 0.7, 0.7, 0.0);
        let road_white = Pixel::from_rgba(1.0, 1.0, 1.0, 0.0);

        let road = if track_segment == 0 || track_segment == 1 {
            road_white
        } else {
            road_gray
        };

        for y in 0..(gfx.height() / 2) {
            let perspective =
                ((gfx.height() as f32 / 2.0) - y as f32) / (gfx.height() as f32 / 2.0);

            let grass = if (20.0 * (1.0 - perspective).powf(3.0) + self.distance * 0.1).sin() > 0.0
            {
                grass_light
            } else {
                grass_dark
            };

            let clipboard =
                if (80.0 * (1.0 - perspective).powf(3.0) + self.distance * 0.1).sin() > 0.0 {
                    clipboard_dark
                } else {
                    clipboard_light
                };

            for x in 0..gfx.width() {
                let middle_point = 0.5 + self.curvature * (1.0 - perspective).powf(3.0);
                let road_width = 0.1 + perspective * 0.8;
                let clipboard_width = road_width * 0.15;

                let half_road_width = road_width * 0.5;
                let left_grass = ((middle_point - half_road_width - clipboard_width)
                    * gfx.width() as f32) as usize;
                let left_clipboard =
                    ((middle_point - half_road_width) * gfx.width() as f32) as usize;
                let right_clipboard =
                    ((middle_point + half_road_width) * gfx.width() as f32) as usize;
                let right_grass = ((middle_point + half_road_width + clipboard_width)
                    * gfx.width() as f32) as usize;

                if x >= 0 && x < left_grass {
                    gfx.draw(x, y, grass);
                }
                if x >= left_grass && x < left_clipboard {
                    gfx.draw(x, y, clipboard);
                }
                if x >= left_clipboard && x < right_clipboard {
                    gfx.draw(x, y, road);
                }
                if x >= right_clipboard && x < right_grass {
                    gfx.draw(x, y, clipboard);
                }
                if x >= right_grass && x < gfx.width() {
                    gfx.draw(x, y, grass);
                }
            }
        }

        // Draw car.
        self.car_pos = self.player_curvature - self.track_curvature; // TODO: This should be done in update().

        let car = [
            0, 1, 0, 1, 1, 0, 1, 0, //
            0, 1, 1, 1, 1, 1, 1, 0, //
            0, 1, 0, 1, 1, 0, 1, 0, //
            0, 0, 0, 1, 1, 0, 0, 0, //
            0, 0, 1, 1, 1, 1, 0, 0, //
            1, 0, 1, 1, 1, 1, 0, 1, //
            1, 1, 1, 1, 1, 1, 1, 1, //
            1, 0, 1, 1, 1, 1, 0, 1, //
        ];
        let car_scale = 2;
        let (car_width, car_height) = (8, 8);
        let car_pos = (gfx.width() as f32 / 2.0 + ((gfx.width() as f32 * self.car_pos) / 2.0)
            - ((car_width as f32 / 2.0) * car_scale as f32)) as usize;
        for y in 0..car_height {
            for x in 0..car_width {
                let cp = car[(car_height - y - 1) * car_width + x];
                if cp == 1 {
                    let car_offset_y = 20;

                    gfx.fill(
                        car_pos + x * car_scale,
                        car_offset_y + y * car_scale,
                        car_pos + (x + 1) * car_scale,
                        car_offset_y + (y + 1) * car_scale,
                        Pixel::from_rgba(0.0, 0.0, 0.0, 0.0),
                    );
                }
            }
        }

        // println!("Distance: {} | Target curvature: {} | Player curvature: {} | Player speed: {} | Track curvature: {}", self.distance, self.curvature, self.player_curvature, self.speed, self.track_curvature);
    }
}

fn main() {
    let mut game = RetroRacer::new();
    let mut engine = GameEngine::new(1280, 720, 4, 4, "Retro Racer!");
    engine.start(&mut game);
}
