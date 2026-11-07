use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use serde_json::{Value, json};

use crate::{
    Customer, CustomerId, MemberId, Organization, OrganizationId, Quote, QuoteId, QuoteLine,
    QuoteLineId, QuoteStatus, VatStatus,
    domain::{
        customer::ports::CustomerRepository,
        customer_contact::ports::CustomerContactRepository,
        customer_context::ports::CustomerContextRepository,
        member::ports::MemberRepository,
        organization::{billing_settings::BillingSettings, ports::OrganizationRepository},
        quote::{
            DepositPercent, DiscountPercent, IssuerSnapshot, QuoteError, QuoteField, QuoteOutcome,
            RecipientSnapshot,
            commands::{
                CreateQuoteCommand, QuoteLineCommand, QuoteTermsCommand, UpdateQuoteCommand,
                UpdateQuoteStatusCommand,
            },
            events::{QuoteCreated, QuoteDeleted, QuoteTransitioned, QuoteUpdated},
            ports::QuoteRepository,
            pricing::totals_of,
        },
    },
};

pub trait QuoteLookup: Send {
    type Organizations: OrganizationRepository;
    type Customers: CustomerRepository;
    type Contacts: CustomerContactRepository;
    type Addresses: CustomerContextRepository;
    type Members: MemberRepository;

    fn organizations(&mut self) -> &mut Self::Organizations;
    fn customers(&mut self) -> &mut Self::Customers;
    fn contacts(&mut self) -> &mut Self::Contacts;
    fn addresses(&mut self) -> &mut Self::Addresses;
    fn members(&mut self) -> &mut Self::Members;
}

pub struct QuoteLookups<'a, O, C, K, A, M> {
    pub organizations: &'a mut O,
    pub customers: &'a mut C,
    pub contacts: &'a mut K,
    pub addresses: &'a mut A,
    pub members: &'a mut M,
}

impl<O, C, K, A, M> QuoteLookup for QuoteLookups<'_, O, C, K, A, M>
where
    O: OrganizationRepository,
    C: CustomerRepository,
    K: CustomerContactRepository,
    A: CustomerContextRepository,
    M: MemberRepository,
{
    type Organizations = O;
    type Customers = C;
    type Contacts = K;
    type Addresses = A;
    type Members = M;

    fn organizations(&mut self) -> &mut O {
        self.organizations
    }

    fn customers(&mut self) -> &mut C {
        self.customers
    }

    fn contacts(&mut self) -> &mut K {
        self.contacts
    }

    fn addresses(&mut self) -> &mut A {
        self.addresses
    }

    fn members(&mut self) -> &mut M {
        self.members
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteChange {
    pub quote: Quote,
    pub from: QuoteStatus,
}

enum Stop {
    Core(CoreError),
    Refused(QuoteError),
}

impl From<CoreError> for Stop {
    fn from(error: CoreError) -> Self {
        Self::Core(error)
    }
}

impl From<QuoteError> for Stop {
    fn from(error: QuoteError) -> Self {
        Self::Refused(error)
    }
}

fn settle<T>(result: Result<T, Stop>) -> Result<QuoteOutcome<T>, CoreError> {
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(Stop::Core(error)) => Err(error),
        Err(Stop::Refused(refusal)) => Ok(Err(refusal)),
    }
}

struct Terms {
    valid_until: Option<NaiveDate>,
    deposit_percent: DepositPercent,
    owner_member_id: Option<MemberId>,
    notes: Option<String>,
    billing_contact_id: Option<crate::CustomerContactId>,
    billing_address_id: Option<crate::CustomerContextId>,
}

impl TryFrom<QuoteTermsCommand> for Terms {
    type Error = QuoteError;

    fn try_from(command: QuoteTermsCommand) -> Result<Self, Self::Error> {
        let notes = normalized(command.notes);
        Ok(Self {
            valid_until: command.valid_until,
            deposit_percent: DepositPercent::try_from(command.deposit_percent)?,
            owner_member_id: command.owner_member_id,
            notes,
            billing_contact_id: command.billing_contact_id,
            billing_address_id: command.billing_address_id,
        })
    }
}

fn normalized(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

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

    pub async fn create_quote<L: QuoteLookup>(
        &mut self,
        command: CreateQuoteCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        settle(self.create(command, lookups, now).await)
    }

    async fn create<L: QuoteLookup>(
        &mut self,
        command: CreateQuoteCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<Quote, Stop> {
        let quote_id = QuoteId(generate_uuid_v7());
        validate_title(&command.title)?;
        let terms = Terms::try_from(command.terms)?;
        let organization = resolve_organization(lookups, command.organization_id).await?;
        let customer =
            require_customer(lookups, command.organization_id, command.customer_id).await?;
        check_links(lookups, command.organization_id, customer.id, &terms, None).await?;
        let lines = build_quote_lines(command.organization_id, quote_id, command.lines, now)?;
        let totals = totals_of(&lines, is_subject_to_vat(&organization))?;

        let created = self
            .repo
            .insert(&Quote {
                id: quote_id,
                organization_id: command.organization_id,
                reference: None,
                title: command.title.trim().to_owned(),
                customer_id: command.customer_id,
                customer_context_id: command.customer_context_id,
                deal_id: command.deal_id,
                status: QuoteStatus::Draft,
                net_cents: totals.net_cents,
                vat_breakdown: totals.vat_breakdown,
                gross_cents: totals.gross_cents,
                valid_until: terms.valid_until,
                deposit_percent: terms.deposit_percent,
                owner_member_id: terms.owner_member_id,
                notes: terms.notes,
                billing_contact_id: terms
                    .billing_contact_id
                    .or(customer.profile.billing_contact_id),
                billing_address_id: terms
                    .billing_address_id
                    .or(customer.profile.billing_address_id),
                responded_at: None,
                responded_by: None,
                decline_reason: None,
                revision_of_id: None,
                superseded_by_id: None,
                issuer_snapshot: None,
                recipient_snapshot: None,
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
        deal_id: Option<crate::DealId>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Quote>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, deal_id, limit, offset)
            .await
    }

    pub async fn update_quote<L: QuoteLookup>(
        &mut self,
        command: UpdateQuoteCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<QuoteOutcome<QuoteChange>, CoreError> {
        settle(self.update(command, lookups, now).await)
    }

    async fn update<L: QuoteLookup>(
        &mut self,
        command: UpdateQuoteCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<QuoteChange, Stop> {
        let existing = self.get_quote(command.id).await?;
        if !existing.is_draft() {
            return Err(QuoteError::Locked.into());
        }
        if !matches!(command.status, QuoteStatus::Draft | QuoteStatus::Sent) {
            return Err(QuoteError::invalid(
                QuoteField::Status,
                "a quote is edited as a draft or sent, answer it through the status endpoint",
            )
            .into());
        }
        validate_title(&command.title)?;
        let terms = Terms::try_from(command.terms)?;
        let organization = resolve_organization(lookups, existing.organization_id).await?;
        require_customer(lookups, existing.organization_id, command.customer_id).await?;
        if command.customer_id != existing.customer_id && !existing.lines.is_empty() {
            return Err(QuoteError::invalid(
                QuoteField::CustomerId,
                "the customer cannot change once the quote has lines",
            )
            .into());
        }
        check_links(
            lookups,
            existing.organization_id,
            command.customer_id,
            &terms,
            Some(&existing),
        )
        .await?;
        let lines = build_quote_lines(existing.organization_id, existing.id, command.lines, now)?;
        let totals = totals_of(&lines, is_subject_to_vat(&organization))?;

        let mut candidate = Quote {
            title: command.title.trim().to_owned(),
            customer_id: command.customer_id,
            customer_context_id: command.customer_context_id,
            deal_id: command.deal_id,
            net_cents: totals.net_cents,
            vat_breakdown: totals.vat_breakdown,
            gross_cents: totals.gross_cents,
            valid_until: terms.valid_until,
            deposit_percent: terms.deposit_percent,
            owner_member_id: terms.owner_member_id,
            notes: terms.notes,
            billing_contact_id: terms.billing_contact_id,
            billing_address_id: terms.billing_address_id,
            lines,
            updated_at: now,
            ..existing.clone()
        };
        if command.status == QuoteStatus::Sent {
            candidate = self.prepare_send(candidate, lookups, now).await?;
        }

        let updated = self.repo.update(&candidate).await?;

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
        if updated.status == QuoteStatus::Sent {
            self.supersede_source(&updated, now).await?;
        }

        Ok(QuoteChange {
            quote: updated,
            from: existing.status,
        })
    }

    pub async fn update_quote_status<L: QuoteLookup>(
        &mut self,
        command: UpdateQuoteStatusCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<QuoteOutcome<QuoteChange>, CoreError> {
        settle(self.change_status(command, lookups, now).await)
    }

    async fn change_status<L: QuoteLookup>(
        &mut self,
        command: UpdateQuoteStatusCommand,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<QuoteChange, Stop> {
        let existing = self.get_quote(command.id).await?;
        let from = existing.status;
        if from == command.status {
            return Ok(QuoteChange {
                quote: existing,
                from,
            });
        }

        let next = match (from, command.status) {
            (QuoteStatus::Draft, QuoteStatus::Sent) => {
                self.prepare_send(existing, lookups, now).await?
            }
            (QuoteStatus::Sent, QuoteStatus::Accepted) => {
                accepted(existing, command.responded_by, now)?
            }
            (QuoteStatus::Sent, QuoteStatus::Declined) => {
                declined(existing, command.decline_reason, now)
            }
            (
                QuoteStatus::Sent | QuoteStatus::Accepted | QuoteStatus::Declined,
                QuoteStatus::Cancelled,
            ) => Quote {
                status: QuoteStatus::Cancelled,
                updated_at: now,
                ..existing
            },
            (_, QuoteStatus::Accepted | QuoteStatus::Declined) => {
                return Err(QuoteError::NotSent.into());
            }
            (_, QuoteStatus::Sent | QuoteStatus::Draft) => {
                return Err(QuoteError::Locked.into());
            }
            (_, QuoteStatus::Cancelled) => {
                return Err(QuoteError::invalid(
                    QuoteField::Status,
                    "only a sent, accepted or declined quote can be cancelled",
                )
                .into());
            }
            (_, QuoteStatus::Superseded) => {
                return Err(QuoteError::invalid(
                    QuoteField::Status,
                    "a quote is superseded by sending its revision",
                )
                .into());
            }
        };

        let saved = self.repo.update_lifecycle(&next).await?;
        self.emit_transition(from, &saved)?;
        if saved.status == QuoteStatus::Sent {
            self.supersede_source(&saved, now).await?;
        }

        Ok(QuoteChange { quote: saved, from })
    }

    async fn prepare_send<L: QuoteLookup>(
        &mut self,
        mut quote: Quote,
        lookups: &mut L,
        now: DateTime<Utc>,
    ) -> Result<Quote, Stop> {
        let today = now.date_naive();
        let organization = resolve_organization(lookups, quote.organization_id).await?;
        let settings = lookups
            .organizations()
            .find_billing_settings(organization.id)
            .await?
            .unwrap_or_default();
        let customer = require_customer(lookups, quote.organization_id, quote.customer_id).await?;
        let recipient = resolve_recipient(lookups, &quote, &customer).await?;

        validate_for_send(&quote, &recipient, today)?;

        let reference = match quote.reference.take() {
            Some(existing) => existing,
            None => {
                self.repo
                    .allocate_number(
                        organization.id,
                        &organization.quote_number_prefix,
                        now.year(),
                        settings.number_padding,
                    )
                    .await?
            }
        };

        quote.reference = Some(reference);
        quote.valid_until = quote.valid_until.or_else(|| validity_end(today, &settings));
        quote.issuer_snapshot = Some(IssuerSnapshot::of(&organization, &settings));
        quote.recipient_snapshot = Some(recipient);
        quote.status = QuoteStatus::Sent;
        quote.updated_at = now;

        Ok(quote)
    }

    async fn supersede_source(
        &mut self,
        sent: &Quote,
        now: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let Some(source_id) = sent.revision_of_id else {
            return Ok(());
        };
        let Some(source) = self.repo.find_by_id(source_id).await? else {
            return Ok(());
        };
        if source.status != QuoteStatus::Sent {
            return Ok(());
        }

        let superseded = self
            .repo
            .update_lifecycle(&Quote {
                status: QuoteStatus::Superseded,
                updated_at: now,
                ..source
            })
            .await?;

        self.emit_transition(QuoteStatus::Sent, &superseded)
    }

    pub async fn revise_quote(
        &mut self,
        id: QuoteId,
        now: DateTime<Utc>,
    ) -> Result<QuoteOutcome<Quote>, CoreError> {
        settle(self.revise(id, now).await)
    }

    async fn revise(&mut self, id: QuoteId, now: DateTime<Utc>) -> Result<Quote, Stop> {
        let source = self.get_quote(id).await?;
        if !matches!(source.status, QuoteStatus::Sent | QuoteStatus::Declined) {
            return Err(QuoteError::NotRevisable.into());
        }

        let revision_id = QuoteId(generate_uuid_v7());
        let lines = source
            .lines
            .iter()
            .map(|line| QuoteLine {
                id: QuoteLineId(generate_uuid_v7()),
                quote_id: revision_id,
                created_at: now,
                updated_at: now,
                ..line.clone()
            })
            .collect();

        let created = self
            .repo
            .insert(&Quote {
                id: revision_id,
                reference: None,
                status: QuoteStatus::Draft,
                valid_until: None,
                responded_at: None,
                responded_by: None,
                decline_reason: None,
                revision_of_id: Some(source.id),
                superseded_by_id: None,
                issuer_snapshot: None,
                recipient_snapshot: None,
                lines,
                deleted_at: None,
                created_at: now,
                updated_at: now,
                ..source
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

    fn emit_transition(&self, from: QuoteStatus, quote: &Quote) -> Result<(), CoreError> {
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

    pub async fn soft_delete_quote(
        &mut self,
        id: QuoteId,
        now: DateTime<Utc>,
    ) -> Result<QuoteOutcome<()>, CoreError> {
        let existing = self.get_quote(id).await?;
        if !existing.is_draft() {
            return Ok(Err(QuoteError::Locked));
        }
        self.repo.soft_delete(id, now).await?;

        self.emitter
            .emit(existing.organization_id, &QuoteDeleted { quote_id: id })?;

        Ok(Ok(()))
    }
}

fn accepted(
    quote: Quote,
    responded_by: Option<String>,
    now: DateTime<Utc>,
) -> Result<Quote, QuoteError> {
    if quote.is_expired(now.date_naive()) {
        return Err(QuoteError::Expired);
    }

    Ok(Quote {
        status: QuoteStatus::Accepted,
        responded_at: Some(now),
        responded_by: normalized(responded_by),
        updated_at: now,
        ..quote
    })
}

fn declined(quote: Quote, reason: Option<String>, now: DateTime<Utc>) -> Quote {
    Quote {
        status: QuoteStatus::Declined,
        responded_at: Some(now),
        decline_reason: normalized(reason),
        updated_at: now,
        ..quote
    }
}

fn validity_end(today: NaiveDate, settings: &BillingSettings) -> Option<NaiveDate> {
    today.checked_add_signed(Duration::days(i64::from(
        settings.quote_validity_days.get(),
    )))
}

fn is_subject_to_vat(organization: &Organization) -> bool {
    matches!(organization.vat_status, Some(VatStatus::Subject { .. }))
}

pub fn validate_for_send(
    quote: &Quote,
    recipient: &RecipientSnapshot,
    today: NaiveDate,
) -> QuoteOutcome<()> {
    validate_title(&quote.title)?;
    if quote.lines.is_empty() {
        return Err(QuoteError::invalid(
            QuoteField::Lines,
            "add at least one line",
        ));
    }
    for line in &quote.lines {
        check_line_fields(
            &line.label,
            line.quantity,
            &line.unit,
            line.unit_price_cents,
        )?;
    }
    if quote.gross_cents <= 0 {
        return Err(QuoteError::invalid(
            QuoteField::Total,
            "must be greater than zero",
        ));
    }
    if !recipient.address.is_postal() {
        return Err(QuoteError::invalid(
            QuoteField::BillingAddress,
            "the customer needs a billing street and city",
        ));
    }
    if quote.valid_until.is_some_and(|until| until < today) {
        return Err(QuoteError::invalid(
            QuoteField::ValidUntil,
            "must not be in the past",
        ));
    }

    Ok(())
}

async fn resolve_organization<L: QuoteLookup>(
    lookups: &mut L,
    organization_id: OrganizationId,
) -> Result<Organization, CoreError> {
    lookups
        .organizations()
        .find_by_id(organization_id)
        .await?
        .ok_or(CoreError::NotFound)
}

async fn require_customer<L: QuoteLookup>(
    lookups: &mut L,
    organization_id: OrganizationId,
    customer_id: CustomerId,
) -> Result<Customer, CoreError> {
    lookups
        .customers()
        .find_by_id(customer_id)
        .await?
        .filter(|customer| customer.organization_id == organization_id)
        .ok_or(CoreError::NotFound)
}

async fn resolve_recipient<L: QuoteLookup>(
    lookups: &mut L,
    quote: &Quote,
    customer: &Customer,
) -> Result<RecipientSnapshot, CoreError> {
    let contact = match quote
        .billing_contact_id
        .or(customer.profile.billing_contact_id)
    {
        Some(id) => lookups
            .contacts()
            .find_by_id(id)
            .await?
            .filter(|contact| contact.customer_id == customer.id),
        None => None,
    };
    let address = match quote
        .billing_address_id
        .or(customer.profile.billing_address_id)
    {
        Some(id) => lookups
            .addresses()
            .find_by_id(id)
            .await?
            .filter(|address| address.customer_id == customer.id),
        None => None,
    };

    Ok(RecipientSnapshot::of(
        customer,
        contact.as_ref(),
        address.as_ref(),
    ))
}

async fn check_links<L: QuoteLookup>(
    lookups: &mut L,
    organization_id: OrganizationId,
    customer_id: CustomerId,
    terms: &Terms,
    current: Option<&Quote>,
) -> Result<(), Stop> {
    let customer_changed = current.is_none_or(|quote| quote.customer_id != customer_id);

    if let Some(owner) = terms
        .owner_member_id
        .filter(|owner| Some(*owner) != current.and_then(|quote| quote.owner_member_id))
    {
        let same_organization = lookups
            .members()
            .find_by_id(owner)
            .await?
            .is_some_and(|member| member.organization_id == organization_id);
        if !same_organization {
            return Err(QuoteError::invalid(
                QuoteField::OwnerMemberId,
                "must be a member of the organization",
            )
            .into());
        }
    }

    if let Some(contact) = terms.billing_contact_id.filter(|contact| {
        customer_changed || Some(*contact) != current.and_then(|quote| quote.billing_contact_id)
    }) && !lookups
        .customers()
        .has_contact(customer_id, contact)
        .await?
    {
        return Err(QuoteError::invalid(
            QuoteField::BillingContactId,
            "must be a contact of the customer",
        )
        .into());
    }

    if let Some(address) = terms.billing_address_id.filter(|address| {
        customer_changed || Some(*address) != current.and_then(|quote| quote.billing_address_id)
    }) && !lookups
        .customers()
        .has_address(customer_id, address)
        .await?
    {
        return Err(QuoteError::invalid(
            QuoteField::BillingAddressId,
            "must be an address of the customer",
        )
        .into());
    }

    Ok(())
}

fn content_diff(existing: &Quote, updated: &Quote) -> (Vec<&'static str>, Value) {
    let mut changed = Vec::new();
    let mut previous = serde_json::Map::new();
    let mut moved = |field: &'static str, was: Value, differs: bool| {
        if differs {
            changed.push(field);
            previous.insert(field.to_owned(), was);
        }
    };

    moved(
        "title",
        json!(existing.title),
        existing.title != updated.title,
    );
    moved(
        "customer_id",
        json!(existing.customer_id.0),
        existing.customer_id != updated.customer_id,
    );
    moved(
        "customer_context_id",
        json!(existing.customer_context_id.0),
        existing.customer_context_id != updated.customer_context_id,
    );
    moved(
        "valid_until",
        json!(existing.valid_until),
        existing.valid_until != updated.valid_until,
    );
    moved(
        "deposit_percent",
        json!(existing.deposit_percent.get().to_string()),
        existing.deposit_percent != updated.deposit_percent,
    );
    moved(
        "owner_member_id",
        json!(existing.owner_member_id.map(|id| id.0)),
        existing.owner_member_id != updated.owner_member_id,
    );
    moved(
        "notes",
        json!(existing.notes),
        existing.notes != updated.notes,
    );
    moved(
        "billing_contact_id",
        json!(existing.billing_contact_id.map(|id| id.0)),
        existing.billing_contact_id != updated.billing_contact_id,
    );
    moved(
        "billing_address_id",
        json!(existing.billing_address_id.map(|id| id.0)),
        existing.billing_address_id != updated.billing_address_id,
    );
    let lines_moved = line_projection(existing) != line_projection(updated);
    moved("lines", line_projection(existing), lines_moved);
    if lines_moved {
        previous.insert("net_cents".to_owned(), json!(existing.net_cents));
        previous.insert("gross_cents".to_owned(), json!(existing.gross_cents));
    }

    (changed, Value::Object(previous))
}

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
                "discount_percent": line.discount_percent.get().normalize().to_string(),
                "billing_frequency": line.billing_frequency.as_str(),
                "service_rate_id": line.service_rate_id.map(|id| id.0),
                "product_id": line.product_id.map(|id| id.0),
                "notes": line.notes,
                "photo_keys": line.photo_keys,
            }))
            .collect::<Vec<_>>()
    )
}

fn validate_title(title: &str) -> QuoteOutcome<()> {
    if title.trim().is_empty() {
        return Err(QuoteError::invalid(QuoteField::Title, "must not be empty"));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Turning an accepted quote into a project's proposed tasks (#298). Pure,
// I/O-free — the read (`propose_tasks_from_quote`) and the two write-side
// guards (`require_quote_accepted`, `validate_quote_plannable`) all operate
// on data the caller already loaded.
// ---------------------------------------------------------------------------

/// One quote line's suggested task, before a human confirms anything.
///
/// `suggested_minutes` is `None` whenever a duration cannot be read off the
/// line rather than guessed — a line priced by the hour has one, a line
/// priced per unit, per square meter, or flat does not. Inventing a number
/// for those would be a suggestion the caller cannot audit, which is worse
/// than no suggestion (see #298's own brief).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskProposal {
    pub quote_line_id: QuoteLineId,
    pub title: String,
    pub suggested_minutes: Option<i32>,
}

/// Builds one proposal per quote line, in line order. Never refuses: a
/// draft or declined quote still has lines worth previewing, even though
/// only an accepted one can actually be turned into a project (see
/// [`require_quote_accepted`], enforced by the read use case instead of
/// here, so this stays a pure projection).
pub fn propose_tasks_from_quote(quote: &Quote) -> Vec<TaskProposal> {
    quote
        .lines
        .iter()
        .map(|line| TaskProposal {
            quote_line_id: line.id,
            title: line.label.clone(),
            suggested_minutes: line_suggested_minutes(line),
        })
        .collect()
}

/// A hard 60 minutes to the hour — the only unit this reads a duration from.
fn line_suggested_minutes(line: &QuoteLine) -> Option<i32> {
    if line.unit != crate::ServiceRateUnit::Hour.as_str() {
        return None;
    }

    (line.quantity * Decimal::from(60)).round().to_i32()
}

/// Refuses a quote that has not been accepted, with its own conflict rather
/// than a generic failure — a quote is a commercial unit, a task is a
/// scheduling unit, and turning a draft's *guesses* into a plan is not what
/// this endpoint is for.
pub fn require_quote_accepted(quote: &Quote) -> Result<(), CoreError> {
    if quote.status != QuoteStatus::Accepted {
        return Err(CoreError::Conflict(
            "only an accepted quote can be turned into a project".to_owned(),
        ));
    }

    Ok(())
}

/// The write-side gate: accepted, and either the first project on this
/// quote or an explicit second one. Two projects on one quote makes the
/// margin ambiguous, which #260 rejected when it moved `quote_id` up from
/// the task onto the project — `force_new` is the caller saying, in so many
/// words, that they know that and want it anyway.
pub fn validate_quote_plannable(
    quote: &Quote,
    already_has_project: bool,
    force_new: bool,
) -> Result<(), CoreError> {
    require_quote_accepted(quote)?;

    if already_has_project && !force_new {
        return Err(CoreError::Conflict(
            "this quote already has a project; pass force_new to create a second one".to_owned(),
        ));
    }

    Ok(())
}

fn build_quote_lines(
    organization_id: OrganizationId,
    quote_id: QuoteId,
    commands: Vec<QuoteLineCommand>,
    now: DateTime<Utc>,
) -> QuoteOutcome<Vec<QuoteLine>> {
    commands
        .into_iter()
        .map(|command| build_quote_line(organization_id, quote_id, command, now))
        .collect()
}

fn build_quote_line(
    organization_id: OrganizationId,
    quote_id: QuoteId,
    command: QuoteLineCommand,
    now: DateTime<Utc>,
) -> QuoteOutcome<QuoteLine> {
    validate_line(&command)?;
    let discount_percent = DiscountPercent::try_from(command.discount_percent)?;

    Ok(QuoteLine {
        id: QuoteLineId(generate_uuid_v7()),
        organization_id,
        quote_id,
        service_rate_id: command.service_rate_id,
        product_id: command.product_id,
        label: command.label,
        quantity: command.quantity,
        unit: command.unit,
        unit_price_cents: command.unit_price_cents,
        vat_rate_bp: command.vat_rate_bp,
        discount_percent,
        billing_frequency: command.billing_frequency,
        notes: command.notes,
        photo_keys: command.photo_keys,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    })
}

fn check_line_fields(
    label: &str,
    quantity: Decimal,
    unit: &str,
    unit_price_cents: i32,
) -> QuoteOutcome<()> {
    let refuse = |rule| Err(QuoteError::invalid(QuoteField::Lines, rule));

    if label.trim().is_empty() {
        return refuse("quote line label cannot be empty");
    }
    if quantity <= Decimal::ZERO {
        return refuse("quote line quantity must be positive");
    }
    if unit.trim().is_empty() {
        return refuse("quote line unit cannot be empty");
    }
    if unit_price_cents < 0 {
        return refuse("quote line unit price cannot be negative");
    }

    Ok(())
}

fn validate_line(command: &QuoteLineCommand) -> QuoteOutcome<()> {
    check_line_fields(
        &command.label,
        command.quantity,
        &command.unit,
        command.unit_price_cents,
    )?;
    let refuse = |rule| Err(QuoteError::invalid(QuoteField::Lines, rule));

    if command
        .vat_rate_bp
        .is_some_and(|rate_bp| !(0..=10_000).contains(&rate_bp))
    {
        return refuse("quote line vat rate must be between 0 and 10000 basis points");
    }
    if command
        .notes
        .as_ref()
        .is_some_and(|notes| notes.trim().is_empty())
    {
        return refuse("quote line notes cannot be empty when present");
    }
    if command.photo_keys.iter().any(|key| key.trim().is_empty()) {
        return refuse("quote line photo keys cannot be empty");
    }
    if command.service_rate_id.is_some() && command.product_id.is_some() {
        return Err(QuoteError::invalid(
            QuoteField::CatalogueReference,
            "a line references a product or a service rate, not both",
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
