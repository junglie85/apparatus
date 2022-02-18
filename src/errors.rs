use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApparatusError {
    #[error("error running game")]
    Game(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("error initialising engine")]
    Initialisation(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("logger error")]
    Logger(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("window error")]
    Window(#[source] Box<dyn std::error::Error + Send + Sync>),
}
