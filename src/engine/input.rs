#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
}

pub trait Input {
    fn is_key_held(&self, key: Key) -> bool;
}
