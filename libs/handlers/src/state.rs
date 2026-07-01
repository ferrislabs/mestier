use common::Config;
use mestier_core::{
    EventHub, MestierAuthService, MestierFileStorageService, MestierRateLimitService,
    MestierUseCase, create_service,
};
use rate_limit::Quota;
use server::errors::ServerError;
use std::sync::Arc;

use args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,

    pub auth: MestierAuthService,
    pub file_storage: MestierFileStorageService,
    pub usecase: MestierUseCase,
    pub rate_limit: MestierRateLimitService,
    pub rate_limit_quota: Quota,
    /// Subscribe-side handle for the in-process realtime event bus.
    /// Populated from the single [`EventHub`] created by `create_service`
    /// so all subscribers share the same broadcast channel.
    pub events: EventHub,
    /// HMAC-SHA256 secret for signing and verifying document access tokens.
    /// Sourced from `DOCUMENT_SIGNING_SECRET` (falls back to `AUTH_CLIENT_SECRET`).
    pub document_signing_secret: Arc<String>,
}

pub async fn state(args: Arc<Args>) -> Result<AppState, ServerError> {
    let config = Config::from(args.as_ref().clone());

    let document_signing_secret = Arc::new(config.document_signing_secret.clone());

    let service = create_service(config).await.unwrap();

    Ok(AppState {
        args,
        auth: service.auth,
        file_storage: service.file_storage,
        usecase: service.usecase,
        rate_limit: service.rate_limit,
        rate_limit_quota: service.rate_limit_quota,
        events: service.events,
        document_signing_secret,
    })
}
