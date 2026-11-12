pub mod auth;
pub mod delegated;
pub mod errors;
pub mod feature;
pub mod membership;
pub mod rate_limit;
pub mod response;
pub mod state;

pub use auth::{IdentityExt, resolve_actor, resolve_user_id};
pub use errors::{ApiError, MiddlewareError};
pub use feature::{require_feature, require_feature_enabled, require_feature_layer};
pub use membership::{require_mcp_access, require_org_membership};
pub use pagination::{Page, PaginationMetadata, PaginationParams};
pub use response::{DataEnvelope, Response};
pub use state::{AppState, state};
