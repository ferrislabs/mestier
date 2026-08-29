use common::{CoreError, OrganizationId};
use events::EventCatalogue;
use uuid::Uuid;

use crate::domain::automation::ports::SubscriptionRepository;

/// The full desired set of event names for a workflow's trigger — always a
/// replacement of whatever is currently selected, never an addition to it.
/// An empty list clears the trigger entirely: the workflow then runs from
/// nothing until a selection is made again.
#[derive(Debug, Clone, PartialEq)]
pub struct SetWorkflowTriggerCommand {
    pub org_id: OrganizationId,
    pub workflow_id: Uuid,
    pub event_names: Vec<String>,
}

/// Refuses any name the event catalogue does not know, before a single
/// repository call is made. An event name from a stale trigger picker, or a
/// hand-crafted request, must never reach `automation.subscription`
/// unvalidated: the dispatcher would join against it forever without ever
/// matching, a silent no-op indistinguishable from a bug (see #225).
fn validate_trigger_event_names(
    catalogue: &EventCatalogue,
    event_names: &[String],
) -> Result<(), CoreError> {
    for name in event_names {
        if !catalogue
            .descriptors()
            .any(|descriptor| descriptor.name == name)
        {
            return Err(CoreError::Conflict(format!(
                "`{name}` is not a known automation event"
            )));
        }
    }
    Ok(())
}

/// Validates, then replaces a workflow's trigger — the write half of #225.
///
/// Generic over [`SubscriptionRepository`] so this is unit-testable against
/// a mock without a database (see the tests below).
/// `application::automation::subscription::MestierUseCase::set_workflow_trigger`
/// wraps this in the transaction that also checks the workflow itself
/// belongs to `org_id`, which this function does not and cannot: it has no
/// way to look a workflow up.
pub async fn set_workflow_trigger<R>(
    repository: &mut R,
    catalogue: &EventCatalogue,
    org_id: OrganizationId,
    workflow_id: Uuid,
    event_names: Vec<String>,
) -> Result<Vec<String>, CoreError>
where
    R: SubscriptionRepository,
{
    validate_trigger_event_names(catalogue, &event_names)?;
    repository
        .set_workflow_trigger(org_id, workflow_id, &event_names)
        .await
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use events::EventDescriptor;
    use serde_json::json;

    use super::*;
    use crate::domain::automation::ports::MockSubscriptionRepository;

    fn catalogue() -> EventCatalogue {
        let mut catalogue = EventCatalogue::new();
        catalogue
            .register(EventDescriptor {
                name: "quote.accepted",
                version: 1,
                label: "Quote accepted",
                subject_kind: "quote",
                payload_example: json!({}),
            })
            .unwrap();
        catalogue
    }

    #[tokio::test]
    async fn an_unknown_event_name_is_rejected_before_the_repository_is_called() {
        let org_id = OrganizationId(generate_uuid_v7());
        let workflow_id = generate_uuid_v7();
        // No expectation configured: the mock panics if it is ever called,
        // which is what proves the repository is never reached.
        let mut repository = MockSubscriptionRepository::new();

        let error = set_workflow_trigger(
            &mut repository,
            &catalogue(),
            org_id,
            workflow_id,
            vec!["not.a.real.event".to_string()],
        )
        .await
        .expect_err("an event name outside the catalogue must be refused");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn one_unknown_name_among_known_ones_still_refuses_the_whole_selection() {
        let org_id = OrganizationId(generate_uuid_v7());
        let workflow_id = generate_uuid_v7();
        let mut repository = MockSubscriptionRepository::new();

        let error = set_workflow_trigger(
            &mut repository,
            &catalogue(),
            org_id,
            workflow_id,
            vec!["quote.accepted".to_string(), "not.a.real.event".to_string()],
        )
        .await
        .expect_err("one bad name spoils the whole batch");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn every_known_event_name_reaches_the_repository_exactly_once() {
        let org_id = OrganizationId(generate_uuid_v7());
        let workflow_id = generate_uuid_v7();
        let mut repository = MockSubscriptionRepository::new();
        repository
            .expect_set_workflow_trigger()
            .times(1)
            .withf(move |o, w, names: &[String]| {
                *o == org_id && *w == workflow_id && names == ["quote.accepted"]
            })
            .returning(|_, _, names| {
                let names = names.to_vec();
                Box::pin(async move { Ok(names) })
            });

        let result = set_workflow_trigger(
            &mut repository,
            &catalogue(),
            org_id,
            workflow_id,
            vec!["quote.accepted".to_string()],
        )
        .await
        .unwrap();

        assert_eq!(result, vec!["quote.accepted".to_string()]);
    }

    #[tokio::test]
    async fn an_empty_selection_is_valid_and_forwarded_to_clear_the_trigger() {
        let org_id = OrganizationId(generate_uuid_v7());
        let workflow_id = generate_uuid_v7();
        let mut repository = MockSubscriptionRepository::new();
        repository
            .expect_set_workflow_trigger()
            .times(1)
            .withf(|_, _, names: &[String]| names.is_empty())
            .returning(|_, _, _| Box::pin(async move { Ok(Vec::new()) }));

        let result = set_workflow_trigger(
            &mut repository,
            &catalogue(),
            org_id,
            workflow_id,
            Vec::new(),
        )
        .await
        .unwrap();

        assert!(result.is_empty());
    }
}
