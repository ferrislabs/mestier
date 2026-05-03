//! Benchmark for `Token::decode_manual`. This runs once per authenticated
//! request, so the allocation profile here is on the critical path.

use auth::Token;

fn main() {
    divan::main();
}

// A real-world Keycloak token, used to keep the bench representative of
// production payload sizes (claims include realm_access, resource_access,
// preferred_username, etc.).
const SAMPLE_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsInR5cCIgOiAiSldUIiwia2lkIiA6ICJiaE9ZRENETC14TFhyWVRGZERTMmlwMzdHdHhFNlpUVVI4a2swSm9CVDhzIn0.eyJleHAiOjE3NjExMTc5NTYsImlhdCI6MTc2MTExNzg5NiwianRpIjoib25ydHJvOjJhMjNjYjkyLTc1MTktYzgzYS0wMGM2LWIxNDQyOTlkYjE1NSIsImlzcyI6Imh0dHA6Ly9sb2NhbGhvc3Q6ODAwMC9yZWFsbXMvbWFzdGVyIiwiYXVkIjoiYWNjb3VudCIsInN1YiI6IjE0NDM0Y2JhLThmMzItNDliYi1hMzllLTgzNzhhN2NkZGVhMyIsInR5cCI6IkJlYXJlciIsImF6cCI6ImFwaSIsInNpZCI6ImY2YjUwZWY2LTJlNjItNjAxNS1lNTJjLTA5NzA4NWUyYTAxOCIsImFjciI6IjEiLCJhbGxvd2VkLW9yaWdpbnMiOlsiLyoiXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRlZmF1bHQtcm9sZXMtbWFzdGVyIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoicHJvZmlsZSBlbWFpbCIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJuYW1lIjoiTmF0aGFlbCBCb25uYWwiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJuYXRoYWVsIiwiZ2l2ZW5fbmFtZSI6Ik5hdGhhZWwiLCJmYW1pbHlfbmFtZSI6IkJvbm5hbCIsImVtYWlsIjoibmF0aGFlbEBib25uYWwuY2xvdWQifQ.ApKQsnjT2gCgqngCndHTNU2W9YJzuHGHRLk4OE-_b4Sk650vSUS0AhMWPuAgEwVjLm2y8UpOJ_64BXDcnQMZzKHNo2_xj5c8P8glvBM-02YJlR_ssbUlReJPvLLKzwFTPdKF_FDsEIXkroV-ds8aU5OmOX8emdxb79XzdHkaWbl13IErHqMnRMsAvh742ZQeCqbedr8R3uH6V5qbbNu7H9kTf2EGX7G66rfpY-Zl8EyR4fWCVwjVLr_5tLsUFteajADf2RtW9dZRsUW9M9g9WIzT_tNdsTQhBj1q3kHkwhhC6hVVz2VaLNgYKikLu8QDfGy4BZ6nHZobrq4eKr3HQg";

#[divan::bench]
fn decode_realistic_token(bencher: divan::Bencher) {
    let token = Token::new(SAMPLE_TOKEN.to_string());
    bencher.bench_local(|| divan::black_box(token.decode_manual().unwrap()));
}

#[divan::bench]
fn extract_claims(bencher: divan::Bencher) {
    let token = Token::new(SAMPLE_TOKEN.to_string());
    bencher.bench_local(|| divan::black_box(token.extract_claims().unwrap()));
}
