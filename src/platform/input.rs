use crate::engine::key::*;
use crate::platform::window::Window;
use std::collections::HashMap;

#[derive(Default, Debug)]
struct KeyState {
    is_down: bool,
    was_down: bool,
}

impl KeyState {
    fn new(is_down: bool, was_down: bool) -> Self {
        Self { is_down, was_down }
    }
}

pub struct Input {
    keys: HashMap<Key, KeyState>,
}

impl Input {
    pub fn new() -> Self {
        let keys = HashMap::new();

        Self { keys }
    }

    pub fn process_input(&mut self, window: &Window) {
        let mut keys = HashMap::new();

        fn get_key_state(
            key: Key,
            window: &Window,
            previous_keys: &HashMap<Key, KeyState>,
        ) -> KeyState {
            let native_key = Into::<NativeKey>::into(key).0;
            let is_down = window.native_window().is_key_down(native_key);
            let was_down = match previous_keys.get(&key) {
                Some(key) => key.is_down,
                None => false,
            };

            KeyState::new(is_down, was_down)
        }

        let key_state = get_key_state(Key::Num1, window, &self.keys);
        keys.insert(Key::Num1, key_state);

        let key_state = get_key_state(Key::Num2, window, &self.keys);
        keys.insert(Key::Num2, key_state);

        let key_state = get_key_state(Key::Num3, window, &self.keys);
        keys.insert(Key::Num3, key_state);

        let key_state = get_key_state(Key::A, window, &self.keys);
        keys.insert(Key::A, key_state);

        let key_state = get_key_state(Key::Q, window, &self.keys);
        keys.insert(Key::Q, key_state);

        let key_state = get_key_state(Key::Z, window, &self.keys);
        keys.insert(Key::Z, key_state);

        let key_state = get_key_state(Key::Up, window, &self.keys);
        keys.insert(Key::Up, key_state);

        let key_state = get_key_state(Key::Down, window, &self.keys);
        keys.insert(Key::Down, key_state);

        let key_state = get_key_state(Key::Left, window, &self.keys);
        keys.insert(Key::Left, key_state);

        let key_state = get_key_state(Key::Right, window, &self.keys);
        keys.insert(Key::Right, key_state);

        let key_state = get_key_state(Key::Space, window, &self.keys);
        keys.insert(Key::Space, key_state);

        self.keys = keys;
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
}

struct NativeKey(minifb::Key);

impl From<Key> for NativeKey {
    fn from(key: Key) -> Self {
        match key {
            Key::Num1 => NativeKey(minifb::Key::Key1),
            Key::Num2 => NativeKey(minifb::Key::Key2),
            Key::Num3 => NativeKey(minifb::Key::Key3),
            Key::A => NativeKey(minifb::Key::A),
            Key::Q => NativeKey(minifb::Key::Q),
            Key::Z => NativeKey(minifb::Key::Z),
            Key::Up => NativeKey(minifb::Key::Up),
            Key::Down => NativeKey(minifb::Key::Down),
            Key::Left => NativeKey(minifb::Key::Left),
            Key::Right => NativeKey(minifb::Key::Right),
            Key::Space => NativeKey(minifb::Key::Space),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_not_pressed_is_not_held() {
        let input = Input::new();

        assert!(!input.is_key_held(Key::Up));
    }

    #[test]
    fn key_pressed_is_held() {
        let mut input = Input::new();
        let key_state = KeyState {
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
        let key_state = KeyState {
            is_down: true,
            was_down: true,
        };
        input.keys.insert(Key::Space, key_state);

        assert!(!input.was_key_released(Key::Space));
    }

    #[test]
    fn key_previously_held_is_released() {
        let mut input = Input::new();
        let key_state = KeyState {
            is_down: false,
            was_down: true,
        };
        input.keys.insert(Key::Space, key_state);

        assert!(input.was_key_released(Key::Space));
    }
}
