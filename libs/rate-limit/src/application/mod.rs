use crate::{Quota, RateLimitDecision, RateLimitError, RateLimitKey, RateLimiter};

#[derive(Clone)]
pub struct RateLimitService<R>
where
    R: RateLimiter,
{
    limiter: R,
}

impl<R> RateLimitService<R>
where
    R: RateLimiter,
{
    pub fn new(limiter: R) -> Self {
        Self { limiter }
    }

    #[tracing::instrument(skip_all, fields(scope = %key.scope, id = %key.identifier), err)]
    pub async fn check(
        &self,
        key: &RateLimitKey,
        quota: Quota,
    ) -> Result<RateLimitDecision, RateLimitError> {
        self.limiter.check(key, quota).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::MockRateLimiter;

    fn allowed(limit: u32, remaining: u32) -> RateLimitDecision {
        RateLimitDecision {
            allowed: true,
            limit,
            remaining,
            retry_after: None,
        }
    }

    fn blocked(limit: u32, retry_after: Duration) -> RateLimitDecision {
        RateLimitDecision {
            allowed: false,
            limit,
            remaining: 0,
            retry_after: Some(retry_after),
        }
    }

    #[tokio::test]
    async fn forwards_allowed_decision() {
        let mut limiter = MockRateLimiter::new();
        limiter
            .expect_check()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(allowed(3, 2)) }));

        let svc = RateLimitService::new(limiter);
        let d = svc
            .check(&RateLimitKey::organization("org-1"), Quota::per_minute(3))
            .await
            .unwrap();

        assert!(d.allowed);
        assert_eq!(d.remaining, 2);
    }

    #[tokio::test]
    async fn forwards_blocked_decision_with_retry_after() {
        let mut limiter = MockRateLimiter::new();
        limiter
            .expect_check()
            .returning(|_, _| Box::pin(async { Ok(blocked(2, Duration::from_secs(30))) }));

        let svc = RateLimitService::new(limiter);
        let d = svc
            .check(&RateLimitKey::organization("org-1"), Quota::per_minute(2))
            .await
            .unwrap();

        assert!(!d.allowed);
        assert_eq!(d.retry_after, Some(Duration::from_secs(30)));
    }

    #[tokio::test]
    async fn passes_key_and_quota_to_limiter() {
        let mut limiter = MockRateLimiter::new();
        limiter
            .expect_check()
            .withf(|key, quota| {
                key.scope == "org" && key.identifier == "org-42" && quota.limit == 10
            })
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(allowed(10, 9)) }));

        let svc = RateLimitService::new(limiter);
        svc.check(&RateLimitKey::organization("org-42"), Quota::per_minute(10))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn propagates_backend_errors() {
        let mut limiter = MockRateLimiter::new();
        limiter.expect_check().returning(|_, _| {
            Box::pin(async {
                Err(RateLimitError::BackendUnavailable {
                    message: "boom".into(),
                })
            })
        });

        let svc = RateLimitService::new(limiter);
        let err = svc
            .check(&RateLimitKey::organization("o"), Quota::per_minute(1))
            .await
            .unwrap_err();

        assert!(matches!(err, RateLimitError::BackendUnavailable { .. }));
    }
}
