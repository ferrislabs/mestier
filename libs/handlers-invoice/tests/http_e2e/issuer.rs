//! A stand-in for FerrisKey, serving one JWKS over a real socket.
//!
//! Identical to `handlers-quote`'s own copy: the point of this test is that
//! the auth middleware runs for real, so the token has to survive the whole
//! chain. Each e2e crate keeps its own copy rather than sharing one, same
//! reason `require_org_membership` is duplicated per crate rather than
//! imported.

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use auth::{Audience, Claims, Subject};
use chrono::Utc;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

const PRIVATE_KEY_PEM: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQDHF23PRKIxgKZC
8b8NihRuCh/PTx8bdX+x5Mp6WR44eVFNirC/j9mhmtK5vezS3CokPgpl0g1CVfBp
iZR1OEZs+Y0cFFJBZTWxuDiUz3jAQIqQlN7WH6dNsOu14FJS799Tv4yAC/wtx7ig
yLpncaQYJE5CkSOAGd+P7YBT2ONtOs0dR0+bdTbDkpu1MlIEsVMojzJFVuGGKtJp
cgLrYVZKACNop5y84tQUJx7vLW2JfdEZleFJ6k4g9DNnl/Y6njLTsKVtCKakORf3
wiqbk80IQxN7labaVQlXd1GooBC+7mBxwyXbFW35eM31GgQBPVaqBOHpBLu60knW
t7hC0x/xAgMBAAECggEAHjaCfg1K1dtRn+Ai37GgJxDXQfUeYeLjZYI0bfu/N8/F
VFCjQPbaDom5x+E4IsmxhX16w3fsdjAng0STKHTJTzlRvjyhPPZYfydXQtH3X6mL
vaQx6umz0Hj0VE3+AEMRr5pmfnoTI3lnHdNIYnFe9yDvVW/EJOkIQcXHjzHfVZBt
ofFGHL8NjJ008VEVwDtscaCq+ibfoEghvI9GMffd/HqZAYd9qhrz+wiT8ZQAFbp5
kTlP6YBUJ+mo2K7OkNdGPivgaxQhijwqc9d53eFMrmnETxliAHN1Alniud16o1j8
TpaIwF0Y+y6trmHrKXWaQkVRbPfYT2QTSmpTeLe1jwKBgQDzEevZNdjBJWvjvqaX
5n5F3ZPQD67XKghgokkNa+uKrIvHrzG4HDXrR7R24SHBxTmHGgw2k3WRfaFBnoHN
n7BoJNK+M8ddP3b0ea2kFpPAkWuWmOxv0VQykt721vfkHohBu5ra5eoXXd4Efnj5
PqX50JCVPT+k5Xl4R9dpbniziwKBgQDRrp4QZoiX3GXEmddqIn5ZwMrY/ia9Z8M0
da3I/+PCUFw23HEP0T6LskS8g64dG63hhrCy0BZN+WrJQu/m82cAJaRsQCbzilIt
K6/3NtXlu4SmXotGxEpn26X03j0YO1osKLFgd2FiT/0KiIQYj1/Ipyst3YghCIjR
zYm1KKx58wKBgBAV4oa4UoTNpisnJb0tqrOS60I8l3RzuqQyeSUjPC4sJv/q7x5g
94x/bUjksygwlhMDvUUrUv9y0eYWyD5EUBdEQJIHuSzJk2SwXLZcLCD1Pqpzqkno
D2tdXtX0+eilwJyg/ql3x5sOQjAH8peD9tXmYHsP15NhAD3eeznl7qTrAoGBAIXj
8pqWXnJaEcHQWnUzQWseaGjXIPWg5E0DN805WL4jgj6l1Kw8+KtLUgjuLKf5nLZ9
wybrKNLxiPaq/3WBxyuY3b0h2b15fa/KTbqWEU94xeNWS6kMflaDMx2BK5HllFbO
RTVMBas5WGL5eSAVrRv7Yt8OrnYpdPRDQsOjDT9xAoGBAMq7pYVEJBWoyFYWDnSY
LoQgUrpiRssRjaCMHOpEBxjtOTv3TzeyzHWD7+r2+y/qToJXcdA8jEyhaSeUa7mr
9e2VtIC/6Ouhmfb0+mwgwO/zQHR0sd/ruyNc7v4FBgYfZ/XqvYtzzTZzhNmvX9gQ
HUim3t4M1KMtX1QmMKKCg4i4
-----END PRIVATE KEY-----"#;

const KID: &str = "test-kid";
const MODULUS: &str = "xxdtz0SiMYCmQvG_DYoUbgofz08fG3V_seTKelkeOHlRTYqwv4_ZoZrSub3s0twqJD4KZdINQlXwaYmUdThGbPmNHBRSQWU1sbg4lM94wECKkJTe1h-nTbDrteBSUu_fU7-MgAv8Lce4oMi6Z3GkGCROQpEjgBnfj-2AU9jjbTrNHUdPm3U2w5KbtTJSBLFTKI8yRVbhhirSaXIC62FWSgAjaKecvOLUFCce7y1tiX3RGZXhSepOIPQzZ5f2Op4y07ClbQimpDkX98Iqm5PNCEMTe5Wm2lUJV3dRqKAQvu5gccMl2xVt-XjN9RoEAT1WqgTh6QS7utJJ1re4QtMf8Q";
const EXPONENT: &str = "AQAB";

/// Serves the JWKS until the test process exits.
///
/// A loop, not a one-shot: nothing caches the key set, so every authenticated
/// request fetches it again.
pub fn spawn() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind the fake issuer");
    let addr = listener.local_addr().expect("read the fake issuer address");
    let body = format!(r#"{{"keys":[{{"kid":"{KID}","n":"{MODULUS}","e":"{EXPONENT}"}}]}}"#);

    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut buf = [0_u8; 2048];
            let _ = stream.read(&mut buf);

            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    format!("http://{addr}")
}

/// A token the middleware will accept, for a caller identified by `sub`.
pub fn mint(sub: &str) -> String {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(KID.to_owned());

    let claims = Claims {
        sub: Subject(sub.to_owned()),
        iss: "http://127.0.0.1/realms/mestier".to_owned(),
        aud: Some(Audience::Single("mestier-api".to_owned())),
        exp: Some((Utc::now() + chrono::Duration::hours(1)).timestamp()),
        email: Some("artisan@example.com".to_owned()),
        email_verified: Some(true),
        name: Some("Artisan Test".to_owned()),
        preferred_username: Some("artisan".to_owned()),
        given_name: Some("Artisan".to_owned()),
        family_name: Some("Test".to_owned()),
        scope: "openid profile email".to_owned(),
        client_id: None,
        extra: serde_json::Map::new(),
    };

    let key = EncodingKey::from_rsa_pem(PRIVATE_KEY_PEM.as_bytes()).expect("build the signing key");

    encode(&header, &claims, &key).expect("sign the test token")
}
