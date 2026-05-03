//! Integration tests for `RedisRateLimiter` against a real Redis instance.
//!
//! Run with: `cargo test -p rate-limit -- --ignored`
//! Requires Docker.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use rate_limit::{Quota, RateLimitKey, RateLimiter, RedisRateLimiter};
use testcontainers::{ContainerAsync, runners::AsyncRunner};
use testcontainers_modules::redis::Redis;

struct RedisFixture {
    _container: ContainerAsync<Redis>,
    pub limiter: RedisRateLimiter,
}

async fn start_redis() -> RedisFixture {
    let container = Redis::default().start().await.expect("start redis");
    let host = container.get_host().await.expect("host");
    let port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("mapped port");
    let url = format!("redis://{host}:{port}");
    let limiter = RedisRateLimiter::connect(&url).await.expect("connect");
    RedisFixture {
        _container: container,
        limiter,
    }
}

#[tokio::test]
#[ignore = "requires docker"]
async fn sliding_window_blocks_after_limit_and_recovers() {
    let fx = start_redis().await;
    let key = RateLimitKey::organization("org-1");
    let quota = Quota {
        limit: 2,
        window: Duration::from_millis(500),
    };

    let d1 = fx.limiter.check(&key, quota).await.unwrap();
    assert!(d1.allowed);
    assert_eq!(d1.remaining, 1);

    let d2 = fx.limiter.check(&key, quota).await.unwrap();
    assert!(d2.allowed);
    assert_eq!(d2.remaining, 0);

    let d3 = fx.limiter.check(&key, quota).await.unwrap();
    assert!(!d3.allowed);
    assert!(d3.retry_after.is_some());

    tokio::time::sleep(Duration::from_millis(700)).await;

    let d4 = fx.limiter.check(&key, quota).await.unwrap();
    assert!(d4.allowed, "window should have slid past earlier requests");
}

#[tokio::test]
#[ignore = "requires docker"]
async fn isolates_keys_across_organizations() {
    let fx = start_redis().await;
    let quota = Quota::per_minute(1);

    let a = fx
        .limiter
        .check(&RateLimitKey::organization("org-a"), quota)
        .await
        .unwrap();
    let b = fx
        .limiter
        .check(&RateLimitKey::organization("org-b"), quota)
        .await
        .unwrap();
    let a_again = fx
        .limiter
        .check(&RateLimitKey::organization("org-a"), quota)
        .await
        .unwrap();

    assert!(a.allowed);
    assert!(b.allowed, "org-b must not share a counter with org-a");
    assert!(!a_again.allowed, "org-a should now be over quota");
}

#[tokio::test]
#[ignore = "requires docker"]
async fn key_expires_after_window_with_no_traffic() {
    let fx = start_redis().await;
    let key = RateLimitKey::organization("ttl-test");
    let quota = Quota {
        limit: 1,
        window: Duration::from_millis(300),
    };

    fx.limiter.check(&key, quota).await.unwrap();

    // After the window passes, ZREMRANGEBYSCORE drops every entry and the next
    // check sees a fresh state. (The PEXPIRE TTL also reaps it server-side.)
    tokio::time::sleep(Duration::from_millis(500)).await;

    let d = fx.limiter.check(&key, quota).await.unwrap();
    assert!(d.allowed);
    assert_eq!(d.remaining, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires docker"]
async fn atomic_under_concurrent_load() {
    // The whole point of using a Lua script: 100 parallel requests with
    // limit=10 must yield exactly 10 allowed and 90 blocked. This is the
    // guarantee an in-memory limiter cannot give across replicas.
    let fx = start_redis().await;
    let limiter = Arc::new(fx.limiter);
    let key = Arc::new(RateLimitKey::organization("burst"));
    let quota = Quota {
        limit: 10,
        window: Duration::from_secs(5),
    };

    let started = Instant::now();
    let mut handles = Vec::with_capacity(100);
    for _ in 0..100 {
        let limiter = Arc::clone(&limiter);
        let key = Arc::clone(&key);
        handles.push(tokio::spawn(async move {
            limiter.check(&key, quota).await.unwrap().allowed
        }));
    }

    let mut allowed = 0;
    let mut blocked = 0;
    for h in handles {
        if h.await.unwrap() {
            allowed += 1;
        } else {
            blocked += 1;
        }
    }

    assert_eq!(allowed, 10, "exactly `limit` requests must pass");
    assert_eq!(blocked, 90);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "test should complete within the window"
    );
}
