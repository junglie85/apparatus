use anyhow::Result;
use apparatus::color;
use apparatus::color::Color;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::any::Any;
use std::f32::consts::PI;

use apparatus::engine::apparatus::{Apparatus, ApparatusSettings};
use apparatus::engine::game::Game;
use apparatus::engine::key::Key;
use apparatus::engine::mouse::MouseButton;
use apparatus::engine::sprite::Sprite;
use apparatus::engine::Point;
use apparatus::errors::ApparatusError;
use apparatus::maths::{clamp, lerp};
use apparatus::renderer::bresenham::BresenhamLine;

// Implementation notes:
// - All units (worms) have circular collision boxes.
// - Pixel level collision.

const SKY: Color = color::css::CYAN;
const LAND: Color = color::css::DARKGREEN;

static mut NEXT_PHYSICS_ID: u128 = 0;

fn get_physics_id() -> u128 {
    unsafe {
        let id = NEXT_PHYSICS_ID;
        let (next, overflow) = NEXT_PHYSICS_ID.overflowing_add(1);
        if overflow {
            panic!("We ran out of physics ID's, fix this");
        }
        NEXT_PHYSICS_ID = next;
        id
    }
}

#[derive(Debug, Copy, Clone)]
enum GameState {
    Reset,
    GenerateTerrain,
    GeneratingTerrain,
    AllocateUnits,
    AllocatingUnits,
    StartPlay,
    CameraMode,
}

struct Worms {
    map_width: u32,
    map_height: u32,
    map: Vec<u8>,
    camera_pos_x: f32,
    camera_pos_y: f32,
    target_camera_pos_x: f32,
    target_camera_pos_y: f32,
    rng: ThreadRng,

    physics_things: Vec<Box<dyn Physics>>,
    object_under_control: Option<u128>,
    camera_tracking_object: Option<u128>,
    is_energising: bool,
    energy_level: f32,
    fire_weapon: bool,

    game_state: GameState,
    next_state: GameState,
    is_game_stable: bool,

    player_has_control: bool,
    player_action_complete: bool,
}

impl Worms {
    fn new(rng: ThreadRng) -> Self {
        let map_width = 1024;
        let map_height = 512;
        let map = vec![0; map_width as usize * map_height as usize];

        let camera_pos_x = 0.0;
        let camera_pos_y = map_height as f32;
        let target_camera_pos_x = 0.0;
        let target_camera_pos_y = map_height as f32;

        let physics_things = Vec::new();

        let object_under_control = None;
        let camera_tracking_object = None;

        let is_energising = false;
        let energy_level = 0.0;
        let fire_weapon = false;

        let game_state = GameState::Reset;
        let next_state = GameState::Reset;
        let is_game_stable = false;

        let player_has_control = false;
        let player_action_complete = false;

        Self {
            map_width,
            map_height,
            map,
            camera_pos_x,
            camera_pos_y,
            target_camera_pos_x,
            target_camera_pos_y,
            rng,
            physics_things,
            object_under_control,
            camera_tracking_object,
            is_energising,
            energy_level,
            fire_weapon,
            game_state,
            next_state,
            is_game_stable,
            player_has_control,
            player_action_complete,
        }
    }

    fn create_map(&mut self) {
        let mut noise_seed = vec![0.0; self.map_width as usize];
        let mut surface = vec![0.0; self.map_width as usize];

        generate_noise_seed(self.map_width, &mut noise_seed, &mut self.rng);
        noise_seed[0] = 0.5; // Hack to set initial and final octave seed to be half way up the map.

        let octaves = 8;
        let scaling_bias = 2.0;
        generate_perlin_noise_1d(
            self.map_width,
            octaves,
            scaling_bias,
            &noise_seed,
            &mut surface,
        );

        for x in 0..self.map_width {
            for y in 0..self.map_height {
                if y as f32 <= surface[x as usize] * self.map_height as f32 {
                    self.map[(y * self.map_width + x) as usize] = 1;
                } else {
                    self.map[(y * self.map_width + x) as usize] = 0;
                }
            }
        }
    }

    fn get_object(&self, id: u128) -> Option<&Box<dyn Physics>> {
        self.physics_things
            .iter()
            .find(|p| p.physics_object().id == id)
    }

    fn get_object_mut(&mut self, id: u128) -> Option<&mut Box<dyn Physics>> {
        self.physics_things
            .iter_mut()
            .find(|p| p.physics_object().id == id)
    }
}

impl Game for Worms {
    fn on_create(_app: &Apparatus) -> std::result::Result<Self, ApparatusError> {
        let rng = rand::thread_rng();
        let worms = Worms::new(rng);

        Ok(worms)
    }

    fn on_update(&mut self, app: &mut Apparatus) {
        if app.was_key_released(Key::M) {
            self.create_map();
        }

        if app.is_key_held(Key::E) && app.was_mouse_button_released(MouseButton::Left) {
            if app.is_key_held(Key::Num1) {
                explosion(
                    Point::new(
                        app.mouse_pos_x() + self.camera_pos_x,
                        app.mouse_pos_y() + self.camera_pos_y,
                    ),
                    10.0,
                    self.map_width,
                    self.map_height,
                    &mut self.map,
                    &mut self.physics_things,
                    &mut self.rng,
                );
            }

            if app.is_key_held(Key::Num2) {
                let dummy = Missile::new(
                    Point::new(
                        app.mouse_pos_x() + self.camera_pos_x,
                        app.mouse_pos_y() + self.camera_pos_y,
                    ),
                    Point::new(0.0, 0.0),
                );
                self.physics_things.push(Box::new(dummy));
            }

            if app.is_key_held(Key::Num3) {
                let x1 = app.mouse_pos_x() + self.camera_pos_x;
                let y1 = app.mouse_pos_y() + self.camera_pos_y;
                let worm = Worm::new(Point::new(x1, y1));
                let id = worm.physics_object.id;
                self.physics_things.push(Box::new(worm));
                self.object_under_control = Some(id);
                self.camera_tracking_object = Some(id);
            }

            if app.is_key_held(Key::Num9) {
                let dummy = Dummy::new(
                    app.mouse_pos_x() + self.camera_pos_x,
                    app.mouse_pos_y() + self.camera_pos_y,
                );
                self.physics_things.push(Box::new(dummy));
            }
        }

        // Map scroll.
        let map_scroll_speed = 400.0;
        let dt = app.elapsed_time().as_secs_f32();
        if app.mouse_pos_x() < 5.0 {
            self.camera_pos_x -= map_scroll_speed * dt;
        }
        if app.mouse_pos_x() > app.screen_width() as f32 - 5.0 {
            self.camera_pos_x += map_scroll_speed * dt;
        }
        if app.mouse_pos_y() < 5.0 {
            self.camera_pos_y -= map_scroll_speed * dt;
        }
        if app.mouse_pos_y() > app.screen_height() as f32 - 5.0 {
            self.camera_pos_y += map_scroll_speed * dt;
        }

        // Game state management.
        match self.game_state {
            GameState::Reset => {
                self.player_has_control = false;
                self.next_state = GameState::GenerateTerrain;
            }
            GameState::GenerateTerrain => {
                self.player_has_control = false;
                self.create_map();
                self.next_state = GameState::GeneratingTerrain;
            }
            GameState::GeneratingTerrain => {
                self.player_has_control = false;
                self.next_state = GameState::AllocateUnits;
            }
            GameState::AllocateUnits => {
                self.player_has_control = false;

                let worm = Worm::new((32.0, self.map_height as f32).into());
                self.object_under_control = Some(worm.physics_object.id);
                self.camera_tracking_object = self.object_under_control;
                self.physics_things.push(Box::new(worm));

                self.next_state = GameState::AllocatingUnits;
            }
            GameState::AllocatingUnits => {
                self.player_has_control = false;
                if self.is_game_stable {
                    self.player_action_complete = false;
                    self.next_state = GameState::StartPlay;
                }
            }
            GameState::StartPlay => {
                self.player_has_control = true;

                if self.player_action_complete {
                    self.next_state = GameState::CameraMode;
                }
            }
            GameState::CameraMode => {
                self.player_has_control = false;
                self.player_action_complete = false;

                if self.is_game_stable {
                    self.camera_tracking_object = self.object_under_control;
                    self.next_state = GameState::StartPlay;
                }
            }
        }

        // Handle user input.
        if self.player_has_control {
            if let Some(id) = self.object_under_control {
                if let Some(object_under_control) = self.get_object_mut(id) {
                    if object_under_control.physics_object().is_stable {
                        if let Some(worm) = object_under_control.as_any_mut().downcast_mut::<Worm>()
                        {
                            let p = &mut worm.physics_object;
                            if app.is_key_pressed(Key::Z) {
                                p.velocity_x = 4.0 * worm.shoot_angle.cos();
                                p.velocity_y = 8.0 * worm.shoot_angle.sin();
                                p.is_stable = false;
                            }

                            if app.is_key_held(Key::A) {
                                worm.shoot_angle += 1.0 * dt;
                                if worm.shoot_angle > -PI {
                                    worm.shoot_angle -= 2.0 * PI;
                                }
                            }

                            if app.is_key_held(Key::S) {
                                worm.shoot_angle -= 1.0 * dt;
                                if worm.shoot_angle < -PI {
                                    worm.shoot_angle += 2.0 * PI;
                                }
                            }

                            if app.is_key_pressed(Key::Space) {
                                self.is_energising = true;
                                self.energy_level = 0.0;
                                self.fire_weapon = false;
                            }

                            if app.is_key_held(Key::Space) && self.is_energising {
                                self.energy_level += 0.75 * dt;
                                if self.energy_level >= 1.0 {
                                    self.energy_level = 1.0;
                                    self.fire_weapon = true;
                                }
                            }

                            if app.was_key_released(Key::Space) {
                                if self.is_energising {
                                    self.fire_weapon = true;
                                }
                                self.is_energising = false;
                            }
                        }
                    }
                }
            }

            if let Some(id) = self.object_under_control {
                if let Some(object_under_control) = self.get_object(id) {
                    if object_under_control.physics_object().is_stable {
                        if let Some(worm) = object_under_control.as_any().downcast_ref::<Worm>() {
                            let p = &worm.physics_object;
                            if self.fire_weapon {
                                let origin_x = p.position_x;
                                let origin_y = p.position_y;

                                let velocity_x = worm.shoot_angle.cos() * 40.0 * self.energy_level;
                                let velocity_y = worm.shoot_angle.sin() * 40.0 * self.energy_level;

                                let missile = Missile::new(
                                    Point::new(origin_x, origin_y),
                                    Point::new(velocity_x, velocity_y),
                                );
                                self.camera_tracking_object = Some(missile.physics_object.id);
                                self.physics_things.push(Box::new(missile));

                                self.fire_weapon = false;
                                self.is_energising = false;
                                self.energy_level = 0.0;

                                self.player_action_complete = true;
                            }
                        }
                    }
                }
            }
        }

        let (x, y) = if let Some(id) = self.camera_tracking_object {
            if let Some(camera_tracking_object) = self.get_object(id) {
                let p = camera_tracking_object.physics_object();
                (
                    p.position_x - app.screen_width() as f32 / 2.0,
                    p.position_y - app.screen_height() as f32 / 2.0,
                )
            } else {
                (self.camera_pos_x, self.camera_pos_y)
            }
        } else {
            (self.camera_pos_x, self.camera_pos_y)
        };

        self.target_camera_pos_x = x;
        self.target_camera_pos_y = y;
        self.camera_pos_x += (self.target_camera_pos_x - self.camera_pos_x) * 5.0 * dt;
        self.camera_pos_y += (self.target_camera_pos_y - self.camera_pos_y) * 5.0 * dt;

        self.camera_pos_x = clamp(
            0.0,
            self.camera_pos_x,
            (self.map_width - app.screen_width() as u32) as f32,
        );
        self.camera_pos_y = clamp(
            0.0,
            self.camera_pos_y,
            (self.map_height - app.screen_height() as u32) as f32,
        );

        // Update physics - 10 times per 1 render cycle. How does this work?
        for _ in 0..10 {
            self.physics_things
                .iter_mut()
                .map(|p| p.physics_object_mut())
                .for_each(|p| {
                    // Apply gravity.
                    p.acceleration_y += -2.0;

                    // Update velocity => integration of acceleration wrt dt.
                    p.velocity_x += p.acceleration_x * dt;
                    p.velocity_y += p.acceleration_y * dt;

                    // Update position => integration velocity wrt dt. Potential position because might be a collision...
                    let potential_x = p.position_x + p.velocity_x * dt;
                    let potential_y = p.position_y + p.velocity_y * dt;

                    // Update acceleration after applying forces. Here we just reset to zero, setting unstable because moving.
                    p.acceleration_x = 0.0;
                    p.acceleration_y = 0.0;
                    p.is_stable = false;

                    // Check for collision with map.
                    let rotation = p.velocity_y.atan2(p.velocity_x);
                    let mut response_x = 0.0;
                    let mut response_y = 0.0;
                    let mut collision = false;

                    for r in (0..8).map(|i| rotation - PI / 2.0 + PI / 8.0 * (i as f32)) {
                        let test_x = p.radius * r.cos() + potential_x;
                        let test_y = p.radius * r.sin() + potential_y;

                        let test_x = clamp(0.0, test_x, self.map_width as f32 - 1.0);
                        let test_y = clamp(0.0, test_y, self.map_height as f32 - 1.0);

                        // Test if any points on semicircle intersect with terrain (which is represented in the map by anything other than a zero).
                        if self.map[test_y as usize * self.map_width as usize + test_x as usize]
                            != 0
                        {
                            response_x += potential_x - test_x;
                            response_y += potential_y - test_y;
                            collision = true;
                        }
                    }

                    let velocity_magnitude =
                        (p.velocity_x * p.velocity_x + p.velocity_y * p.velocity_y).sqrt();
                    let response_magnitude =
                        (response_x * response_x + response_y * response_y).sqrt();

                    if collision {
                        p.is_stable = true;

                        // Calculate reflection vector and apply friction to it.
                        let dot = p.velocity_x * (response_x / response_magnitude)
                            + p.velocity_y * (response_y / response_magnitude);
                        p.velocity_x +=
                            p.friction * (-2.0 * dot * (response_x / response_magnitude));
                        p.velocity_y +=
                            p.friction * (-2.0 * dot * (response_y / response_magnitude));

                        // Some objects will "die" after several bounces.
                        if let Some(bounces) = p.bounce_before_death {
                            let bounces_remaining = bounces - 1;
                            p.bounce_before_death = Some(bounces_remaining);
                            p.is_dead = bounces_remaining == 0;
                        }
                    } else {
                        p.position_x = potential_x;
                        p.position_y = potential_y;
                    }

                    if velocity_magnitude < 0.1 {
                        p.is_stable = true;
                    }
                });

            let mut dead = Vec::new();
            for (i, pt) in self.physics_things.iter().enumerate() {
                let p = pt.physics_object();
                if p.is_dead {
                    dead.push(i);
                }
            }
            for i in dead {
                let pt = &self.physics_things[i];
                let p = pt.physics_object();
                if let DeathAction::Explode(radius) = p.bounce_death_action() {
                    explosion(
                        Point::new(p.position_x, p.position_y),
                        *radius,
                        self.map_width,
                        self.map_height,
                        &mut self.map,
                        &mut self.physics_things,
                        &mut self.rng,
                    );
                    self.camera_tracking_object = None;
                }
            }

            self.physics_things.retain(|p| !p.physics_object().is_dead);
        }

        // Draw landscape.
        for x in 0..app.screen_width() {
            for y in 0..app.screen_height() {
                match self.map[(y + self.camera_pos_y as usize) * self.map_width as usize
                    + (x + self.camera_pos_x as usize)]
                {
                    0 => app.draw(x as f32, y as f32, SKY),
                    1 => app.draw(x as f32, y as f32, LAND),
                    _ => unreachable!("Tried to draw an unknown pixel type"),
                }
            }
        }

        for p in &self.physics_things {
            p.draw(app, self.camera_pos_x, self.camera_pos_y);

            if let Some(id) = self.object_under_control {
                if p.physics_object().id == id {
                    if let Some(worm) = p.as_any().downcast_ref::<Worm>() {
                        let po = &worm.physics_object;
                        let center_x =
                            po.position_x + 8.0 * worm.shoot_angle.cos() - self.camera_pos_x;
                        let center_y =
                            po.position_y + 8.0 * worm.shoot_angle.sin() - self.camera_pos_y;

                        // Direction cursor.
                        app.draw(center_x, center_y, color::css::BLACK);
                        app.draw(center_x + 1.0, center_y, color::css::BLACK);
                        app.draw(center_x - 1.0, center_y, color::css::BLACK);
                        app.draw(center_x, center_y - 1.0, color::css::BLACK);
                        app.draw(center_x, center_y + 1.0, color::css::BLACK);

                        // Weapon energising energy level.
                        if self.is_energising {
                            for i in 0..=(10.0 * self.energy_level) as u32 {
                                app.draw(
                                    po.position_x - 5.0 + i as f32 - self.camera_pos_x,
                                    po.position_y + 12.0 - self.camera_pos_y,
                                    color::css::GREEN,
                                );
                                app.draw(
                                    po.position_x - 5.0 + i as f32 - self.camera_pos_x,
                                    po.position_y + 11.0 - self.camera_pos_y,
                                    color::css::RED,
                                );
                            }
                        }
                    }
                }
            }
        }

        self.is_game_stable = true;
        for pt in &self.physics_things {
            let p = pt.physics_object();
            if !p.is_stable {
                self.is_game_stable = false;
                break;
            }
        }

        if self.is_game_stable {
            app.draw_filled_rectangle(
                2.0,
                app.screen_height() as f32 - 8.0,
                6.0,
                6.0,
                color::css::RED,
            );
        }

        self.game_state = self.next_state;
    }
}

fn main() -> Result<()> {
    let settings = ApparatusSettings::default()
        .with_screen_size(256, 160)
        .with_pixel_size(6, 6);
    let engine = Apparatus::new("Worms", settings)?;
    engine.run::<Worms>()?;

    Ok(())
}

fn generate_noise_seed(output_size: u32, noise_seed: &mut Vec<f32>, rng: &mut ThreadRng) {
    unsafe { noise_seed.set_len(output_size as usize) };
    for i in noise_seed.iter_mut() {
        *i = rng.gen_range(0.0..=1.0);
    }
}

fn generate_perlin_noise_1d(
    count: u32,
    octaves: u32,
    bias: f32,
    seed: &[f32],
    output: &mut Vec<f32>,
) {
    let (count, octaves) = (count as usize, octaves as usize);
    let bias = 1.0 / bias;

    for (i, x) in output.iter_mut().enumerate() {
        let mut noise = 0.0;
        let mut scale = 1.0;
        let mut scale_accumulator = 0.0;

        for octave in 0..octaves {
            let pitch = count >> octave;
            let sample_1 = (i / pitch) * pitch;
            let sample_2 = (sample_1 + pitch) % count;

            let blend = (i - sample_1) as f32 / pitch as f32;
            let sample = lerp(seed[sample_2], seed[sample_1], blend);
            noise += sample * scale;
            scale_accumulator += scale;
            scale *= bias;
        }

        *x = noise / scale_accumulator;
    }
}

fn explosion(
    position: Point,
    radius: f32,
    map_width: u32,
    map_height: u32,
    map: &mut Vec<u8>,
    physics_things: &mut Vec<Box<dyn Physics>>,
    rng: &mut ThreadRng,
) {
    // Form a crater.
    fn bresenham_circle(
        x: i32,
        y: i32,
        radius: i32,
        map_width: u32,
        map_height: u32,
        map: &mut Vec<u8>,
    ) {
        let mut x0 = 0;
        let mut y0 = radius;
        let mut d = 3 - 2 * radius;

        fn draw_line(
            x0: f32,
            y0: f32,
            x1: f32,
            y1: f32,
            width: u32,
            height: u32,
            map: &mut Vec<u8>,
        ) {
            let x0 = (clamp(0.0, x0.floor(), width as f32) + 0.5) as u32;
            let y0 = (clamp(0.0, y0.floor(), height as f32) + 0.5) as u32;
            let x1 = (clamp(0.0, x1.floor(), width as f32) + 0.5) as u32;
            let y1 = (clamp(0.0, y1.floor(), height as f32) + 0.5) as u32;

            let line = BresenhamLine::new(x0, y0, x1, y1);
            for (x, y) in line {
                map[y as usize * width as usize + x as usize] = 0;
            }
        }

        while y0 >= x0 {
            draw_line(
                (x - x0) as f32,
                (y - y0) as f32,
                (x + x0) as f32,
                (y - y0) as f32,
                map_width,
                map_height,
                map,
            );
            draw_line(
                (x - y0) as f32,
                (y - x0) as f32,
                (x + y0) as f32,
                (y - x0) as f32,
                map_width,
                map_height,
                map,
            );
            draw_line(
                (x - y0) as f32,
                (y + x0) as f32,
                (x + y0) as f32,
                (y + x0) as f32,
                map_width,
                map_height,
                map,
            );
            draw_line(
                (x - x0) as f32,
                (y + y0) as f32,
                (x + x0) as f32,
                (y + y0) as f32,
                map_width,
                map_height,
                map,
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

    bresenham_circle(
        position.x() as i32,
        position.y() as i32,
        radius as i32,
        map_width,
        map_height,
        map,
    );

    // Shockwave.
    physics_things.iter_mut().for_each(|p| {
        let mut p = p.physics_object_mut();
        let dx = p.position_x - position.x();
        let dy = p.position_y - position.y();
        let mut distance = (dx * dx + dy * dy).sqrt(); // Or we could compare to radius squared and save the division.
        if distance < 0.0001 {
            distance = 0.0001;
        }

        if distance <= radius {
            p.velocity_x = (dx / distance) * radius;
            p.velocity_y = (dy / distance) * radius;
            p.is_stable = false;
        }
    });

    // Launch debris.
    for _ in 0..radius as u32 {
        let debris = Debris::new(position.x(), position.y(), rng);
        physics_things.push(Box::new(debris));
    }
}

#[derive(Debug)]
enum DeathAction {
    None,
    Explode(f32),
}

#[derive(Debug)]
struct PhysicsObject {
    id: u128,

    position_x: f32,     // or just `x`?
    position_y: f32,     // or just `y`?
    velocity_x: f32,     // or `dx` for 1st differential of x?
    velocity_y: f32,     // or `dy` for 1st differential of y?
    acceleration_x: f32, // or `ddx` for 2nd differential of x?
    acceleration_y: f32, // or `ddy` for 2nd differential of y?
    friction: f32,

    radius: f32,
    is_stable: bool,

    bounce_before_death: Option<u32>,
    bounce_death_action: DeathAction,
    is_dead: bool,
}

impl PhysicsObject {
    fn new(x: f32, y: f32) -> Self {
        let id = get_physics_id();
        Self {
            id,
            position_x: x,
            position_y: y,
            velocity_x: 0.0,
            velocity_y: 0.0,
            acceleration_x: 0.0,
            acceleration_y: 0.0,
            friction: 0.8,
            radius: 4.0,
            is_stable: false,
            bounce_before_death: None,
            bounce_death_action: DeathAction::None,
            is_dead: false,
        }
    }

    fn bounce_death_action(&self) -> &DeathAction {
        &self.bounce_death_action
    }
}

trait Physics {
    fn physics_object(&self) -> &PhysicsObject;

    fn physics_object_mut(&mut self) -> &mut PhysicsObject;

    fn draw(&self, app: &mut Apparatus, camera_offset_x: f32, camera_offset_y: f32);

    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct Dummy {
    physics_object: PhysicsObject,
}

impl Dummy {
    fn new(x: f32, y: f32) -> Self {
        let physics_object = PhysicsObject::new(x, y);

        Self { physics_object }
    }
}

impl Physics for Dummy {
    fn physics_object(&self) -> &PhysicsObject {
        &self.physics_object
    }

    fn physics_object_mut(&mut self) -> &mut PhysicsObject {
        &mut self.physics_object
    }

    fn draw(&self, app: &mut Apparatus, camera_offset_x: f32, camera_offset_y: f32) {
        let rotation = self
            .physics_object
            .velocity_y
            .atan2(self.physics_object.velocity_x);

        let x = self.physics_object.position_x;
        let y = self.physics_object.position_y;
        let radius = self.physics_object.radius;
        let direction_x = x + (radius * rotation.cos() - rotation.sin());
        let direction_y = y + (rotation.cos() + radius * rotation.sin());

        app.draw_line(
            self.physics_object.position_x - camera_offset_x,
            self.physics_object.position_y - camera_offset_y,
            direction_x - camera_offset_x,
            direction_y - camera_offset_y,
            color::css::WHITE,
        );

        app.draw_wireframe_circle(
            self.physics_object.position_x - camera_offset_x,
            self.physics_object.position_y - camera_offset_y,
            radius,
            color::css::WHITE,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Debris {
    physics_object: PhysicsObject,
}

impl Debris {
    const MODEL: [Point; 4] = [
        Point::new(0.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(1.0, 1.0),
        Point::new(1.0, 0.0),
    ];

    fn new(x: f32, y: f32, rng: &mut ThreadRng) -> Self {
        let mut physics_object = PhysicsObject::new(x, y);
        physics_object.velocity_x = 10.0 * (rng.gen_range(0.0..=1.0) * 2.0 * PI).cos();
        physics_object.velocity_y = 10.0 * (rng.gen_range(0.0..=1.0) * 2.0 * PI).sin();
        physics_object.radius = 1.0;
        physics_object.friction = 0.8;
        physics_object.bounce_before_death = Some(5);

        Self { physics_object }
    }
}

impl Physics for Debris {
    fn physics_object(&self) -> &PhysicsObject {
        &self.physics_object
    }

    fn physics_object_mut(&mut self) -> &mut PhysicsObject {
        &mut self.physics_object
    }

    fn draw(&self, app: &mut Apparatus, camera_offset_x: f32, camera_offset_y: f32) {
        let rotation = self
            .physics_object
            .velocity_y
            .atan2(self.physics_object.velocity_x);

        app.draw_wireframe_model(
            (
                self.physics_object.position_x - camera_offset_x,
                self.physics_object.position_y - camera_offset_y,
            )
                .into(),
            rotation,
            self.physics_object.radius,
            &Self::MODEL,
            LAND,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Missile {
    physics_object: PhysicsObject,
}

impl Missile {
    const MODEL: [Point; 12] = [
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(2.0, 1.0),
        Point::new(2.5, 0.0),
        Point::new(2.0, -1.0),
        Point::new(1.0, -1.0),
        Point::new(0.0, 0.0),
        Point::new(-1.0, -1.0),
        Point::new(-2.5, -1.0),
        Point::new(-2.0, 0.0),
        Point::new(-2.5, 1.0),
        Point::new(-1.0, 1.0),
    ];

    fn new(position: Point, velocity: Point) -> Self {
        let mut physics_object = PhysicsObject::new(position.x(), position.y());
        physics_object.velocity_x = velocity.x();
        physics_object.velocity_y = velocity.y();
        physics_object.radius = 2.5;
        physics_object.friction = 0.5;
        physics_object.bounce_before_death = Some(1);
        physics_object.bounce_death_action = DeathAction::Explode(20.0); // Big explosion!

        Self { physics_object }
    }
}

impl Physics for Missile {
    fn physics_object(&self) -> &PhysicsObject {
        &self.physics_object
    }

    fn physics_object_mut(&mut self) -> &mut PhysicsObject {
        &mut self.physics_object
    }

    fn draw(&self, app: &mut Apparatus, camera_offset_x: f32, camera_offset_y: f32) {
        // Negative y because we flipped the y axis when we draw.
        let rotation = (-self.physics_object.velocity_y).atan2(self.physics_object.velocity_x);

        app.draw_wireframe_model(
            (
                self.physics_object.position_x - camera_offset_x,
                self.physics_object.position_y - camera_offset_y,
            )
                .into(),
            rotation,
            self.physics_object.radius * 0.4,
            &Self::MODEL,
            color::css::YELLOW,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Worm {
    sprite: Sprite,
    physics_object: PhysicsObject,
    shoot_angle: f32,
}

impl Worm {
    fn new(position: Point) -> Self {
        let sprite_bytes = include_bytes!("assets/worm.png");
        let sprite = Sprite::from_bytes(sprite_bytes);

        let mut physics_object = PhysicsObject::new(position.x(), position.y());
        physics_object.velocity_x = 0.0;
        physics_object.velocity_y = 0.0;
        physics_object.radius = 3.5;
        physics_object.friction = 0.2;
        physics_object.bounce_before_death = None;

        let shooting_angle = 0.0;

        Self {
            sprite,
            physics_object,
            shoot_angle: shooting_angle,
        }
    }
}

impl Physics for Worm {
    fn physics_object(&self) -> &PhysicsObject {
        &self.physics_object
    }

    fn physics_object_mut(&mut self) -> &mut PhysicsObject {
        &mut self.physics_object
    }

    fn draw(&self, app: &mut Apparatus, camera_offset_x: f32, camera_offset_y: f32) {
        app.draw_sprite(
            self.physics_object.position_x - camera_offset_x - self.physics_object.radius,
            self.physics_object.position_y - camera_offset_y - self.physics_object.radius - 1.0,
            &self.sprite,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
