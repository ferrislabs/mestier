use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid token: {message}")]
    InvalidToken { message: String },

    #[error("network: {message}")]
    Network { message: String },

    #[error("key not found: {key}")]
    KeyNotFound { key: String },

    #[error("internal: {message}")]
    Internal { message: String },

    #[error("token expired")]
    Expired,
}
