use thiserror::Error;

/// Errors surfaced by an [`Iam`](crate::Iam) implementation.
///
/// Adapters translate provider-specific failures (HTTP status codes,
/// transport errors, parsing errors) into these variants. Use-cases then
/// translate further into `CoreError` so the business layer never depends
/// on a specific IAM.
#[derive(Debug, Error)]
pub enum IamError {
    /// The requested entity does not exist in the IAM.
    #[error("IAM entity not found")]
    NotFound,

    /// A uniqueness constraint enforced by the IAM was violated
    /// (e.g. duplicate username, e-mail, role name).
    #[error("IAM conflict: {0}")]
    Conflict(String),

    /// The service-account credentials used by Mestier were rejected by the IAM.
    /// Indicates a misconfiguration or a revoked client — never a user-facing
    /// 401.
    #[error("IAM authentication failed")]
    Unauthorized,

    /// The IAM is reachable but refused the operation for authorization
    /// reasons (insufficient scope on the service account).
    #[error("IAM forbade the operation")]
    Forbidden,

    /// The payload was rejected by the IAM (validation, schema, etc.).
    #[error("IAM rejected the payload: {0}")]
    InvalidInput(String),

    /// The IAM is unreachable or returned a 5xx — typically retryable.
    #[error("IAM unavailable: {0}")]
    Unavailable(String),

    /// Any other failure (deserialization, unexpected response shape, ...).
    #[error("IAM internal error: {0}")]
    Internal(String),
}
