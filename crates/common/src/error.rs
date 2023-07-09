use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Bad config: {0}")]
    BadConfig(String),
    #[error("Connecting to database: {0}")]
    ConnectingToDatabase(String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
