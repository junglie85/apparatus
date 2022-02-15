use crate::engine::input::*;

pub struct PlatformInput {
    pub keys: Vec<Key>,
}

impl PlatformInput {
    pub fn new() -> Self {
        let keys = Vec::new();

        Self { keys }
    }
}

impl Input for PlatformInput {
    fn is_key_held(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Input;

    #[test]
    fn key_not_pressed_is_not_held() {
        let input = PlatformInput::new();

        assert!(!input.is_key_held(Key::Up));
    }

    #[test]
    fn key_pressed_is_held() {
        let mut input = PlatformInput::new();
        input.keys.push(Key::Up);

        assert!(input.is_key_held(Key::Up));
    }
}
