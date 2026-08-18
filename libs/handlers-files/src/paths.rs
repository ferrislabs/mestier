use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/files")]
pub struct FilesPath;

/// The key is a query parameter, not a path segment: object keys contain
/// slashes (`uploads/quotes/<uuid>`), and a percent-encoded slash in a path is
/// normalised inconsistently between proxies.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/files/url")]
pub struct FileUrlPath;
