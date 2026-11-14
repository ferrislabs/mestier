use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use handlers::{
    ApiError, AppState,
    rate_limit::{client_ip, inject_headers},
};
use http::{HeaderValue, Method, header::RETRY_AFTER};
use mestier_core::PublicSlug;
use rate_limit::{Quota, RateLimitDecision, RateLimitKey, RateLimitService, RateLimiter};

use super::PUBLIC_SURVEYS_PREFIX;

pub const PUBLIC_READ_LIMIT_PER_MINUTE: u32 = 600;
pub const PUBLIC_START_LIMIT_PER_MINUTE: u32 = 60;
pub const PUBLIC_PHOTO_LIMIT_PER_MINUTE: u32 = 30;
pub const SURVEY_START_LIMIT_PER_MINUTE: u32 = 600;
pub const SURVEY_PHOTO_LIMIT_PER_MINUTE: u32 = 300;
pub const SURVEY_SAVE_LIMIT_PER_MINUTE: u32 = 3000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicLimit {
    Read,
    Start,
    Photo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurveyLimit {
    Start,
    Photo,
    Save,
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

    pub fn per_survey(self, method: &Method) -> Option<SurveyLimit> {
        match self {
            Self::Start => Some(SurveyLimit::Start),
            Self::Photo => Some(SurveyLimit::Photo),
            Self::Read if method == Method::PUT => Some(SurveyLimit::Save),
            Self::Read => None,
        }
    }
}

impl SurveyLimit {
    pub fn scope(self) -> &'static str {
        match self {
            Self::Start => "survey-public-start-per-survey",
            Self::Photo => "survey-public-photo-per-survey",
            Self::Save => "survey-public-save-per-survey",
        }
    }

    pub fn per_minute(self) -> u32 {
        match self {
            Self::Start => SURVEY_START_LIMIT_PER_MINUTE,
            Self::Photo => SURVEY_PHOTO_LIMIT_PER_MINUTE,
            Self::Save => SURVEY_SAVE_LIMIT_PER_MINUTE,
        }
    }
}

pub fn client_address(request: &Request, peer: SocketAddr) -> IpAddr {
    let chosen = client_ip(request, peer);
    chosen
        .parse::<IpAddr>()
        .or_else(|_| chosen.parse::<SocketAddr>().map(|socket| socket.ip()))
        .unwrap_or(peer.ip())
}

fn slug_of(path: &str) -> Option<PublicSlug> {
    let value = path
        .strip_prefix(PUBLIC_SURVEYS_PREFIX)?
        .strip_prefix('/')?
        .split('/')
        .next()?;
    PublicSlug::new(value.to_owned()).ok()
}

pub(crate) fn keys(
    limit: PublicLimit,
    request: &Request,
    peer: SocketAddr,
) -> Vec<(RateLimitKey, u32)> {
    let mut keys = vec![(
        RateLimitKey {
            scope: limit.scope().to_owned(),
            identifier: client_address(request, peer).to_string(),
        },
        limit.per_minute(),
    )];
    if let Some(survey) = limit.per_survey(request.method())
        && let Some(slug) = slug_of(request.uri().path())
    {
        keys.push((
            RateLimitKey {
                scope: survey.scope().to_owned(),
                identifier: slug.as_str().to_owned(),
            },
            survey.per_minute(),
        ));
    }
    keys
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
    keys: &[(RateLimitKey, u32)],
) -> Option<Response> {
    for (key, per_minute) in keys {
        match limiter.check(key, Quota::per_minute(*per_minute)).await {
            Ok(decision) if !decision.allowed => return Some(too_many(&decision)),
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(%error, scope = %key.scope, "the public survey rate limiter is unavailable, allowing the call");
            }
        }
    }
    None
}

pub async fn public_limit(
    State((state, limit)): State<(AppState, PublicLimit)>,
    ConnectInfo(socket): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let keys = keys(limit, &request, socket);
    match refusal(&state.rate_limit, &keys).await {
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

    const PEER: &str = "192.0.2.1:4000";
    const TOKEN: &str = "jeton-de-reprise-secret-0123456789abcdefghijk";

    fn peer() -> SocketAddr {
        PEER.parse().unwrap()
    }

    fn request(method: Method, path: &str, forwarded_for: Option<&str>) -> Request {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(value) = forwarded_for {
            builder = builder.header("x-forwarded-for", value);
        }
        builder.body(axum::body::Body::empty()).unwrap()
    }

    fn path_of(limit: PublicLimit) -> String {
        match limit {
            PublicLimit::Read => "/api/v1/public/surveys/abcdefgh12".to_owned(),
            PublicLimit::Start => "/api/v1/public/surveys/abcdefgh12/responses".to_owned(),
            PublicLimit::Photo => {
                format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}/photos")
            }
        }
    }

    fn method_of(limit: PublicLimit) -> Method {
        match limit {
            PublicLimit::Read => Method::GET,
            PublicLimit::Start | PublicLimit::Photo => Method::POST,
        }
    }

    fn keys_for(limit: PublicLimit, ip: &str) -> Vec<(RateLimitKey, u32)> {
        keys(
            limit,
            &request(method_of(limit), &path_of(limit), Some(ip)),
            peer(),
        )
    }

    async fn nth_plus_one_is_refused(limit: PublicLimit, expected: u32) {
        let limiter = RateLimitService::new(CountingLimiter::default());
        assert_eq!(limit.per_minute(), expected);
        for call in 1..=expected {
            assert!(
                refusal(&limiter, &keys_for(limit, "203.0.113.7"))
                    .await
                    .is_none(),
                "{limit:?}: call {call} of {expected} must pass"
            );
        }

        let refused = refusal(&limiter, &keys_for(limit, "203.0.113.7")).await;
        let other_ip = refusal(&limiter, &keys_for(limit, "203.0.113.8")).await;

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
    fn a_garbage_forwarded_for_falls_back_to_the_peer_address() {
        let garbage = request(
            Method::GET,
            &path_of(PublicLimit::Read),
            Some("pas-une-adresse"),
        );
        let spaced = request(
            Method::GET,
            &path_of(PublicLimit::Read),
            Some("203.0.113.9 , 10.0.0.1"),
        );
        let absent = request(Method::GET, &path_of(PublicLimit::Read), None);

        assert_eq!(client_address(&garbage, peer()), peer().ip());
        assert_eq!(client_address(&absent, peer()), peer().ip());
        assert_eq!(
            client_address(&spaced, peer()),
            "203.0.113.9".parse::<IpAddr>().unwrap()
        );
    }

    #[tokio::test]
    async fn rotating_garbage_forwarded_for_values_are_limited_by_the_peer_address() {
        let limiter = RateLimitService::new(CountingLimiter::default());
        let start = |n: u32| {
            keys(
                PublicLimit::Start,
                &request(
                    Method::POST,
                    &path_of(PublicLimit::Start),
                    Some(&format!("garbage-{n}")),
                ),
                peer(),
            )
        };
        for n in 1..=PUBLIC_START_LIMIT_PER_MINUTE {
            assert!(refusal(&limiter, &start(n)).await.is_none(), "start {n}");
        }

        let refused = refusal(&limiter, &start(0)).await;

        assert_eq!(start(7)[0].0.identifier, "192.0.2.1");
        assert_eq!(
            refused.map(|response| response.status()),
            Some(StatusCode::TOO_MANY_REQUESTS)
        );
    }

    async fn survey_flooded_through_rotating_addresses(
        limit: PublicLimit,
        method: Method,
        path: &str,
        ceiling: u32,
    ) {
        let limiter = RateLimitService::new(CountingLimiter::default());
        let call = |n: u32| {
            let ip = format!("10.{}.{}.{}", (n >> 16) as u8, (n >> 8) as u8, n as u8);
            keys(limit, &request(method.clone(), path, Some(&ip)), peer())
        };
        for n in 1..=ceiling {
            assert!(
                refusal(&limiter, &call(n)).await.is_none(),
                "{limit:?} call {n}"
            );
        }

        let refused = refusal(&limiter, &call(ceiling + 1)).await;
        let other_survey = refusal(
            &limiter,
            &keys(
                limit,
                &request(
                    method.clone(),
                    &path.replace("abcdefgh12", "zyxwvuts98"),
                    Some("10.200.0.1"),
                ),
                peer(),
            ),
        )
        .await;

        assert_eq!(
            refused.map(|response| response.status()),
            Some(StatusCode::TOO_MANY_REQUESTS),
            "{limit:?}"
        );
        assert!(
            other_survey.is_none(),
            "{limit:?}: another survey keeps its own budget"
        );
    }

    #[tokio::test]
    async fn a_survey_is_protected_from_flooding_even_when_every_request_rotates_its_address() {
        survey_flooded_through_rotating_addresses(
            PublicLimit::Start,
            Method::POST,
            &path_of(PublicLimit::Start),
            600,
        )
        .await;
        survey_flooded_through_rotating_addresses(
            PublicLimit::Photo,
            Method::POST,
            &path_of(PublicLimit::Photo),
            300,
        )
        .await;
        survey_flooded_through_rotating_addresses(
            PublicLimit::Read,
            Method::PUT,
            &format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}"),
            3000,
        )
        .await;
    }

    #[test]
    fn reading_resuming_and_completing_have_no_per_survey_ceiling() {
        assert_eq!(PublicLimit::Read.per_survey(&Method::GET), None);
        assert_eq!(PublicLimit::Read.per_survey(&Method::POST), None);
        assert_eq!(
            PublicLimit::Read.per_survey(&Method::PUT),
            Some(SurveyLimit::Save)
        );
        assert_eq!(
            [SurveyLimit::Start, SurveyLimit::Photo, SurveyLimit::Save]
                .map(SurveyLimit::per_minute),
            [600, 300, 3000]
        );
    }

    #[test]
    fn a_limiter_key_never_holds_the_resume_token() {
        for (method, path) in [
            (
                Method::POST,
                format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}/photos"),
            ),
            (
                Method::PUT,
                format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}"),
            ),
            (
                Method::GET,
                format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}"),
            ),
            (
                Method::POST,
                format!("/api/v1/public/surveys/abcdefgh12/responses/{TOKEN}/complete"),
            ),
        ] {
            for limit in [PublicLimit::Read, PublicLimit::Start, PublicLimit::Photo] {
                for (key, _) in keys(
                    limit,
                    &request(method.clone(), &path, Some("203.0.113.7")),
                    peer(),
                ) {
                    let written = key.to_string();
                    assert!(!written.contains(TOKEN), "{written}");
                }
            }
        }
        let photo = keys_for(PublicLimit::Photo, "203.0.113.7");
        assert_eq!(
            photo
                .iter()
                .map(|(key, _)| key.to_string())
                .collect::<Vec<_>>(),
            [
                "ratelimit:survey-public-photo:203.0.113.7",
                "ratelimit:survey-public-photo-per-survey:abcdefgh12"
            ]
        );
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
