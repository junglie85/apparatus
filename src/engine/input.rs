#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Num1,
    Num2,
    Num3,
    A,
    Q,
    Z,
    Up,
    Down,
    Left,
    Right,
    Space,
}

pub trait Input {
    fn is_key_held(&self, key: Key) -> bool;

    fn was_key_released(&self, key: Key) -> bool;
}
