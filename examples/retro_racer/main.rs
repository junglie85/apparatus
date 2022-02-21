// Retro Racer is a derivative of [Code-It-Yourself! Retro Arcade Racing Game - Programming from
// Scratch (Quick and Simple C++)](https://youtu.be/KkMZI5Jbf18) by javidx9 and is subject to the
// following license:
//
// License (OLC-3)
// Copyright 2018-2021 OneLoneCoder.com
//
// Redistribution and use in source and binary forms, with or without modification, are permitted
// provided that the following conditions are met:
//
// Redistributions or derivations of source code must retain the above copyright notice, this list
// of conditions and the following disclaimer.
//
// Redistributions or derivative works in binary form must reproduce the above copyright notice.
// This list of conditions and the following disclaimer must be reproduced in the documentation
// and/or other materials provided with the distribution.
//
// Neither the name of the copyright holder nor the names of its contributors may be used to endorse
// or promote products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY
// WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::VecDeque;
use std::time::Duration;

use anyhow::Result;

use apparatus::color;
use apparatus::color::Color;
use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::engine::key::Key;
use apparatus::engine::sprite::Sprite;
use apparatus::errors::ApparatusError;
use apparatus::maths::clamp;

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

enum Direction {
    Forward,
    Left,
    Right,
}

struct RetroRacer {
    sprites: Vec<Sprite>,
    car_pos: f32,
    distance: f32,
    speed: f32,
    direction: Direction,
    target_curvature: f32,
    track_curvature: f32,
    player_curvature: f32,
    track_distance: f32,
    current_lap_time: Duration,
    track: Vec<TrackSegment>,
    track_segment: usize,
    lap_times: VecDeque<Duration>,
}

impl Game for RetroRacer {
    fn on_create(_app: &Apparatus) -> Result<Self, ApparatusError> {
        let mut sprites = Vec::new();

        let car_sprite_bytes = include_bytes!("assets/red_racer_32x32.png");
        let car_sprite = Sprite::from_bytes(car_sprite_bytes);
        sprites.push(car_sprite);

        let car_left_sprite_bytes = include_bytes!("assets/red_racer_left_32x32.png");
        let car_left_sprite = Sprite::from_bytes(car_left_sprite_bytes);
        sprites.push(car_left_sprite);

        let car_right_sprite_bytes = include_bytes!("assets/red_racer_right_32x32.png");
        let car_right_sprite = Sprite::from_bytes(car_right_sprite_bytes);
        sprites.push(car_right_sprite);

        let track: Vec<TrackSegment> = [
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

        let track_distance = track
            .iter()
            .map(|TrackSegment { distance, .. }| distance)
            .sum();

        let retro_racer = Self {
            sprites,
            car_pos: 0.0,
            distance: 0.0,
            speed: 0.0,
            direction: Direction::Forward,
            target_curvature: 0.0,
            track_curvature: 0.0,
            player_curvature: 0.0,
            track_distance,
            current_lap_time: Duration::from_millis(0),
            track,
            track_segment: 0,
            lap_times: VecDeque::from([Duration::from_millis(0); 5]),
        };

        Ok(retro_racer)
    }

    fn on_update(&mut self, app: &mut Apparatus) {
        let dt = app.elapsed_time();

        if app.is_key_held(Key::Up) {
            self.speed += 2.0 * dt.as_secs_f32();
            self.distance += 100.0 * dt.as_secs_f32();
        } else {
            self.speed -= 1.0;
        }

        self.direction = Direction::Forward;

        if app.is_key_held(Key::Left) {
            self.player_curvature -= 0.7 * dt.as_secs_f32();
            self.direction = Direction::Left;
        }

        if app.is_key_held(Key::Right) {
            self.player_curvature += 0.7 * dt.as_secs_f32();
            self.direction = Direction::Right;
        }

        if (self.player_curvature - self.track_curvature).abs() >= 0.8 {
            self.speed -= 5.0 * dt.as_secs_f32();
        }

        self.speed = clamp(0.0, self.speed, 1.0);
        self.distance += 70.0 * self.speed * dt.as_secs_f32();
        self.current_lap_time += dt;

        if self.distance > self.track_distance {
            self.distance -= self.track_distance;
            self.lap_times.push_front(self.current_lap_time);
            self.lap_times.pop_back();
            self.current_lap_time = Duration::from_millis(0);
        }

        let mut offset = 0.0;
        let mut track_segment = 0;

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
            (target_curvature - self.target_curvature) * dt.as_secs_f32() * self.speed;
        self.target_curvature += track_curvature_difference;
        self.track_curvature += self.target_curvature * dt.as_secs_f32() * self.speed;

        self.car_pos = self.player_curvature - self.track_curvature;
        self.car_pos = clamp(-0.95, self.car_pos, 0.95);

        // ------------------------------- Render ---------------------------------
        app.clear(Color::rgba(204, 51, 204, 0));

        let screen_height = app.screen_height();
        let screen_width = app.screen_width();

        // Draw scenery.
        for y in (screen_height / 2)..=screen_height {
            let sky = if (y as f32) < screen_height as f32 * 0.75 {
                color::css::LIGHTSKYBLUE
            } else {
                color::css::DEEPSKYBLUE
            };
            for x in 0..screen_width {
                let x = x as f32;
                let y = y as f32;
                app.draw(x, y, sky);
            }
        }

        for x in 0..screen_width {
            let hill_height =
                ((x as f32 * 0.01 * self.track_curvature).sin() * 32.0).abs() as usize;
            for y in (screen_height / 2)..((screen_height / 2) + hill_height) {
                let x = x as f32;
                let y = y as f32;
                app.draw(x, y, color::css::DARKGOLDENROD);
            }
        }

        // Draw track.
        let road = if self.track_segment == 0 || self.track_segment == 1 {
            color::css::WHITE
        } else {
            color::css::LIGHTGREY
        };

        for y in 0..(screen_height / 2) {
            let perspective =
                ((screen_height as f32 / 2.0) - y as f32) / (screen_height as f32 / 2.0);

            let grass = if (20.0 * (1.0 - perspective).powf(3.0) + self.distance * 0.1).sin() > 0.0
            {
                color::css::LAWNGREEN
            } else {
                color::css::DARKGREEN
            };

            let clipboard =
                if (80.0 * (1.0 - perspective).powf(3.0) + self.distance * 0.1).sin() > 0.0 {
                    color::css::DARKRED
                } else {
                    color::css::WHITE
                };

            for x in 0..screen_width {
                let middle_point = 0.5 + self.target_curvature * (1.0 - perspective).powf(3.0);
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
                if x < left_grass {
                    app.draw(x, y, grass);
                }
                if x >= left_grass && x < left_clipboard {
                    app.draw(x, y, clipboard);
                }
                if x >= left_clipboard && x < right_clipboard {
                    app.draw(x, y, road);
                }
                if x >= right_clipboard && x < right_grass {
                    app.draw(x, y, clipboard);
                }
                if x >= right_grass && x < screen_width as f32 {
                    app.draw(x, y, grass);
                }
            }
        }

        // Draw car.
        let sprite_idx = match self.direction {
            Direction::Forward => 0,
            Direction::Left => 1,
            Direction::Right => 2,
        };

        if let Some(car_sprite) = self.sprites.get(sprite_idx) {
            let scale = 1.0;
            let car_x = (((screen_width as f32 / 2.0)
                + ((screen_width as f32 * self.car_pos) / 2.0))
                * scale)
                - (car_sprite.width() as f32 / 2.0);
            let car_y = 30.0 * scale;

            app.draw_sprite(car_x as f32, car_y as f32, car_sprite);
        }

        // Draw stats.
        app.draw_string(
            format!("Distance: {:.2}", self.distance),
            10.0,
            app.window_height() - 20.0,
            color::css::WHITE,
            12.0,
        );
        app.draw_string(
            format!("Speed: {:.2}", self.speed),
            10.0,
            app.window_height() - 30.0,
            color::css::WHITE,
            12.0,
        );
        app.draw_string(
            format!("Target curvature:: {:.2}", self.target_curvature),
            10.0,
            app.window_height() - 40.0,
            color::css::WHITE,
            12.0,
        );
        app.draw_string(
            format!("Player curvature: {:.2}", self.player_curvature),
            10.0,
            app.window_height() - 50.0,
            color::css::WHITE,
            12.0,
        );
        app.draw_string(
            format!("Track curvature: {:.2}", self.track_curvature),
            10.0,
            app.window_height() - 60.0,
            color::css::WHITE,
            12.0,
        );

        fn format_lap_time(lap_time: &Duration) -> String {
            let minutes = lap_time.as_secs() / 60;
            let seconds = lap_time.as_secs() - (minutes * 60);
            let millis = lap_time.subsec_millis();
            format!("{:0>2}:{:>02}:{:3}", minutes, seconds, millis)
        }

        app.draw_string(
            format!("Lap 0: {}", format_lap_time(&self.current_lap_time)),
            10.0,
            app.window_height() - 80.0,
            color::css::WHITE,
            12.0,
        );

        for (lap, lap_time) in self.lap_times.iter().enumerate() {
            app.draw_string(
                format!("Lap {}: {}", lap + 1, format_lap_time(lap_time)),
                10.0,
                app.window_height() as f32 - (90.0 + 10.0 * lap as f32),
                color::css::WHITE,
                12.0,
            );
        }
    }
}

fn main() -> Result<()> {
    let settings = ApparatusSettings::default()
        .with_screen_size(320, 180)
        .with_pixel_size(4, 4);

    let app = Apparatus::new("Retro Racer!", settings)?;
    app.run::<RetroRacer>()?;

    Ok(())
}
