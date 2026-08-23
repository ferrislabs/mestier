use std::collections::BTreeMap;

use chrono::{Datelike, Utc};
use common::{CoreError, generate_uuid_v7};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use events::EventEmitter;
use serde_json::{Value, json};

use crate::{
    Organization, OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineId, QuoteStatus,
    QuoteVatBreakdownLine,
    domain::{
        organization::{legal_identity::VatStatus, ports::OrganizationRepository},
        quote::{
            commands::{
                CreateQuoteCommand, QuoteLineCommand, UpdateQuoteCommand, UpdateQuoteStatusCommand,
            },
            events::{QuoteCreated, QuoteDeleted, QuoteTransitioned, QuoteUpdated},
            ports::QuoteRepository,
        },
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

    /// `organization_repository` reads only the organization's VAT status —
    /// method-scoped like `task_label_repository` on
    /// `OrganizationService::create_organization`, so nothing else on this
    /// service becomes aware of the organization aggregate.
    pub async fn create_quote<O>(
        &mut self,
        command: CreateQuoteCommand,
        mut organization_repository: O,
    ) -> Result<Quote, CoreError>
    where
        O: OrganizationRepository,
    {
        let now = Utc::now();
        let quote_id = QuoteId(generate_uuid_v7());
        validate_title(&command.title)?;
        let organization =
            resolve_organization(&mut organization_repository, command.organization_id).await?;
        let lines = build_quote_lines(command.organization_id, quote_id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;

        let created = self
            .repo
            .insert(&Quote {
                id: quote_id,
                organization_id: command.organization_id,
                // Allocated when the quote first leaves `Draft`, not here:
                // see `QuoteRepository::allocate_number`.
                reference: None,
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                status: crate::QuoteStatus::Draft,
                net_cents: totals.net_cents,
                vat_breakdown: totals.vat_breakdown,
                gross_cents: totals.gross_cents,
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

    pub async fn update_quote<O>(
        &mut self,
        command: UpdateQuoteCommand,
        mut organization_repository: O,
    ) -> Result<Quote, CoreError>
    where
        O: OrganizationRepository,
    {
        let existing = self.get_quote(command.id).await?;
        let now = Utc::now();
        validate_title(&command.title)?;
        let organization =
            resolve_organization(&mut organization_repository, existing.organization_id).await?;
        let lines = build_quote_lines(existing.organization_id, existing.id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;
        let newly_allocated = self
            .allocate_reference_if_entering_sent(&existing, &organization, command.status)
            .await?;

        let updated = self
            .repo
            .update(&Quote {
                id: existing.id,
                organization_id: existing.organization_id,
                // Whichever number the quote already carries, or the one
                // just allocated — the two can never both be `Some`,
                // because `allocate_reference_if_entering_sent` never
                // allocates a second one.
                reference: existing.reference.clone().or(newly_allocated),
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                status: command.status,
                net_cents: totals.net_cents,
                vat_breakdown: totals.vat_breakdown,
                gross_cents: totals.gross_cents,
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

    pub async fn update_quote_status<O>(
        &mut self,
        command: UpdateQuoteStatusCommand,
        mut organization_repository: O,
    ) -> Result<Quote, CoreError>
    where
        O: OrganizationRepository,
    {
        let existing = self.get_quote(command.id).await?;
        let organization =
            resolve_organization(&mut organization_repository, existing.organization_id).await?;
        let reference = self
            .allocate_reference_if_entering_sent(&existing, &organization, command.status)
            .await?;

        let updated = self
            .repo
            .update_status(command.id, command.status, reference, Utc::now())
            .await?;

        self.emit_transition(existing.status, &updated)?;

        Ok(updated)
    }

    /// `Some` only on the one call that allocates a fresh number — the
    /// transition into `Sent` on a quote that has none yet. `None` in
    /// every other case: already numbered, or not entering `Sent` at all.
    /// Mirrors `QuoteRepository::update_status`'s contract exactly, so its
    /// result can be handed straight to that port method; `update_quote`
    /// still has to fold it with the quote's existing reference itself,
    /// since it persists the whole struct rather than a `COALESCE`.
    async fn allocate_reference_if_entering_sent(
        &mut self,
        existing: &Quote,
        organization: &Organization,
        target_status: QuoteStatus,
    ) -> Result<Option<String>, CoreError> {
        if existing.reference.is_some() || target_status != QuoteStatus::Sent {
            return Ok(None);
        }

        let year = Utc::now().year();
        let number = self
            .repo
            .allocate_number(organization.id, &organization.quote_number_prefix, year)
            .await?;

        Ok(Some(number))
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

/// The organization a quote belongs to — its VAT status (`None` when not
/// stated yet, treated the same as "not subject" for totals: quote
/// drafting must not block on legal-identity completeness, only issuing a
/// document does, see #314) and its quote number prefix.
async fn resolve_organization(
    organization_repository: &mut impl OrganizationRepository,
    organization_id: OrganizationId,
) -> Result<Organization, CoreError> {
    organization_repository
        .find_by_id(organization_id)
        .await?
        .ok_or(CoreError::NotFound)
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
        previous.insert("net_cents".to_owned(), json!(existing.net_cents));
        previous.insert("gross_cents".to_owned(), json!(existing.gross_cents));
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
                "vat_rate_bp": line.vat_rate_bp,
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
        vat_rate_bp: command.vat_rate_bp,
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
        .vat_rate_bp
        .is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp))
    {
        return Err(CoreError::Conflict(
            "quote line vat rate must be between 0 and 10000 basis points".to_owned(),
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

/// A single line's net amount, in cents, rounded the house way (half to
/// even). Shared by `calculate_total_cents` and `calculate_totals` so the
/// net a document shows and the base VAT is computed on are, structurally,
/// the same number.
fn line_net_cents(quantity: Decimal, unit_price_cents: i32) -> Result<i64, CoreError> {
    let line_total = (quantity * Decimal::from(unit_price_cents)).round_dp(0);

    line_total.to_i64().ok_or_else(|| {
        CoreError::Conflict("quote line total is outside supported bounds".to_owned())
    })
}

fn calculate_total_cents(lines: &[QuoteLine]) -> Result<i32, CoreError> {
    let total = lines.iter().try_fold(0_i64, |sum, line| {
        let line_total = line_net_cents(line.quantity, line.unit_price_cents)?;

        sum.checked_add(line_total).ok_or_else(|| {
            CoreError::Conflict("quote total is outside supported bounds".to_owned())
        })
    })?;

    i32::try_from(total)
        .map_err(|_| CoreError::Conflict("quote total is outside supported bounds".to_owned()))
}

struct QuoteTotals {
    net_cents: i32,
    vat_breakdown: Vec<QuoteVatBreakdownLine>,
    gross_cents: i32,
}

/// Net, VAT broken down per rate, and gross — computed once, here, per
/// `CLAUDE.md`: every other reader (the API response, the PDF) receives
/// these figures rather than recomputing them, so the two can never
/// disagree.
///
/// VAT is rounded per line, then summed per rate — not summed then rounded.
/// The two disagree exactly at a half-cent boundary; see
/// `three_half_cent_vat_lines_round_per_line_then_sum` for the case that
/// tells them apart.
///
/// An organization not subject to VAT, or one that has not stated a status
/// yet, produces `gross_cents == net_cents` and an empty breakdown — never
/// a breakdown of zeros. "Exempt" and "nothing to report" are different
/// facts, and only one of them is true here.
fn calculate_totals(
    lines: &[QuoteLine],
    vat_status: Option<&VatStatus>,
) -> Result<QuoteTotals, CoreError> {
    let net_cents = calculate_total_cents(lines)?;

    if !matches!(vat_status, Some(VatStatus::Subject { .. })) {
        return Ok(QuoteTotals {
            net_cents,
            vat_breakdown: Vec::new(),
            gross_cents: net_cents,
        });
    }

    let mut vat_by_rate: BTreeMap<i32, i64> = BTreeMap::new();
    let mut vat_total: i64 = 0;

    for line in lines {
        let rate_bp = line.vat_rate_bp.unwrap_or(0);
        let line_net = line_net_cents(line.quantity, line.unit_price_cents)?;
        let line_vat = div_round_half_even(line_net * i64::from(rate_bp), 10_000);

        *vat_by_rate.entry(rate_bp).or_insert(0) += line_vat;
        vat_total = vat_total.checked_add(line_vat).ok_or_else(|| {
            CoreError::Conflict("quote VAT total is outside supported bounds".to_owned())
        })?;
    }

    let vat_breakdown = vat_by_rate
        .into_iter()
        .map(|(rate_bp, vat_cents)| {
            i32::try_from(vat_cents)
                .map(|vat_cents| QuoteVatBreakdownLine { rate_bp, vat_cents })
                .map_err(|_| {
                    CoreError::Conflict("quote VAT total is outside supported bounds".to_owned())
                })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    let gross_cents = i64::from(net_cents)
        .checked_add(vat_total)
        .and_then(|total| i32::try_from(total).ok())
        .ok_or_else(|| CoreError::Conflict("quote total is outside supported bounds".to_owned()))?;

    Ok(QuoteTotals {
        net_cents,
        vat_breakdown,
        gross_cents,
    })
}

/// Divides, rounding a half to the even neighbour. Both arguments are
/// non-negative here (a line's net cents and a VAT rate in basis points
/// cannot be negative — both are validated at the boundary), so no sign
/// handling: a negative would be a bug upstream rather than something to
/// interpret. Duplicated from `domain::profitability::service`, which owns
/// the same rule for a different arithmetic: two occurrences, not yet a
/// pattern worth a shared module.
fn div_round_half_even(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let doubled_remainder = (numerator % denominator) * 2;

    if doubled_remainder > denominator {
        return quotient + 1;
    }
    if doubled_remainder < denominator {
        return quotient;
    }

    if quotient % 2 == 0 {
        quotient
    } else {
        quotient + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CustomerContextId, CustomerId, Organization, QuoteStatus, ServiceRateUnit,
        domain::{
            organization::ports::MockOrganizationRepository, quote::ports::MockQuoteRepository,
        },
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
            vat_rate_bp: None,
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
            // A draft has no number yet — see `Quote::reference`.
            reference: None,
            title: "Rénovation cuisine".to_owned(),
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            status: QuoteStatus::Draft,
            net_cents: 5500,
            vat_breakdown: Vec::new(),
            gross_cents: 5500,
            lines: vec![QuoteLine {
                id: QuoteLineId(Uuid::new_v4()),
                organization_id,
                quote_id: id,
                service_rate_id: None,
                label: "Taille de haie".to_owned(),
                quantity: Decimal::new(1, 0),
                unit: ServiceRateUnit::Hour,
                unit_price_cents: 5500,
                vat_rate_bp: None,
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

    fn organization_without_vat_status(id: OrganizationId) -> Organization {
        let now = Utc::now();
        Organization {
            id,
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: crate::UserId(Uuid::new_v4()),
            legal_name: None,
            legal_form: None,
            registration_number: None,
            vat_status: None,
            share_capital_cents: None,
            address_line1: None,
            address_line2: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            field_clock_enabled: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn organization_subject_to_vat(id: OrganizationId) -> Organization {
        Organization {
            vat_status: Some(VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }),
            ..organization_without_vat_status(id)
        }
    }

    fn mock_organization_repository(organization: Organization) -> MockOrganizationRepository {
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        repo
    }

    #[tokio::test]
    async fn create_quote_calculates_total_from_lines() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let created = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation cuisine".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![
                        line_command(Decimal::new(25, 1), 1200),
                        line_command(Decimal::new(1, 0), 500),
                    ],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.status, QuoteStatus::Draft);
        assert_eq!(
            created.reference, None,
            "a draft has no number until it is sent"
        );
        assert_eq!(created.title, "Rénovation cuisine");
        assert_eq!(created.net_cents, 3500);
        assert_eq!(created.gross_cents, 3500);
        assert!(created.vat_breakdown.is_empty());
    }

    #[tokio::test]
    async fn create_quote_breaks_vat_down_per_rate_for_a_subject_organization() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let mut reduced_rate = line_command(Decimal::new(1, 0), 10_000);
        reduced_rate.vat_rate_bp = Some(550); // 5.5 %
        let mut standard_rate = line_command(Decimal::new(1, 0), 10_000);
        standard_rate.vat_rate_bp = Some(2000); // 20 %

        let created = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![reduced_rate, standard_rate],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, 20_000);
        assert_eq!(
            created.vat_breakdown,
            vec![
                QuoteVatBreakdownLine {
                    rate_bp: 550,
                    vat_cents: 550
                },
                QuoteVatBreakdownLine {
                    rate_bp: 2000,
                    vat_cents: 2000
                },
            ]
        );
        assert_eq!(created.gross_cents, 22_550);
    }

    #[tokio::test]
    async fn a_not_subject_organization_produces_gross_equal_to_net() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let organization = Organization {
            vat_status: Some(VatStatus::NotSubject {
                basis: "Article 293 B du CGI".into(),
            }),
            ..organization_without_vat_status(organization_id)
        };
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let mut line = line_command(Decimal::new(1, 0), 10_000);
        line.vat_rate_bp = Some(2000); // ignored: the organization is exempt

        let created = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line],
                },
                mock_organization_repository(organization),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, created.gross_cents);
        assert!(created.vat_breakdown.is_empty());
    }

    /// The distinguishing case named in #312: three lines whose VAT lands
    /// exactly on a half cent. Rounding per line then summing (the house
    /// choice) sends each 10.5 to its even neighbour, 10, for a total of 30.
    /// Summing then rounding would instead round 31.5 to 32. The two must
    /// disagree here, or this test is not testing the decision.
    #[tokio::test]
    async fn three_half_cent_vat_lines_round_per_line_then_sum() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let mut line = line_command(Decimal::new(1, 0), 105); // 1.05 € net
        line.vat_rate_bp = Some(1000); // 10 % -> 10.5 cents of VAT, exactly

        let created = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Trois lignes".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line.clone(), line.clone(), line],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, 315);
        assert_eq!(
            created.vat_breakdown,
            vec![QuoteVatBreakdownLine {
                rate_bp: 1000,
                vat_cents: 30
            }]
        );
        assert_eq!(created.gross_cents, 345);
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

        let organization_id = quote(id).organization_id;
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let updated = service
            .update_quote(
                UpdateQuoteCommand {
                    id,
                    title: "Version ajustée".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    status: QuoteStatus::Draft,
                    lines: vec![line_command(Decimal::new(3, 0), 2000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, QuoteStatus::Draft);
        assert_eq!(
            updated.reference, None,
            "staying in draft allocates nothing"
        );
        assert_eq!(updated.title, "Version ajustée");
        assert_eq!(updated.net_cents, 6000);
    }

    #[tokio::test]
    async fn update_quote_allocates_a_number_when_it_first_enters_sent() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_allocate_number()
            .withf(|_, prefix, year| prefix == "DEV" && *year == chrono::Utc::now().year())
            .times(1)
            .returning(|_, prefix, year| {
                let reference = format!("{prefix}-{year}-0001");
                Box::pin(async move { Ok(reference) })
            });
        repo.expect_update().times(1).returning(|q| {
            let quote = q.clone();
            Box::pin(async move { Ok(quote) })
        });

        let organization_id = quote(id).organization_id;
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let updated = service
            .update_quote(
                UpdateQuoteCommand {
                    id,
                    title: "Version ajustée".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    status: QuoteStatus::Sent,
                    lines: vec![line_command(Decimal::new(3, 0), 2000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(updated.status, QuoteStatus::Sent);
        let year = chrono::Utc::now().year();
        assert_eq!(updated.reference, Some(format!("DEV-{year}-0001")));
    }

    #[tokio::test]
    async fn update_quote_status_delegates_without_recalculating_lines() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        // No `expect_allocate_number`: `Accepted` is not `Sent`, so nothing
        // is allocated — a call the mock has not been told to expect would
        // panic and fail this test.
        repo.expect_update_status()
            .withf(move |quote_id, status, reference, _| {
                *quote_id == id && *status == QuoteStatus::Accepted && reference.is_none()
            })
            .returning(move |_, _, _, _| Box::pin(async move { Ok(quote(id)) }));

        let organization_id = quote(id).organization_id;
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());

        service
            .update_quote_status(
                UpdateQuoteStatusCommand {
                    id,
                    status: QuoteStatus::Accepted,
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_quote_status_allocates_a_number_when_it_first_enters_sent() {
        let id = QuoteId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(quote(id))) }));
        repo.expect_allocate_number()
            .times(1)
            .returning(|_, prefix, year| {
                let reference = format!("{prefix}-{year}-0001");
                Box::pin(async move { Ok(reference) })
            });
        repo.expect_update_status()
            .withf(move |quote_id, status, reference, _| {
                *quote_id == id && *status == QuoteStatus::Sent && reference.is_some()
            })
            .returning(move |_, status, reference, _| {
                let mut q = quote(id);
                q.status = status;
                q.reference = reference;
                Box::pin(async move { Ok(q) })
            });

        let organization_id = quote(id).organization_id;
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());

        let updated = service
            .update_quote_status(
                UpdateQuoteStatusCommand {
                    id,
                    status: QuoteStatus::Sent,
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert!(updated.reference.is_some());
    }

    #[tokio::test]
    async fn a_quote_that_already_carries_a_number_never_gets_a_second_one() {
        let id = QuoteId(Uuid::new_v4());
        let already_sent = Quote {
            status: QuoteStatus::Sent,
            reference: Some("DEV-2026-0007".to_owned()),
            ..quote(id)
        };
        let mut repo = MockQuoteRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let q = already_sent.clone();
            Box::pin(async move { Ok(Some(q)) })
        });
        // No `expect_allocate_number`: restating `Sent` on an already-sent
        // quote must not reallocate.
        repo.expect_update_status()
            .withf(|_, _, reference, _| reference.is_none())
            .returning(move |_, _, _, _| {
                let q = Quote {
                    status: QuoteStatus::Sent,
                    reference: Some("DEV-2026-0007".to_owned()),
                    ..quote(id)
                };
                Box::pin(async move { Ok(q) })
            });

        let organization_id = quote(id).organization_id;
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());

        let updated = service
            .update_quote_status(
                UpdateQuoteStatusCommand {
                    id,
                    status: QuoteStatus::Sent,
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(updated.reference.as_deref(), Some("DEV-2026-0007"));
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
        let organization_id = OrganizationId(Uuid::new_v4());
        let repo = MockQuoteRepository::new();
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation cuisine".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line_command(Decimal::ZERO, 1000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn rejects_a_vat_rate_outside_the_valid_range() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let repo = MockQuoteRepository::new();
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let mut line = line_command(Decimal::new(1, 0), 1000);
        line.vat_rate_bp = Some(10_001);

        let result = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation cuisine".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn rejects_empty_quote_title() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let repo = MockQuoteRepository::new();
        let mut service = QuoteService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: " ".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line_command(Decimal::new(1, 0), 1000)],
                },
                MockOrganizationRepository::new(),
            )
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
        CustomerContextId, CustomerId, QuoteStatus,
        domain::{
            organization::ports::MockOrganizationRepository, quote::ports::MockQuoteRepository,
        },
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
            vat_rate_bp: None,
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

    fn no_vat_status_repo() -> MockOrganizationRepository {
        let mut repo = MockOrganizationRepository::new();
        repo.expect_find_by_id().returning(|id| {
            let now = Utc::now();
            let organization = crate::Organization {
                id,
                name: "Acme".into(),
                slug: "acme".into(),
                owner_id: crate::UserId(Uuid::new_v4()),
                legal_name: None,
                legal_form: None,
                registration_number: None,
                vat_status: None,
                share_capital_cents: None,
                address_line1: None,
                address_line2: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                contact_email: None,
                contact_phone: None,
                insurance_mention: None,
                quote_number_prefix: "DEV".to_owned(),
                field_clock_enabled: false,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            Box::pin(async move { Ok(Some(organization)) })
        });
        repo
    }

    #[tokio::test]
    async fn creating_a_quote_emits_created_and_nothing_else() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation cuisine".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line_command(Decimal::new(1, 0), 5500)],
                },
                no_vat_status_repo(),
            )
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
            .update_quote(
                update_command(id, "Nouveau titre", existing.status, &existing),
                no_vat_status_repo(),
            )
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

        service
            .update_quote(command, no_vat_status_repo())
            .await
            .unwrap();

        let payload = emitter.only("quote.updated").payload;
        assert_eq!(payload["changed_fields"], json!(["lines"]));
        assert_eq!(payload["previous"]["net_cents"], json!(5500));
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
        repo.expect_allocate_number()
            .times(1)
            .returning(|_, prefix, year| {
                let reference = format!("{prefix}-{year}-0001");
                Box::pin(async move { Ok(reference) })
            });
        repo.expect_update().returning(|q| {
            let q = q.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .update_quote(
                update_command(id, "Nouveau titre", QuoteStatus::Sent, &existing),
                no_vat_status_repo(),
            )
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
            // Only reached when `status` is `Sent`, on the one iteration
            // whose starting quote (`quote(id)`, a draft) has no number yet
            // — the mock permits any number of calls, including zero, when
            // `.times()` is not stated.
            repo.expect_allocate_number().returning(|_, prefix, year| {
                let reference = format!("{prefix}-{year}-0001");
                Box::pin(async move { Ok(reference) })
            });
            repo.expect_update_status().returning(move |_, _, _, _| {
                let q = landed.clone();
                Box::pin(async move { Ok(q) })
            });
            let emitter = RecordingEmitter::new();
            let mut service = QuoteService::new(repo, &emitter);

            service
                .update_quote_status(
                    UpdateQuoteStatusCommand { id, status },
                    no_vat_status_repo(),
                )
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
        repo.expect_update_status().returning(move |_, _, _, _| {
            let q = landed.clone();
            Box::pin(async move { Ok(q) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        service
            .update_quote_status(
                UpdateQuoteStatusCommand {
                    id,
                    status: existing.status,
                },
                no_vat_status_repo(),
            )
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
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockQuoteRepository::new();
        repo.expect_insert()
            .returning(|_| Box::pin(async { Err(CoreError::Conflict("nope".into())) }));
        let emitter = RecordingEmitter::new();
        let mut service = QuoteService::new(repo, &emitter);

        let outcome = service
            .create_quote(
                CreateQuoteCommand {
                    organization_id,
                    title: "Rénovation cuisine".to_owned(),
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    lines: vec![line_command(Decimal::new(1, 0), 5500)],
                },
                no_vat_status_repo(),
            )
            .await;

        assert!(outcome.is_err());
        assert!(emitter.names().is_empty());
    }
}
