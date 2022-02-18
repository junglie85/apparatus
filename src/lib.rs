pub use crate::engine::apparatus::{Engine, Settings};
pub use crate::engine::game::Game;
pub use crate::engine::input::*;
pub use crate::engine::renderer::Renderer;
pub use crate::engine::sprite::*;
use crate::errors::ApparatusError;
pub use crate::maths::*;

pub mod color;
mod engine;
pub mod errors;
pub mod font;
mod maths;
mod platform;
mod renderer;
mod util;
