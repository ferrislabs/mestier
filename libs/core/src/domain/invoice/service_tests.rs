use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use chrono::{TimeZone, Utc};
use events::testing::RecordingEmitter;
use mockall::predicate::eq;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::*;
use crate::{
    AddressKind, BillingFrequency, CustomerContext, CustomerKind, CustomerProfile, CustomerStatus,
    DiscountPercent, InvoiceDisplayState, MockProjectRepository, MockQuoteRepository,
    PaymentMethod, QuoteLine, QuoteLineId, QuoteStatus, UserId,
    domain::{
        customer::ports::MockCustomerRepository,
        customer_context::ports::MockCustomerContextRepository,
        invoice::{
            commands::{CreditLineRequest, CreditRequest},
            ports::MockInvoiceRepository,
        },
        member::{Member, MemberId, ports::MockMemberRepository},
        organization::{
            billing_settings::BillingSettingsInput, numbering::format_number,
            ports::MockOrganizationRepository,
        },
        role::ports::MockRoleRepository,
    },
};
use authz::{Decision, MockAuthorizer};

fn system_actor() -> Subject {
    Subject::system()
}

fn at(month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, month, day, 9, 0, 0).unwrap()
}

fn date(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).unwrap()
}

fn dec(text: &str) -> Decimal {
    text.parse().unwrap()
}

#[derive(Default)]
struct State {
    invoices: Vec<Invoice>,
    payments: Vec<InvoicePayment>,
    events: Vec<InvoiceEvent>,
    counters: HashMap<(InvoiceNumberSeries, i32), i32>,
}

type Store = Arc<Mutex<State>>;

fn state(store: &Store) -> MutexGuard<'_, State> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn collection_of(state: &State, id: InvoiceId) -> InvoiceCollection {
    InvoiceCollection {
        invoice_id: id,
        paid_cents: state
            .payments
            .iter()
            .filter(|payment| payment.invoice_id == id && payment.deleted_at.is_none())
            .map(|payment| i64::from(payment.amount_cents))
            .sum(),
        credited_cents: state
            .invoices
            .iter()
            .filter(|invoice| {
                invoice.source_invoice_id == Some(id)
                    && !matches!(
                        invoice.status,
                        InvoiceStatus::Draft | InvoiceStatus::Cancelled
                    )
            })
            .map(|invoice| i64::from(invoice.gross_cents))
            .sum(),
    }
}

fn repository_over(store: &Store) -> MockInvoiceRepository {
    let mut repo = MockInvoiceRepository::new();

    let shared = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = state(&shared)
            .invoices
            .iter()
            .find(|invoice| invoice.id == id)
            .cloned();
        Box::pin(async move { Ok(found) })
    });

    let shared = store.clone();
    repo.expect_list_by_organization()
        .returning(move |organization_id, _, _| {
            let invoices: Vec<Invoice> = state(&shared)
                .invoices
                .iter()
                .filter(|invoice| invoice.organization_id == organization_id)
                .cloned()
                .collect();
            let total = invoices.len() as u64;
            Box::pin(async move { Ok((invoices, total)) })
        });

    let shared = store.clone();
    repo.expect_list_by_project().returning(move |project_id| {
        let invoices: Vec<Invoice> = state(&shared)
            .invoices
            .iter()
            .filter(|invoice| invoice.project_id == Some(project_id))
            .cloned()
            .collect();
        Box::pin(async move { Ok(invoices) })
    });

    let shared = store.clone();
    repo.expect_list_by_source_invoice()
        .returning(move |source| {
            let notes: Vec<Invoice> = state(&shared)
                .invoices
                .iter()
                .filter(|invoice| invoice.source_invoice_id == Some(source))
                .cloned()
                .collect();
            Box::pin(async move { Ok(notes) })
        });

    let shared = store.clone();
    repo.expect_list_by_quote().returning(move |quote_id| {
        let invoices: Vec<Invoice> = state(&shared)
            .invoices
            .iter()
            .filter(|invoice| invoice.quote_id == Some(quote_id))
            .cloned()
            .collect();
        Box::pin(async move { Ok(invoices) })
    });

    let shared = store.clone();
    repo.expect_insert_draft().returning(move |draft| {
        let invoice = draft.invoice().clone();
        state(&shared).invoices.push(invoice.clone());
        Box::pin(async move { Ok(invoice) })
    });

    let shared = store.clone();
    repo.expect_update_draft().returning(move |draft| {
        let invoice = draft.invoice().clone();
        let mut guard = state(&shared);
        let stored = guard
            .invoices
            .iter_mut()
            .find(|stored| stored.id == invoice.id)
            .expect("update_draft on an invoice that was never inserted");
        *stored = invoice.clone();
        drop(guard);
        Box::pin(async move { Ok(invoice) })
    });

    let shared = store.clone();
    repo.expect_update_status()
        .returning(move |id, status, updated_at| {
            let mut guard = state(&shared);
            let invoice = guard
                .invoices
                .iter_mut()
                .find(|invoice| invoice.id == id)
                .expect("update_status on an invoice that was never inserted");
            invoice.status = status;
            invoice.updated_at = updated_at;
            let result = invoice.clone();
            drop(guard);
            Box::pin(async move { Ok(result) })
        });

    let shared = store.clone();
    repo.expect_soft_delete().returning(move |id, _| {
        state(&shared).invoices.retain(|invoice| invoice.id != id);
        Box::pin(async { Ok(()) })
    });

    let shared = store.clone();
    repo.expect_allocate_number()
        .returning(move |_, series, prefix, year, padding| {
            let mut guard = state(&shared);
            let counter = guard.counters.entry((series, year)).or_insert(0);
            *counter += 1;
            let number = format_number(prefix, year, *counter, padding);
            drop(guard);
            Box::pin(async move { Ok(number) })
        });

    let shared = store.clone();
    repo.expect_issue().returning(
        move |id, number, issued_at, issued_on, due_at, issuer, updated_at| {
            let mut guard = state(&shared);
            let invoice = guard
                .invoices
                .iter_mut()
                .find(|invoice| invoice.id == id)
                .expect("issue called on an invoice that was never inserted");
            invoice.number = Some(number);
            invoice.status = InvoiceStatus::Issued;
            invoice.issued_at = Some(issued_at);
            invoice.issued_on = Some(issued_on);
            invoice.due_at = due_at;
            invoice.issuer_identity = Some(issuer.clone());
            invoice.updated_at = updated_at;
            let result = invoice.clone();
            drop(guard);
            Box::pin(async move { Ok(result) })
        },
    );

    let shared = store.clone();
    repo.expect_last_issued_on().returning(move |_| {
        let last = state(&shared)
            .invoices
            .iter()
            .filter(|invoice| invoice.kind != InvoiceKind::CreditNote)
            .filter_map(|invoice| invoice.issued_on)
            .max();
        Box::pin(async move { Ok(last) })
    });

    let shared = store.clone();
    repo.expect_last_credit_note_on().returning(move |_| {
        let last = state(&shared)
            .invoices
            .iter()
            .filter(|invoice| invoice.kind == InvoiceKind::CreditNote)
            .filter_map(|invoice| invoice.issued_on)
            .max();
        Box::pin(async move { Ok(last) })
    });

    let shared = store.clone();
    repo.expect_collections().returning(move |ids| {
        let guard = state(&shared);
        let collections: Vec<InvoiceCollection> =
            ids.iter().map(|id| collection_of(&guard, *id)).collect();
        drop(guard);
        Box::pin(async move { Ok(collections) })
    });

    let shared = store.clone();
    repo.expect_insert_payment().returning(move |payment| {
        state(&shared).payments.push(payment.clone());
        let payment = payment.clone();
        Box::pin(async move { Ok(payment) })
    });

    let shared = store.clone();
    repo.expect_list_events().returning(move |id| {
        let _ = id;
        let events = state(&shared).events.clone();
        Box::pin(async move { Ok(events) })
    });

    let shared = store.clone();
    repo.expect_insert_event().returning(move |_, _, event| {
        state(&shared).events.push(event.clone());
        let event = event.clone();
        Box::pin(async move { Ok(event) })
    });

    let shared = store.clone();
    repo.expect_mark_sent().returning(move |id, sent_at| {
        if let Some(invoice) = state(&shared)
            .invoices
            .iter_mut()
            .find(|invoice| invoice.id == id)
        {
            invoice.sent_at = Some(sent_at);
        }
        Box::pin(async { Ok(()) })
    });

    let shared = store.clone();
    repo.expect_set_internal_note()
        .returning(move |id, note, _| {
            if let Some(invoice) = state(&shared)
                .invoices
                .iter_mut()
                .find(|invoice| invoice.id == id)
            {
                invoice.internal_note = (!note.is_empty()).then(|| note.to_owned());
            }
            Box::pin(async { Ok(()) })
        });

    repo
}

fn new_permissive_service<'a>(
    store: &Store,
    emitter: &'a RecordingEmitter,
) -> InvoiceService<
    MockInvoiceRepository,
    &'a RecordingEmitter,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
> {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

    InvoiceService::new(
        repository_over(store),
        emitter,
        MockMemberRepository::new(),
        MockRoleRepository::new(),
        authz,
    )
}

fn stage_org_membership(
    members: &mut MockMemberRepository,
    roles: &mut MockRoleRepository,
    organization_id: OrganizationId,
    user_id: UserId,
) {
    let member_id = MemberId(Uuid::new_v4());
    members
        .expect_find_by_org_and_user()
        .with(eq(organization_id), eq(user_id))
        .times(1)
        .returning(move |organization_id, user_id| {
            let m = Member {
                id: member_id,
                organization_id,
                user_id: Some(user_id),
                last_name: "Member".to_owned(),
                first_name: None,
                joined_at: Some(Utc::now()),
                created_at: Utc::now(),
                deleted_at: None,
            };
            Box::pin(async move { Ok(Some(m)) })
        });
    members
        .expect_list_role_ids()
        .with(eq(member_id))
        .times(1)
        .returning(|_| Box::pin(async { Ok(Vec::new()) }));
    roles
        .expect_list_by_organization()
        .with(eq(organization_id))
        .times(1)
        .returning(|_| Box::pin(async { Ok(Vec::new()) }));
}

fn denying_authorizer(organization_id: OrganizationId) -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .withf(move |req| {
            req.action.name == "invoice.manage"
                && req.resource.r#type == "organization"
                && req.resource.id == organization_id.0.to_string()
        })
        .times(1)
        .returning(|_| Box::pin(async { Ok(Decision::deny()) }));
    authz
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
        invoice_number_prefix: "FAC".to_owned(),
        field_clock_enabled: false,
        vat_on_debits: false,
        mcp_enabled: false,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn complete_organization(id: OrganizationId, subject_to_vat: bool) -> Organization {
    Organization {
        legal_name: Some("Acme SARL".into()),
        legal_form: Some("SARL".into()),
        registration_number: Some("123 456 789 00012".into()),
        vat_status: Some(if subject_to_vat {
            VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }
        } else {
            VatStatus::NotSubject {
                basis: "Article 293 B du CGI".into(),
            }
        }),
        share_capital_cents: Some(1_000_000),
        address_line1: Some("12 rue des Artisans".into()),
        address_postal_code: Some("75001".into()),
        address_city: Some("Paris".into()),
        address_country: Some("FR".into()),
        insurance_mention: Some("RC Pro n°123456 - MAAF Assurances".into()),
        ..organization_without_vat_status(id)
    }
}

fn settings_with(
    padding: i64,
    iban: Option<&str>,
    late_penalty: &str,
    recovery_fee_cents: i64,
    footer: &str,
) -> BillingSettings {
    BillingSettings::try_from(BillingSettingsInput {
        payment_terms_days: 30,
        quote_validity_days: 30,
        default_vat_rate_bp: 2000,
        late_penalty: late_penalty.to_owned(),
        recovery_fee_cents,
        invoice_footer: footer.to_owned(),
        number_padding: padding,
        iban: iban.map(str::to_owned),
        bic: None,
    })
    .unwrap()
}

fn organization_repository_with_settings(
    organization: Organization,
    settings: BillingSettings,
) -> MockOrganizationRepository {
    let mut repo = MockOrganizationRepository::new();
    repo.expect_find_by_id().returning(move |_| {
        let organization = organization.clone();
        Box::pin(async move { Ok(Some(organization)) })
    });
    repo.expect_find_billing_settings().returning(move |_| {
        let settings = settings.clone();
        Box::pin(async move { Ok(Some(settings)) })
    });
    repo
}

struct World {
    organization_id: OrganizationId,
    customer_id: CustomerId,
    context_id: CustomerContextId,
    subject_to_vat: bool,
    kind: CustomerKind,
    registration_number: Option<String>,
    payment_terms: Option<i64>,
    postal_address: bool,
    settings: BillingSettings,
    store: Store,
    emitter: RecordingEmitter,
}

impl World {
    fn new() -> Self {
        Self {
            organization_id: OrganizationId(Uuid::new_v4()),
            customer_id: CustomerId(Uuid::new_v4()),
            context_id: CustomerContextId(Uuid::new_v4()),
            subject_to_vat: false,
            kind: CustomerKind::Professional,
            registration_number: Some("12345678901234".to_owned()),
            payment_terms: None,
            postal_address: true,
            settings: settings_with(4, None, "trois fois le taux légal", 4000, ""),
            store: Arc::default(),
            emitter: RecordingEmitter::new(),
        }
    }

    fn service(
        &self,
    ) -> InvoiceService<
        MockInvoiceRepository,
        &RecordingEmitter,
        MockMemberRepository,
        MockRoleRepository,
        MockAuthorizer,
    > {
        new_permissive_service(&self.store, &self.emitter)
    }

    fn organizations(&self) -> MockOrganizationRepository {
        organization_repository_with_settings(
            complete_organization(self.organization_id, self.subject_to_vat),
            self.settings.clone(),
        )
    }

    fn customers(&self) -> MockCustomerRepository {
        let mut repo = MockCustomerRepository::new();
        let organization_id = self.organization_id;
        let kind = self.kind;
        let registration_number = self.registration_number.clone();
        let payment_terms = self.payment_terms;
        repo.expect_find_by_id().returning(move |id| {
            let now = Utc::now();
            let customer = Customer {
                id,
                organization_id,
                status: CustomerStatus::Client,
                name: "Duval Masonry".to_owned(),
                registration_number: registration_number.clone(),
                phone: None,
                email: None,
                profile: CustomerProfile {
                    kind,
                    payment_terms: payment_terms
                        .map(|days| PaymentTermsDays::try_from(days).unwrap()),
                    ..CustomerProfile::default()
                },
                converted_at: None,
                archived_at: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            Box::pin(async move { Ok(Some(customer)) })
        });
        repo
    }

    fn contexts(&self) -> MockCustomerContextRepository {
        let mut repo = MockCustomerContextRepository::new();
        let customer_id = self.customer_id;
        let postal = self.postal_address;
        repo.expect_find_by_id().returning(move |id| {
            let now = Utc::now();
            let context = CustomerContext {
                id,
                customer_id,
                label: "Siège".to_owned(),
                kind: AddressKind::Billing,
                address_line: postal.then(|| "12 rue des Lilas".to_owned()),
                address_line2: None,
                postal_code: Some("69003".to_owned()),
                city: postal.then(|| "Lyon".to_owned()),
                country: "FR".to_owned(),
                photo_key: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            Box::pin(async move { Ok(Some(context)) })
        });
        repo
    }

    fn line(&self, quantity: &str, price: i32, rate: Option<i32>) -> InvoiceLineCommand {
        InvoiceLineCommand::plain("Pose de carrelage".to_owned(), dec(quantity), price, rate)
    }

    fn create_command(&self, lines: Vec<InvoiceLineCommand>) -> CreateInvoiceCommand {
        CreateInvoiceCommand {
            actor: system_actor(),
            organization_id: self.organization_id,
            kind: InvoiceKind::Standard,
            project_id: None,
            customer_id: self.customer_id,
            customer_context_id: self.context_id,
            title: "Salle de bain".to_owned(),
            payment_terms_days: None,
            legal_notice: None,
            notes: None,
            operation_nature: None,
            delivery_address: None,
            lines,
        }
    }

    async fn create(&self, lines: Vec<InvoiceLineCommand>) -> Invoice {
        self.service()
            .create_invoice(
                self.create_command(lines),
                &mut self.organizations(),
                &mut self.customers(),
                at(11, 7),
            )
            .await
            .unwrap()
            .unwrap()
    }

    async fn issue_on(
        &self,
        id: InvoiceId,
        issued_on: Option<NaiveDate>,
        now: DateTime<Utc>,
    ) -> InvoiceOutcome<Invoice> {
        self.service()
            .issue_invoice(
                IssueInvoiceCommand {
                    id,
                    actor: system_actor(),
                    issued_on,
                    allow_exceeding_total: false,
                },
                &mut self.organizations(),
                &mut MockProjectRepository::new(),
                &mut MockQuoteRepository::new(),
                &mut self.customers(),
                &mut self.contexts(),
                now,
            )
            .await
            .unwrap()
    }

    async fn issued(&self, lines: Vec<InvoiceLineCommand>) -> Invoice {
        let draft = self.create(lines).await;

        self.issue_on(draft.id, None, at(11, 7)).await.unwrap()
    }

    async fn pay(
        &self,
        id: InvoiceId,
        amount_cents: i32,
        now: DateTime<Utc>,
    ) -> InvoiceOutcome<InvoicePayment> {
        self.service()
            .record_payment(
                RecordInvoicePaymentCommand {
                    invoice_id: id,
                    actor: system_actor(),
                    amount_cents,
                    paid_on: now.date_naive(),
                    method: PaymentMethod::Transfer,
                    reference: None,
                    note: None,
                    recorded_by: UserId(Uuid::new_v4()),
                },
                now,
            )
            .await
            .unwrap()
    }

    async fn credit(
        &self,
        source: InvoiceId,
        request: CreditRequest,
        now: DateTime<Utc>,
    ) -> InvoiceOutcome<Invoice> {
        self.service()
            .issue_credit_note(
                IssueCreditNoteCommand {
                    source_invoice_id: source,
                    actor: system_actor(),
                    reason: "Malfaçon constatée".to_owned(),
                    request,
                    notes: None,
                },
                &mut self.organizations(),
                now,
            )
            .await
            .unwrap()
    }

    async fn fetch(&self, id: InvoiceId, today: NaiveDate) -> Invoice {
        self.service().get_invoice(id, today).await.unwrap()
    }

    fn quote(&self, lines: Vec<(&str, i32, Option<i32>, &str)>, status: QuoteStatus) -> Quote {
        let now = at(11, 1);
        let id = QuoteId(Uuid::new_v4());
        let quote_lines: Vec<QuoteLine> = lines
            .into_iter()
            .map(|(quantity, price, rate, discount)| QuoteLine {
                id: QuoteLineId(Uuid::new_v4()),
                organization_id: self.organization_id,
                quote_id: id,
                service_rate_id: None,
                product_id: None,
                label: "Pose".to_owned(),
                quantity: dec(quantity),
                unit: "unit".to_owned(),
                unit_price_cents: price,
                vat_rate_bp: rate,
                discount_percent: DiscountPercent::try_from(dec(discount)).unwrap(),
                billing_frequency: BillingFrequency::OneShot,
                notes: None,
                photo_keys: vec![],
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .collect();
        let net: i64 = quote_lines
            .iter()
            .map(|line| line.net_cents().unwrap())
            .sum();

        Quote {
            reference: Some("DEV-2026-0007".to_owned()),
            status,
            net_cents: i32::try_from(net).unwrap(),
            gross_cents: i32::try_from(net).unwrap(),
            lines: quote_lines,
            ..Quote::draft(
                id,
                self.organization_id,
                self.customer_id,
                self.context_id,
                "Salle de bain".to_owned(),
                now,
            )
        }
    }

    fn quotes(&self, quote: &Quote) -> MockQuoteRepository {
        let mut repo = MockQuoteRepository::new();
        let quote = quote.clone();
        repo.expect_find_by_id().returning(move |_| {
            let quote = quote.clone();
            Box::pin(async move { Ok(Some(quote)) })
        });
        repo
    }

    async fn invoice_from_quote(
        &self,
        quote: &Quote,
        mode: QuoteInvoiceMode,
        now: DateTime<Utc>,
    ) -> InvoiceOutcome<Invoice> {
        self.service()
            .create_invoice_from_quote(
                CreateInvoiceFromQuoteCommand {
                    quote_id: quote.id,
                    actor: system_actor(),
                    mode,
                },
                &mut self.organizations(),
                &mut self.quotes(quote),
                &mut self.customers(),
                now,
            )
            .await
            .unwrap()
    }

    fn stored(&self, id: InvoiceId) -> Invoice {
        state(&self.store)
            .invoices
            .iter()
            .find(|invoice| invoice.id == id)
            .cloned()
            .expect("the invoice is in the store")
    }
}

fn refusal<T: std::fmt::Debug>(outcome: InvoiceOutcome<T>) -> InvoiceError {
    outcome.expect_err("the rule must refuse")
}

#[tokio::test]
async fn create_invoice_calculates_totals_from_lines() {
    let world = World::new();

    let created = world
        .create(vec![
            world.line("4", 4_500, None),
            world.line("12.5", 3_800, None),
        ])
        .await;

    assert_eq!(created.net_cents, 65_500);
    assert_eq!(created.gross_cents, 65_500);
    assert_eq!(created.status, InvoiceStatus::Draft);
    assert_eq!(created.title, "Salle de bain");
    assert_eq!(world.emitter.names(), vec!["invoice.created".to_owned()]);
}

#[tokio::test]
async fn money_vat_is_computed_once_per_rate_on_the_summed_base() {
    let mut world = World::new();
    world.subject_to_vat = true;

    let created = world
        .create(vec![
            world.line("1", 10_033, Some(2000)),
            world.line("1", 10_033, Some(2000)),
        ])
        .await;

    assert_eq!(created.net_cents, 20_066);
    assert_eq!(created.vat_breakdown.len(), 1);
    assert_eq!(created.vat_breakdown[0].vat_cents, 4_013);
    assert_eq!(created.gross_cents, 24_079);
}

#[tokio::test]
async fn create_invoice_breaks_vat_down_per_rate() {
    let mut world = World::new();
    world.subject_to_vat = true;

    let created = world
        .create(vec![
            world.line("1", 10_000, Some(2000)),
            world.line("1", 20_000, Some(550)),
        ])
        .await;

    assert_eq!(created.net_cents, 30_000);
    assert_eq!(
        created
            .vat_breakdown
            .iter()
            .map(|line| (line.rate_bp, line.vat_cents))
            .collect::<Vec<_>>(),
        vec![(550, 1_100), (2000, 2_000)]
    );
    assert_eq!(created.gross_cents, 33_100);
}

#[tokio::test]
async fn money_a_line_discount_applies_before_vat() {
    let mut world = World::new();
    world.subject_to_vat = true;
    let mut command = world.line("1", 10_000, Some(2000));
    command.discount_percent = dec("10");

    let created = world.create(vec![command]).await;

    assert_eq!(created.net_cents, 9_000);
    assert_eq!(created.gross_cents, 10_800);
    assert_eq!(created.lines[0].discount_percent.get(), dec("10"));
}

#[tokio::test]
async fn rejects_an_invoice_with_no_lines() {
    let world = World::new();

    let outcome = world
        .service()
        .create_invoice(
            world.create_command(vec![]),
            &mut world.organizations(),
            &mut world.customers(),
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(refusal(outcome).field(), InvoiceField::Lines);
    assert!(state(&world.store).invoices.is_empty());
}

#[tokio::test]
async fn a_line_references_at_most_one_catalogue_item() {
    let world = World::new();
    let mut command = world.line("1", 100, None);
    command.product_id = Some(crate::ProductId(Uuid::new_v4()));
    command.service_rate_id = Some(crate::ServiceRateId(Uuid::new_v4()));

    let outcome = world
        .service()
        .create_invoice(
            world.create_command(vec![command]),
            &mut world.organizations(),
            &mut world.customers(),
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(refusal(outcome).field(), InvoiceField::Lines);
}

#[tokio::test]
async fn a_catalogue_reference_is_stored_on_the_line() {
    let world = World::new();
    let product = crate::ProductId(Uuid::new_v4());
    let mut command = world.line("1", 100, None);
    command.product_id = Some(product);

    let created = world.create(vec![command]).await;

    assert_eq!(created.lines[0].product_id, Some(product));
}

#[tokio::test]
async fn spec_crm_5_4_payment_terms_default_from_the_customer_then_the_settings() {
    let mut world = World::new();
    world.payment_terms = Some(45);
    let from_customer = world.create(vec![world.line("1", 100, None)]).await;
    world.payment_terms = None;
    let from_settings = world.create(vec![world.line("1", 100, None)]).await;

    assert_eq!(from_customer.payment_terms_days, 45);
    assert_eq!(from_settings.payment_terms_days, 30);
}

#[tokio::test]
async fn spec_crm_5_4_payment_terms_above_sixty_days_are_refused() {
    let world = World::new();
    let mut command = world.create_command(vec![world.line("1", 100, None)]);
    command.payment_terms_days = Some(61);

    let outcome = world
        .service()
        .create_invoice(
            command,
            &mut world.organizations(),
            &mut world.customers(),
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(refusal(outcome).field(), InvoiceField::PaymentTermsDays);
}

#[tokio::test]
async fn spec_crm_5_5_a_draft_carries_the_penalty_and_the_recovery_fee_for_a_professional() {
    let mut world = World::new();
    world.kind = CustomerKind::Professional;
    let professional = world.create(vec![world.line("1", 100, None)]).await;
    world.kind = CustomerKind::Individual;
    let individual = world.create(vec![world.line("1", 100, None)]).await;

    assert!(
        professional
            .legal_notice
            .contains("Pénalités de retard : trois fois le taux légal.")
    );
    assert!(
        professional
            .legal_notice
            .contains("Indemnité forfaitaire pour frais de recouvrement : 40,00")
    );
    assert_eq!(
        individual.legal_notice,
        "Pénalités de retard : trois fois le taux légal."
    );
}

#[tokio::test]
async fn spec_crm_5_5_a_notice_typed_by_the_artisan_is_kept() {
    let world = World::new();
    let mut command = world.create_command(vec![world.line("1", 100, None)]);
    command.legal_notice = Some("  Mention libre.  ".to_owned());

    let created = world
        .service()
        .create_invoice(
            command,
            &mut world.organizations(),
            &mut world.customers(),
            at(11, 7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(created.legal_notice, "Mention libre.");
}

#[tokio::test]
async fn update_invoice_recalculates_totals_on_a_draft() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;

    let updated = world
        .service()
        .update_invoice(
            UpdateInvoiceCommand {
                id: draft.id,
                actor: system_actor(),
                project_id: None,
                customer_id: world.customer_id,
                customer_context_id: world.context_id,
                title: Some("Nouveau titre".to_owned()),
                payment_terms_days: Some(15),
                legal_notice: None,
                notes: None,
                operation_nature: None,
                delivery_address: None,
                lines: vec![world.line("2", 2_500, None)],
            },
            &mut world.organizations(),
            at(11, 8),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.net_cents, 5_000);
    assert_eq!(updated.title, "Nouveau titre");
    assert_eq!(updated.payment_terms_days, 15);
    assert_eq!(updated.legal_notice, draft.legal_notice);
}

#[tokio::test]
async fn spec_crm_5_6_an_issued_invoice_cannot_be_edited() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;

    let outcome = world
        .service()
        .update_invoice(
            UpdateInvoiceCommand {
                id: issued.id,
                actor: system_actor(),
                project_id: None,
                customer_id: world.customer_id,
                customer_context_id: world.context_id,
                title: None,
                payment_terms_days: None,
                legal_notice: None,
                notes: None,
                operation_nature: None,
                delivery_address: None,
                lines: vec![world.line("9", 9_000, None)],
            },
            &mut world.organizations(),
            at(11, 8),
        )
        .await
        .unwrap();

    assert_eq!(refusal(outcome), InvoiceError::Locked);
    assert_eq!(world.stored(issued.id).net_cents, 1_000);
}

#[tokio::test]
async fn spec_crm_5_6_an_issued_invoice_cannot_be_deleted_and_a_draft_can() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;

    let locked = world
        .service()
        .soft_delete_invoice(issued.id, system_actor(), at(11, 8))
        .await
        .unwrap();
    let deleted = world
        .service()
        .soft_delete_invoice(draft.id, system_actor(), at(11, 8))
        .await
        .unwrap();

    assert_eq!(refusal(locked), InvoiceError::Locked);
    assert_eq!(deleted, Ok(()));
    assert!(
        state(&world.store)
            .invoices
            .iter()
            .all(|invoice| invoice.id != draft.id)
    );
    assert!(
        world
            .emitter
            .names()
            .contains(&"invoice.deleted".to_owned())
    );
}

#[tokio::test]
async fn spec_crm_5_6_a_draft_deleted_consumes_no_number() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    world
        .service()
        .soft_delete_invoice(draft.id, system_actor(), at(11, 8))
        .await
        .unwrap()
        .unwrap();

    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;

    assert_eq!(issued.number.as_deref(), Some("FAC-2026-0001"));
}

#[tokio::test]
async fn spec_crm_5_1_issuing_needs_a_draft() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;

    let outcome = world.issue_on(issued.id, None, at(11, 8)).await;

    assert_eq!(refusal(outcome), InvoiceError::Locked);
}

#[tokio::test]
async fn spec_crm_5_1_issuing_refuses_a_missing_billing_street_and_city() {
    let mut world = World::new();
    world.postal_address = false;
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;

    let outcome = world.issue_on(draft.id, None, at(11, 7)).await;

    assert_eq!(refusal(outcome).field(), InvoiceField::BillingAddress);
    assert_eq!(world.stored(draft.id).status, InvoiceStatus::Draft);
}

#[tokio::test]
async fn spec_crm_5_1_issuing_refuses_a_professional_without_a_siret() {
    let mut world = World::new();
    world.registration_number = None;
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;

    let outcome = world.issue_on(draft.id, None, at(11, 7)).await;

    assert_eq!(refusal(outcome).field(), InvoiceField::RegistrationNumber);

    world.kind = CustomerKind::Individual;
    assert!(world.issue_on(draft.id, None, at(11, 7)).await.is_ok());
}

#[tokio::test]
async fn spec_crm_5_1_issuing_refuses_a_zero_total() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 0, None)]).await;

    let outcome = world.issue_on(draft.id, None, at(11, 7)).await;

    assert_eq!(refusal(outcome).field(), InvoiceField::Total);
}

#[tokio::test]
async fn spec_crm_5_1_issuing_for_an_unknown_customer_is_not_found() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    let mut customers = MockCustomerRepository::new();
    customers
        .expect_find_by_id()
        .returning(|_| Box::pin(async { Ok(None) }));

    let outcome = world
        .service()
        .issue_invoice(
            IssueInvoiceCommand {
                id: draft.id,
                actor: system_actor(),
                issued_on: None,
                allow_exceeding_total: false,
            },
            &mut world.organizations(),
            &mut MockProjectRepository::new(),
            &mut MockQuoteRepository::new(),
            &mut customers,
            &mut world.contexts(),
            at(11, 7),
        )
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
}

#[tokio::test]
async fn spec_crm_5_2_an_issue_date_in_the_future_is_refused() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;

    let outcome = world.issue_on(draft.id, Some(date(11, 8)), at(11, 7)).await;

    assert_eq!(refusal(outcome).code(), "E_INVOICE_DATE_RANGE");
    assert_eq!(world.stored(draft.id).status, InvoiceStatus::Draft);
}

#[tokio::test]
async fn spec_crm_5_2_an_issue_date_before_the_last_issued_invoice_is_refused() {
    let world = World::new();
    let first = world.create(vec![world.line("1", 1_000, None)]).await;
    let second = world.create(vec![world.line("1", 1_000, None)]).await;
    world
        .issue_on(first.id, Some(date(11, 5)), at(11, 7))
        .await
        .unwrap();

    let before = world
        .issue_on(second.id, Some(date(11, 4)), at(11, 7))
        .await;
    let same_day = world
        .issue_on(second.id, Some(date(11, 5)), at(11, 7))
        .await;

    assert_eq!(refusal(before).code(), "E_INVOICE_DATE_RANGE");
    assert!(same_day.is_ok());
}

#[tokio::test]
async fn spec_crm_5_3_the_number_is_assigned_at_issue_with_the_year_of_the_issue_date() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    assert_eq!(draft.number, None);
    let last_year = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let now = Utc.with_ymd_and_hms(2026, 1, 2, 9, 0, 0).unwrap();

    let issued = world
        .issue_on(draft.id, Some(last_year), now)
        .await
        .unwrap();

    assert_eq!(issued.number.as_deref(), Some("FAC-2025-0001"));
    assert_eq!(issued.issued_on, Some(last_year));
}

#[tokio::test]
async fn spec_crm_5_4_the_due_date_is_the_issue_date_plus_the_payment_terms() {
    let world = World::new();
    let mut command = world.create_command(vec![world.line("1", 1_000, None)]);
    command.payment_terms_days = Some(45);
    let draft = world
        .service()
        .create_invoice(
            command,
            &mut world.organizations(),
            &mut world.customers(),
            at(11, 7),
        )
        .await
        .unwrap()
        .unwrap();

    let issued = world
        .issue_on(draft.id, Some(date(11, 3)), at(11, 7))
        .await
        .unwrap();

    assert_eq!(issued.due_on(), Some(date(12, 18)));
    assert_eq!(issued.derived.display_state, InvoiceDisplayState::Unpaid);
    assert!(world.emitter.names().contains(&"invoice.issued".to_owned()));
}

#[tokio::test]
async fn spec_crm_1_4_a_standard_invoice_keeps_the_invoice_series() {
    let world = World::new();

    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;

    assert_eq!(issued.number.as_deref(), Some("FAC-2026-0001"));
}

#[tokio::test]
async fn spec_crm_1_6_an_issued_invoice_keeps_the_billing_mentions_it_had_at_issue() {
    let mut world = World::new();
    let first = world.create(vec![world.line("1", 1_000, None)]).await;
    let second = world.create(vec![world.line("1", 1_000, None)]).await;
    let before = settings_with(
        4,
        Some("FR76 3000 6000 0112 3456 7890 189"),
        "trois fois le taux légal",
        4000,
        "Merci.",
    );
    let after = settings_with(
        4,
        Some("FR14 2004 1010 0505 0001 3M02 606"),
        "dix pour cent",
        0,
        "",
    );

    world.settings = before.clone();
    world.issue_on(first.id, None, at(11, 7)).await.unwrap();
    world.settings = after.clone();
    world.issue_on(second.id, None, at(11, 7)).await.unwrap();

    let frozen = |id: InvoiceId| world.stored(id).issuer_identity.and_then(|i| i.billing);
    assert_eq!(frozen(first.id), Some(IssuerBilling::from(&before)));
    assert_eq!(frozen(second.id), Some(IssuerBilling::from(&after)));
    assert_ne!(frozen(first.id), frozen(second.id));
}

#[tokio::test]
async fn spec_crm_5_7_a_full_invoice_copies_the_lines_and_deducts_each_issued_deposit_per_rate() {
    let mut world = World::new();
    world.subject_to_vat = true;
    let quote = world.quote(
        vec![
            ("2", 50_000, Some(2000), "10"),
            ("1", 20_000, Some(1000), "0"),
        ],
        QuoteStatus::Accepted,
    );
    let deposit_draft = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("30") },
            at(11, 7),
        )
        .await
        .unwrap();
    let issued_deposit = world
        .issue_on(deposit_draft.id, None, at(11, 7))
        .await
        .unwrap();
    world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("10") },
            at(11, 7),
        )
        .await
        .unwrap();

    let full = world
        .invoice_from_quote(&quote, QuoteInvoiceMode::Full, at(11, 8))
        .await
        .unwrap();

    assert_eq!(full.kind, InvoiceKind::Standard);
    assert_eq!(full.quote_id, Some(quote.id));
    assert_eq!(full.title, "Salle de bain");
    assert_eq!(full.lines.len(), 4);
    assert_eq!(full.lines[0].discount_percent.get(), dec("10"));
    let deductions: Vec<(Option<i32>, i32)> = full.lines[2..]
        .iter()
        .map(|line| (line.vat_rate_basis_points, line.unit_price_cents))
        .collect();
    assert_eq!(
        deductions,
        vec![(Some(1000), -6_000), (Some(2000), -27_000)]
    );
    assert!(
        full.lines[2]
            .label
            .contains(issued_deposit.number.as_deref().unwrap_or("?"))
    );
    assert_eq!(full.net_cents, 90_000 + 20_000 - 33_000);
}

#[tokio::test]
async fn spec_crm_5_7_a_quote_that_is_not_accepted_cannot_be_invoiced() {
    let world = World::new();
    let quote = world.quote(vec![("1", 10_000, None, "0")], QuoteStatus::Sent);

    let outcome = world
        .invoice_from_quote(&quote, QuoteInvoiceMode::Full, at(11, 7))
        .await;

    assert_eq!(refusal(outcome), InvoiceError::QuoteNotAccepted);
    assert!(state(&world.store).invoices.is_empty());
}

#[tokio::test]
async fn spec_crm_5_8_a_deposit_has_one_lump_line_per_rate_priced_on_the_percentage() {
    let mut world = World::new();
    world.subject_to_vat = true;
    let quote = world.quote(
        vec![
            ("1", 100_000, Some(2000), "0"),
            ("1", 50_000, Some(1000), "0"),
        ],
        QuoteStatus::Accepted,
    );

    let deposit = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("30") },
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(deposit.kind, InvoiceKind::Deposit);
    assert_eq!(
        deposit
            .lines
            .iter()
            .map(|line| (line.vat_rate_basis_points, line.unit_price_cents))
            .collect::<Vec<_>>(),
        vec![(Some(1000), 15_000), (Some(2000), 30_000)]
    );
    assert_eq!(deposit.net_cents, 45_000);
}

#[tokio::test]
async fn spec_crm_5_8_a_deposit_above_the_remaining_percentage_is_refused() {
    let world = World::new();
    let quote = world.quote(vec![("1", 100_000, None, "0")], QuoteStatus::Accepted);
    world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("60") },
            at(11, 7),
        )
        .await
        .unwrap();

    let too_much = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit {
                percent: dec("40.1"),
            },
            at(11, 7),
        )
        .await;
    let exact = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("40") },
            at(11, 7),
        )
        .await;
    let covered = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("1") },
            at(11, 7),
        )
        .await;

    assert_eq!(refusal(too_much), InvoiceError::DepositExceedsRemaining);
    assert!(exact.is_ok());
    assert_eq!(refusal(covered), InvoiceError::DepositExceedsRemaining);
}

#[tokio::test]
async fn spec_crm_5_8_a_deposit_needs_a_positive_percentage() {
    let world = World::new();
    let quote = world.quote(vec![("1", 100_000, None, "0")], QuoteStatus::Accepted);

    let outcome = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("0") },
            at(11, 7),
        )
        .await;

    assert_eq!(refusal(outcome).field(), InvoiceField::Percent);
}

#[tokio::test]
async fn spec_crm_5_9_an_active_standard_invoice_blocks_the_quote_and_a_cancelled_one_does_not() {
    let world = World::new();
    let quote = world.quote(vec![("1", 100_000, None, "0")], QuoteStatus::Accepted);
    let full = world
        .invoice_from_quote(&quote, QuoteInvoiceMode::Full, at(11, 7))
        .await
        .unwrap();

    let again = world
        .invoice_from_quote(&quote, QuoteInvoiceMode::Full, at(11, 7))
        .await;
    let deposit = world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit { percent: dec("10") },
            at(11, 7),
        )
        .await;
    assert_eq!(refusal(again), InvoiceError::QuoteAlreadyInvoiced);
    assert_eq!(refusal(deposit), InvoiceError::QuoteAlreadyInvoiced);

    world.issue_on(full.id, None, at(11, 7)).await.unwrap();
    world
        .credit(full.id, CreditRequest::Full, at(11, 7))
        .await
        .unwrap();
    let after_cancellation = world
        .invoice_from_quote(&quote, QuoteInvoiceMode::Full, at(11, 8))
        .await;

    assert!(after_cancellation.is_ok());
}

#[tokio::test]
async fn spec_crm_5_9_a_deposit_covered_quote_reads_through_the_invoicing_view() {
    let world = World::new();
    let quote = world.quote(vec![("1", 100_000, None, "0")], QuoteStatus::Accepted);
    world
        .invoice_from_quote(
            &quote,
            QuoteInvoiceMode::Deposit {
                percent: dec("100"),
            },
            at(11, 7),
        )
        .await
        .unwrap();

    let view = world
        .service()
        .quote_invoicing(quote.id, &mut world.quotes(&quote), date(11, 7))
        .await
        .unwrap();

    assert_eq!(view.organization_id, world.organization_id);
    assert!(view.invoicing.can_full);
    assert!(!view.invoicing.can_deposit);
    assert_eq!(view.invoicing.invoiced_percent, dec("100.0"));
    assert_eq!(view.invoices.len(), 1);
}

#[tokio::test]
async fn spec_crm_5_10_a_payment_is_refused_above_the_balance() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let over = world.pay(issued.id, 10_001, at(11, 7)).await;
    let exact = world.pay(issued.id, 10_000, at(11, 7)).await;

    assert_eq!(refusal(over), InvoiceError::PaymentExceedsBalance);
    assert!(exact.is_ok());
    assert_eq!(state(&world.store).payments.len(), 1);
}

#[tokio::test]
async fn spec_crm_5_10_a_refund_is_refused_above_the_net_amount_collected() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    world.pay(issued.id, 6_000, at(11, 7)).await.unwrap();

    let over = world.pay(issued.id, -6_001, at(11, 7)).await;
    let refund = world.pay(issued.id, -2_500, at(11, 7)).await;
    let rest = world.pay(issued.id, -3_501, at(11, 7)).await;

    assert_eq!(refusal(over), InvoiceError::RefundExceedsCollected);
    assert!(refund.is_ok());
    assert_eq!(refusal(rest), InvoiceError::RefundExceedsCollected);
    assert_eq!(
        world
            .fetch(issued.id, date(11, 7))
            .await
            .derived
            .settlement
            .paid_cents,
        3_500
    );
}

#[tokio::test]
async fn spec_crm_5_11_a_payment_needs_a_non_zero_amount_and_a_date_not_in_the_future() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let zero = world.pay(issued.id, 0, at(11, 7)).await;
    let future = world
        .service()
        .record_payment(
            RecordInvoicePaymentCommand {
                invoice_id: issued.id,
                actor: system_actor(),
                amount_cents: 100,
                paid_on: date(11, 8),
                method: PaymentMethod::Cash,
                reference: None,
                note: None,
                recorded_by: UserId(Uuid::new_v4()),
            },
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(refusal(zero).field(), InvoiceField::AmountCents);
    assert_eq!(refusal(future).field(), InvoiceField::PaidOn);
}

#[tokio::test]
async fn spec_crm_5_11_a_draft_cannot_receive_a_payment() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 10_000, None)]).await;

    let outcome = world.pay(draft.id, 100, at(11, 7)).await;

    assert_eq!(refusal(outcome), InvoiceError::NotIssued);
}

#[tokio::test]
async fn spec_crm_5_12_the_display_state_follows_the_payments_and_the_due_date() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let unpaid = world.fetch(issued.id, date(11, 7)).await;
    world.pay(issued.id, 4_000, at(11, 7)).await.unwrap();
    let partial = world.fetch(issued.id, date(11, 7)).await;
    let overdue = world.fetch(issued.id, date(12, 8)).await;
    world.pay(issued.id, 6_000, at(11, 7)).await.unwrap();
    let paid = world.fetch(issued.id, date(12, 8)).await;

    assert_eq!(unpaid.derived.display_state, InvoiceDisplayState::Unpaid);
    assert_eq!(partial.derived.display_state, InvoiceDisplayState::Partial);
    assert_eq!(partial.derived.settlement.balance_cents, 6_000);
    assert_eq!(overdue.derived.display_state, InvoiceDisplayState::Overdue);
    assert_eq!(paid.derived.display_state, InvoiceDisplayState::Paid);
    assert_eq!(paid.status, InvoiceStatus::Paid);
}

#[tokio::test]
async fn spec_crm_5_12_a_payment_then_a_full_credit_is_a_refund_due() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    world.pay(issued.id, 10_000, at(11, 7)).await.unwrap();
    world
        .credit(issued.id, CreditRequest::Full, at(11, 7))
        .await
        .unwrap();

    let source = world.fetch(issued.id, date(11, 7)).await;

    assert_eq!(source.derived.display_state, InvoiceDisplayState::RefundDue);
    assert_eq!(source.derived.settlement.balance_cents, -10_000);
}

#[tokio::test]
async fn spec_crm_5_13_overdue_means_the_due_date_is_strictly_before_today() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    let due = issued.due_on().unwrap();

    let on_the_day = world.fetch(issued.id, due).await;
    let next_day = world.fetch(issued.id, due.succ_opt().unwrap()).await;

    assert_eq!(
        on_the_day.derived.display_state,
        InvoiceDisplayState::Unpaid
    );
    assert_eq!(next_day.derived.display_state, InvoiceDisplayState::Overdue);
}

#[tokio::test]
async fn spec_crm_5_18_list_rows_carry_paid_credited_balance_and_display_state() {
    let world = World::new();
    let paid = world.issued(vec![world.line("1", 10_000, None)]).await;
    let credited = world.issued(vec![world.line("2", 5_000, None)]).await;
    let draft = world.create(vec![world.line("1", 700, None)]).await;
    world.pay(paid.id, 10_000, at(11, 7)).await.unwrap();
    world
        .credit(
            credited.id,
            CreditRequest::Lines(vec![CreditLineRequest {
                source_line_id: credited.lines[0].id,
                quantity: dec("1"),
            }]),
            at(11, 7),
        )
        .await
        .unwrap();

    let (rows, total) = world
        .service()
        .list_invoices(world.organization_id, 50, 0, date(11, 7))
        .await
        .unwrap();

    let row = |id: InvoiceId| rows.iter().find(|row| row.id == id).unwrap();
    assert_eq!(total, 4);
    assert_eq!(row(paid.id).derived.settlement.paid_cents, 10_000);
    assert_eq!(row(paid.id).derived.settlement.balance_cents, 0);
    assert_eq!(
        row(paid.id).derived.display_state,
        InvoiceDisplayState::Paid
    );
    assert_eq!(row(credited.id).derived.settlement.credited_cents, 5_000);
    assert_eq!(row(credited.id).derived.settlement.balance_cents, 5_000);
    assert_eq!(
        row(credited.id).derived.display_state,
        InvoiceDisplayState::Unpaid
    );
    assert_eq!(
        row(draft.id).derived.display_state,
        InvoiceDisplayState::Draft
    );
    assert_eq!(row(draft.id).derived.settlement.balance_cents, 0);
}

#[tokio::test]
async fn spec_crm_5_14_a_credit_note_needs_an_issued_invoice() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 10_000, None)]).await;

    let outcome = world.credit(draft.id, CreditRequest::Full, at(11, 7)).await;

    assert_eq!(refusal(outcome), InvoiceError::NotIssued);
}

#[tokio::test]
async fn spec_crm_5_14_a_credit_note_needs_a_reason() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let outcome = world
        .service()
        .issue_credit_note(
            IssueCreditNoteCommand {
                source_invoice_id: issued.id,
                actor: system_actor(),
                reason: "   ".to_owned(),
                request: CreditRequest::Full,
                notes: None,
            },
            &mut world.organizations(),
            at(11, 7),
        )
        .await
        .unwrap();

    assert_eq!(refusal(outcome).field(), InvoiceField::Reason);
    assert_eq!(state(&world.store).invoices.len(), 1);
}

#[tokio::test]
async fn spec_crm_5_14_a_credit_note_cannot_credit_a_credit_note() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    let note = world
        .credit(issued.id, CreditRequest::Full, at(11, 7))
        .await
        .unwrap();

    let outcome = world
        .service()
        .issue_credit_note(
            IssueCreditNoteCommand {
                source_invoice_id: note.id,
                actor: system_actor(),
                reason: "x".to_owned(),
                request: CreditRequest::Full,
                notes: None,
            },
            &mut world.organizations(),
            at(11, 7),
        )
        .await;

    assert!(matches!(outcome, Err(CoreError::Conflict(_))));
}

#[tokio::test]
async fn spec_crm_5_14_lines_beyond_what_remains_on_the_source_line_are_refused() {
    let world = World::new();
    let issued = world.issued(vec![world.line("4", 5_000, None)]).await;
    let line = issued.lines[0].id;
    let partial = |quantity: &str| {
        CreditRequest::Lines(vec![CreditLineRequest {
            source_line_id: line,
            quantity: dec(quantity),
        }])
    };
    world
        .credit(issued.id, partial("3"), at(11, 7))
        .await
        .unwrap();

    let over = world.credit(issued.id, partial("1.5"), at(11, 7)).await;
    let rest = world.credit(issued.id, partial("1"), at(11, 7)).await;

    assert_eq!(refusal(over), InvoiceError::CreditExceedsRemaining);
    assert!(rest.is_ok());
}

#[tokio::test]
async fn spec_crm_5_15_a_credit_note_is_dated_today_and_numbered_in_the_av_series() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let note = world
        .credit(issued.id, CreditRequest::Full, at(11, 9))
        .await
        .unwrap();

    assert_eq!(note.kind, InvoiceKind::CreditNote);
    assert_eq!(note.issued_on, Some(date(11, 9)));
    assert_eq!(note.number.as_deref(), Some("AV-2026-0001"));
    assert_eq!(note.source_invoice_id, Some(issued.id));
    assert_eq!(note.reason.as_deref(), Some("Malfaçon constatée"));
    assert_eq!(note.status, InvoiceStatus::Issued);
}

#[tokio::test]
async fn spec_crm_5_15_a_credit_note_is_refused_before_the_last_credit_note_or_the_invoice() {
    let world = World::new();
    let first = world.issued(vec![world.line("2", 5_000, None)]).await;
    let line = first.lines[0].id;
    let one = |quantity: &str| {
        CreditRequest::Lines(vec![CreditLineRequest {
            source_line_id: line,
            quantity: dec(quantity),
        }])
    };
    world.credit(first.id, one("1"), at(11, 9)).await.unwrap();

    let before_last_note = world.credit(first.id, one("1"), at(11, 8)).await;
    let same_day = world.credit(first.id, one("1"), at(11, 9)).await;

    assert_eq!(refusal(before_last_note).code(), "E_INVOICE_DATE_RANGE");
    assert!(same_day.is_ok());
}

#[tokio::test]
async fn spec_crm_5_15_a_credit_note_is_refused_before_the_invoice_it_corrects() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 10_000, None)]).await;
    let issued = world
        .issue_on(draft.id, Some(date(11, 7)), at(11, 7))
        .await
        .unwrap();

    let outcome = world
        .credit(issued.id, CreditRequest::Full, at(11, 6))
        .await;

    assert_eq!(refusal(outcome).code(), "E_INVOICE_DATE_RANGE");
}

#[tokio::test]
async fn spec_crm_5_16_crediting_every_remaining_line_sets_full_cancellation_and_leaves_the_source_unchanged()
 {
    let world = World::new();
    let issued = world
        .issued(vec![
            world.line("4", 5_000, None),
            world.line("1", 30_000, None),
        ])
        .await;
    let first_line = issued.lines[0].id;
    let partial = world
        .credit(
            issued.id,
            CreditRequest::Lines(vec![CreditLineRequest {
                source_line_id: first_line,
                quantity: dec("4"),
            }]),
            at(11, 7),
        )
        .await
        .unwrap();

    let rest = world
        .credit(issued.id, CreditRequest::Full, at(11, 8))
        .await
        .unwrap();

    assert!(!partial.full_cancellation);
    assert!(rest.full_cancellation);
    assert_eq!(rest.lines.len(), 1);
    let source = world.stored(issued.id);
    assert_eq!(source.status, InvoiceStatus::Issued);
    assert_eq!(source.net_cents, issued.net_cents);
    assert_eq!(source.lines, issued.lines);
    let read = world.fetch(issued.id, date(11, 8)).await;
    assert_eq!(read.status, InvoiceStatus::Issued);
    assert_eq!(read.derived.display_state, InvoiceDisplayState::Cancelled);
    assert_eq!(read.derived.settlement.balance_cents, 0);
}

#[tokio::test]
async fn spec_crm_5_17_sending_records_sent_at_and_an_event_without_touching_the_balance() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;

    let sent = world
        .service()
        .send_invoice(
            SendInvoiceCommand {
                id: issued.id,
                actor: system_actor(),
                recorded_by: UserId(Uuid::new_v4()),
            },
            at(11, 8),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(sent.sent_at, Some(at(11, 8)));
    assert_eq!(sent.derived.events.len(), 1);
    assert_eq!(sent.derived.events[0].kind, InvoiceEventKind::Sent);
    assert_eq!(sent.derived.settlement.balance_cents, 10_000);
}

#[tokio::test]
async fn spec_crm_5_17_reminding_records_the_days_late() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    let due = issued.due_on().unwrap();
    let now = Utc.from_utc_datetime(
        &(due + chrono::Duration::days(6))
            .and_hms_opt(9, 0, 0)
            .unwrap(),
    );

    let reminded = world
        .service()
        .remind_invoice(
            RemindInvoiceCommand {
                id: issued.id,
                actor: system_actor(),
                recorded_by: UserId(Uuid::new_v4()),
            },
            now,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(reminded.derived.events.len(), 1);
    assert_eq!(reminded.derived.events[0].kind, InvoiceEventKind::Reminder);
    assert_eq!(
        reminded.derived.events[0].detail,
        "Relance envoyée (6 jours de retard)"
    );
    assert_eq!(reminded.derived.settlement.balance_cents, 10_000);
    assert_eq!(reminded.sent_at, None);
}

#[tokio::test]
async fn spec_crm_5_17_a_draft_can_be_neither_sent_nor_reminded() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 10_000, None)]).await;
    let recorded_by = UserId(Uuid::new_v4());

    let sent = world
        .service()
        .send_invoice(
            SendInvoiceCommand {
                id: draft.id,
                actor: system_actor(),
                recorded_by,
            },
            at(11, 8),
        )
        .await
        .unwrap();
    let reminded = world
        .service()
        .remind_invoice(
            RemindInvoiceCommand {
                id: draft.id,
                actor: system_actor(),
                recorded_by,
            },
            at(11, 8),
        )
        .await
        .unwrap();

    assert_eq!(refusal(sent), InvoiceError::NotIssued);
    assert_eq!(refusal(reminded), InvoiceError::NotIssued);
    assert!(state(&world.store).events.is_empty());
}

#[tokio::test]
async fn spec_crm_5_17_a_note_sets_the_internal_note_and_records_an_event() {
    let world = World::new();
    let issued = world.issued(vec![world.line("1", 10_000, None)]).await;
    let note = |text: &str| NoteInvoiceCommand {
        id: issued.id,
        actor: system_actor(),
        recorded_by: UserId(Uuid::new_v4()),
        note: text.to_owned(),
    };

    let noted = world
        .service()
        .note_invoice(note("  Rappeler mardi  "), at(11, 8))
        .await
        .unwrap()
        .unwrap();
    let cleared = world
        .service()
        .note_invoice(note("  "), at(11, 9))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(noted.internal_note.as_deref(), Some("Rappeler mardi"));
    assert_eq!(noted.derived.events.len(), 1);
    assert_eq!(noted.derived.events[0].kind, InvoiceEventKind::Note);
    assert_eq!(noted.derived.events[0].detail, "Rappeler mardi");
    assert_eq!(cleared.internal_note, None);
    assert_eq!(cleared.derived.events.len(), 1);
}

#[tokio::test]
async fn cancel_invoice_transitions_a_draft_and_refuses_a_second_time() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    let cancel = || CancelInvoiceCommand {
        id: draft.id,
        actor: system_actor(),
    };

    let cancelled = world.service().cancel_invoice(cancel()).await.unwrap();
    let again = world.service().cancel_invoice(cancel()).await;

    assert_eq!(cancelled.status, InvoiceStatus::Cancelled);
    assert!(matches!(again, Err(CoreError::Conflict(_))));
    assert!(
        world
            .emitter
            .names()
            .contains(&"invoice.cancelled".to_owned())
    );
}

fn complete_project(
    organization_id: OrganizationId,
    quote_id: Option<QuoteId>,
    customer_id: CustomerId,
    context_id: CustomerContextId,
) -> Project {
    let now = Utc::now();
    Project {
        id: ProjectId(Uuid::new_v4()),
        organization_id,
        name: "Chantier".to_owned(),
        customer_id: Some(customer_id),
        customer_context_id: Some(context_id),
        quote_id,
        archived_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn project_repo_with(project: Project) -> MockProjectRepository {
    let mut repo = MockProjectRepository::new();
    repo.expect_find_by_id().returning(move |_| {
        let project = project.clone();
        Box::pin(async move { Ok(Some(project)) })
    });
    repo
}

fn net_quote(world: &World, net_cents: i32) -> Quote {
    Quote {
        reference: Some("DEV-2026-0001".to_owned()),
        status: QuoteStatus::Accepted,
        net_cents,
        gross_cents: net_cents,
        ..Quote::draft(
            QuoteId(Uuid::new_v4()),
            world.organization_id,
            world.customer_id,
            world.context_id,
            "Chantier".to_owned(),
            Utc::now(),
        )
    }
}

#[tokio::test]
async fn deposits_then_final_invoice_round_to_a_zero_remainder() {
    let world = World::new();
    let quote = net_quote(&world, 100_007);
    let project = complete_project(
        world.organization_id,
        Some(quote.id),
        world.customer_id,
        world.context_id,
    );
    let deposit = |bp: i32| IssueDepositCommand {
        project_id: project.id,
        actor: system_actor(),
        percentage_bp: bp,
        payment_terms_days: None,
        notes: None,
        allow_exceeding_total: false,
    };

    let issue = || async {
        world
            .service()
            .issue_deposit(
                deposit(3000),
                &mut world.organizations(),
                &mut project_repo_with(project.clone()),
                &mut world.quotes(&quote),
                &mut world.customers(),
                &mut world.contexts(),
                at(11, 7),
            )
            .await
            .unwrap()
            .unwrap()
    };
    let first = issue().await;
    let second = issue().await;
    let last = world
        .service()
        .issue_final_invoice(
            IssueFinalInvoiceCommand {
                actor: system_actor(),
                project_id: project.id,
                payment_terms_days: None,
                notes: None,
                allow_exceeding_total: false,
            },
            &mut world.organizations(),
            &mut project_repo_with(project.clone()),
            &mut world.quotes(&quote),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(first.net_cents, 30_002);
    assert_eq!(second.net_cents, 30_002);
    assert_eq!(last.net_cents, 40_003);
    assert_eq!(
        first.net_cents + second.net_cents + last.net_cents,
        100_007,
        "the remainder after every invoice must round to exactly zero"
    );
    assert_eq!(first.kind, InvoiceKind::Deposit);
    assert_eq!(last.kind, InvoiceKind::Final);
    assert_eq!(last.derived.display_state, InvoiceDisplayState::Unpaid);
}

#[tokio::test]
async fn issue_deposit_refuses_to_exceed_the_quoted_total_unless_allowed() {
    let world = World::new();
    let quote = net_quote(&world, 10_000);
    let project = complete_project(
        world.organization_id,
        Some(quote.id),
        world.customer_id,
        world.context_id,
    );
    let command = |allow: bool| IssueDepositCommand {
        project_id: project.id,
        actor: system_actor(),
        percentage_bp: 9000,
        payment_terms_days: None,
        notes: None,
        allow_exceeding_total: allow,
    };
    let first = world
        .service()
        .issue_deposit(
            command(false),
            &mut world.organizations(),
            &mut project_repo_with(project.clone()),
            &mut world.quotes(&quote),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await
        .unwrap();
    assert!(first.is_ok());

    let refused = world
        .service()
        .issue_deposit(
            command(false),
            &mut world.organizations(),
            &mut project_repo_with(project.clone()),
            &mut world.quotes(&quote),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await;
    let allowed = world
        .service()
        .issue_deposit(
            command(true),
            &mut world.organizations(),
            &mut project_repo_with(project.clone()),
            &mut world.quotes(&quote),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 8),
        )
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(refused, Err(CoreError::Conflict(_))));
    assert_eq!(state(&world.store).invoices.len(), 2);
    assert_eq!(allowed.status, InvoiceStatus::Issued);
}

#[tokio::test]
async fn issue_deposit_refuses_a_project_with_no_quote() {
    let world = World::new();
    let project = complete_project(
        world.organization_id,
        None,
        world.customer_id,
        world.context_id,
    );

    let refused = world
        .service()
        .issue_deposit(
            IssueDepositCommand {
                project_id: project.id,
                actor: system_actor(),
                percentage_bp: 3000,
                payment_terms_days: None,
                notes: None,
                allow_exceeding_total: false,
            },
            &mut world.organizations(),
            &mut project_repo_with(project),
            &mut MockQuoteRepository::new(),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await;

    assert!(matches!(refused, Err(CoreError::Conflict(_))));
}

#[tokio::test]
async fn issue_final_invoice_refuses_a_project_with_no_quote() {
    let world = World::new();
    let project = complete_project(
        world.organization_id,
        None,
        world.customer_id,
        world.context_id,
    );

    let refused = world
        .service()
        .issue_final_invoice(
            IssueFinalInvoiceCommand {
                actor: system_actor(),
                project_id: project.id,
                payment_terms_days: None,
                notes: None,
                allow_exceeding_total: false,
            },
            &mut world.organizations(),
            &mut project_repo_with(project),
            &mut MockQuoteRepository::new(),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await;

    assert!(matches!(refused, Err(CoreError::Conflict(_))));
}

#[tokio::test]
async fn issue_invoice_refuses_an_incomplete_organization_legal_identity() {
    let world = World::new();
    let draft = world.create(vec![world.line("1", 1_000, None)]).await;
    let mut organizations = MockOrganizationRepository::new();
    let incomplete = organization_without_vat_status(world.organization_id);
    organizations.expect_find_by_id().returning(move |_| {
        let organization = incomplete.clone();
        Box::pin(async move { Ok(Some(organization)) })
    });

    let refused = world
        .service()
        .issue_invoice(
            IssueInvoiceCommand {
                id: draft.id,
                actor: system_actor(),
                issued_on: None,
                allow_exceeding_total: false,
            },
            &mut organizations,
            &mut MockProjectRepository::new(),
            &mut MockQuoteRepository::new(),
            &mut world.customers(),
            &mut world.contexts(),
            at(11, 7),
        )
        .await;

    assert!(matches!(refused, Err(CoreError::Conflict(_))));
    assert_eq!(world.stored(draft.id).status, InvoiceStatus::Draft);
}

async fn assert_forbidden<F, T>(
    organization_id: OrganizationId,
    user_id: UserId,
    repo: MockInvoiceRepository,
    call: F,
) where
    F: for<'a> FnOnce(
        &'a mut InvoiceService<
            MockInvoiceRepository,
            RecordingEmitter,
            MockMemberRepository,
            MockRoleRepository,
            MockAuthorizer,
        >,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, CoreError>> + 'a>,
    >,
    T: std::fmt::Debug,
{
    let mut member_repository = MockMemberRepository::new();
    let mut role_repository = MockRoleRepository::new();
    stage_org_membership(
        &mut member_repository,
        &mut role_repository,
        organization_id,
        user_id,
    );
    let mut service = InvoiceService::new(
        repo,
        RecordingEmitter::new(),
        member_repository,
        role_repository,
        denying_authorizer(organization_id),
    );

    let result = call(&mut service).await;

    assert!(
        matches!(result, Err(CoreError::Forbidden { .. })),
        "{result:?}"
    );
}

fn found_in(existing: Invoice) -> MockInvoiceRepository {
    let mut repo = MockInvoiceRepository::new();
    repo.expect_find_by_id().returning(move |_| {
        let existing = existing.clone();
        Box::pin(async move { Ok(Some(existing)) })
    });
    repo
}

fn actor_of(user_id: UserId) -> Subject {
    policy::user_subject(user_id, Vec::new())
}

async fn draft_of(world: &World) -> Invoice {
    world.create(vec![world.line("1", 1_000, None)]).await
}

#[tokio::test]
async fn every_write_refuses_an_actor_without_manage_invoices() {
    let world = World::new();
    let draft = draft_of(&world).await;
    let issued = world.issued(vec![world.line("1", 1_000, None)]).await;
    let organization_id = world.organization_id;
    let user_id = UserId(Uuid::new_v4());

    assert_forbidden(
        organization_id,
        user_id,
        MockInvoiceRepository::new(),
        |service| {
            let command = world.create_command(vec![world.line("1", 1_000, None)]);
            let command = CreateInvoiceCommand {
                actor: actor_of(user_id),
                ..command
            };
            Box::pin(async move {
                service
                    .create_invoice(
                        command,
                        &mut MockOrganizationRepository::new(),
                        &mut MockCustomerRepository::new(),
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(draft.clone()),
        |service| {
            let id = draft.id;
            Box::pin(async move {
                service
                    .soft_delete_invoice(id, actor_of(user_id), at(11, 7))
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(issued.clone()),
        |service| {
            let id = issued.id;
            Box::pin(async move {
                service
                    .record_payment(
                        RecordInvoicePaymentCommand {
                            invoice_id: id,
                            actor: actor_of(user_id),
                            amount_cents: 100,
                            paid_on: date(11, 7),
                            method: PaymentMethod::Card,
                            reference: None,
                            note: None,
                            recorded_by: user_id,
                        },
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(issued.clone()),
        |service| {
            let id = issued.id;
            Box::pin(async move {
                service
                    .issue_credit_note(
                        IssueCreditNoteCommand {
                            source_invoice_id: id,
                            actor: actor_of(user_id),
                            reason: "x".to_owned(),
                            request: CreditRequest::Full,
                            notes: None,
                        },
                        &mut MockOrganizationRepository::new(),
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(issued.clone()),
        |service| {
            let id = issued.id;
            Box::pin(async move {
                service
                    .send_invoice(
                        SendInvoiceCommand {
                            id,
                            actor: actor_of(user_id),
                            recorded_by: user_id,
                        },
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(issued.clone()),
        |service| {
            let id = issued.id;
            Box::pin(async move {
                service
                    .remind_invoice(
                        RemindInvoiceCommand {
                            id,
                            actor: actor_of(user_id),
                            recorded_by: user_id,
                        },
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(issued.clone()),
        |service| {
            let id = issued.id;
            Box::pin(async move {
                service
                    .note_invoice(
                        NoteInvoiceCommand {
                            id,
                            actor: actor_of(user_id),
                            recorded_by: user_id,
                            note: "x".to_owned(),
                        },
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;

    assert_forbidden(
        organization_id,
        user_id,
        found_in(draft.clone()),
        |service| {
            let id = draft.id;
            Box::pin(async move {
                service
                    .issue_invoice(
                        IssueInvoiceCommand {
                            id,
                            actor: actor_of(user_id),
                            issued_on: None,
                            allow_exceeding_total: false,
                        },
                        &mut MockOrganizationRepository::new(),
                        &mut MockProjectRepository::new(),
                        &mut MockQuoteRepository::new(),
                        &mut MockCustomerRepository::new(),
                        &mut MockCustomerContextRepository::new(),
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;
}

#[tokio::test]
async fn creating_from_a_quote_is_refused_without_manage_invoices() {
    let world = World::new();
    let quote = world.quote(vec![("1", 1_000, None, "0")], QuoteStatus::Accepted);
    let user_id = UserId(Uuid::new_v4());

    assert_forbidden(
        world.organization_id,
        user_id,
        MockInvoiceRepository::new(),
        |service| {
            let quotes = world.quotes(&quote);
            let id = quote.id;
            Box::pin(async move {
                let mut quotes = quotes;
                service
                    .create_invoice_from_quote(
                        CreateInvoiceFromQuoteCommand {
                            quote_id: id,
                            actor: actor_of(user_id),
                            mode: QuoteInvoiceMode::Full,
                        },
                        &mut MockOrganizationRepository::new(),
                        &mut quotes,
                        &mut MockCustomerRepository::new(),
                        at(11, 7),
                    )
                    .await
            })
        },
    )
    .await;
}
