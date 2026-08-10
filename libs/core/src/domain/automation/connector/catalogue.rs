use std::collections::HashMap;

use serde::Serialize;
use thiserror::Error;

use super::descriptor::ConnectorDescriptor;

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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::domain::automation::connector::descriptor::AuthRequirement;

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
}
