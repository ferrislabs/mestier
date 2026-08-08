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
    fn a_subscription_to_a_known_event_is_accepted() {
        let catalogue = event_catalogue();

        assert!(validate_event_names(&["quote.accepted".to_owned()], &catalogue).is_ok());
    }

    /// A typo would otherwise create an endpoint that looks healthy and never
    /// fires — the worst kind of failure to debug.
    #[test]
    fn a_typo_is_refused_and_named() {
        let catalogue = event_catalogue();

        let error = validate_event_names(&["quote.acepted".to_owned()], &catalogue)
            .expect_err("a typo must not be accepted");

        assert!(format!("{error:?}").contains("quote.acepted"), "{error:?}");
    }

    #[test]
    fn a_subscription_to_nothing_is_refused() {
        assert!(validate_event_names(&[], &event_catalogue()).is_err());
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

/// Refuses a subscription to an event the product cannot emit.
///
/// A free-text event name is a silent dead end: the endpoint is created, looks
/// healthy, and never fires. Checking against the catalogue turns a typo into
/// an error at the moment it is made.
pub fn validate_event_names(
    names: &[String],
    catalogue: &EventCatalogue,
) -> Result<(), common::CoreError> {
    if names.is_empty() {
        return Err(common::CoreError::Conflict(
            "a subscription with no event listens to nothing".to_owned(),
        ));
    }

    let unknown: Vec<&str> = names
        .iter()
        .filter(|name| !catalogue.descriptors().any(|d| d.name == name.as_str()))
        .map(String::as_str)
        .collect();

    if !unknown.is_empty() {
        return Err(common::CoreError::Conflict(format!(
            "unknown event names: {}",
            unknown.join(", ")
        )));
    }

    Ok(())
}
