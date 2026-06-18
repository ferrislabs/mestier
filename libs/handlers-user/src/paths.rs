use axum_extra::routing::TypedPath;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/users")]
pub struct UsersPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/users/{id}")]
pub struct UserPath {
    pub id: String,
}
