use aws_lc_rs::{constant_time, digest};
use base64::{Engine, engine::general_purpose::STANDARD};

use crate::infrastructure::automation::connectors::http_client::HttpAuth;

#[cfg(test)]
thread_local! {
    static COMPARISONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub fn constant_time_equals(left: &[u8], right: &[u8]) -> bool {
    #[cfg(test)]
    COMPARISONS.with(|count| count.set(count.get() + 1));
    let left = digest::digest(&digest::SHA256, left);
    let right = digest::digest(&digest::SHA256, right);
    constant_time::verify_slices_are_equal(left.as_ref(), right.as_ref()).is_ok()
}

pub fn presents<'a>(expected: &HttpAuth, header: impl Fn(&str) -> Option<&'a str>) -> bool {
    match expected {
        HttpAuth::Bearer { token } => header("authorization")
            .and_then(|value| after_scheme(value, "bearer"))
            .is_some_and(|presented| constant_time_equals(presented.as_bytes(), token.as_bytes())),
        HttpAuth::Basic { username, password } => header("authorization")
            .and_then(|value| after_scheme(value, "basic"))
            .and_then(|encoded| STANDARD.decode(encoded).ok())
            .is_some_and(|decoded| {
                constant_time_equals(&decoded, format!("{username}:{password}").as_bytes())
            }),
        HttpAuth::Header { name, value } => header(name)
            .is_some_and(|presented| constant_time_equals(presented.as_bytes(), value.as_bytes())),
    }
}

fn after_scheme<'a>(value: &'a str, scheme: &str) -> Option<&'a str> {
    let (presented, rest) = value.split_once(' ')?;
    presented.eq_ignore_ascii_case(scheme).then_some(rest)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> impl Fn(&str) -> Option<&'static str> {
        let map: HashMap<String, &'static str> = pairs
            .iter()
            .map(|(name, value)| (name.to_ascii_lowercase(), *value))
            .collect();
        move |name| map.get(&name.to_ascii_lowercase()).copied()
    }

    fn bearer() -> HttpAuth {
        HttpAuth::Bearer {
            token: "s3cret".to_owned(),
        }
    }

    fn basic() -> HttpAuth {
        HttpAuth::Basic {
            username: "alice".to_owned(),
            password: "p:ss word".to_owned(),
        }
    }

    fn header_auth() -> HttpAuth {
        HttpAuth::Header {
            name: "X-Api-Key".to_owned(),
            value: "k3y".to_owned(),
        }
    }

    fn comparisons_during(work: impl FnOnce()) -> usize {
        COMPARISONS.with(|count| count.set(0));
        work();
        COMPARISONS.with(|count| count.get())
    }

    #[test]
    fn equal_values_are_equal_and_nothing_else_is() {
        assert!(constant_time_equals(b"abc", b"abc"));
        assert!(constant_time_equals(b"", b""));
        assert!(!constant_time_equals(b"abc", b"abd"));
        assert!(!constant_time_equals(b"abc", b"abcd"));
        assert!(!constant_time_equals(b"abcd", b"abc"));
        assert!(!constant_time_equals(b"", b"a"));
    }

    #[test]
    fn a_bearer_token_must_be_presented_as_a_bearer_authorization() {
        let expected = bearer();

        assert!(presents(
            &expected,
            headers(&[("Authorization", "Bearer s3cret")])
        ));
        assert!(presents(
            &expected,
            headers(&[("authorization", "bearer s3cret")])
        ));
        assert!(presents(
            &expected,
            headers(&[("authorization", "BEARER s3cret")])
        ));
    }

    #[test]
    fn a_bearer_token_that_is_wrong_missing_or_in_another_scheme_is_refused() {
        let expected = bearer();

        for value in [
            "Bearer wrong",
            "Bearer s3cre",
            "Bearer s3cret2",
            "Bearer ",
            "Bearer  s3cret",
            "Bearer s3cret ",
            "Bearers3cret",
            "Basic s3cret",
            "Token s3cret",
            "s3cret",
            "",
        ] {
            assert!(
                !presents(&expected, headers(&[("authorization", value)])),
                "`{value}`"
            );
        }
        assert!(!presents(&expected, headers(&[])));
        assert!(!presents(
            &expected,
            headers(&[("x-authorization", "Bearer s3cret")])
        ));
    }

    #[test]
    fn a_basic_credential_is_presented_as_base64_of_user_colon_password() {
        let expected = basic();
        let good = format!("Basic {}", STANDARD.encode("alice:p:ss word"));
        let good: &'static str = Box::leak(good.into_boxed_str());

        assert!(presents(&expected, headers(&[("authorization", good)])));
    }

    #[test]
    fn a_basic_credential_that_is_wrong_or_malformed_is_refused() {
        let expected = basic();
        let encode = |text: &str| -> &'static str {
            Box::leak(format!("Basic {}", STANDARD.encode(text)).into_boxed_str())
        };

        for value in [
            encode("alice:wrong"),
            encode("bob:p:ss word"),
            encode("alice"),
            encode("alice:p:ss word "),
            encode(":p:ss word"),
            "Basic !!!not-base64!!!",
            "Basic ",
            "Bearer YWxpY2U6cDpzcyB3b3Jk",
        ] {
            assert!(
                !presents(&expected, headers(&[("authorization", value)])),
                "`{value}`"
            );
        }
        assert!(!presents(&expected, headers(&[])));
    }

    #[test]
    fn a_header_credential_is_the_named_header_with_exactly_its_value() {
        let expected = header_auth();

        assert!(presents(&expected, headers(&[("X-Api-Key", "k3y")])));
        assert!(presents(&expected, headers(&[("x-api-key", "k3y")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "K3Y")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "k3y ")])));
        assert!(!presents(&expected, headers(&[("x-api-key", "")])));
        assert!(!presents(&expected, headers(&[("x-other-key", "k3y")])));
        assert!(!presents(&expected, headers(&[])));
    }

    #[test]
    fn a_credential_never_authorizes_through_the_wrong_header() {
        assert!(!presents(&bearer(), headers(&[("x-api-key", "s3cret")])));
        assert!(!presents(
            &header_auth(),
            headers(&[("authorization", "Bearer k3y")])
        ));
        assert!(!presents(
            &basic(),
            headers(&[("authorization", "Bearer s3cret")])
        ));
    }

    #[test]
    fn every_secret_is_compared_through_the_constant_time_comparison() {
        let cases: [(HttpAuth, &'static [(&'static str, &'static str)]); 3] = [
            (bearer(), &[("authorization", "Bearer nope")]),
            (basic(), &[("authorization", "Basic YWxpY2U6bm9wZQ==")]),
            (header_auth(), &[("x-api-key", "nope")]),
        ];

        for (expected, pairs) in cases {
            let count = comparisons_during(|| {
                assert!(!presents(&expected, headers(pairs)));
            });

            assert_eq!(count, 1, "{expected:?}");
        }
    }
}
