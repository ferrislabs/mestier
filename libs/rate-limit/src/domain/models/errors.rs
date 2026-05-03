use thiserror::Error;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("backend unavailable: {message}")]
    BackendUnavailable { message: String },

    #[error("internal: {message}")]
    Internal { message: String },
}
