pub use crate::engine::apparatus::{Apparatus, ApparatusSettings};
pub use crate::engine::game::Game;
pub use crate::engine::key::*;
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
