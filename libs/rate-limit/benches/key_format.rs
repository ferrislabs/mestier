//! Benchmarks for the `RateLimitKey` constructors and `Display` impl.
//!
//! These run on every rate-limited request (`key.to_string()` is fed into the
//! Redis script), so any allocation regression here multiplies by request rate.

use rate_limit::RateLimitKey;

fn main() {
    divan::main();
}

#[divan::bench]
fn ip_v4_key(bencher: divan::Bencher) {
    bencher.bench(|| {
        let key = RateLimitKey::ip(divan::black_box("203.0.113.42"));
        divan::black_box(key.to_string())
    });
}

#[divan::bench]
fn ip_v6_key(bencher: divan::Bencher) {
    bencher.bench(|| {
        let key = RateLimitKey::ip(divan::black_box("2001:db8::1"));
        divan::black_box(key.to_string())
    });
}

#[divan::bench]
fn organization_key(bencher: divan::Bencher) {
    bencher.bench(|| {
        let key = RateLimitKey::organization(divan::black_box("org-7f3c8a91"));
        divan::black_box(key.to_string())
    });
}
