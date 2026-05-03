use common::CoreError;

/// Maps a [`sqlx::Error`] to a [`CoreError`].
///
/// On unique-violation, the returned [`CoreError::Conflict`] payload is the
/// **constraint name** (e.g. `organizations_slug_key`) — never the raw
/// database message, which can leak internals to API clients. The full
/// SQL error is logged at `error` level for diagnostics.
///
/// Service layers are expected to translate constraint names into business
/// errors (e.g. `organizations_slug_key` → "slug already taken").
pub fn map_sqlx_error(error: sqlx::Error) -> CoreError {
    match &error {
        sqlx::Error::RowNotFound => CoreError::NotFound,
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            tracing::error!(error = %error, "unique constraint violation");
            let constraint = db_err.constraint().unwrap_or("unknown").to_owned();
            CoreError::Conflict(constraint)
        }
        _ => {
            tracing::error!(error = %error, "database error");
            CoreError::Database(error.to_string())
        }
    }
}
