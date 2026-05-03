//! Authorization domain for Mestier.
//!
//! Exposes a port trait ([`Authorizer`]) and a set of types that mirror
//! the [OpenID AuthZen 1.0] Access Evaluation API on the wire. Concrete
//! Policy Decision Points (in-process or remote) implement the port; the
//! application layer never depends on a specific engine.
//!
//! Types are kept structurally identical to AuthZen so that requests can
//! be serialized to / deserialized from a compliant PDP without translation.
//!
//! [OpenID AuthZen 1.0]: https://openid.net/specs/authorization-api-1_0.html

pub mod domain;
pub mod infrastructure;

#[cfg(any(test, feature = "mock"))]
pub use domain::ports::MockAuthorizer;
pub use domain::{
    errors::AuthzError,
    ports::Authorizer,
    types::{
        action::Action,
        context::Context,
        decision::Decision,
        evaluation::{
            AccessEvaluationRequest, AccessEvaluationsRequest, AccessEvaluationsResponse,
            BoxedEvaluation, BoxedEvaluationDecision,
        },
        resource::Resource,
        subject::{Subject, SubjectKind},
    },
};
pub use infrastructure::local::{
    DEFAULT_SUPER_ADMIN_ROLE, LocalPolicyEngine, LocalPolicyEngineBuilder, SUBJECT_IAM_ROLES_KEY,
    SUBJECT_PERMISSIONS_KEY,
};
