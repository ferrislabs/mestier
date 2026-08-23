use common::CoreError;

/// Postgres SQLSTATE for an exclusion constraint violation. sqlx's
/// `DatabaseError::kind()` only special-cases unique/foreign-key/not-null/
/// check violations (SQLSTATE classes `23505`/`23503`/`23502`/`23514`); an
/// exclusion violation falls through to `ErrorKind::Other`, so it has to be
/// matched on the raw code.
const EXCLUSION_VIOLATION: &str = "23P01";

/// Maps a [`sqlx::Error`] to a [`CoreError`].
///
/// On unique-violation or exclusion-violation, the returned
/// [`CoreError::Conflict`] payload is the **constraint name** (e.g.
/// `organizations_slug_key`) — never the raw database message, which can leak
/// internals to API clients. The full SQL error is logged at `error` level
/// for diagnostics.
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
        // The overlap guard on `employee_cost_bases` (and any future
        // exclusion constraint) used to surface here as a bare 500 — a
        // known sharp edge, not a hypothetical: `map_sqlx_error` only ever
        // special-cased unique violations, so this was the first exclusion
        // constraint in the schema to actually hit it.
        sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some(EXCLUSION_VIOLATION) => {
            tracing::error!(error = %error, "exclusion constraint violation");
            let constraint = db_err.constraint().unwrap_or("unknown").to_owned();
            CoreError::Conflict(constraint)
        }
        _ => {
            tracing::error!(error = %error, "database error");
            CoreError::Database(error.to_string())
        }
    }
}
