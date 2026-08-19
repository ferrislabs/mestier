use chrono::{Datelike, Utc};
use common::{CoreError, generate_uuid_v7};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use events::EventEmitter;
use serde_json::{Value, json};

use crate::{
    OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineId,
    domain::quote::{
        commands::{
            CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
        },
        events::{QuoteCreated, QuoteDeleted, QuoteTransitioned, QuoteUpdated},
        ports::QuoteRepository,
    },
};

pub struct QuoteService<R, E>
where
    R: QuoteRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> QuoteService<R, E>
where
    R: QuoteRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create_quote(&mut self, command: CreateQuoteCommand) -> Result<Quote, CoreError> {
        let now = Utc::now();
        let quote_id = QuoteId(generate_uuid_v7());
        validate_title(&command.title)?;
        let reference = self
            .repo
            .next_reference(command.organization_id, now.year())
            .await?;
        let lines = build_quote_lines(command.organization_id, quote_id, command.lines, now)?;
        let total_cents = calculate_total_cents(&lines)?;

        let created = self
            .repo
            .insert(&Quote {
                id: quote_id,
                organization_id: command.organization_id,
                reference,
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                status: crate::QuoteStatus::Draft,
                total_cents,
                lines,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.emitter.emit(
            created.organization_id,
            &QuoteCreated {
                quote: created.clone(),
            },
        )?;

        Ok(created)
    }

    pub async fn get_quote(&mut self, id: QuoteId) -> Result<Quote, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_quotes(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_quote(&mut self, command: UpdateQuoteCommand) -> Result<Quote, CoreError> {
        let existing = self.get_quote(command.id).await?;
        let now = Utc::now();
        validate_title(&command.title)?;
        let lines = build_quote_lines(existing.organization_id, existing.id, command.lines, now)?;
        let total_cents = calculate_total_cents(&lines)?;

        let updated = self
            .repo
            .update(&Quote {
                id: existing.id,
                organization_id: existing.organization_id,
                reference: existing.reference.clone(),
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                status: command.status,
                total_cents,
                lines,
                deleted_at: existing.deleted_at,
                created_at: existing.created_at,
                updated_at: now,
            })
            .await?;

        // Content first, then the transition. The two perimeters never
        // overlap: `changed_fields` cannot contain the status, and the
        // transition carries nothing about the content.
        let (changed_fields, previous) = content_diff(&existing, &updated);
        if !changed_fields.is_empty() {
            self.emitter.emit(
                updated.organization_id,
                &QuoteUpdated {
                    quote: updated.clone(),
                    changed_fields,
                    previous,
                },
            )?;
        }
        self.emit_transition(existing.status, &updated)?;

        Ok(updated)
    }

    pub async fn update_quote_status(
        &mut self,
        command: UpdateQuoteStatusCommand,
    ) -> Result<Quote, CoreError> {
        let existing = self.get_quote(command.id).await?;
        let updated = self
            .repo
            .update_status(command.id, command.status, Utc::now())
            .await?;

        self.emit_transition(existing.status, &updated)?;

        Ok(updated)
    }

    /// Emits the named event for a landed transition. Silent when the status
    /// did not move, and when the destination has no name the product uses.
    fn emit_transition(&self, from: crate::QuoteStatus, quote: &Quote) -> Result<(), CoreError> {
        if from == quote.status || QuoteTransitioned::event_name(quote.status).is_none() {
            return Ok(());
        }

        self.emitter.emit(
            quote.organization_id,
            &QuoteTransitioned {
                quote: quote.clone(),
                from,
            },
        )
    }

    pub async fn soft_delete_quote(&mut self, id: QuoteId) -> Result<(), CoreError> {
        let existing = self.get_quote(id).await?;
        self.repo.soft_delete(id, Utc::now()).await?;

        self.emitter
            .emit(existing.organization_id, &QuoteDeleted { quote_id: id })
    }
}

/// Which content fields moved, and what they held before.
///
/// `status` is absent by construction: a status change is a transition, and
/// reporting it here would make the two perimeters overlap.
fn content_diff(existing: &Quote, updated: &Quote) -> (Vec<&'static str>, Value) {
    let mut changed = Vec::new();
    let mut previous = serde_json::Map::new();

    if existing.title != updated.title {
        changed.push("title");
        previous.insert("title".to_owned(), json!(existing.title));
    }
    if existing.customer_id != updated.customer_id {
        changed.push("customer_id");
        previous.insert("customer_id".to_owned(), json!(existing.customer_id.0));
    }
    if existing.customer_context_id != updated.customer_context_id {
        changed.push("customer_context_id");
        previous.insert(
            "customer_context_id".to_owned(),
            json!(existing.customer_context_id.0),
        );
    }
    if line_projection(existing) != line_projection(updated) {
        changed.push("lines");
        previous.insert("lines".to_owned(), line_projection(existing));
        previous.insert("total_cents".to_owned(), json!(existing.total_cents));
    }

    (changed, Value::Object(previous))
}

/// Lines compared on what a subscriber can observe, not on their ids: a line
/// rebuilt with the same content is not a change worth waking an automation.
fn line_projection(quote: &Quote) -> Value {
    json!(
        quote
            .lines
            .iter()
            .map(|line| json!({
                "label": line.label,
                "quantity": line.quantity.to_string(),
                "unit": line.unit,
                "unit_price_cents": line.unit_price_cents,
                "notes": line.notes,
                "photo_keys": line.photo_keys,
            }))
            .collect::<Vec<_>>()
    )
}

fn validate_title(title: &str) -> Result<(), CoreError> {
    if title.trim().is_empty() {
        return Err(CoreError::Conflict(
            "quote title cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn build_quote_lines(
    organization_id: OrganizationId,
    quote_id: QuoteId,
    commands: Vec<QuoteLineCommand>,
    now: chrono::DateTime<Utc>,
) -> Result<Vec<QuoteLine>, CoreError> {
    commands
        .into_iter()
        .map(|command| build_quote_line(organization_id, quote_id, command, now))
        .collect()
}

fn build_quote_line(
    organization_id: OrganizationId,
    quote_id: QuoteId,
    command: QuoteLineCommand,
    now: chrono::DateTime<Utc>,
) -> Result<QuoteLine, CoreError> {
    validate_line(&command)?;

    Ok(QuoteLine {
        id: QuoteLineId(generate_uuid_v7()),
        organization_id,
        quote_id,
        service_rate_id: command.service_rate_id,
        label: command.label,
        quantity: command.quantity,
        unit: command.unit,
        unit_price_cents: command.unit_price_cents,
        notes: command.notes,
        photo_keys: command.photo_keys,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn validate_line(command: &QuoteLineCommand) -> Result<(), CoreError> {
    if command.label.trim().is_empty() {
        return Err(CoreError::Conflict(
            "quote line label cannot be empty".to_owned(),
        ));
    }

    if command.quantity <= Decimal::ZERO {
        return Err(CoreError::Conflict(
            "quote line quantity must be positive".to_owned(),
        ));
    }

    if command.unit_price_cents < 0 {
        return Err(CoreError::Conflict(
            "quote line unit price cannot be negative".to_owned(),
        ));
    }

    if command
        .notes
        .as_ref()
        .is_some_and(|notes| notes.trim().is_empty())
    {
        return Err(CoreError::Conflict(
            "quote line notes cannot be empty when present".to_owned(),
        ));
    }

    if command.photo_keys.iter().any(|key| key.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "quote line photo keys cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn calculate_total_cents(lines: &[QuoteLine]) -> Result<i32, CoreError> {
    let total = lines.iter().try_fold(0_i64, |sum, line| {
        let line_total = (line.quantity * Decimal::from(line.unit_price_cents)).round_dp(0);
        let line_total = line_total.to_i64().ok_or_else(|| {
            CoreError::Conflict("quote line total is outside supported bounds".to_owned())
        })?;

        sum.checked_add(line_total).ok_or_else(|| {
            CoreError::Conflict("quote total is outside supported bounds".to_owned())
        })
    })?;

    i32::try_from(total)
        .map_err(|_| CoreError::Conflict("quote total is outside supported bounds".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CustomerContextId, CustomerId, QuoteStatus, ServiceRateUnit,
        domain::quote::ports::MockQuoteRepository,
    };
    use mockall::predicate::eq;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    pub(super) fn line_command(quantity: Decimal, unit_price_cents: i32) -> QuoteLineCommand {
        QuoteLineCommand {
            service_rate_id: None,
            label: "Taille de haie".to_owned(),
            quantity,
            unit: ServiceRateUnit::Ml,
            unit_price_cents,
            notes: Some("Acces jardin".to_owned()),
            photo_keys: vec!["quotes/photo-1.jpg".to_owned()],
        }
    }

    pub(super) fn quote(id: QuoteId) -> Quote {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        Quote {
            id,
            organization_id,
            reference: "DEV-2026-0001".to_owned(),
            title: "Rénovation cuisine".to_owned(),
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            status: QuoteStatus::Draft,
            total_cents: 5500,
            lines: vec![QuoteLine {
                id: QuoteLineId(Uuid::new_v4()),
                organization_id,
                quote_id: id,
                service_rate_id: None,
                label: "Taille de haie".to_owned(),
                quantity: Decimal::new(1, 0),
                unit: ServiceRateUnit::Hour,
                unit_price_cents: 5500,
                notes: None,
                photo_keys: vec![],
                deleted_at: None,
                created_at: now,
                updated_at: now,
            }],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_quote_calculates_total_from_lines() {
        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let created = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                lines: vec![
                    line_command(Decimal::new(25, 1), 1200),
                    line_command(Decimal::new(1, 0), 500),
                ],
            })
            .await
            .unwrap();

        assert_eq!(created.status, QuoteStatus::Draft);
        assert_eq!(created.reference, "DEV-2026-0001");
        assert_eq!(created.title, "Rénovation cuisine");
        assert_eq!(created.total_cents, 3500);
    }

    #[tokio::test]
    async fn update_quote_recalculates_total() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_update().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let updated = service
            .update_quote(UpdateQuoteCommand {
                id,
                title: "Version ajustée".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                status: QuoteStatus::Sent,
                lines: vec![line_command(Decimal::new(3, 0), 2000)],
            })
            .await
            .unwrap();

        assert_eq!(updated.status, QuoteStatus::Sent);
        assert_eq!(updated.reference, "DEV-2026-0001");
        assert_eq!(updated.title, "Version ajustée");
        assert_eq!(updated.total_cents, 6000);
    }

    #[tokio::test]
    async fn update_quote_status_delegates_without_recalculating_lines() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_update_status()
            .withf(move |quote_id, status, _| *quote_id == id && *status == QuoteStatus::Accepted)
            .returning(move |_, _, _| Box::pin(async move { Ok(quote(id)) }));

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());

        service
            .update_quote_status(UpdateQuoteStatusCommand {
                id,
                status: QuoteStatus::Accepted,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn list_quotes_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(25), eq(50))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![quote(QuoteId(Uuid::new_v4()))], 1)) })
            });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let (items, total) = service.list_quotes(org_id, 25, 50).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_quote_checks_existence_then_deletes() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());

        service.soft_delete_quote(id).await.unwrap();
    }

    /// The same vectors the webapp pins in `pages/quotes/types.test.ts`.
    ///
    /// The draft total shown while typing is computed in the browser, so two
    /// implementations of one pricing rule exist. These are what stops them
    /// drifting: every product below lands exactly on a half cent, which is
    /// where the rounding mode decides the answer. `round_dp` rounds halves to
    /// even, so each one goes to the even neighbour rather than up. Change a
    /// number here and the webapp's copy has to change with it.
    #[test]
    fn products_landing_on_a_half_cent_round_to_even() {
        for (quantity, unit_price_cents, expected) in [
            ("631.9", 38_875, 24_565_112),
            ("1723.1", 5_455, 9_399_510),
            ("710.1", 13_485, 9_575_698),
            ("1274.7", 13_895, 17_711_956),
            ("1024.5", 37_525, 38_444_362),
            ("1744.9", 26_385, 46_039_186),
        ] {
            let line = QuoteLine {
                quantity: quantity.parse().expect("a decimal quantity"),
                unit_price_cents,
                ..quote(QuoteId(Uuid::new_v4())).lines.remove(0)
            };

            assert_eq!(
                calculate_total_cents(&[line]).expect("a total within bounds"),
                expected,
                "{quantity} x {unit_price_cents} cents"
            );
        }
    }

    #[tokio::test]
    async fn rejects_invalid_line_input() {
        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                lines: vec![line_command(Decimal::ZERO, 1000)],
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn rejects_empty_quote_title() {
        let repo = MockQuoteRepository::new();
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: " ".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                lines: vec![line_command(Decimal::new(1, 0), 1000)],
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}

#[cfg(test)]
mod emission_tests {
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use rust_decimal::Decimal;
    use serde_json::json;
    use uuid::Uuid;

    use super::tests::{line_command, quote};
    use super::*;
    use crate::{
        CustomerContextId, CustomerId, QuoteStatus, domain::quote::ports::MockQuoteRepository,
    };

    /// Mirrors the line the `quote` fixture holds, so a test that means to
    /// change only the title does not silently change the lines too.
    fn unchanged_line() -> QuoteLineCommand {
        QuoteLineCommand {
            service_rate_id: None,
            label: "Taille de haie".to_owned(),
            quantity: Decimal::new(1, 0),
            unit: crate::ServiceRateUnit::Hour,
            unit_price_cents: 5500,
            notes: None,
            photo_keys: vec![],
        }
    }

    fn update_command(
        id: QuoteId,
        title: &str,
        status: QuoteStatus,
        from: &Quote,
    ) -> UpdateQuoteCommand {
        UpdateQuoteCommand {
            id,
            title: title.to_owned(),
            customer_id: from.customer_id,
            customer_context_id: from.customer_context_id,
            status,
            lines: vec![unchanged_line()],
        }
    }

    #[tokio::test]
    async fn creating_a_quote_emits_created_and_nothing_else() {
        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        repo.expect_insert().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                lines: vec![line_command(Decimal::new(1, 0), 5500)],
            })
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["quote.created"]);
    }

    #[tokio::test]
    async fn a_content_change_reports_only_the_fields_that_moved() {
        let id = QuoteId(Uuid::new_v4());
        let existing = quote(id);
        let mut repo = MockQuoteRepository::new();
        let stored = existing.clone();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let q = stored.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        repo.expect_update().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .update_quote(update_command(
                id,
                "Nouveau titre",
                existing.status,
                &existing,
            ))
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["quote.updated"]);
        let payload = emitter.only("quote.updated").payload;
        assert_eq!(payload["changed_fields"], json!(["title"]));
        assert_eq!(payload["previous"]["title"], json!(existing.title));
    }

    #[tokio::test]
    async fn changing_a_line_is_reported_as_a_content_change() {
        let id = QuoteId(Uuid::new_v4());
        let existing = quote(id);
        let mut repo = MockQuoteRepository::new();
        let stored = existing.clone();
        repo.expect_find_by_id().returning(move |_| {
            let q = stored.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        repo.expect_update().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        let mut command = update_command(id, &existing.title, existing.status, &existing);
        command.lines = vec![line_command(Decimal::new(2, 0), 9000)];

        service.update_quote(command).await.unwrap();

        let payload = emitter.only("quote.updated").payload;
        assert_eq!(payload["changed_fields"], json!(["lines"]));
        assert_eq!(payload["previous"]["total_cents"], json!(5500));
    }

    /// The rule the whole taxonomy rests on: content and status never overlap.
    #[tokio::test]
    async fn a_write_touching_content_and_status_emits_both_on_disjoint_perimeters() {
        let id = QuoteId(Uuid::new_v4());
        let existing = quote(id);
        let mut repo = MockQuoteRepository::new();
        let stored = existing.clone();
        repo.expect_find_by_id().returning(move |_| {
            let q = stored.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        repo.expect_update().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .update_quote(update_command(
                id,
                "Nouveau titre",
                QuoteStatus::Sent,
                &existing,
            ))
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["quote.updated", "quote.sent"]);
        let updated = emitter.only("quote.updated").payload;
        assert_eq!(
            updated["changed_fields"],
            json!(["title"]),
            "the status change must not be reported as a content change"
        );
        let sent = emitter.only("quote.sent").payload;
        assert_eq!(sent["from"], json!("DRAFT"));
        assert_eq!(sent["to"], json!("SENT"));
    }

    #[tokio::test]
    async fn each_status_transition_has_its_own_event() {
        for (status, expected) in [
            (QuoteStatus::Sent, "quote.sent"),
            (QuoteStatus::Accepted, "quote.accepted"),
            (QuoteStatus::Declined, "quote.declined"),
            (QuoteStatus::Cancelled, "quote.cancelled"),
        ] {
            let id = QuoteId(Uuid::new_v4());
            let existing = quote(id);
            let mut repo = MockQuoteRepository::new();
            let stored = existing.clone();
            repo.expect_find_by_id().returning(move |_| {
                let q = stored.clone();
                Box::pin(async move { Ok(Some(q)) })
            });
            let mut landed = existing.clone();
            landed.status = status;
            repo.expect_update_status().returning(move |_, _, _| {
                let q = landed.clone();
                Box::pin(async move { Ok(q) })
            });
            let emitter = RecordingEmitter::new();
            let mut service = QuoteService::new(repo, &emitter);

            service
                .update_quote_status(UpdateQuoteStatusCommand { id, status })
                .await
                .unwrap();

            assert_eq!(emitter.names(), vec![expected]);
        }
    }

    #[tokio::test]
    async fn restating_the_current_status_emits_nothing() {
        let id = QuoteId(Uuid::new_v4());
        let existing = quote(id);
        let mut repo = MockQuoteRepository::new();
        let stored = existing.clone();
        repo.expect_find_by_id().returning(move |_| {
            let q = stored.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        let landed = existing.clone();
        repo.expect_update_status().returning(move |_, _, _| {
            let q = landed.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .update_quote_status(UpdateQuoteStatusCommand {
                id,
                status: existing.status,
            })
            .await
            .unwrap();

        assert!(emitter.names().is_empty(), "{:?}", emitter.names());
    }

    #[tokio::test]
    async fn deleting_a_quote_emits_deleted() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        let stored = quote(id);
        repo.expect_find_by_id().returning(move |_| {
            let q = stored.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        repo.expect_soft_delete()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service.soft_delete_quote(id).await.unwrap();

        assert_eq!(emitter.names(), vec!["quote.deleted"]);
    }

    #[tokio::test]
    async fn a_write_that_fails_emits_nothing() {
        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        repo.expect_insert()
            .returning(|_| Box::pin(async { Err(CoreError::Conflict("nope".into())) }));
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        let outcome = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                lines: vec![line_command(Decimal::new(1, 0), 5500)],
            })
            .await;

        assert!(outcome.is_err());
        assert!(emitter.names().is_empty());
    }
}
