use common::CoreError;
use iam::IamError;

/// Everything that can go wrong while seeding a preview environment.
///
/// Deliberately flat rather than nested per source: `main` only ever does
/// one thing with a failure — log it and exit non-zero — so there is no
/// caller that benefits from matching a specific variant.
#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("use case failed: {0}")]
    Core(#[from] CoreError),

    #[error("ferriskey (IAM) call failed: {0}")]
    Iam(#[from] IamError),

    /// `IdentityExt::user_id` (the real request path) parses a FerrisKey
    /// `sub` straight as a UUID, so `create_organization` looks the owner up
    /// by treating its `owner_id` as a `sub`. Seeding relies on the same
    /// assumption; a FerrisKey deployment that hands out non-UUID subjects
    /// would break the real login path too, not just this binary.
    #[error("ferriskey returned a non-uuid subject `{0}` for the demo owner")]
    NonUuidSubject(String),
}
