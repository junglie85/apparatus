use crate::engine::apparatus::Apparatus;
use crate::errors::ApparatusError;

pub trait Game<Game = Self> {
    /// Called once, after the engine has initialised.
    fn on_create(screen_width: usize, screen_height: usize) -> Result<Game, ApparatusError>;

    /// Called once per frame.
    fn on_update(&mut self, _app: &mut Apparatus) {}
}
