use chrono::{Datelike, Utc};
use common::{CoreError, generate_uuid_v7};
use rust_decimal::Decimal;

use crate::{
    OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineId,
    domain::{
        billing::{BillingLine, compute_totals},
        quote::{
            commands::{
                CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
            },
            ports::QuoteRepository,
        },
    },
};

pub struct QuoteService<R>
where
    R: QuoteRepository,
{
    repo: R,
}

impl<R> QuoteService<R>
where
    R: QuoteRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
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
        let (total_ht_cents, total_vat_cents, total_ttc_cents) =
            compute_quote_totals(&lines)?;
        let total_cents = total_ttc_cents; // spec invariant: total_cents mirrors TTC

        let quote = self
            .repo
            .insert(&Quote {
                id: quote_id,
                organization_id: command.organization_id,
                reference,
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                emitter_context_id: command.emitter_context_id,
                status: crate::QuoteStatus::Draft,
                deposit_basis: command.deposit_basis,
                deposit_value: command.deposit_value,
                total_cents,
                total_ht_cents,
                total_vat_cents,
                total_ttc_cents,
                lines,
                legal_mention_template_ids: vec![],
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.repo
            .replace_legal_mention_templates(&quote, &command.legal_mention_template_ids)
            .await?;

        Ok(Quote {
            legal_mention_template_ids: command.legal_mention_template_ids,
            ..quote
        })
    }

    pub async fn get_quote(&mut self, id: QuoteId) -> Result<Quote, CoreError> {
        let quote = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        let legal_mention_template_ids =
            self.repo.find_legal_mention_template_ids(id).await?;
        Ok(Quote {
            legal_mention_template_ids,
            ..quote
        })
    }

    pub async fn list_quotes(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        let (quotes, total) = self
            .repo
            .list_by_organization(organization_id, limit, offset)
            .await?;

        let mut enriched = Vec::with_capacity(quotes.len());
        for quote in quotes {
            let legal_mention_template_ids =
                self.repo.find_legal_mention_template_ids(quote.id).await?;
            enriched.push(Quote {
                legal_mention_template_ids,
                ..quote
            });
        }

        Ok((enriched, total))
    }

    pub async fn update_quote(&mut self, command: UpdateQuoteCommand) -> Result<Quote, CoreError> {
        let existing = self.get_quote(command.id).await?;
        let now = Utc::now();
        validate_title(&command.title)?;
        let lines = build_quote_lines(existing.organization_id, existing.id, command.lines, now)?;
        let (total_ht_cents, total_vat_cents, total_ttc_cents) =
            compute_quote_totals(&lines)?;
        let total_cents = total_ttc_cents; // spec invariant: total_cents mirrors TTC

        let quote = self
            .repo
            .update(&Quote {
                id: existing.id,
                organization_id: existing.organization_id,
                reference: existing.reference,
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                emitter_context_id: command.emitter_context_id,
                status: command.status,
                deposit_basis: command.deposit_basis,
                deposit_value: command.deposit_value,
                total_cents,
                total_ht_cents,
                total_vat_cents,
                total_ttc_cents,
                lines,
                legal_mention_template_ids: vec![],
                deleted_at: existing.deleted_at,
                created_at: existing.created_at,
                updated_at: now,
            })
            .await?;

        self.repo
            .replace_legal_mention_templates(&quote, &command.legal_mention_template_ids)
            .await?;

        Ok(Quote {
            legal_mention_template_ids: command.legal_mention_template_ids,
            ..quote
        })
    }

    pub async fn update_quote_status(
        &mut self,
        command: UpdateQuoteStatusCommand,
    ) -> Result<Quote, CoreError> {
        self.get_quote(command.id).await?;
        let quote = self
            .repo
            .update_status(command.id, command.status, Utc::now())
            .await?;
        let legal_mention_template_ids =
            self.repo.find_legal_mention_template_ids(command.id).await?;
        Ok(Quote {
            legal_mention_template_ids,
            ..quote
        })
    }

    pub async fn soft_delete_quote(&mut self, id: QuoteId) -> Result<(), CoreError> {
        self.get_quote(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
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
        vat_rate: command.vat_rate,
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

fn compute_quote_totals(lines: &[QuoteLine]) -> Result<(i32, i32, i32), CoreError> {
    let billing_lines: Vec<BillingLine> = lines
        .iter()
        .map(|line| BillingLine {
            quantity: line.quantity,
            unit_price_cents: line.unit_price_cents as i64,
            vat_rate: line.vat_rate,
        })
        .collect();

    let totals = compute_totals(&billing_lines);

    let total_ht_cents = i32::try_from(totals.total_ht_cents)
        .map_err(|_| CoreError::Conflict("quote HT total is outside supported bounds".to_owned()))?;
    let total_vat_cents = i32::try_from(totals.total_vat_cents).map_err(|_| {
        CoreError::Conflict("quote VAT total is outside supported bounds".to_owned())
    })?;
    let total_ttc_cents = i32::try_from(totals.total_ttc_cents).map_err(|_| {
        CoreError::Conflict("quote TTC total is outside supported bounds".to_owned())
    })?;

    Ok((total_ht_cents, total_vat_cents, total_ttc_cents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CustomerContextId, CustomerId, LegalMentionTemplateId, QuoteStatus, ServiceRateUnit,
        domain::quote::ports::MockQuoteRepository,
    };
    use mockall::predicate::eq;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn line_command(quantity: Decimal, unit_price_cents: i32) -> QuoteLineCommand {
        QuoteLineCommand {
            service_rate_id: None,
            label: "Taille de haie".to_owned(),
            quantity,
            unit: ServiceRateUnit::Ml,
            unit_price_cents,
            vat_rate: Decimal::from(20u32),
            notes: Some("Acces jardin".to_owned()),
            photo_keys: vec!["quotes/photo-1.jpg".to_owned()],
        }
    }

    fn quote(id: QuoteId) -> Quote {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        Quote {
            id,
            organization_id,
            reference: "DEV-2026-0001".to_owned(),
            title: "Rénovation cuisine".to_owned(),
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            emitter_context_id: None,
            status: QuoteStatus::Draft,
            deposit_basis: None,
            deposit_value: None,
            total_cents: 6600,
            total_ht_cents: 5500,
            total_vat_cents: 1100,
            total_ttc_cents: 6600,
            lines: vec![QuoteLine {
                id: QuoteLineId(Uuid::new_v4()),
                organization_id,
                quote_id: id,
                service_rate_id: None,
                label: "Taille de haie".to_owned(),
                quantity: Decimal::new(1, 0),
                unit: ServiceRateUnit::Hour,
                unit_price_cents: 5500,
                vat_rate: Decimal::from(20u32),
                notes: None,
                photo_keys: vec![],
                deleted_at: None,
                created_at: now,
                updated_at: now,
            }],
            legal_mention_template_ids: vec![],
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
        repo.expect_replace_legal_mention_templates()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo);
        let created = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![
                    line_command(Decimal::new(25, 1), 1200),
                    line_command(Decimal::new(1, 0), 500),
                ],
                legal_mention_template_ids: vec![],
            })
            .await
            .unwrap();

        assert_eq!(created.status, QuoteStatus::Draft);
        assert_eq!(created.reference, "DEV-2026-0001");
        assert_eq!(created.title, "Rénovation cuisine");
        // 2.5*1200=3000 + 1*500=500 → HT=3500, VAT@20%=700, TTC=4200
        assert_eq!(created.total_ht_cents, 3500);
        assert_eq!(created.total_vat_cents, 700);
        assert_eq!(created.total_ttc_cents, 4200);
        assert_eq!(created.total_cents, 4200);
    }

    #[tokio::test]
    async fn create_quote_persists_and_reads_back_legal_mention_template_ids() {
        let template_a = LegalMentionTemplateId(Uuid::new_v4());
        let template_b = LegalMentionTemplateId(Uuid::new_v4());
        let template_ids = vec![template_a, template_b];
        let template_ids_clone = template_ids.clone();

        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });
        repo.expect_replace_legal_mention_templates()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo);
        let created = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Devis avec mentions".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![line_command(Decimal::new(1, 0), 1000)],
                legal_mention_template_ids: template_ids_clone,
            })
            .await
            .unwrap();

        assert_eq!(created.legal_mention_template_ids, template_ids);
    }

    #[tokio::test]
    async fn update_quote_replaces_legal_mention_template_ids() {
        let id = QuoteId(Uuid::new_v4());
        let template_a = LegalMentionTemplateId(Uuid::new_v4());
        let template_b = LegalMentionTemplateId(Uuid::new_v4());
        let new_ids = vec![template_b];
        let new_ids_clone = new_ids.clone();

        let mut repo = MockQuoteRepository::new();
        // get_quote called inside update_quote
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| {
                let mut q = quote(id);
                q.legal_mention_template_ids = vec![template_a];
                Box::pin(async move { Ok(Some(q)) })
            });
        repo.expect_find_legal_mention_template_ids()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));
        repo.expect_update().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });
        repo.expect_replace_legal_mention_templates()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo);
        let updated = service
            .update_quote(UpdateQuoteCommand {
                id,
                title: "Version ajustée".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                status: QuoteStatus::Sent,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![line_command(Decimal::new(3, 0), 2000)],
                legal_mention_template_ids: new_ids_clone,
            })
            .await
            .unwrap();

        assert_eq!(updated.legal_mention_template_ids, new_ids);
    }

    #[tokio::test]
    async fn update_quote_recalculates_total() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_find_legal_mention_template_ids()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));
        repo.expect_update().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });
        repo.expect_replace_legal_mention_templates()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo);
        let updated = service
            .update_quote(UpdateQuoteCommand {
                id,
                title: "Version ajustée".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                status: QuoteStatus::Sent,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![line_command(Decimal::new(3, 0), 2000)],
                legal_mention_template_ids: vec![],
            })
            .await
            .unwrap();

        assert_eq!(updated.status, QuoteStatus::Sent);
        assert_eq!(updated.reference, "DEV-2026-0001");
        assert_eq!(updated.title, "Version ajustée");
        // 3*2000=6000 HT, VAT@20%=1200, TTC=7200
        assert_eq!(updated.total_ht_cents, 6000);
        assert_eq!(updated.total_vat_cents, 1200);
        assert_eq!(updated.total_ttc_cents, 7200);
        assert_eq!(updated.total_cents, 7200);
    }

    #[tokio::test]
    async fn update_quote_status_delegates_without_recalculating_lines() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        // get_quote (existence check) calls find_by_id + find_legal_mention_template_ids
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_find_legal_mention_template_ids()
            .with(eq(id))
            .times(2) // once for get_quote check, once for update_quote_status result
            .returning(|_| Box::pin(async { Ok(vec![]) }));
        repo.expect_update_status()
            .withf(move |quote_id, status, _| *quote_id == id && *status == QuoteStatus::Accepted)
            .returning(move |_, _, _| Box::pin(async move { Ok(quote(id)) }));

        let mut service = QuoteService::new(repo);

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
        let quote_id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(25), eq(50))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![quote(quote_id)], 1)) })
            });
        repo.expect_find_legal_mention_template_ids()
            .with(eq(quote_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = QuoteService::new(repo);
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
        repo.expect_find_legal_mention_template_ids()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = QuoteService::new(repo);

        service.soft_delete_quote(id).await.unwrap();
    }

    #[tokio::test]
    async fn rejects_invalid_line_input() {
        let mut repo = MockQuoteRepository::new();
        repo.expect_next_reference()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok("DEV-2026-0001".to_owned()) }));
        let mut service = QuoteService::new(repo);
        let result = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: "Rénovation cuisine".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![line_command(Decimal::ZERO, 1000)],
                legal_mention_template_ids: vec![],
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn rejects_empty_quote_title() {
        let repo = MockQuoteRepository::new();
        let mut service = QuoteService::new(repo);
        let result = service
            .create_quote(CreateQuoteCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                title: " ".to_owned(),
                customer_id: CustomerId(Uuid::new_v4()),
                customer_context_id: CustomerContextId(Uuid::new_v4()),
                emitter_context_id: None,
                deposit_basis: None,
                deposit_value: None,
                lines: vec![line_command(Decimal::new(1, 0), 1000)],
                legal_mention_template_ids: vec![],
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }
}
