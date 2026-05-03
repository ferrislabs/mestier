use std::time::Duration;

use chrono::Utc;
use redis::{Script, aio::ConnectionManager};

use crate::{Quota, RateLimitDecision, RateLimitError, RateLimitKey, RateLimiter};

const SLIDING_WINDOW_SCRIPT: &str = r#"
local key = KEYS[1]
local now = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local limit = tonumber(ARGV[3])
local member = ARGV[4]

redis.call('ZREMRANGEBYSCORE', key, 0, now - window)
local count = redis.call('ZCARD', key)

if count < limit then
    redis.call('ZADD', key, now, member)
    redis.call('PEXPIRE', key, window)
    return {1, limit - count - 1, 0}
end

local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
local retry_after = 0
if oldest[2] then
    retry_after = (tonumber(oldest[2]) + window) - now
    if retry_after < 0 then retry_after = 0 end
end
return {0, 0, retry_after}
"#;

#[derive(Clone)]
pub struct RedisRateLimiter {
    conn: ConnectionManager,
    script: Script,
}

impl RedisRateLimiter {
    pub async fn connect(url: &str) -> Result<Self, RateLimitError> {
        let client = redis::Client::open(url).map_err(|e| RateLimitError::BackendUnavailable {
            message: e.to_string(),
        })?;
        let conn = ConnectionManager::new(client).await.map_err(|e| {
            RateLimitError::BackendUnavailable {
                message: e.to_string(),
            }
        })?;
        Ok(Self {
            conn,
            script: Script::new(SLIDING_WINDOW_SCRIPT),
        })
    }
}

impl RateLimiter for RedisRateLimiter {
    #[tracing::instrument(
        name = "redis.rate_limit.eval",
        skip_all,
        fields(
            otel.kind = "client",
            db.system = "redis",
            db.operation = "EVALSHA",
            rate_limit.key = %key,
            rate_limit.limit = quota.limit,
        ),
        err
    )]
    async fn check(
        &self,
        key: &RateLimitKey,
        quota: Quota,
    ) -> Result<RateLimitDecision, RateLimitError> {
        let mut conn = self.conn.clone();
        let now_ms = Utc::now().timestamp_millis();
        let window_ms = quota.window.as_millis() as i64;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let member = format!("{now_ms}-{nanos:x}");

        let result: (i64, i64, i64) = self
            .script
            .key(key.to_string())
            .arg(now_ms)
            .arg(window_ms)
            .arg(quota.limit as i64)
            .arg(member)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| RateLimitError::BackendUnavailable {
                message: e.to_string(),
            })?;

        let (allowed, remaining, retry_after_ms) = result;
        Ok(RateLimitDecision {
            allowed: allowed == 1,
            limit: quota.limit,
            remaining: remaining.max(0) as u32,
            retry_after: if retry_after_ms > 0 {
                Some(Duration::from_millis(retry_after_ms as u64))
            } else {
                None
            },
        })
    }
}
