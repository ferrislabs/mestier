use super::model::{ApiKeyLocation, SecurityKind, SecurityScheme};

pub const UNSUPPORTED_SECURITY_SCHEME: &str = "unsupported_security_scheme";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offered {
    Bearer,
    Basic,
    Header,
    Query,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identification {
    Anonymous,
    Credential,
    HeaderNamed(String),
    QueryNamed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SecurityRefusal {
    #[error("{UNSUPPORTED_SECURITY_SCHEME}: scheme `{name}` ({kind}) cannot be applied")]
    Unsupported { name: String, kind: String },
    #[error("the credential offers {offered} but the collection declares {declared}")]
    Mismatch { offered: String, declared: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Declared<'a> {
    Bearer,
    Basic,
    HeaderKey(&'a str),
    QueryKey(&'a str),
}

fn classify(scheme: &SecurityScheme) -> Result<Declared<'_>, String> {
    match &scheme.kind {
        SecurityKind::Http { scheme: flavour } if flavour.eq_ignore_ascii_case("bearer") => {
            Ok(Declared::Bearer)
        }
        SecurityKind::Http { scheme: flavour } if flavour.eq_ignore_ascii_case("basic") => {
            Ok(Declared::Basic)
        }
        SecurityKind::Http { scheme: flavour } => Err(format!("http {flavour}")),
        SecurityKind::ApiKey {
            location: ApiKeyLocation::Header,
            parameter_name,
        } => Ok(Declared::HeaderKey(parameter_name)),
        SecurityKind::ApiKey {
            location: ApiKeyLocation::Query,
            parameter_name,
        } => Ok(Declared::QueryKey(parameter_name)),
        SecurityKind::ApiKey { location, .. } => Err(format!(
            "api_key in {}",
            match location {
                ApiKeyLocation::Header => "header",
                ApiKeyLocation::Query => "query",
                ApiKeyLocation::Cookie => "cookie",
            }
        )),
        SecurityKind::Oauth2 => Err("oauth2".to_owned()),
        SecurityKind::OpenIdConnect => Err("openid_connect".to_owned()),
        SecurityKind::Other { declared_type } => Err(declared_type.clone()),
    }
}

impl Offered {
    fn label(self) -> &'static str {
        match self {
            Self::Bearer => "bearer",
            Self::Basic => "basic",
            Self::Header => "a header key",
            Self::Query => "a query key",
            Self::Other => "another kind of credential",
        }
    }
}

pub fn select_identification(
    schemes: &[SecurityScheme],
    offered: Option<Offered>,
) -> Result<Identification, SecurityRefusal> {
    if schemes.is_empty() {
        return Ok(match offered {
            None => Identification::Anonymous,
            Some(_) => Identification::Credential,
        });
    }

    let mut supported: Vec<Declared<'_>> = Vec::new();
    let mut first_unsupported: Option<(&SecurityScheme, String)> = None;
    for scheme in schemes {
        match classify(scheme) {
            Ok(declared) => supported.push(declared),
            Err(kind) => {
                first_unsupported.get_or_insert((scheme, kind));
            }
        }
    }

    if supported.is_empty() {
        let (scheme, kind) = first_unsupported.expect("a non-empty scheme list has a first entry");
        return Err(SecurityRefusal::Unsupported {
            name: scheme.name.clone(),
            kind,
        });
    }

    let Some(offered) = offered else {
        return Ok(Identification::Anonymous);
    };

    supported
        .iter()
        .find_map(|declared| match (offered, declared) {
            (Offered::Bearer, Declared::Bearer) | (Offered::Basic, Declared::Basic) => {
                Some(Identification::Credential)
            }
            (Offered::Header, Declared::HeaderKey(name)) => {
                Some(Identification::HeaderNamed((*name).to_owned()))
            }
            (Offered::Query, Declared::QueryKey(name)) => {
                Some(Identification::QueryNamed((*name).to_owned()))
            }
            _ => None,
        })
        .ok_or_else(|| SecurityRefusal::Mismatch {
            offered: offered.label().to_owned(),
            declared: schemes
                .iter()
                .map(|scheme| format!("`{}`", scheme.name))
                .collect::<Vec<_>>()
                .join(", "),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::automation::collection::model::{ApiKeyLocation, SecurityKind};

    fn scheme(name: &str, kind: SecurityKind) -> SecurityScheme {
        SecurityScheme {
            name: name.to_owned(),
            kind,
        }
    }

    fn http(name: &str, flavour: &str) -> SecurityScheme {
        scheme(
            name,
            SecurityKind::Http {
                scheme: flavour.to_owned(),
            },
        )
    }

    fn api_key(name: &str, location: ApiKeyLocation, parameter: &str) -> SecurityScheme {
        scheme(
            name,
            SecurityKind::ApiKey {
                location,
                parameter_name: parameter.to_owned(),
            },
        )
    }

    #[test]
    fn with_no_declared_scheme_a_credential_is_applied_as_it_is_and_none_means_anonymous() {
        assert_eq!(
            select_identification(&[], None),
            Ok(Identification::Anonymous)
        );
        for offered in [
            Offered::Bearer,
            Offered::Basic,
            Offered::Header,
            Offered::Query,
            Offered::Other,
        ] {
            assert_eq!(
                select_identification(&[], Some(offered)),
                Ok(Identification::Credential)
            );
        }
    }

    #[test]
    fn a_bearer_scheme_takes_a_bearer_credential() {
        let schemes = [http("auth", "bearer")];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Bearer)),
            Ok(Identification::Credential)
        );
    }

    #[test]
    fn the_http_scheme_name_is_matched_without_regard_to_case() {
        let schemes = [http("auth", "Bearer"), http("legacy", "BASIC")];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Bearer)),
            Ok(Identification::Credential)
        );
        assert_eq!(
            select_identification(&schemes[1..], Some(Offered::Basic)),
            Ok(Identification::Credential)
        );
    }

    #[test]
    fn a_basic_scheme_takes_a_basic_credential() {
        let schemes = [http("auth", "basic")];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Basic)),
            Ok(Identification::Credential)
        );
    }

    #[test]
    fn an_api_key_in_a_header_takes_a_header_credential_under_the_declared_name() {
        let schemes = [api_key("key", ApiKeyLocation::Header, "X-API-Key")];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Header)),
            Ok(Identification::HeaderNamed("X-API-Key".to_owned()))
        );
    }

    #[test]
    fn the_credential_picks_among_several_declared_schemes() {
        let schemes = [
            http("auth", "bearer"),
            api_key("key", ApiKeyLocation::Header, "X-Key"),
        ];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Header)),
            Ok(Identification::HeaderNamed("X-Key".to_owned()))
        );
        assert_eq!(
            select_identification(&schemes, Some(Offered::Bearer)),
            Ok(Identification::Credential)
        );
    }

    #[test]
    fn a_declared_scheme_without_a_credential_is_an_anonymous_call() {
        let schemes = [http("auth", "bearer")];

        assert_eq!(
            select_identification(&schemes, None),
            Ok(Identification::Anonymous)
        );
    }

    #[test]
    fn oauth2_alone_is_refused_with_the_stable_code_and_the_scheme_named() {
        let schemes = [scheme("petstore_auth", SecurityKind::Oauth2)];

        for offered in [None, Some(Offered::Bearer)] {
            let refusal = select_identification(&schemes, offered).unwrap_err();

            assert_eq!(
                refusal,
                SecurityRefusal::Unsupported {
                    name: "petstore_auth".to_owned(),
                    kind: "oauth2".to_owned(),
                }
            );
            let message = refusal.to_string();
            assert!(
                message.starts_with("unsupported_security_scheme"),
                "{message}"
            );
            assert!(message.contains("petstore_auth"), "{message}");
        }
    }

    #[test]
    fn an_api_key_in_the_query_takes_a_query_credential_under_the_declared_name() {
        let schemes = [api_key("key", ApiKeyLocation::Query, "api_key")];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Query)),
            Ok(Identification::QueryNamed("api_key".to_owned()))
        );
        assert_eq!(
            select_identification(&schemes, None),
            Ok(Identification::Anonymous)
        );
    }

    #[test]
    fn an_api_key_in_the_query_refuses_a_credential_of_another_shape() {
        let schemes = [api_key("key", ApiKeyLocation::Query, "api_key")];

        for offered in [
            Offered::Header,
            Offered::Bearer,
            Offered::Basic,
            Offered::Other,
        ] {
            let refusal = select_identification(&schemes, Some(offered)).unwrap_err();

            assert!(
                matches!(refusal, SecurityRefusal::Mismatch { .. }),
                "{offered:?}: {refusal:?}"
            );
        }
        let header = [api_key("key", ApiKeyLocation::Header, "X-Key")];
        assert!(matches!(
            select_identification(&header, Some(Offered::Query)),
            Err(SecurityRefusal::Mismatch { .. })
        ));
    }

    #[test]
    fn an_api_key_in_a_cookie_is_still_refused() {
        let schemes = [api_key("key", ApiKeyLocation::Cookie, "session")];

        let refusal = select_identification(&schemes, Some(Offered::Header)).unwrap_err();

        assert!(
            matches!(refusal, SecurityRefusal::Unsupported { .. }),
            "{refusal:?}"
        );
        assert!(refusal.to_string().contains("api_key"));
    }

    #[test]
    fn the_other_declared_kinds_are_refused_too() {
        for kind in [
            SecurityKind::OpenIdConnect,
            SecurityKind::Other {
                declared_type: "mutualTLS".to_owned(),
            },
            SecurityKind::Http {
                scheme: "digest".to_owned(),
            },
        ] {
            let refusal = select_identification(&[scheme("s", kind)], None).unwrap_err();

            assert!(
                matches!(refusal, SecurityRefusal::Unsupported { .. }),
                "{refusal:?}"
            );
        }
    }

    #[test]
    fn an_unsupported_scheme_beside_a_supported_one_does_not_block_the_call() {
        let schemes = [
            scheme("oauth", SecurityKind::Oauth2),
            http("auth", "bearer"),
        ];

        assert_eq!(
            select_identification(&schemes, Some(Offered::Bearer)),
            Ok(Identification::Credential)
        );
        assert_eq!(
            select_identification(&schemes, None),
            Ok(Identification::Anonymous)
        );
    }

    #[test]
    fn a_credential_that_fits_no_supported_scheme_is_a_mismatch_naming_both_sides() {
        let schemes = [http("auth", "bearer")];

        let refusal = select_identification(&schemes, Some(Offered::Basic)).unwrap_err();

        assert!(
            matches!(refusal, SecurityRefusal::Mismatch { .. }),
            "{refusal:?}"
        );
        let message = refusal.to_string();
        assert!(
            message.contains("basic") && message.contains("auth"),
            "{message}"
        );
    }
}
