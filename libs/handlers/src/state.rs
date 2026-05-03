use common::Config;
use mestier_core::{MestierAuthService, MestierRateLimitService, MestierUseCase, create_service};
use rate_limit::Quota;
use server::errors::ServerError;
use std::sync::Arc;

use args::Args;

#[derive(Clone)]
pub struct AppState {
    pub args: Arc<Args>,

    pub auth: MestierAuthService,
    pub usecase: MestierUseCase,
    pub rate_limit: MestierRateLimitService,
    pub rate_limit_quota: Quota,
}

pub async fn state(args: Arc<Args>) -> Result<AppState, ServerError> {
    let config = Config::from(args.as_ref().clone());

    let service = create_service(config).await.unwrap();

    Ok(AppState {
        args,
        auth: service.auth,
        usecase: service.usecase,
        rate_limit: service.rate_limit,
        rate_limit_quota: service.rate_limit_quota,
    })
}
