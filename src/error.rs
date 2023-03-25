use thiserror::Error;
#[derive(Debug, Error)]
pub enum GitError {
    #[error("init : {0}")]
    GitInitError(String),
    #[error("head : {0}")]
    InitHeadError(String),
    #[error("add : {0}")]
    StagedAddError(String),
    #[error("rm : {0}")]
    StagedRemoveError(String),
    #[error("No reason to remove the file")]
    StagedRemoveNoReasonError,
    #[error("commit : {0}")]
    CommitError(String),
    #[error("File {0} does not exist.")]
    FileNotExistError(String),
    #[error("file op fatal: {0}")]
    FileOpError(String),
    #[error("serialized/deserialized fatal: {0}")]
    SerdeOpError(String),
    #[error("crypto error: {0}")]
    CryptoError(String),
}
