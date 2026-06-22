use auth::Identity;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use http::{HeaderValue, header::AUTHORIZATION};
use mestier_core::CreateUserCommand;
use tracing::error;

use crate::{
    errors::{ApiError, MiddlewareError},
    state::AppState,
};

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, MiddlewareError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or(MiddlewareError::MissingAuthHeader)?;

    let token = extract_bearer(auth_header).map_err(|_| MiddlewareError::InvalidAuthHeader)?;

    let identity = state.auth.get_identity(token).await.map_err(|e| {
        error!("Auth middleware: failed to identify user {:?}", e);
        MiddlewareError::AuthenticationFailed(e.to_string())
    })?;

    if let Identity::User(user) = &identity {
        let name = user.name.as_deref().unwrap_or_else(|| &user.username);

        let email = user.email.as_deref().unwrap_or_else(|| {
            error!("Auth middleware: user {} has no email", name);
            "unknown"
        });

        if let Err(err) = state
            .usecase
            .create_user(CreateUserCommand {
                name: name.to_string(),
                username: user.username.clone(),
                email: email.to_string(),
                sub: user.id.clone(),
            })
            .await
        {
            error!("auth middleware: failed to create user {:?}", err);
        }
    }

    req.extensions_mut().insert(identity);

    Ok(next.run(req).await)
}

pub fn extract_bearer(auth_header: &HeaderValue) -> Result<&str, ApiError> {
    auth_header
        .to_str()
        .map_err(|_| ApiError::TokenNotFound)?
        .strip_prefix("Bearer ")
        .ok_or(ApiError::TokenNotFound)
}
