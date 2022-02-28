use minifb::MouseMode;
use std::collections::HashMap;

use crate::engine::key::Key;
use crate::engine::mouse::MouseButton;
use crate::platform::window::Window;

#[derive(Default, Debug)]
struct MouseState {
    x: f32,
    y: f32,
    buttons: HashMap<MouseButton, ButtonState>,
}

#[derive(Default, Debug)]
struct ButtonState {
    is_down: bool,
    was_down: bool,
}

impl ButtonState {
    fn new(is_down: bool, was_down: bool) -> Self {
        Self { is_down, was_down }
    }
}

pub struct Input {
    keys: HashMap<Key, ButtonState>,
    mouse: MouseState,
}

impl Input {
    pub fn new() -> Self {
        let keys = HashMap::new();
        let mouse = MouseState::default();

        Self { mouse, keys }
    }

    pub fn process_input(&mut self, window: &Window) {
        self.keys = process_keys(window, &self.keys);
        self.mouse = process_mouse(window, &self.mouse.buttons);
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        match self.keys.get(&key) {
            Some(key) => key.is_down && !key.was_down,
            None => false,
        }
    }

    pub fn is_key_held(&self, key: Key) -> bool {
        match self.keys.get(&key) {
            Some(key) => key.is_down && key.was_down,
            None => false,
        }
    }

    pub fn was_key_released(&self, key: Key) -> bool {
        match self.keys.get(&key) {
            Some(key) => !key.is_down && key.was_down,
            None => false,
        }
    }

    pub fn mouse_pos_x(&self) -> f32 {
        self.mouse.x
    }

    pub fn mouse_pos_y(&self) -> f32 {
        self.mouse.y
    }

    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        match self.mouse.buttons.get(&button) {
            Some(button) => button.is_down && button.was_down,
            None => false,
        }
    }

    pub fn was_mouse_button_released(&self, button: MouseButton) -> bool {
        match self.mouse.buttons.get(&button) {
            Some(button) => !button.is_down && button.was_down,
            None => false,
        }
    }
}

fn process_keys(
    window: &Window,
    previous_keys: &HashMap<Key, ButtonState>,
) -> HashMap<Key, ButtonState> {
    let mut keys = HashMap::new();

    fn get_key_state(
        key: Key,
        window: &Window,
        previous_keys: &HashMap<Key, ButtonState>,
    ) -> ButtonState {
        let native_key = Into::<NativeKey>::into(key).0;
        let is_down = window.native_window().is_key_down(native_key);
        let was_down = match previous_keys.get(&key) {
            Some(key) => key.is_down,
            None => false,
        };

        ButtonState::new(is_down, was_down)
    }

    let key_state = get_key_state(Key::Num1, window, previous_keys);
    keys.insert(Key::Num1, key_state);

    let key_state = get_key_state(Key::Num2, window, previous_keys);
    keys.insert(Key::Num2, key_state);

    let key_state = get_key_state(Key::Num3, window, previous_keys);
    keys.insert(Key::Num3, key_state);

    let key_state = get_key_state(Key::Num4, window, previous_keys);
    keys.insert(Key::Num4, key_state);

    let key_state = get_key_state(Key::Num5, window, previous_keys);
    keys.insert(Key::Num5, key_state);

    let key_state = get_key_state(Key::Num6, window, previous_keys);
    keys.insert(Key::Num6, key_state);

    let key_state = get_key_state(Key::Num7, window, previous_keys);
    keys.insert(Key::Num7, key_state);

    let key_state = get_key_state(Key::Num8, window, previous_keys);
    keys.insert(Key::Num8, key_state);

    let key_state = get_key_state(Key::Num9, window, previous_keys);
    keys.insert(Key::Num9, key_state);

    let key_state = get_key_state(Key::Num0, window, previous_keys);
    keys.insert(Key::Num0, key_state);

    let key_state = get_key_state(Key::A, window, previous_keys);
    keys.insert(Key::A, key_state);

    let key_state = get_key_state(Key::B, window, previous_keys);
    keys.insert(Key::B, key_state);

    let key_state = get_key_state(Key::C, window, previous_keys);
    keys.insert(Key::C, key_state);

    let key_state = get_key_state(Key::D, window, previous_keys);
    keys.insert(Key::D, key_state);

    let key_state = get_key_state(Key::E, window, previous_keys);
    keys.insert(Key::E, key_state);

    let key_state = get_key_state(Key::F, window, previous_keys);
    keys.insert(Key::F, key_state);

    let key_state = get_key_state(Key::G, window, previous_keys);
    keys.insert(Key::G, key_state);

    let key_state = get_key_state(Key::H, window, previous_keys);
    keys.insert(Key::H, key_state);

    let key_state = get_key_state(Key::I, window, previous_keys);
    keys.insert(Key::I, key_state);

    let key_state = get_key_state(Key::J, window, previous_keys);
    keys.insert(Key::J, key_state);

    let key_state = get_key_state(Key::K, window, previous_keys);
    keys.insert(Key::K, key_state);

    let key_state = get_key_state(Key::L, window, previous_keys);
    keys.insert(Key::L, key_state);

    let key_state = get_key_state(Key::M, window, previous_keys);
    keys.insert(Key::M, key_state);

    let key_state = get_key_state(Key::N, window, previous_keys);
    keys.insert(Key::N, key_state);

    let key_state = get_key_state(Key::O, window, previous_keys);
    keys.insert(Key::O, key_state);

    let key_state = get_key_state(Key::P, window, previous_keys);
    keys.insert(Key::P, key_state);

    let key_state = get_key_state(Key::Q, window, previous_keys);
    keys.insert(Key::Q, key_state);

    let key_state = get_key_state(Key::R, window, previous_keys);
    keys.insert(Key::R, key_state);

    let key_state = get_key_state(Key::S, window, previous_keys);
    keys.insert(Key::S, key_state);

    let key_state = get_key_state(Key::T, window, previous_keys);
    keys.insert(Key::T, key_state);

    let key_state = get_key_state(Key::U, window, previous_keys);
    keys.insert(Key::U, key_state);

    let key_state = get_key_state(Key::V, window, previous_keys);
    keys.insert(Key::V, key_state);

    let key_state = get_key_state(Key::W, window, previous_keys);
    keys.insert(Key::W, key_state);

    let key_state = get_key_state(Key::X, window, previous_keys);
    keys.insert(Key::X, key_state);

    let key_state = get_key_state(Key::Y, window, previous_keys);
    keys.insert(Key::Y, key_state);

    let key_state = get_key_state(Key::Z, window, previous_keys);
    keys.insert(Key::Z, key_state);

    let key_state = get_key_state(Key::Up, window, previous_keys);
    keys.insert(Key::Up, key_state);

    let key_state = get_key_state(Key::Down, window, previous_keys);
    keys.insert(Key::Down, key_state);

    let key_state = get_key_state(Key::Left, window, previous_keys);
    keys.insert(Key::Left, key_state);

    let key_state = get_key_state(Key::Right, window, previous_keys);
    keys.insert(Key::Right, key_state);

    let key_state = get_key_state(Key::Space, window, previous_keys);
    keys.insert(Key::Space, key_state);

    keys
}

fn process_mouse(
    window: &Window,
    previous_buttons: &HashMap<MouseButton, ButtonState>,
) -> MouseState {
    let mut mouse = MouseState::default();

    let (mouse_pos_x, mouse_pos_y) = window
        .native_window()
        .get_mouse_pos(MouseMode::Pass)
        .expect("MouseMode::Pass always returns a position");

    // (0, 0) is bottom left.
    let (_, window_height) = window.native_window().get_size();
    mouse.x = mouse_pos_x;
    mouse.y = window_height as f32 - mouse_pos_y;

    fn get_mouse_button_state(
        button: MouseButton,
        window: &Window,
        previous_buttons: &HashMap<MouseButton, ButtonState>,
    ) -> ButtonState {
        let native_button = Into::<NativeMouseButton>::into(button).0;
        let is_down = window.native_window().get_mouse_down(native_button);
        let was_down = match previous_buttons.get(&button) {
            Some(button) => button.is_down,
            None => false,
        };

        ButtonState::new(is_down, was_down)
    }

    let button_state = get_mouse_button_state(MouseButton::Left, window, previous_buttons);
    mouse.buttons.insert(MouseButton::Left, button_state);

    let button_state = get_mouse_button_state(MouseButton::Middle, window, previous_buttons);
    mouse.buttons.insert(MouseButton::Middle, button_state);

    let button_state = get_mouse_button_state(MouseButton::Right, window, previous_buttons);
    mouse.buttons.insert(MouseButton::Right, button_state);

    mouse
}

struct NativeKey(minifb::Key);

impl From<Key> for NativeKey {
    fn from(key: Key) -> Self {
        match key {
            Key::Num1 => NativeKey(minifb::Key::Key1),
            Key::Num2 => NativeKey(minifb::Key::Key2),
            Key::Num3 => NativeKey(minifb::Key::Key3),
            Key::Num4 => NativeKey(minifb::Key::Key4),
            Key::Num5 => NativeKey(minifb::Key::Key5),
            Key::Num6 => NativeKey(minifb::Key::Key6),
            Key::Num7 => NativeKey(minifb::Key::Key7),
            Key::Num8 => NativeKey(minifb::Key::Key8),
            Key::Num9 => NativeKey(minifb::Key::Key9),
            Key::Num0 => NativeKey(minifb::Key::Key0),
            Key::A => NativeKey(minifb::Key::A),
            Key::B => NativeKey(minifb::Key::B),
            Key::C => NativeKey(minifb::Key::C),
            Key::D => NativeKey(minifb::Key::D),
            Key::E => NativeKey(minifb::Key::E),
            Key::F => NativeKey(minifb::Key::F),
            Key::G => NativeKey(minifb::Key::G),
            Key::H => NativeKey(minifb::Key::H),
            Key::I => NativeKey(minifb::Key::I),
            Key::J => NativeKey(minifb::Key::J),
            Key::K => NativeKey(minifb::Key::K),
            Key::L => NativeKey(minifb::Key::L),
            Key::M => NativeKey(minifb::Key::M),
            Key::N => NativeKey(minifb::Key::N),
            Key::O => NativeKey(minifb::Key::O),
            Key::P => NativeKey(minifb::Key::P),
            Key::Q => NativeKey(minifb::Key::Q),
            Key::R => NativeKey(minifb::Key::R),
            Key::S => NativeKey(minifb::Key::S),
            Key::T => NativeKey(minifb::Key::T),
            Key::U => NativeKey(minifb::Key::U),
            Key::V => NativeKey(minifb::Key::V),
            Key::W => NativeKey(minifb::Key::W),
            Key::X => NativeKey(minifb::Key::X),
            Key::Y => NativeKey(minifb::Key::Y),
            Key::Z => NativeKey(minifb::Key::Z),
            Key::Up => NativeKey(minifb::Key::Up),
            Key::Down => NativeKey(minifb::Key::Down),
            Key::Left => NativeKey(minifb::Key::Left),
            Key::Right => NativeKey(minifb::Key::Right),
            Key::Space => NativeKey(minifb::Key::Space),
        }
    }
}

struct NativeMouseButton(minifb::MouseButton);

impl From<MouseButton> for NativeMouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => NativeMouseButton(minifb::MouseButton::Left),
            MouseButton::Middle => NativeMouseButton(minifb::MouseButton::Middle),
            MouseButton::Right => NativeMouseButton(minifb::MouseButton::Right),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_not_pressed_is_not_pressed() {
        let input = Input::new();

        assert!(!input.is_key_pressed(Key::Up));
    }

    #[test]
    fn key_pressed_is_pressed() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: true,
            was_down: false,
        };
        input.keys.insert(Key::Up, key_state);

        assert!(input.is_key_pressed(Key::Up));
    }

    #[test]
    fn key_pressed_is_not_held() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: true,
            was_down: false,
        };
        input.keys.insert(Key::Up, key_state);

        assert!(!input.is_key_held(Key::Up));
    }

    #[test]
    fn key_pressed_is_not_released() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: true,
            was_down: false,
        };
        input.keys.insert(Key::Up, key_state);

        assert!(!input.was_key_released(Key::Up));
    }

    #[test]
    fn key_not_pressed_is_not_held() {
        let input = Input::new();

        assert!(!input.is_key_held(Key::Up));
    }

    #[test]
    fn key_pressed_is_held() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: true,
            was_down: true,
        };
        input.keys.insert(Key::Up, key_state);

        assert!(input.is_key_held(Key::Up));
    }

    #[test]
    fn key_not_pressed_is_not_released() {
        let input = Input::new();

        assert!(!input.was_key_released(Key::Space));
    }

    #[test]
    fn key_held_is_not_released() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: true,
            was_down: true,
        };
        input.keys.insert(Key::Space, key_state);

        assert!(!input.was_key_released(Key::Space));
    }

    #[test]
    fn key_previously_held_is_released() {
        let mut input = Input::new();
        let key_state = ButtonState {
            is_down: false,
            was_down: true,
        };
        input.keys.insert(Key::Space, key_state);

        assert!(input.was_key_released(Key::Space));
    }

    #[test]
    fn mouse_button_not_pressed_is_not_held() {
        let input = Input::new();

        assert!(!input.is_mouse_button_held(MouseButton::Left));
    }

    #[test]
    fn mouse_button_pressed_is_held() {
        let mut input = Input::new();
        let button_state = ButtonState {
            is_down: true,
            was_down: true,
        };
        input.mouse.buttons.insert(MouseButton::Left, button_state);

        assert!(input.is_mouse_button_held(MouseButton::Left));
    }

    #[test]
    fn mouse_button_not_pressed_is_not_released() {
        let input = Input::new();

        assert!(!input.was_mouse_button_released(MouseButton::Left));
    }

    #[test]
    fn mouse_button_held_is_not_released() {
        let mut input = Input::new();
        let button_state = ButtonState {
            is_down: true,
            was_down: true,
        };
        input.mouse.buttons.insert(MouseButton::Left, button_state);

        assert!(!input.was_mouse_button_released(MouseButton::Left));
    }

    #[test]
    fn mouse_button_previously_held_is_released() {
        let mut input = Input::new();
        let button_state = ButtonState {
            is_down: false,
            was_down: true,
        };
        input.mouse.buttons.insert(MouseButton::Left, button_state);

        assert!(input.was_mouse_button_released(MouseButton::Left));
    }
}
