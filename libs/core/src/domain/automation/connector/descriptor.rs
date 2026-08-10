use serde::Serialize;
use serde_json::Value;

use super::field::Field;

/// What a connector accepts as authentication, checked against a
/// credential's scheme when a workflow is saved (#199).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum AuthRequirement {
    /// The connector talks to nothing external — a control-flow connector,
    /// for instance — so no credential can be attached to it.
    None,
    /// Only this one scheme authenticates the connector.
    Exactly(&'static str),
    /// Any of these schemes authenticates the connector — an HTTP call that
    /// accepts either a bearer token or a header, say.
    AnyOf(&'static [&'static str]),
}

impl AuthRequirement {
    /// The predicate #199 replays when a workflow is saved: does this
    /// credential's scheme satisfy what the connector needs.
    pub fn accepts(&self, scheme_kind: &str) -> bool {
        match self {
            AuthRequirement::None => false,
            AuthRequirement::Exactly(kind) => *kind == scheme_kind,
            AuthRequirement::AnyOf(kinds) => kinds.contains(&scheme_kind),
        }
    }

    /// The schemes named here, for the catalogue's consistency test and for
    /// the frontend's credential picker.
    pub fn scheme_kinds(&self) -> &'static [&'static str] {
        match *self {
            AuthRequirement::None => &[],
            // `Exactly` carries a single `&'static str`, not a `&'static`
            // slice: there is no existing 'static storage to borrow a
            // one-element slice from. Leaking a one-pointer allocation turns
            // it into one without unsafe code; `scheme_kinds` is called at
            // catalogue-build and API-response time, never in a hot loop.
            AuthRequirement::Exactly(kind) => vec![kind].leak(),
            AuthRequirement::AnyOf(kinds) => kinds,
        }
    }
}

/// A connector type: one action a workflow graph can pose and configure
/// (#199 poses it, #200 executes it). Static and versioned with the binary,
/// exactly like `events::EventDescriptor` on the trigger side.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConnectorDescriptor {
    pub kind: &'static str,
    pub version: u16,
    pub family: &'static str,
    pub label: &'static str,
    pub auth: AuthRequirement,
    pub fields: &'static [Field],
    /// What this connector's output looks like, so the editor can offer
    /// `connectors.c1.output.id` in the mapping autocomplete before a single
    /// run has ever happened.
    pub output_example: Value,
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::*;
    use crate::domain::automation::connector::{FieldKind, SelectOption, VisibleWhen};

    #[test]
    fn no_auth_accepts_no_scheme() {
        let requirement = AuthRequirement::None;

        assert!(!requirement.accepts("bearer_token"));
        assert!(!requirement.accepts("http_basic"));
        assert_eq!(requirement.scheme_kinds(), &[] as &[&str]);
    }

    #[test]
    fn exactly_accepts_only_its_own_scheme() {
        let requirement = AuthRequirement::Exactly("odoo_api");

        assert!(requirement.accepts("odoo_api"));
        assert!(!requirement.accepts("bearer_token"));
        assert_eq!(requirement.scheme_kinds(), &["odoo_api"]);
    }

    #[test]
    fn any_of_accepts_each_of_its_schemes_and_nothing_else() {
        let requirement = AuthRequirement::AnyOf(&["bearer_token", "http_header"]);

        assert!(requirement.accepts("bearer_token"));
        assert!(requirement.accepts("http_header"));
        assert!(!requirement.accepts("http_basic"));
        assert_eq!(
            requirement.scheme_kinds(),
            &["bearer_token", "http_header"]
        );
    }

    /// The catalogue leaves the crate as JSON (#203): every field must
    /// survive the trip, including the nested ones.
    #[test]
    fn a_descriptor_serializes_without_losing_its_fields() {
        let descriptor = ConnectorDescriptor {
            kind: "odoo.create_partner",
            version: 1,
            family: "odoo",
            label: "Create partner",
            auth: AuthRequirement::AnyOf(&["odoo_api", "bearer_token"]),
            fields: &[Field {
                name: "kind",
                label: "Kind",
                required: true,
                kind: FieldKind::Select {
                    options: &[SelectOption {
                        value: "company",
                        label: "Company",
                    }],
                },
                expression: true,
                secret: false,
                visible_when: Some(VisibleWhen {
                    field: "type",
                    any_of: &["b2b"],
                }),
            }],
            output_example: json!({ "id": 42 }),
        };

        let json = to_value(&descriptor).expect("descriptor serializes");

        assert_eq!(json["kind"], "odoo.create_partner");
        assert_eq!(json["version"], 1);
        assert_eq!(json["family"], "odoo");
        assert_eq!(json["output_example"], json!({ "id": 42 }));
        assert_eq!(json["auth"], json!({ "AnyOf": ["odoo_api", "bearer_token"] }));
        assert_eq!(json["fields"][0]["name"], "kind");
        assert_eq!(json["fields"][0]["required"], true);
        assert_eq!(json["fields"][0]["expression"], true);
        assert_eq!(json["fields"][0]["secret"], false);
        assert_eq!(
            json["fields"][0]["kind"],
            json!({ "Select": { "options": [{ "value": "company", "label": "Company" }] } })
        );
        assert_eq!(
            json["fields"][0]["visible_when"],
            json!({ "field": "type", "any_of": ["b2b"] })
        );
    }
}
