use std::collections::HashMap;

use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use super::descriptor::{AuthRequirement, ConnectorDescriptor};
use super::field::{Field, FieldKind};

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
    label: "Items",
    required: true,
    kind: FieldKind::Json,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CONDITION_FIELDS: &[Field] = &[Field {
    name: "predicate",
    label: "Predicate",
    required: true,
    kind: FieldKind::Text,
    expression: true,
    secret: false,
    visible_when: None,
}];

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
    flow_descriptors()
}

/// The two flow-control connectors: they act on the graph itself rather than
/// an external system, so they need no authentication and are declared here
/// instead of being owned by a bounded context. Described now, executed in
/// #200 — two real cases are enough to prove the descriptor shape without
/// running anything.
fn flow_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "flow.loop",
            version: 1,
            family: "flow",
            label: "Loop",
            auth: AuthRequirement::None,
            fields: LOOP_FIELDS,
            output_example: json!({ "item": "…", "index": 0 }),
        },
        ConnectorDescriptor {
            kind: "flow.condition",
            version: 1,
            family: "flow",
            label: "Condition",
            auth: AuthRequirement::None,
            fields: CONDITION_FIELDS,
            output_example: json!({ "matched": true }),
        },
    ]
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
