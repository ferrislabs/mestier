//! Benchmarks for the rate-limit middleware helpers. These run on every HTTP
//! request, so allocation regressions multiply by request rate.

use std::{net::SocketAddr, time::Duration};

use axum::extract::Request;
use handlers::rate_limit::{client_ip, inject_headers};
use http::{HeaderMap, header::FORWARDED};
use rate_limit::RateLimitDecision;

fn main() {
    divan::main();
}

fn fallback() -> SocketAddr {
    "127.0.0.1:54321".parse().unwrap()
}

fn make_request(headers: &[(&str, &str)]) -> Request {
    let mut req = Request::builder()
        .uri("/")
        .body(axum::body::Body::empty())
        .unwrap();
    for (k, v) in headers {
        req.headers_mut().insert(
            http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
            http::HeaderValue::from_str(v).unwrap(),
        );
    }
    req
}

#[divan::bench]
fn client_ip_x_forwarded_for(bencher: divan::Bencher) {
    let req = make_request(&[("x-forwarded-for", "203.0.113.42, 10.0.0.1")]);
    bencher.bench_local(|| divan::black_box(client_ip(&req, fallback())));
}

#[divan::bench]
fn client_ip_x_real_ip(bencher: divan::Bencher) {
    let req = make_request(&[("x-real-ip", "203.0.113.42")]);
    bencher.bench_local(|| divan::black_box(client_ip(&req, fallback())));
}

#[divan::bench]
fn client_ip_forwarded_header(bencher: divan::Bencher) {
    let req = make_request(&[(FORWARDED.as_str(), r#"for="203.0.113.42";proto=https"#)]);
    bencher.bench_local(|| divan::black_box(client_ip(&req, fallback())));
}

#[divan::bench]
fn client_ip_fallback_only(bencher: divan::Bencher) {
    let req = make_request(&[]);
    bencher.bench_local(|| divan::black_box(client_ip(&req, fallback())));
}

#[divan::bench]
fn inject_headers_allowed(bencher: divan::Bencher) {
    let decision = RateLimitDecision {
        allowed: true,
        limit: 1000,
        remaining: 742,
        retry_after: None,
    };
    bencher.bench_local(|| {
        let mut headers = HeaderMap::new();
        inject_headers(&mut headers, divan::black_box(&decision));
        divan::black_box(headers);
    });
}

#[divan::bench]
fn inject_headers_blocked(bencher: divan::Bencher) {
    let decision = RateLimitDecision {
        allowed: false,
        limit: 1000,
        remaining: 0,
        retry_after: Some(Duration::from_secs(42)),
    };
    bencher.bench_local(|| {
        let mut headers = HeaderMap::new();
        inject_headers(&mut headers, divan::black_box(&decision));
        divan::black_box(headers);
    });
}
