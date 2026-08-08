use events::{EventCatalogue, EventDescriptor};

use crate::domain::quote;

/// Every event the product can emit, assembled from the modules that own them.
///
/// A module contributes its descriptors here and nowhere else; adding one is a
/// line in this function, never a change to the backbone.
pub fn event_catalogue() -> EventCatalogue {
    let mut catalogue = EventCatalogue::new();

    for descriptor in descriptors() {
        catalogue
            .register(descriptor)
            .expect("the catalogue is built from static descriptors, so a clash is a bug");
    }

    catalogue
}

fn descriptors() -> Vec<EventDescriptor> {
    quote::events::descriptors()
}

#[cfg(test)]
mod tests {
    use events::testing::{EventKey, drift};

    use super::*;

    /// The check that keeps explicit emission honest.
    ///
    /// Explicit emission buys a documented, stable contract; the price is that
    /// nothing forces a module to keep the two in step. `emitted_events`
    /// enumerates every `DomainEvent` the quote module can construct, so an
    /// event added without a descriptor — or a descriptor left behind by an
    /// event that no longer exists — fails here rather than reaching a
    /// subscriber.
    #[test]
    fn the_catalogue_describes_exactly_what_the_modules_emit() {
        let emitted: Vec<EventKey> = quote::events::emitted_events()
            .iter()
            .map(|(name, version)| EventKey::new(*name, *version))
            .collect();

        let report = drift(&event_catalogue(), &emitted);

        assert!(report.is_clean(), "{report:#?}");
    }

    #[test]
    fn every_descriptor_carries_a_payload_example() {
        for descriptor in event_catalogue().descriptors() {
            assert!(
                !descriptor.payload_example.is_null(),
                "`{}` has no payload example, so nothing documents its shape",
                descriptor.name
            );
        }
    }
}
