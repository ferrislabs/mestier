use serde::Serialize;

use super::field::{Field, FieldKind};

/// The field layout a connector accepts as authentication.
///
/// Not a credential: an `AuthScheme` is a static shape declared here, in
/// Rust, and versioned with the binary. A credential is the row an
/// organization stores against one of these shapes (#198) — this module
/// never sees it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AuthScheme {
    pub kind: &'static str,
    pub label: &'static str,
    pub fields: &'static [Field],
}

const fn text_field(name: &'static str, label: &'static str, secret: bool) -> Field {
    Field {
        name,
        label,
        required: true,
        kind: FieldKind::Text,
        expression: false,
        secret,
        visible_when: None,
    }
}

const HTTP_HEADER_FIELDS: &[Field] = &[
    text_field("header_name", "Header name", false),
    text_field("header_value", "Header value", true),
];

const HTTP_BASIC_FIELDS: &[Field] = &[
    text_field("username", "Username", false),
    text_field("password", "Password", true),
];

const BEARER_TOKEN_FIELDS: &[Field] = &[text_field("token", "Token", true)];

const ODOO_API_FIELDS: &[Field] = &[
    text_field("base_url", "Instance URL", false),
    text_field("database", "Database", false),
    text_field("username", "Username", false),
    text_field("api_key", "API key", true),
];

const AUTH_SCHEMES: &[AuthScheme] = &[
    AuthScheme {
        kind: "http_header",
        label: "HTTP header",
        fields: HTTP_HEADER_FIELDS,
    },
    AuthScheme {
        kind: "http_basic",
        label: "HTTP basic auth",
        fields: HTTP_BASIC_FIELDS,
    },
    AuthScheme {
        kind: "bearer_token",
        label: "Bearer token",
        fields: BEARER_TOKEN_FIELDS,
    },
    AuthScheme {
        kind: "odoo_api",
        label: "Odoo API key",
        fields: ODOO_API_FIELDS,
    },
];

/// Every authentication scheme a connector can require.
pub fn auth_schemes() -> &'static [AuthScheme] {
    AUTH_SCHEMES
}

pub fn auth_scheme(kind: &str) -> Option<&'static AuthScheme> {
    AUTH_SCHEMES.iter().find(|scheme| scheme.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field_names(scheme: &AuthScheme) -> Vec<&'static str> {
        scheme.fields.iter().map(|field| field.name).collect()
    }

    fn secret_field_names(scheme: &AuthScheme) -> Vec<&'static str> {
        scheme
            .fields
            .iter()
            .filter(|field| field.secret)
            .map(|field| field.name)
            .collect()
    }

    #[test]
    fn http_header_has_a_name_and_a_secret_value() {
        let scheme = auth_scheme("http_header").expect("http_header is a known scheme");

        assert_eq!(field_names(scheme), vec!["header_name", "header_value"]);
        assert_eq!(secret_field_names(scheme), vec!["header_value"]);
    }

    #[test]
    fn http_basic_has_a_username_and_a_secret_password() {
        let scheme = auth_scheme("http_basic").expect("http_basic is a known scheme");

        assert_eq!(field_names(scheme), vec!["username", "password"]);
        assert_eq!(secret_field_names(scheme), vec!["password"]);
    }

    #[test]
    fn bearer_token_has_a_single_secret_token_field() {
        let scheme = auth_scheme("bearer_token").expect("bearer_token is a known scheme");

        assert_eq!(field_names(scheme), vec!["token"]);
        assert_eq!(secret_field_names(scheme), vec!["token"]);
    }

    #[test]
    fn odoo_api_has_its_four_fields_with_only_the_api_key_secret() {
        let scheme = auth_scheme("odoo_api").expect("odoo_api is a known scheme");

        assert_eq!(
            field_names(scheme),
            vec!["base_url", "database", "username", "api_key"]
        );
        assert_eq!(secret_field_names(scheme), vec!["api_key"]);
    }

    /// The one check that would catch a fifth scheme silently forgotten here.
    #[test]
    fn auth_schemes_lists_exactly_the_four_known_kinds() {
        let kinds: Vec<&'static str> = auth_schemes().iter().map(|scheme| scheme.kind).collect();

        assert_eq!(
            kinds,
            vec!["http_header", "http_basic", "bearer_token", "odoo_api"]
        );
    }

    #[test]
    fn an_unknown_scheme_kind_reads_back_as_none() {
        assert!(auth_scheme("not_a_real_scheme").is_none());
    }

    /// Every field a scheme declares secret must stay masked, and nothing
    /// else should be — a non-secret field silently carrying a password is
    /// how a credential leaks into a log.
    #[test]
    fn only_the_documented_fields_are_secret() {
        let expected: &[(&str, &[&str])] = &[
            ("http_header", &["header_value"]),
            ("http_basic", &["password"]),
            ("bearer_token", &["token"]),
            ("odoo_api", &["api_key"]),
        ];

        for (kind, secrets) in expected {
            let scheme = auth_scheme(kind).unwrap_or_else(|| panic!("{kind} is a known scheme"));
            assert_eq!(secret_field_names(scheme), *secrets, "scheme `{kind}`");
        }
    }
}
