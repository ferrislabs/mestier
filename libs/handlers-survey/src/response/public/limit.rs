use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use handlers::{
    ApiError, AppState,
    rate_limit::{client_ip, inject_headers},
};
use http::{HeaderValue, header::RETRY_AFTER};
use rate_limit::{Quota, RateLimitDecision, RateLimitKey, RateLimitService, RateLimiter};

pub const PUBLIC_READ_LIMIT_PER_MINUTE: u32 = 600;
pub const PUBLIC_START_LIMIT_PER_MINUTE: u32 = 60;
pub const PUBLIC_PHOTO_LIMIT_PER_MINUTE: u32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicLimit {
    Read,
    Start,
    Photo,
}

impl PublicLimit {
    pub fn scope(self) -> &'static str {
        match self {
            Self::Read => "survey-public-read",
            Self::Start => "survey-public-start",
            Self::Photo => "survey-public-photo",
        }
    }

    pub fn per_minute(self) -> u32 {
        match self {
            Self::Read => PUBLIC_READ_LIMIT_PER_MINUTE,
            Self::Start => PUBLIC_START_LIMIT_PER_MINUTE,
            Self::Photo => PUBLIC_PHOTO_LIMIT_PER_MINUTE,
        }
    }

    fn key(self, ip: String) -> RateLimitKey {
        RateLimitKey {
            scope: self.scope().to_owned(),
            identifier: ip,
        }
    }
}

fn too_many(decision: &RateLimitDecision) -> Response {
    let mut response = ApiError::TooManyRequests.into_response();
    inject_headers(response.headers_mut(), decision);
    if let Some(retry_after) = decision.retry_after
        && let Ok(value) = HeaderValue::from_str(&retry_after.as_secs().max(1).to_string())
    {
        response.headers_mut().insert(RETRY_AFTER, value);
    }
    response
}

pub(crate) async fn refusal<L: RateLimiter>(
    limiter: &RateLimitService<L>,
    limit: PublicLimit,
    ip: String,
) -> Option<Response> {
    let key = limit.key(ip);
    match limiter
        .check(&key, Quota::per_minute(limit.per_minute()))
        .await
    {
        Ok(decision) if !decision.allowed => Some(too_many(&decision)),
        Ok(_) => None,
        Err(error) => {
            tracing::warn!(%error, scope = %key.scope, "the public survey rate limiter is unavailable, allowing the call");
            None
        }
    }
}

pub async fn public_limit(
    State((state, limit)): State<(AppState, PublicLimit)>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let ip = client_ip(&request, socket);
    match refusal(&state.rate_limit, limit, ip).await {
        Some(refused) => refused,
        None => next.run(request).await,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use axum::body::to_bytes;
    use http::StatusCode;
    use rate_limit::RateLimitError;

    use super::*;

    #[derive(Clone, Default)]
    struct CountingLimiter {
        hits: Arc<Mutex<HashMap<String, u32>>>,
    }

    impl RateLimiter for CountingLimiter {
        async fn check(
            &self,
            key: &RateLimitKey,
            quota: Quota,
        ) -> Result<RateLimitDecision, RateLimitError> {
            let mut hits = self.hits.lock().unwrap();
            let count = hits.entry(key.to_string()).or_insert(0);
            *count += 1;
            let allowed = *count <= quota.limit;
            Ok(RateLimitDecision {
                allowed,
                limit: quota.limit,
                remaining: quota.limit.saturating_sub(*count),
                retry_after: (!allowed).then_some(Duration::from_secs(60)),
            })
        }
    }

    async fn nth_plus_one_is_refused(limit: PublicLimit, expected: u32) {
        let limiter = RateLimitService::new(CountingLimiter::default());
        assert_eq!(limit.per_minute(), expected);
        for call in 1..=expected {
            assert!(
                refusal(&limiter, limit, "203.0.113.7".to_owned())
                    .await
                    .is_none(),
                "{limit:?}: call {call} of {expected} must pass"
            );
        }

        let refused = refusal(&limiter, limit, "203.0.113.7".to_owned()).await;
        let other_ip = refusal(&limiter, limit, "203.0.113.8".to_owned()).await;

        let Some(refused) = refused else {
            panic!("{limit:?}: call {} must be refused", expected + 1);
        };
        let headers = refused.headers().clone();
        let status = refused.status();
        let body = to_bytes(refused.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(json["code"], "E_TOO_MANY_REQUESTS");
        assert_eq!(headers["retry-after"], "60");
        assert_eq!(headers["x-ratelimit-remaining"], "0");
        assert!(
            other_ip.is_none(),
            "{limit:?}: another address keeps its own budget"
        );
    }

    #[tokio::test]
    async fn the_read_limiter_refuses_the_six_hundred_and_first_call_of_a_minute() {
        nth_plus_one_is_refused(PublicLimit::Read, 600).await;
    }

    #[tokio::test]
    async fn the_start_limiter_refuses_the_sixty_first_call_of_a_minute() {
        nth_plus_one_is_refused(PublicLimit::Start, 60).await;
    }

    #[tokio::test]
    async fn the_photo_limiter_refuses_the_thirty_first_call_of_a_minute() {
        nth_plus_one_is_refused(PublicLimit::Photo, 30).await;
    }

    #[test]
    fn starting_a_response_and_uploading_a_photo_are_stricter_than_reading() {
        const { assert!(PUBLIC_START_LIMIT_PER_MINUTE < PUBLIC_READ_LIMIT_PER_MINUTE) };
        const { assert!(PUBLIC_PHOTO_LIMIT_PER_MINUTE < PUBLIC_READ_LIMIT_PER_MINUTE) };
    }

    #[test]
    fn each_limiter_counts_in_a_scope_of_its_own() {
        let scopes =
            [PublicLimit::Read, PublicLimit::Start, PublicLimit::Photo].map(PublicLimit::scope);
        assert_eq!(
            scopes,
            [
                "survey-public-read",
                "survey-public-start",
                "survey-public-photo"
            ]
        );
    }
}
