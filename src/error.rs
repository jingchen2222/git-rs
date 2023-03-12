use thiserror::Error;
#[derive(Debug, Error)]
pub enum GitError {
    #[error("init fatal: {0}")]
    GitInitError(String),
}
