use std::collections::HashMap;

use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use super::descriptor::{AuthRequirement, ConnectorDescriptor};
use super::field::{Field, FieldKind, SelectOption};
use crate::domain::automation::workflow::Branch;

/// Every connector kind the product can pose in a workflow graph, keyed by
/// kind and version.
///
/// Mirrors `events::EventCatalogue` on purpose: the trigger picker and the
/// action picker are the same UI pattern, fed by two catalogues built the
/// same way.
#[derive(Debug, Default, Serialize)]
pub struct ConnectorCatalogue {
    /// Nested rather than keyed by `(kind, version)`: lookups come from the
    /// database with a runtime `&str`, which cannot be turned into the
    /// `&'static str` half of a tuple key without scanning the whole map.
    descriptors: HashMap<&'static str, HashMap<u16, ConnectorDescriptor>>,
}

impl ConnectorCatalogue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        descriptor: ConnectorDescriptor,
    ) -> Result<(), ConnectorCatalogueError> {
        let versions = self.descriptors.entry(descriptor.kind).or_default();

        if versions.contains_key(&descriptor.version) {
            return Err(ConnectorCatalogueError::Duplicate {
                kind: descriptor.kind,
                version: descriptor.version,
            });
        }

        versions.insert(descriptor.version, descriptor);
        Ok(())
    }

    pub fn get(&self, kind: &str, version: u16) -> Option<&ConnectorDescriptor> {
        self.descriptors.get(kind)?.get(&version)
    }

    /// Every descriptor, in no particular order. The action picker and the
    /// public connector documentation are both built from this.
    pub fn descriptors(&self) -> impl Iterator<Item = &ConnectorDescriptor> {
        self.descriptors.values().flat_map(HashMap::values)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ConnectorCatalogueError {
    #[error("connector `{kind}` version {version} is already registered")]
    Duplicate { kind: &'static str, version: u16 },
}

const LOOP_FIELDS: &[Field] = &[Field {
    name: "items",
    label: "Éléments",
    required: true,
    kind: FieldKind::Json,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CONDITION_FIELDS: &[Field] = &[Field {
    name: "predicate",
    label: "Condition",
    required: true,
    kind: FieldKind::Text,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CONFIG_FIELDS: &[Field] = &[Field {
    name: "variables",
    label: "Variables",
    required: true,
    kind: FieldKind::Json,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CUSTOMER_CREATE_FIELDS: &[Field] = &[
    Field {
        name: "name",
        label: "Nom",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const HTTP_REQUEST_FIELDS: &[Field] = &[
    Field {
        name: "method",
        label: "Méthode",
        required: true,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "GET",
                    label: "GET",
                },
                SelectOption {
                    value: "POST",
                    label: "POST",
                },
                SelectOption {
                    value: "PUT",
                    label: "PUT",
                },
                SelectOption {
                    value: "PATCH",
                    label: "PATCH",
                },
                SelectOption {
                    value: "DELETE",
                    label: "DELETE",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "url",
        label: "URL",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "headers",
        label: "En-têtes",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "body",
        label: "Corps",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "timeout_seconds",
        label: "Délai d'expiration (secondes)",
        required: false,
        kind: FieldKind::Number,
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "signing_credential_id",
        label: "Clé de signature sortante",
        required: false,
        kind: FieldKind::Text,
        expression: false,
        secret: false,
        visible_when: None,
    },
];

const ODOO_CREATE_PARTNER_FIELDS: &[Field] = &[
    Field {
        name: "name",
        label: "Nom",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const ODOO_UPDATE_PARTNER_FIELDS: &[Field] = &[
    Field {
        name: "partner_id",
        label: "Identifiant du partenaire",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "name",
        label: "Nom",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const ODOO_CREATE_INVOICE_FIELDS: &[Field] = &[
    Field {
        name: "partner_id",
        label: "Identifiant du partenaire",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "description",
        label: "Libellé de la ligne",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "amount",
        label: "Montant",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

/// Assembles the catalogue. Single point of extension: a module contributes
/// a line here, never a change to the backbone.
pub fn connector_catalogue() -> ConnectorCatalogue {
    let mut catalogue = ConnectorCatalogue::new();

    for descriptor in descriptors() {
        catalogue
            .register(descriptor)
            .expect("the catalogue is built from static descriptors, so a clash is a bug");
    }

    catalogue
}

fn descriptors() -> Vec<ConnectorDescriptor> {
    let mut all = flow_descriptors();
    all.extend(customer_descriptors());
    all.extend(http_descriptors());
    all.extend(odoo_descriptors());
    all.extend(task_recurrence_descriptors());
    all
}

/// The flow-control connectors: they act on the graph itself rather than an
/// external system, so they need no authentication and are declared here
/// instead of being owned by a bounded context.
fn flow_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "flow.loop",
            version: 1,
            family: "flow",
            label: "Pour chaque",
            auth: AuthRequirement::None,
            fields: LOOP_FIELDS,
            branches: &[Branch::Each, Branch::After],
            output_example: json!({ "item": "…", "index": 0 }),
        },
        ConnectorDescriptor {
            kind: "flow.condition",
            version: 1,
            family: "flow",
            label: "Condition",
            auth: AuthRequirement::None,
            fields: CONDITION_FIELDS,
            branches: &[Branch::Then, Branch::Else],
            output_example: json!({ "matched": true }),
        },
        ConnectorDescriptor {
            kind: "flow.config",
            version: 1,
            family: "flow",
            label: "Configuration",
            auth: AuthRequirement::None,
            fields: CONFIG_FIELDS,
            branches: &[],
            output_example: json!({ "customer_id": "…", "retry_limit": 3 }),
        },
    ]
}

/// The one internal connector proved by the run engine (#200): it calls
/// Mestier's own `create_customer` use case, so it needs no credential of
/// its own — the run already acts inside this instance.
fn customer_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "mestier.customer.create",
        version: 1,
        family: "mestier",
        label: "Créer un client",
        auth: AuthRequirement::None,
        fields: CUSTOMER_CREATE_FIELDS,
        branches: &[],
        output_example: json!({ "id": "…", "name": "…" }),
    }]
}

/// The one network connector proved by #202: an outbound call to wherever
/// the resolved `url` points, authenticated with any of three schemes and
/// optionally signed with a second, distinct credential
/// (`signing_credential_id`). `credential_id` stays `Option` — an
/// unauthenticated call is a real, supported case.
fn http_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "http.request",
        version: 1,
        family: "http",
        label: "Requête HTTP",
        auth: AuthRequirement::Optional(&["bearer_token", "http_basic", "http_header"]),
        fields: HTTP_REQUEST_FIELDS,
        branches: &[],
        output_example: json!({
            "status": 200,
            "headers": { "content-type": "application/json" },
            "body": {},
        }),
    }]
}

/// The three Odoo actions #202 ships: each a typed envelope over the same
/// authenticate-then-`execute_kw` HTTP call `http.request` could make by
/// hand — named fields and an imposed `odoo_api` credential in exchange for
/// giving up the generality.
fn odoo_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "odoo.create_partner",
            version: 1,
            family: "odoo",
            label: "Créer un partenaire Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_CREATE_PARTNER_FIELDS,
            branches: &[],
            output_example: json!({ "id": 42 }),
        },
        ConnectorDescriptor {
            kind: "odoo.update_partner",
            version: 1,
            family: "odoo",
            label: "Mettre à jour un partenaire Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_UPDATE_PARTNER_FIELDS,
            branches: &[],
            output_example: json!({ "id": 42, "updated": true }),
        },
        ConnectorDescriptor {
            kind: "odoo.create_invoice",
            version: 1,
            family: "odoo",
            label: "Créer une facture Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_CREATE_INVOICE_FIELDS,
            branches: &[],
            output_example: json!({ "id": 99 }),
        },
    ]
}

/// The one connector the horizon-extension pass (#293) runs: no fields, no
/// credential — it reads `org_id` off the run itself and extends whatever
/// that organization's recurrences need, the same "calls an existing use
/// case, so every business rule already enforced there applies here too"
/// shape as `mestier.customer.create`. Placed by
/// `MestierUseCase::find_or_create_recurrence_horizon_workflow`, never by a
/// human dragging it into a graph — but described here anyway, like every
/// other kind, so the anti-drift test in `infrastructure::automation::connectors`
/// still catches an implementation with no matching descriptor or the
/// reverse.
fn task_recurrence_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "mestier.task_recurrence.extend_horizon",
        version: 1,
        family: "mestier",
        label: "Étendre l'horizon de récurrence",
        auth: AuthRequirement::None,
        fields: &[],
        branches: &[],
        output_example: json!({ "materialized": 0 }),
    }]
}

#[cfg(test)]
mod tests {
    use serde_json::to_value;

    use super::*;
    use crate::domain::automation::connector::auth_scheme;

    fn descriptor(kind: &'static str, version: u16) -> ConnectorDescriptor {
        ConnectorDescriptor {
            kind,
            version,
            family: "flow",
            label: "Loop",
            auth: AuthRequirement::None,
            fields: &[],
            branches: &[],
            output_example: json!({ "index": 0 }),
        }
    }

    #[test]
    fn a_registered_descriptor_can_be_read_back() {
        let mut catalogue = ConnectorCatalogue::new();

        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("first registration succeeds");

        assert_eq!(
            catalogue.get("flow.loop", 1),
            Some(&descriptor("flow.loop", 1))
        );
    }

    #[test]
    fn registering_the_same_kind_and_version_twice_fails() {
        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("first registration succeeds");

        let result = catalogue.register(descriptor("flow.loop", 1));

        assert_eq!(
            result,
            Err(ConnectorCatalogueError::Duplicate {
                kind: "flow.loop",
                version: 1
            })
        );
    }

    #[test]
    fn two_versions_of_the_same_kind_coexist() {
        let mut catalogue = ConnectorCatalogue::new();

        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("version 1 registers");
        catalogue
            .register(descriptor("flow.loop", 2))
            .expect("version 2 registers alongside version 1");

        assert!(catalogue.get("flow.loop", 1).is_some());
        assert!(catalogue.get("flow.loop", 2).is_some());
    }

    #[test]
    fn an_unknown_kind_reads_back_as_none() {
        let catalogue = ConnectorCatalogue::new();

        assert_eq!(catalogue.get("flow.loop", 1), None);
    }

    #[test]
    fn the_catalogue_contains_the_flow_connectors() {
        let catalogue = connector_catalogue();

        assert!(catalogue.get("flow.loop", 1).is_some());
        assert!(catalogue.get("flow.condition", 1).is_some());
        assert!(catalogue.get("flow.config", 1).is_some());
    }

    #[test]
    fn the_catalogue_contains_the_flow_config_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("flow.config", 1)
            .expect("flow.config is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.branches.is_empty());
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "variables" && f.required && f.expression)
        );
    }

    /// The one internal connector the run engine (#200) proves itself
    /// against, described here so the anti-drift test in
    /// `infrastructure::automation::connectors` has something to match its
    /// implementation against.
    #[test]
    fn the_catalogue_contains_the_customer_create_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.customer.create", 1)
            .expect("mestier.customer.create is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "name" && f.required)
        );
    }

    /// The connector the horizon-extension pass (#293) drives — no fields,
    /// no credential, since it acts on the run's own `org_id`.
    #[test]
    fn the_catalogue_contains_the_recurrence_horizon_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.task_recurrence.extend_horizon", 1)
            .expect("mestier.task_recurrence.extend_horizon is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.fields.is_empty());
    }

    /// `http.request`'s auth stays optional: `credential_id` is `Option`, so
    /// an unauthenticated call is a supported case, not an oversight.
    #[test]
    fn the_catalogue_contains_the_http_request_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("http.request", 1)
            .expect("http.request is described");
        assert_eq!(
            descriptor.auth,
            AuthRequirement::Optional(&["bearer_token", "http_basic", "http_header"])
        );
        assert!(descriptor.auth.is_satisfied_without_a_credential());
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "url" && f.required && f.expression)
        );
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "method" && f.required)
        );
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "signing_credential_id" && !f.required),
            "signing is a second, distinct credential slot from `credential_id`"
        );
    }

    /// Each Odoo action imposes exactly one auth scheme — no `AnyOf`, unlike
    /// `http.request` — and a credential-bearing model action requires the
    /// fields that identify what it acts on.
    #[test]
    fn the_catalogue_contains_the_three_odoo_connectors() {
        let catalogue = connector_catalogue();

        for kind in [
            "odoo.create_partner",
            "odoo.update_partner",
            "odoo.create_invoice",
        ] {
            let descriptor = catalogue
                .get(kind, 1)
                .unwrap_or_else(|| panic!("{kind} is described"));
            assert_eq!(descriptor.auth, AuthRequirement::Exactly("odoo_api"));
            assert_eq!(descriptor.family, "odoo");
        }

        let create_partner = catalogue.get("odoo.create_partner", 1).unwrap();
        assert!(
            create_partner
                .fields
                .iter()
                .any(|f| f.name == "name" && f.required)
        );

        let update_partner = catalogue.get("odoo.update_partner", 1).unwrap();
        assert!(
            update_partner
                .fields
                .iter()
                .any(|f| f.name == "partner_id" && f.required)
        );

        let create_invoice = catalogue.get("odoo.create_invoice", 1).unwrap();
        assert!(
            create_invoice
                .fields
                .iter()
                .any(|f| f.name == "amount" && f.required)
        );
    }

    /// The check that keeps a typo in an `AuthRequirement` from reaching
    /// production: a scheme named here but absent from `auth_schemes()`
    /// would otherwise only surface once someone tries to attach a
    /// credential to the connector.
    #[test]
    fn every_scheme_named_by_the_catalogue_is_a_known_auth_scheme() {
        let catalogue = connector_catalogue();

        for descriptor in catalogue.descriptors() {
            for scheme_kind in descriptor.auth.scheme_kinds() {
                assert!(
                    auth_scheme(scheme_kind).is_some(),
                    "`{}` requires unknown auth scheme `{scheme_kind}`",
                    descriptor.kind
                );
            }
        }
    }

    #[test]
    fn every_descriptor_carries_an_output_example() {
        for descriptor in connector_catalogue().descriptors() {
            assert!(
                !descriptor.output_example.is_null(),
                "`{}` has no output example, so nothing documents its shape",
                descriptor.kind
            );
        }
    }

    /// The catalogue leaves the crate as JSON (#203): both registered
    /// versions of a kind must survive the trip.
    #[test]
    fn the_catalogue_serializes_every_registered_descriptor() {
        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("version 1 registers");
        catalogue
            .register(descriptor("flow.loop", 2))
            .expect("version 2 registers");

        let json = to_value(&catalogue).expect("catalogue serializes");

        assert_eq!(json["descriptors"]["flow.loop"]["1"]["version"], 1);
        assert_eq!(json["descriptors"]["flow.loop"]["2"]["version"], 2);
    }
}
