use std::collections::HashMap;

use serde_json::Value;
use thiserror::Error;

/// What the product knows about one event, independently of any emission.
///
/// This is what will populate the trigger picker of the workflow editor and
/// what documents the public contract of the outbound webhooks. It is declared
/// alongside the event it describes, by the module that owns it.
#[derive(Debug, Clone, PartialEq)]
pub struct EventDescriptor {
    pub name: &'static str,
    pub version: u16,
    /// Human-readable, shown in the trigger picker.
    pub label: &'static str,
    pub subject_kind: &'static str,
    pub payload_example: Value,
}

#[derive(Debug, Error, PartialEq)]
pub enum CatalogueError {
    #[error("event `{name}` version {version} is already registered")]
    Duplicate { name: &'static str, version: u16 },
}

/// Every event the product can emit, keyed by name and version.
///
/// Two versions of the same event coexist on purpose: an incompatible payload
/// change increments the version instead of mutating the existing shape, so a
/// subscriber keeps receiving what it subscribed to.
#[derive(Debug, Default)]
pub struct EventCatalogue {
    /// Nested rather than keyed by `(name, version)`: lookups come from the
    /// database with a runtime `&str`, which cannot be turned into the
    /// `&'static str` half of a tuple key without scanning the whole map.
    descriptors: HashMap<&'static str, HashMap<u16, EventDescriptor>>,
}

impl EventCatalogue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, descriptor: EventDescriptor) -> Result<(), CatalogueError> {
        let versions = self.descriptors.entry(descriptor.name).or_default();

        if versions.contains_key(&descriptor.version) {
            return Err(CatalogueError::Duplicate {
                name: descriptor.name,
                version: descriptor.version,
            });
        }

        versions.insert(descriptor.version, descriptor);
        Ok(())
    }

    pub fn get(&self, name: &str, version: u16) -> Option<&EventDescriptor> {
        self.descriptors.get(name)?.get(&version)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn descriptor(name: &'static str, version: u16) -> EventDescriptor {
        EventDescriptor {
            name,
            version,
            label: "Quote accepted",
            subject_kind: "quote",
            payload_example: json!({ "quote_id": "…" }),
        }
    }

    #[test]
    fn a_registered_descriptor_can_be_read_back() {
        let mut catalogue = EventCatalogue::new();

        catalogue
            .register(descriptor("quote.accepted", 1))
            .expect("first registration succeeds");

        assert_eq!(
            catalogue.get("quote.accepted", 1),
            Some(&descriptor("quote.accepted", 1))
        );
    }

    #[test]
    fn registering_the_same_name_and_version_twice_fails() {
        let mut catalogue = EventCatalogue::new();
        catalogue
            .register(descriptor("quote.accepted", 1))
            .expect("first registration succeeds");

        let result = catalogue.register(descriptor("quote.accepted", 1));

        assert_eq!(
            result,
            Err(CatalogueError::Duplicate {
                name: "quote.accepted",
                version: 1
            })
        );
    }

    #[test]
    fn two_versions_of_the_same_event_coexist() {
        let mut catalogue = EventCatalogue::new();

        catalogue
            .register(descriptor("quote.accepted", 1))
            .expect("version 1 registers");
        catalogue
            .register(descriptor("quote.accepted", 2))
            .expect("version 2 registers alongside version 1");

        assert!(catalogue.get("quote.accepted", 1).is_some());
        assert!(catalogue.get("quote.accepted", 2).is_some());
    }

    #[test]
    fn an_unknown_event_reads_back_as_none() {
        let catalogue = EventCatalogue::new();

        assert_eq!(catalogue.get("quote.accepted", 1), None);
    }
}
