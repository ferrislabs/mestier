use events::{EventCatalogue, EventDescriptor};

use crate::domain::{
    absence, assignment_report, chat, customer, deal, extra_period, invitation, invoice,
    knowledge::tables, maintenance, member, quote, supplier_invoice, support, survey, time_entry,
};

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
    [
        quote::events::descriptors(),
        invoice::events::descriptors(),
        member::events::descriptors(),
        invitation::events::descriptors(),
        time_entry::events::descriptors(),
        assignment_report::events::descriptors(),
        maintenance::events::descriptors(),
        supplier_invoice::events::descriptors(),
        support::events::descriptors(),
        customer::events::descriptors(),
        deal::events::descriptors(),
        absence::events::descriptors(),
        extra_period::events::descriptors(),
        chat::events::descriptors(),
        tables::events::descriptors(),
        survey::events::descriptors(),
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use events::testing::{EventKey, drift};

    use super::*;

    /// The check that keeps explicit emission honest.
    ///
    /// Explicit emission buys a documented, stable contract; the price is that
    /// nothing forces a module to keep the two in step. Each module's
    /// `emitted_events` enumerates every `DomainEvent` it can construct, so an
    /// event added without a descriptor — or a descriptor left behind by an
    /// event that no longer exists — fails here rather than reaching a
    /// subscriber.
    #[test]
    fn the_catalogue_describes_exactly_what_the_modules_emit() {
        let emitted: Vec<EventKey> = [
            quote::events::emitted_events(),
            invoice::events::emitted_events(),
            member::events::emitted_events(),
            invitation::events::emitted_events(),
            time_entry::events::emitted_events(),
            assignment_report::events::emitted_events(),
            maintenance::events::emitted_events(),
            supplier_invoice::events::emitted_events(),
            support::events::emitted_events(),
            customer::events::emitted_events(),
            deal::events::emitted_events(),
            absence::events::emitted_events(),
            extra_period::events::emitted_events(),
            chat::events::emitted_events(),
            tables::events::emitted_events(),
            survey::events::emitted_events(),
        ]
        .concat()
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

    #[test]
    fn the_catalogue_lists_the_absence_and_extra_period_events() {
        let catalogue = event_catalogue();
        let names: Vec<&str> = catalogue
            .descriptors()
            .map(|descriptor| descriptor.name)
            .collect();

        for expected in [
            "absence.requested",
            "absence.approved",
            "absence.rejected",
            "absence.cancelled",
            "extra_period.requested",
            "extra_period.approved",
            "extra_period.rejected",
            "extra_period.cancelled",
        ] {
            assert!(names.contains(&expected), "`{expected}` is not catalogued");
        }
    }
}
