use std::collections::BTreeMap;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde_json::{Value, json};

use crate::{
    DraftInvoice, Invoice, InvoiceId, InvoiceLine, InvoiceLineId, InvoiceStatus,
    InvoiceVatBreakdownLine, Organization, OrganizationId,
    domain::{
        invoice::{
            commands::{
                CancelInvoiceCommand, CreateInvoiceCommand, InvoiceLineCommand,
                UpdateInvoiceCommand,
            },
            events::{InvoiceCreated, InvoiceDeleted, InvoiceTransitioned, InvoiceUpdated},
            ports::InvoiceRepository,
        },
        organization::{legal_identity::VatStatus, ports::OrganizationRepository},
    },
};

pub struct InvoiceService<R, E>
where
    R: InvoiceRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> InvoiceService<R, E>
where
    R: InvoiceRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create_invoice<O>(
        &mut self,
        command: CreateInvoiceCommand,
        mut organization_repository: O,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
    {
        let now = Utc::now();
        let invoice_id = InvoiceId(generate_uuid_v7());
        let organization =
            resolve_organization(&mut organization_repository, command.organization_id).await?;
        let lines = build_invoice_lines(command.organization_id, invoice_id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;

        let draft = DraftInvoice::try_from_invoice(Invoice {
            id: invoice_id,
            organization_id: command.organization_id,
            number: None,
            kind: command.kind,
            project_id: command.project_id,
            customer_id: command.customer_id,
            customer_context_id: command.customer_context_id,
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: command.due_at,
            notes: command.notes,
            net_cents: totals.net_cents,
            vat_breakdown: totals.vat_breakdown,
            gross_cents: totals.gross_cents,
            issuer_identity: None,
            lines,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .expect("just constructed with status Draft");

        let created = self.repo.insert_draft(&draft).await?;

        self.emitter.emit(
            created.organization_id,
            &InvoiceCreated {
                invoice: created.clone(),
            },
        )?;

        Ok(created)
    }

    pub async fn get_invoice(&mut self, id: InvoiceId) -> Result<Invoice, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_invoices(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Invoice>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn list_invoices_by_project(
        &mut self,
        project_id: crate::ProjectId,
    ) -> Result<Vec<Invoice>, CoreError> {
        self.repo.list_by_project(project_id).await
    }

    /// Refused, by [`DraftInvoice::try_from_invoice`], on anything but a
    /// draft: the compile-time guarantee is that `update_draft` cannot be
    /// called with anything else.
    pub async fn update_invoice<O>(
        &mut self,
        command: UpdateInvoiceCommand,
        mut organization_repository: O,
    ) -> Result<Invoice, CoreError>
    where
        O: OrganizationRepository,
    {
        let existing = self.get_invoice(command.id).await?;
        let now = Utc::now();
        let organization =
            resolve_organization(&mut organization_repository, existing.organization_id).await?;
        let lines = build_invoice_lines(existing.organization_id, existing.id, command.lines, now)?;
        let totals = calculate_totals(&lines, organization.vat_status.as_ref())?;

        let mut draft = DraftInvoice::try_from_invoice(existing.clone())?;
        draft.set_project(command.project_id);
        draft.set_customer(command.customer_id, command.customer_context_id);
        draft.set_due_at(command.due_at);
        draft.set_notes(command.notes);
        draft.set_lines_and_totals(
            lines,
            totals.net_cents,
            totals.vat_breakdown,
            totals.gross_cents,
        );
        draft.touch(now);

        let updated = self.repo.update_draft(&draft).await?;

        let (changed_fields, previous) = content_diff(&existing, &updated);
        if !changed_fields.is_empty() {
            self.emitter.emit(
                updated.organization_id,
                &InvoiceUpdated {
                    invoice: updated.clone(),
                    changed_fields,
                    previous,
                },
            )?;
        }

        Ok(updated)
    }

    /// Draft or issued, never a document already `Cancelled`, `Paid` or
    /// `PartiallyPaid` — those transitions belong to issuing (#317) and to
    /// payment recording (#318, #320), never to a bare status write here.
    pub async fn cancel_invoice(
        &mut self,
        command: CancelInvoiceCommand,
    ) -> Result<Invoice, CoreError> {
        let existing = self.get_invoice(command.id).await?;
        if matches!(
            existing.status,
            InvoiceStatus::Cancelled | InvoiceStatus::Paid | InvoiceStatus::PartiallyPaid
        ) {
            return Err(CoreError::Conflict(format!(
                "invoice {} is {} and cannot be cancelled",
                existing.id, existing.status
            )));
        }

        let updated = self
            .repo
            .update_status(command.id, InvoiceStatus::Cancelled, Utc::now())
            .await?;

        self.emit_transition(existing.status, &updated)?;

        Ok(updated)
    }

    /// Refused unless the invoice is still a draft: an issued invoice is a
    /// legal document, corrected with a credit note (#318), never deleted.
    pub async fn soft_delete_invoice(&mut self, id: InvoiceId) -> Result<(), CoreError> {
        let existing = self.get_invoice(id).await?;
        if existing.status != InvoiceStatus::Draft {
            return Err(CoreError::Conflict(format!(
                "invoice {} is {} and cannot be deleted; only a draft can be",
                existing.id, existing.status
            )));
        }

        self.repo.soft_delete(id, Utc::now()).await?;

        self.emitter
            .emit(existing.organization_id, &InvoiceDeleted { invoice_id: id })
    }

    fn emit_transition(&self, from: InvoiceStatus, invoice: &Invoice) -> Result<(), CoreError> {
        if from == invoice.status || InvoiceTransitioned::event_name(invoice.status).is_none() {
            return Ok(());
        }

        self.emitter.emit(
            invoice.organization_id,
            &InvoiceTransitioned {
                invoice: invoice.clone(),
                from,
            },
        )
    }
}

async fn resolve_organization(
    organization_repository: &mut impl OrganizationRepository,
    organization_id: OrganizationId,
) -> Result<Organization, CoreError> {
    organization_repository
        .find_by_id(organization_id)
        .await?
        .ok_or(CoreError::NotFound)
}

/// Which content fields moved, and what they held before. `status` is
/// absent by construction, same rule as `quote::service::content_diff`.
fn content_diff(existing: &Invoice, updated: &Invoice) -> (Vec<&'static str>, Value) {
    let mut changed = Vec::new();
    let mut previous = serde_json::Map::new();

    if existing.project_id != updated.project_id {
        changed.push("project_id");
        previous.insert(
            "project_id".to_owned(),
            json!(existing.project_id.map(|id| id.0)),
        );
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
    if existing.due_at != updated.due_at {
        changed.push("due_at");
        previous.insert("due_at".to_owned(), json!(existing.due_at));
    }
    if existing.notes != updated.notes {
        changed.push("notes");
        previous.insert("notes".to_owned(), json!(existing.notes));
    }
    if line_projection(existing) != line_projection(updated) {
        changed.push("lines");
        previous.insert("lines".to_owned(), line_projection(existing));
        previous.insert("net_cents".to_owned(), json!(existing.net_cents));
        previous.insert("gross_cents".to_owned(), json!(existing.gross_cents));
    }

    (changed, Value::Object(previous))
}

fn line_projection(invoice: &Invoice) -> Value {
    json!(
        invoice
            .lines
            .iter()
            .map(|line| json!({
                "label": line.label,
                "quantity": line.quantity.to_string(),
                "unit_price_cents": line.unit_price_cents,
                "vat_rate_basis_points": line.vat_rate_basis_points,
                "position": line.position,
            }))
            .collect::<Vec<_>>()
    )
}

fn build_invoice_lines(
    organization_id: OrganizationId,
    invoice_id: InvoiceId,
    commands: Vec<InvoiceLineCommand>,
    now: chrono::DateTime<Utc>,
) -> Result<Vec<InvoiceLine>, CoreError> {
    if commands.is_empty() {
        return Err(CoreError::Conflict(
            "an invoice must have at least one line".to_owned(),
        ));
    }

    commands
        .into_iter()
        .enumerate()
        .map(|(position, command)| {
            build_invoice_line(organization_id, invoice_id, command, position as i32, now)
        })
        .collect()
}

fn build_invoice_line(
    organization_id: OrganizationId,
    invoice_id: InvoiceId,
    command: InvoiceLineCommand,
    position: i32,
    now: chrono::DateTime<Utc>,
) -> Result<InvoiceLine, CoreError> {
    validate_line(&command)?;

    Ok(InvoiceLine {
        id: InvoiceLineId(generate_uuid_v7()),
        organization_id,
        invoice_id,
        label: command.label.trim().to_owned(),
        quantity: command.quantity,
        unit_price_cents: command.unit_price_cents,
        vat_rate_basis_points: command.vat_rate_basis_points,
        position,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn validate_line(command: &InvoiceLineCommand) -> Result<(), CoreError> {
    if command.label.trim().is_empty() {
        return Err(CoreError::Conflict(
            "invoice line label cannot be empty".to_owned(),
        ));
    }

    if command.quantity <= Decimal::ZERO {
        return Err(CoreError::Conflict(
            "invoice line quantity must be positive".to_owned(),
        ));
    }

    if command.unit_price_cents < 0 {
        return Err(CoreError::Conflict(
            "invoice line unit price cannot be negative".to_owned(),
        ));
    }

    if command
        .vat_rate_basis_points
        .is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp))
    {
        return Err(CoreError::Conflict(
            "invoice line vat rate must be between 0 and 10000 basis points".to_owned(),
        ));
    }

    Ok(())
}

pub(crate) struct InvoiceTotals {
    pub net_cents: i32,
    pub vat_breakdown: Vec<InvoiceVatBreakdownLine>,
    pub gross_cents: i32,
}

fn line_net_cents(quantity: Decimal, unit_price_cents: i32) -> Result<i64, CoreError> {
    let line_total = (quantity * Decimal::from(unit_price_cents)).round_dp(0);

    line_total.to_i64().ok_or_else(|| {
        CoreError::Conflict("invoice line total is outside supported bounds".to_owned())
    })
}

fn calculate_total_cents(lines: &[InvoiceLine]) -> Result<i32, CoreError> {
    let total = lines.iter().try_fold(0_i64, |sum, line| {
        let line_total = line_net_cents(line.quantity, line.unit_price_cents)?;

        sum.checked_add(line_total).ok_or_else(|| {
            CoreError::Conflict("invoice total is outside supported bounds".to_owned())
        })
    })?;

    i32::try_from(total)
        .map_err(|_| CoreError::Conflict("invoice total is outside supported bounds".to_owned()))
}

/// Net, VAT broken down per rate, and gross — same rule as
/// `quote::service::calculate_totals`: rounded per line then summed per
/// rate, not summed then rounded, and an organization not (yet) subject to
/// VAT produces `gross_cents == net_cents` with an empty breakdown.
pub(crate) fn calculate_totals(
    lines: &[InvoiceLine],
    vat_status: Option<&VatStatus>,
) -> Result<InvoiceTotals, CoreError> {
    let net_cents = calculate_total_cents(lines)?;

    if !matches!(vat_status, Some(VatStatus::Subject { .. })) {
        return Ok(InvoiceTotals {
            net_cents,
            vat_breakdown: Vec::new(),
            gross_cents: net_cents,
        });
    }

    let mut vat_by_rate: BTreeMap<i32, i64> = BTreeMap::new();
    let mut vat_total: i64 = 0;

    for line in lines {
        let rate_bp = line.vat_rate_basis_points.unwrap_or(0);
        let line_net = line_net_cents(line.quantity, line.unit_price_cents)?;
        let line_vat = div_round_half_even(line_net * i64::from(rate_bp), 10_000);

        *vat_by_rate.entry(rate_bp).or_insert(0) += line_vat;
        vat_total = vat_total.checked_add(line_vat).ok_or_else(|| {
            CoreError::Conflict("invoice VAT total is outside supported bounds".to_owned())
        })?;
    }

    let vat_breakdown = vat_by_rate
        .into_iter()
        .map(|(rate_bp, vat_cents)| {
            i32::try_from(vat_cents)
                .map(|vat_cents| InvoiceVatBreakdownLine { rate_bp, vat_cents })
                .map_err(|_| {
                    CoreError::Conflict("invoice VAT total is outside supported bounds".to_owned())
                })
        })
        .collect::<Result<Vec<_>, CoreError>>()?;

    let gross_cents = i64::from(net_cents)
        .checked_add(vat_total)
        .and_then(|total| i32::try_from(total).ok())
        .ok_or_else(|| {
            CoreError::Conflict("invoice total is outside supported bounds".to_owned())
        })?;

    Ok(InvoiceTotals {
        net_cents,
        vat_breakdown,
        gross_cents,
    })
}

/// Divides, rounding a half to the even neighbour. Duplicated from
/// `quote::service::div_round_half_even` — two occurrences, not yet a
/// pattern worth sharing.
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
    use crate::{CustomerContextId, CustomerId, InvoiceKind, Organization, UserId};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn line_command(quantity: Decimal, unit_price_cents: i32) -> InvoiceLineCommand {
        InvoiceLineCommand {
            label: "Acompte travaux".to_owned(),
            quantity,
            unit_price_cents,
            vat_rate_basis_points: None,
        }
    }

    fn invoice(id: InvoiceId) -> Invoice {
        let now = Utc::now();
        let organization_id = OrganizationId(Uuid::new_v4());
        Invoice {
            id,
            organization_id,
            number: None,
            kind: InvoiceKind::Standard,
            project_id: None,
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            status: InvoiceStatus::Draft,
            issued_at: None,
            due_at: None,
            notes: None,
            net_cents: 5500,
            vat_breakdown: Vec::new(),
            gross_cents: 5500,
            issuer_identity: None,
            lines: vec![InvoiceLine {
                id: InvoiceLineId(Uuid::new_v4()),
                organization_id,
                invoice_id: id,
                label: "Acompte travaux".to_owned(),
                quantity: Decimal::new(1, 0),
                unit_price_cents: 5500,
                vat_rate_basis_points: None,
                position: 0,
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
            owner_id: UserId(Uuid::new_v4()),
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

    fn mock_organization_repository(
        organization: Organization,
    ) -> crate::domain::organization::ports::MockOrganizationRepository {
        let mut repo = crate::domain::organization::ports::MockOrganizationRepository::new();
        repo.expect_find_by_id().times(1).returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        repo
    }

    #[tokio::test]
    async fn create_invoice_calculates_totals_from_lines() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    lines: vec![
                        line_command(Decimal::new(25, 1), 1200),
                        line_command(Decimal::new(1, 0), 500),
                    ],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.status, InvoiceStatus::Draft);
        assert_eq!(created.number, None, "a draft has no number");
        assert_eq!(created.net_cents, 3500);
        assert_eq!(created.gross_cents, 3500);
    }

    #[tokio::test]
    async fn create_invoice_breaks_vat_down_per_rate() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let mut reduced_rate = line_command(Decimal::new(1, 0), 10_000);
        reduced_rate.vat_rate_basis_points = Some(550);
        let mut standard_rate = line_command(Decimal::new(1, 0), 10_000);
        standard_rate.vat_rate_basis_points = Some(2000);

        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
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
                InvoiceVatBreakdownLine {
                    rate_bp: 550,
                    vat_cents: 550
                },
                InvoiceVatBreakdownLine {
                    rate_bp: 2000,
                    vat_cents: 2000
                },
            ]
        );
        assert_eq!(created.gross_cents, 22_550);
    }

    #[tokio::test]
    async fn rejects_an_invoice_with_no_lines() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());

        let result = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    lines: vec![],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn update_invoice_recalculates_totals_on_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_update_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let organization_id = invoice(id).organization_id;
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let updated = service
            .update_invoice(
                UpdateInvoiceCommand {
                    id,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: Some("Merci".to_owned()),
                    lines: vec![line_command(Decimal::new(3, 0), 2000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(updated.net_cents, 6000);
        assert_eq!(updated.notes.as_deref(), Some("Merci"));
    }

    #[tokio::test]
    async fn update_invoice_refuses_an_issued_invoice() {
        let id = InvoiceId(Uuid::new_v4());
        let issued = Invoice {
            status: InvoiceStatus::Issued,
            number: Some("FAC-2026-0001".to_owned()),
            issued_at: Some(Utc::now()),
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let issued = issued.clone();
            Box::pin(async move { Ok(Some(issued)) })
        });
        // No `expect_update_draft`: the call must never reach the
        // repository, it has to be refused before that.

        let organization_id = invoice(id).organization_id;
        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service
            .update_invoice(
                UpdateInvoiceCommand {
                    id,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    lines: vec![line_command(Decimal::new(1, 0), 1000)],
                },
                mock_organization_repository(organization_without_vat_status(organization_id)),
            )
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn cancel_invoice_transitions_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_update_status()
            .withf(move |invoice_id, status, _| {
                *invoice_id == id && *status == InvoiceStatus::Cancelled
            })
            .returning(move |_, status, _| {
                let mut i = invoice(id);
                i.status = status;
                Box::pin(async move { Ok(i) })
            });

        let emitter = events::testing::RecordingEmitter::new();
        let mut service = InvoiceService::new(repo, &emitter);

        service
            .cancel_invoice(CancelInvoiceCommand { id })
            .await
            .unwrap();

        assert_eq!(emitter.names(), vec!["invoice.cancelled"]);
    }

    #[tokio::test]
    async fn cancel_invoice_refuses_an_already_cancelled_invoice() {
        let id = InvoiceId(Uuid::new_v4());
        let cancelled = Invoice {
            status: InvoiceStatus::Cancelled,
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let cancelled = cancelled.clone();
            Box::pin(async move { Ok(Some(cancelled)) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service.cancel_invoice(CancelInvoiceCommand { id }).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn soft_delete_refuses_anything_but_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let issued = Invoice {
            status: InvoiceStatus::Issued,
            ..invoice(id)
        };
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let issued = issued.clone();
            Box::pin(async move { Ok(Some(issued)) })
        });
        // No `expect_soft_delete`: must be refused before the repository is
        // reached.

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let result = service.soft_delete_invoice(id).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn soft_delete_deletes_a_draft() {
        let id = InvoiceId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(invoice(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let emitter = events::testing::RecordingEmitter::new();
        let mut service = InvoiceService::new(repo, &emitter);

        service.soft_delete_invoice(id).await.unwrap();

        assert_eq!(emitter.names(), vec!["invoice.deleted"]);
    }

    /// The distinguishing rounding case, mirrored from
    /// `quote::service::three_half_cent_vat_lines_round_per_line_then_sum`.
    #[tokio::test]
    async fn three_half_cent_vat_lines_round_per_line_then_sum() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = crate::domain::invoice::ports::MockInvoiceRepository::new();
        repo.expect_insert_draft().times(1).returning(|d| {
            let invoice = d.invoice().clone();
            Box::pin(async move { Ok(invoice) })
        });

        let mut service = InvoiceService::new(repo, events::testing::RecordingEmitter::new());
        let mut line = line_command(Decimal::new(1, 0), 105);
        line.vat_rate_basis_points = Some(1000);

        let created = service
            .create_invoice(
                CreateInvoiceCommand {
                    organization_id,
                    kind: InvoiceKind::Standard,
                    project_id: None,
                    customer_id: CustomerId(Uuid::new_v4()),
                    customer_context_id: CustomerContextId(Uuid::new_v4()),
                    due_at: None,
                    notes: None,
                    lines: vec![line.clone(), line.clone(), line],
                },
                mock_organization_repository(organization_subject_to_vat(organization_id)),
            )
            .await
            .unwrap();

        assert_eq!(created.net_cents, 315);
        assert_eq!(
            created.vat_breakdown,
            vec![InvoiceVatBreakdownLine {
                rate_bp: 1000,
                vat_cents: 30
            }]
        );
        assert_eq!(created.gross_cents, 345);
    }
}
