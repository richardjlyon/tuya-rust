#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("I/O error")]
    ConfigLoad,
    #[error("auth error")]
    Authorisation,
    #[error("not found")]
    NotFound,
    #[error("network error")]
    NetworkError,
    #[error("could not parse {0}: {1}")]
    ParseError(serde_path_to_error::Path, serde_json::Error),
    #[error("read error")]
    ReadError,
    #[error("other error")]
    Other,
}
