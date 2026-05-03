use crate::domain::{
    errors::AuthzError,
    types::{
        decision::Decision,
        evaluation::{
            AccessEvaluationRequest, AccessEvaluationsRequest, AccessEvaluationsResponse,
        },
    },
};

/// Domain port for a Policy Decision Point (PDP).
///
/// Mirrors the [OpenID AuthZen 1.0] Access Evaluation API. Implementations
/// can be in-process (a local policy engine) or remote (an HTTP client
/// targeting an AuthZen-compliant PDP) without affecting callers.
///
/// Implementations must be safe to share across tasks (`Send + Sync`) and
/// must never panic on a denial — denials are values of type
/// [`Decision`], not errors. [`AuthzError`] is reserved for operational
/// failures of the PDP itself.
///
/// [OpenID AuthZen 1.0]: https://openid.net/specs/authorization-api-1_0.html
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait Authorizer: Send + Sync {
    /// Evaluate a single access request. Returns a [`Decision`] for both
    /// allow and deny outcomes; [`AuthzError`] only signals that the PDP
    /// itself failed to produce a decision.
    fn evaluate(
        &self,
        request: &AccessEvaluationRequest,
    ) -> impl Future<Output = Result<Decision, AuthzError>> + Send;

    /// Evaluate a batch of requests sharing common defaults (boxed
    /// evaluations). Returns one decision per entry, in input order.
    fn evaluations(
        &self,
        request: &AccessEvaluationsRequest,
    ) -> impl Future<Output = Result<AccessEvaluationsResponse, AuthzError>> + Send;
}

/// Forwards to the inner authorizer so a service generic over
/// `A: Authorizer` can be parameterized with `&LocalPolicyEngine` (the
/// pattern used by the [`#[transactional]`](mestier_macros::transactional)
/// macro when it injects `&self.authz`).
impl<T: Authorizer + ?Sized> Authorizer for &T {
    fn evaluate(
        &self,
        request: &AccessEvaluationRequest,
    ) -> impl Future<Output = Result<Decision, AuthzError>> + Send {
        T::evaluate(*self, request)
    }

    fn evaluations(
        &self,
        request: &AccessEvaluationsRequest,
    ) -> impl Future<Output = Result<AccessEvaluationsResponse, AuthzError>> + Send {
        T::evaluations(*self, request)
    }
}
