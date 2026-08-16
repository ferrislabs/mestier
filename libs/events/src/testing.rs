//! Test utilities for the modules that emit events.
//!
//! Compiled unconditionally so a consumer crate can use it from its own test
//! suite. Nothing in here is meant to run in production.

use std::collections::BTreeSet;
use std::sync::Mutex;

use common::{CoreError, OrganizationId};

use crate::{Actor, DomainEvent, EmissionContext, EventCatalogue, EventEmitter, EventEnvelope};

/// An [`EventEmitter`] that keeps what it was given instead of persisting it.
///
/// Lets a domain service be tested for what it publishes without a database,
/// a transaction, or a mock whose expectations restate the implementation.
#[derive(Debug, Default)]
pub struct RecordingEmitter {
    recorded: Mutex<Vec<EventEnvelope>>,
}

impl RecordingEmitter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recorded(&self) -> Vec<EventEnvelope> {
        self.recorded.lock().expect("recorder lock").clone()
    }

    /// Event names in emission order — what most assertions actually care about.
    pub fn names(&self) -> Vec<String> {
        self.recorded()
            .into_iter()
            .map(|envelope| envelope.name)
            .collect()
    }

    pub fn only(&self, name: &str) -> EventEnvelope {
        let mut matching: Vec<_> = self
            .recorded()
            .into_iter()
            .filter(|envelope| envelope.name == name)
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "expected exactly one `{name}`, got {:?}",
            self.names()
        );
        matching.remove(0)
    }
}

impl EventEmitter for RecordingEmitter {
    fn emit<E: DomainEvent>(&self, org_id: OrganizationId, event: &E) -> Result<(), CoreError> {
        let envelope = EventEnvelope::from_event(
            event,
            &EmissionContext {
                org_id,
                actor: Actor::system(),
                correlation_id: None,
            },
        );
        self.recorded.lock().expect("recorder lock").push(envelope);
        Ok(())
    }
}

impl EventEmitter for &RecordingEmitter {
    fn emit<E: DomainEvent>(&self, org_id: OrganizationId, event: &E) -> Result<(), CoreError> {
        (*self).emit(org_id, event)
    }
}

/// One event, identified the way the catalogue keys it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventKey {
    pub name: String,
    pub version: u16,
}

impl EventKey {
    pub fn new(name: impl Into<String>, version: u16) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }
}

/// The two ways a catalogue can stop telling the truth.
///
/// Explicit emission buys a stable, documented contract; the price is that
/// nothing forces a module to keep the two in step. This report is what makes
/// the divergence a failing test instead of a support ticket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriftReport {
    /// Emitted by a module, absent from the catalogue.
    pub undocumented: Vec<EventKey>,
    /// Described in the catalogue, emitted by nobody.
    pub unemitted: Vec<EventKey>,
}

impl DriftReport {
    pub fn is_clean(&self) -> bool {
        self.undocumented.is_empty() && self.unemitted.is_empty()
    }
}

/// Compare what a module emits against what the catalogue describes.
///
/// Both sides of the report are sorted, so a failing assertion reads the same
/// way twice — map iteration order must not leak into a test's output.
pub fn drift(catalogue: &EventCatalogue, emitted: &[EventKey]) -> DriftReport {
    let documented: BTreeSet<EventKey> = catalogue
        .descriptors()
        .map(|descriptor| EventKey::new(descriptor.name, descriptor.version))
        .collect();
    let emitted: BTreeSet<EventKey> = emitted.iter().cloned().collect();

    DriftReport {
        undocumented: emitted.difference(&documented).cloned().collect(),
        unemitted: documented.difference(&emitted).cloned().collect(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::EventDescriptor;

    fn catalogue_of(entries: &[(&'static str, u16)]) -> EventCatalogue {
        let mut catalogue = EventCatalogue::new();
        for (name, version) in entries {
            catalogue
                .register(EventDescriptor {
                    name,
                    version: *version,
                    label: "…",
                    subject_kind: "quote",
                    payload_example: json!({}),
                })
                .expect("fixture registers");
        }
        catalogue
    }

    #[test]
    fn a_catalogue_matching_what_is_emitted_does_not_drift() {
        let catalogue = catalogue_of(&[("quote.accepted", 1), ("quote.sent", 1)]);

        let report = drift(
            &catalogue,
            &[
                EventKey::new("quote.accepted", 1),
                EventKey::new("quote.sent", 1),
            ],
        );

        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn an_emitted_event_with_no_descriptor_is_reported_undocumented() {
        let catalogue = catalogue_of(&[("quote.accepted", 1)]);

        let report = drift(
            &catalogue,
            &[
                EventKey::new("quote.accepted", 1),
                EventKey::new("quote.declined", 1),
            ],
        );

        assert_eq!(
            report.undocumented,
            vec![EventKey::new("quote.declined", 1)]
        );
        assert_eq!(report.unemitted, vec![]);
    }

    #[test]
    fn a_descriptor_nobody_emits_is_reported_unemitted() {
        let catalogue = catalogue_of(&[("quote.accepted", 1), ("quote.declined", 1)]);

        let report = drift(&catalogue, &[EventKey::new("quote.accepted", 1)]);

        assert_eq!(report.undocumented, vec![]);
        assert_eq!(report.unemitted, vec![EventKey::new("quote.declined", 1)]);
    }

    /// Bumping the emitted version without documenting it is the realistic
    /// failure: the name still matches, so a name-only comparison would pass.
    #[test]
    fn a_version_bump_left_undocumented_drifts_on_both_sides() {
        let catalogue = catalogue_of(&[("quote.accepted", 1)]);

        let report = drift(&catalogue, &[EventKey::new("quote.accepted", 2)]);

        assert_eq!(
            report.undocumented,
            vec![EventKey::new("quote.accepted", 2)]
        );
        assert_eq!(report.unemitted, vec![EventKey::new("quote.accepted", 1)]);
    }

    #[test]
    fn both_sides_of_the_report_are_sorted() {
        let catalogue = catalogue_of(&[("b.event", 1), ("a.event", 1)]);

        let report = drift(
            &catalogue,
            &[EventKey::new("d.event", 1), EventKey::new("c.event", 1)],
        );

        assert_eq!(
            report.undocumented,
            vec![EventKey::new("c.event", 1), EventKey::new("d.event", 1)]
        );
        assert_eq!(
            report.unemitted,
            vec![EventKey::new("a.event", 1), EventKey::new("b.event", 1)]
        );
    }
}
