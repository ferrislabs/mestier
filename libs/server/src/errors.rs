use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("internal server error: {message}")]
    Internal { message: String },

    #[error("socket resolve error: {0}")]
    SocketResolveError(String),

    #[error("socket not found: {0}")]
    SocketNotFound(String),
}
