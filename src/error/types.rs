use thiserror::Error;

pub type GameResult<T> = Result<T, GameError>;

#[derive(Error, Debug)]
pub enum GameError {
    #[allow(dead_code)]
    #[error("position out of bounds: ({0}, {1})")]
    PositionOutOfBounds(u16, u16),

    #[allow(dead_code)]
    #[error("tile at ({0}, {1}) is not passable: {2}")]
    TileNotPassable(u16, u16, String),

    #[error("unknown command: '{0}'. type 'help' for available commands")]
    UnknownCommand(String),

    #[error("empty input")]
    EmptyInput,

    #[error("squad '{0}' not found")]
    SquadNotFound(String),

    #[error("save error: {0}")]
    SaveError(String),

    #[error("load error: {0}")]
    LoadError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
