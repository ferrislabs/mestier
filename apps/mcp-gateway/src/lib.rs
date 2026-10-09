pub mod app;
pub mod config;
pub mod exchange;
pub mod pdp;
pub mod rpc;
pub mod verifier;

use std::time::Duration;

use reqwest::{Client, redirect::Policy};

use crate::{
    app::{AppState, Gateway},
    config::Config,
    exchange::{CachedExchanger, FerrisKeyExchanger},
    pdp::AuthzenPdp,
    verifier::JwksVerifier,
};

pub type ProductionState = AppState<JwksVerifier, AuthzenPdp, CachedExchanger<FerrisKeyExchanger>>;

pub fn build_state(config: &Config) -> Result<ProductionState, reqwest::Error> {
    let http = Client::builder()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .build()?;
    let exchange_http = Client::builder()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(10))
        .build()?;
    Ok(AppState::new(Gateway {
        verifier: JwksVerifier::new(
            exchange_http.clone(),
            &config.issuer,
            config.resource_url.clone(),
            config.jwks_refresh_interval(),
        ),
        pdp: AuthzenPdp::new(exchange_http.clone(), &config.pdp_url),
        exchanger: CachedExchanger::new(FerrisKeyExchanger::new(
            exchange_http,
            &config.issuer,
            config.exchange_client_id.clone(),
            config.exchange_client_secret.clone(),
            config.exchange_audience.clone(),
            config.exchange_auth_method,
        )),
        http,
        resource_url: config.resource_url.clone(),
        issuer: config.issuer.clone(),
        upstream_url: config.upstream_url.clone(),
    }))
}
