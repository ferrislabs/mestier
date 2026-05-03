use thiserror::Error;

use crate::domain::types::context::Context;

/// Errors surfaced by an [`Authorizer`](crate::Authorizer) implementation.
///
/// `Denied` carries the optional AuthZen `context` returned by the PDP
/// alongside `decision: false` (reasons, obligations, ...). All other
/// variants represent operational failures of the PDP itself, not a
/// negative authorization decision.
#[derive(Debug, Error)]
pub enum AuthzError {
    /// The PDP evaluated the request and returned `decision: false`. The
    /// optional `context` mirrors the AuthZen response and may carry
    /// localized reasons, obligations, etc.
    #[error("authorization denied")]
    Denied(Option<Context>),

    /// The request payload was rejected by the PDP (invalid subject /
    /// action / resource shape, unknown action, ...).
    #[error("authorization request invalid: {0}")]
    InvalidRequest(String),

    /// The PDP is reachable but refused to answer (auth failure on the
    /// PDP itself, quota, ...). Distinct from [`AuthzError::Denied`].
    #[error("authorization service refused: {0}")]
    ServiceRefused(String),

    /// The PDP is unreachable or returned a 5xx — typically retryable.
    #[error("authorization service unavailable: {0}")]
    Unavailable(String),

    /// Any other failure (deserialization, unexpected response shape, ...).
    #[error("authorization internal error: {0}")]
    Internal(String),
}
