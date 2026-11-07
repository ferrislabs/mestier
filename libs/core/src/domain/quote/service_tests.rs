use std::sync::{
    Arc, Mutex, MutexGuard,
    atomic::{AtomicUsize, Ordering},
};

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use common::CoreError;
use events::testing::RecordingEmitter;
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

use super::*;
use crate::{
    BillingFrequency, Customer, CustomerContact, CustomerContactId, CustomerContext,
    CustomerContextId, CustomerProfile, CustomerStatus, DepositPercent, Member, MemberId,
    OrganizationId, QuoteVatBreakdownLine, RecurringTotal, ServiceRateUnit, UserId,
    domain::{
        customer::ports::MockCustomerRepository,
        customer_contact::ports::MockCustomerContactRepository,
        customer_context::ports::MockCustomerContextRepository,
        member::ports::MockMemberRepository,
        organization::{
            billing_settings::{BillingSettings, BillingSettingsInput},
            numbering::format_number,
            ports::MockOrganizationRepository,
        },
        quote::{ports::MockQuoteRepository, pricing::fixtures::priced_line},
    },
};

fn at(day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, day, 9, 0, 0).unwrap()
}

fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
}

fn dec(text: &str) -> Decimal {
    text.parse().unwrap()
}

pub(super) fn line_command(quantity: Decimal, unit_price_cents: i32) -> QuoteLineCommand {
    QuoteLineCommand {
        service_rate_id: None,
        product_id: None,
        label: "Taille de haie".to_owned(),
        quantity,
        unit: ServiceRateUnit::Ml.as_str().to_owned(),
        unit_price_cents,
        vat_rate_bp: None,
        discount_percent: Decimal::ZERO,
        billing_frequency: BillingFrequency::OneShot,
        notes: Some("Acces jardin".to_owned()),
        photo_keys: vec!["quotes/photo-1.jpg".to_owned()],
    }
}

fn unchanged_line() -> QuoteLineCommand {
    QuoteLineCommand {
        notes: None,
        photo_keys: vec![],
        unit: ServiceRateUnit::Hour.as_str().to_owned(),
        label: "Taille de haie".to_owned(),
        ..line_command(Decimal::new(1, 0), 5500)
    }
}

struct Ids {
    organization: OrganizationId,
    customer: CustomerId,
    context: CustomerContextId,
}

impl Ids {
    fn new() -> Self {
        Self {
            organization: OrganizationId(Uuid::new_v4()),
            customer: CustomerId(Uuid::new_v4()),
            context: CustomerContextId(Uuid::new_v4()),
        }
    }
}

fn stored_line(ids: &Ids, quote_id: QuoteId) -> QuoteLine {
    QuoteLine {
        organization_id: ids.organization,
        quote_id,
        label: "Taille de haie".to_owned(),
        unit: ServiceRateUnit::Hour.as_str().to_owned(),
        unit_price_cents: 5500,
        vat_rate_bp: None,
        ..priced_line("1", 5500, 0, "0", BillingFrequency::OneShot)
    }
}

fn draft(ids: &Ids) -> Quote {
    let id = QuoteId(Uuid::new_v4());
    Quote {
        net_cents: 5500,
        gross_cents: 5500,
        lines: vec![stored_line(ids, id)],
        ..Quote::draft(
            id,
            ids.organization,
            ids.customer,
            ids.context,
            "Rénovation cuisine".to_owned(),
            at(1),
        )
    }
}

fn sent(ids: &Ids) -> Quote {
    Quote {
        status: QuoteStatus::Sent,
        reference: Some("DEV-2026-0007".to_owned()),
        valid_until: Some(date(30)),
        ..draft(ids)
    }
}

fn organization(id: OrganizationId, vat: Option<VatStatus>) -> Organization {
    let now = at(1);
    Organization {
        id,
        name: "Acme".into(),
        slug: "acme".into(),
        owner_id: UserId(Uuid::new_v4()),
        legal_name: Some("Acme SARL".into()),
        legal_form: Some("SARL".into()),
        registration_number: Some("12345678900011".into()),
        vat_status: vat,
        share_capital_cents: None,
        address_line1: Some("1 rue de la Paix".into()),
        address_line2: None,
        address_postal_code: Some("75002".into()),
        address_city: Some("Paris".into()),
        address_country: Some("FR".into()),
        contact_email: Some("contact@acme.test".into()),
        contact_phone: None,
        insurance_mention: Some("RC Pro 42".into()),
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

fn customer(ids: &Ids) -> Customer {
    Customer {
        id: ids.customer,
        organization_id: ids.organization,
        status: CustomerStatus::Prospect,
        name: "Duval Maçonnerie".to_owned(),
        registration_number: Some("12345678901234".to_owned()),
        phone: None,
        email: None,
        profile: CustomerProfile {
            billing_address_id: Some(ids.context),
            vat_number: Some("FR99123456789".to_owned()),
            ..CustomerProfile::default()
        },
        converted_at: None,
        archived_at: None,
        deleted_at: None,
        created_at: at(1),
        updated_at: at(1),
    }
}

fn billing_address(ids: &Ids, line: Option<&str>, city: Option<&str>) -> CustomerContext {
    CustomerContext {
        id: ids.context,
        customer_id: ids.customer,
        label: "Siège".to_owned(),
        kind: crate::AddressKind::Billing,
        address_line: line.map(str::to_owned),
        address_line2: None,
        postal_code: Some("69003".to_owned()),
        city: city.map(str::to_owned),
        country: "FR".to_owned(),
        photo_key: None,
        deleted_at: None,
        created_at: at(1),
        updated_at: at(1),
    }
}

fn billing_contact(ids: &Ids) -> CustomerContact {
    CustomerContact {
        id: CustomerContactId(Uuid::new_v4()),
        customer_id: ids.customer,
        first_name: "Anne".to_owned(),
        last_name: "Duval".to_owned(),
        role: None,
        phone: None,
        mobile: None,
        email: Some("anne@duval.test".to_owned()),
        is_primary: true,
        deleted_at: None,
        created_at: at(1),
        updated_at: at(1),
    }
}

fn settings(validity_days: i64, padding: i64) -> BillingSettings {
    BillingSettings::try_from(BillingSettingsInput {
        payment_terms_days: 30,
        quote_validity_days: validity_days,
        default_vat_rate_bp: 2000,
        late_penalty: "trois fois le taux légal".to_owned(),
        recovery_fee_cents: 4000,
        invoice_footer: "Merci".to_owned(),
        number_padding: padding,
        iban: Some("FR7612345".to_owned()),
        bic: None,
    })
    .unwrap()
}

fn with_address(build: impl FnOnce(&Ids) -> Option<CustomerContext>) -> Scenario {
    let scenario = Scenario::new();
    let address = build(&scenario.ids);
    Scenario {
        address,
        ..scenario
    }
}

struct Doubles {
    organizations: MockOrganizationRepository,
    customers: MockCustomerRepository,
    contacts: MockCustomerContactRepository,
    addresses: MockCustomerContextRepository,
    members: MockMemberRepository,
}

struct Scenario {
    ids: Ids,
    vat: Option<VatStatus>,
    settings: Option<BillingSettings>,
    address: Option<CustomerContext>,
    owner_in_organization: bool,
    contact_belongs: bool,
    address_belongs: bool,
}

impl Scenario {
    fn new() -> Self {
        let ids = Ids::new();
        let address = Some(billing_address(
            &ids,
            Some("12 rue des Lilas"),
            Some("Lyon"),
        ));
        Self {
            ids,
            vat: None,
            settings: None,
            address,
            owner_in_organization: true,
            contact_belongs: true,
            address_belongs: true,
        }
    }

    fn doubles(&self) -> Doubles {
        let mut organizations = MockOrganizationRepository::new();
        let organization = organization(self.ids.organization, self.vat.clone());
        organizations.expect_find_by_id().returning(move |_| {
            let organization = organization.clone();
            Box::pin(async move { Ok(Some(organization)) })
        });
        let settings = self.settings.clone();
        organizations
            .expect_find_billing_settings()
            .returning(move |_| {
                let settings = settings.clone();
                Box::pin(async move { Ok(settings) })
            });

        let mut customers = MockCustomerRepository::new();
        let stored = customer(&self.ids);
        customers.expect_find_by_id().returning(move |_| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(stored)) })
        });
        let contact_belongs = self.contact_belongs;
        customers
            .expect_has_contact()
            .returning(move |_, _| Box::pin(async move { Ok(contact_belongs) }));
        let address_belongs = self.address_belongs;
        customers
            .expect_has_address()
            .returning(move |_, _| Box::pin(async move { Ok(address_belongs) }));

        let mut contacts = MockCustomerContactRepository::new();
        let contact = billing_contact(&self.ids);
        contacts.expect_find_by_id().returning(move |_| {
            let contact = contact.clone();
            Box::pin(async move { Ok(Some(contact)) })
        });

        let mut addresses = MockCustomerContextRepository::new();
        let address = self.address.clone();
        addresses.expect_find_by_id().returning(move |_| {
            let address = address.clone();
            Box::pin(async move { Ok(address) })
        });

        let mut members = MockMemberRepository::new();
        let organization_id = self.ids.organization;
        let owner_in_organization = self.owner_in_organization;
        members.expect_find_by_id().returning(move |id| {
            let member = owner_in_organization.then(|| Member {
                id,
                organization_id,
                user_id: None,
                last_name: "Martin".to_owned(),
                first_name: None,
                joined_at: None,
                created_at: at(1),
                deleted_at: None,
            });
            Box::pin(async move { Ok(member) })
        });

        Doubles {
            organizations,
            customers,
            contacts,
            addresses,
            members,
        }
    }
}

impl Doubles {
    fn lookups(
        &mut self,
    ) -> QuoteLookups<
        '_,
        MockOrganizationRepository,
        MockCustomerRepository,
        MockCustomerContactRepository,
        MockCustomerContextRepository,
        MockMemberRepository,
    > {
        QuoteLookups {
            organizations: &mut self.organizations,
            customers: &mut self.customers,
            contacts: &mut self.contacts,
            addresses: &mut self.addresses,
            members: &mut self.members,
        }
    }
}

type Store = Arc<Mutex<Vec<Quote>>>;

fn rows(store: &Store) -> MutexGuard<'_, Vec<Quote>> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct Memory {
    store: Store,
    allocations: Arc<AtomicUsize>,
    emitter: RecordingEmitter,
}

impl Memory {
    fn with(quotes: Vec<Quote>) -> Self {
        Self {
            store: Arc::new(Mutex::new(quotes)),
            allocations: Arc::new(AtomicUsize::new(0)),
            emitter: RecordingEmitter::new(),
        }
    }

    fn empty() -> Self {
        Self::with(Vec::new())
    }

    fn allocated(&self) -> usize {
        self.allocations.load(Ordering::SeqCst)
    }

    fn stored(&self, id: QuoteId) -> Quote {
        rows(&self.store)
            .iter()
            .find(|quote| quote.id == id)
            .cloned()
            .expect("the quote is stored")
    }

    fn service(&self) -> QuoteService<MockQuoteRepository, &RecordingEmitter> {
        QuoteService::new(self.repository(), &self.emitter)
    }

    fn repository(&self) -> MockQuoteRepository {
        let mut repo = MockQuoteRepository::new();

        let allocations = self.allocations.clone();
        repo.expect_allocate_number()
            .returning(move |_, prefix, year, padding| {
                let number = allocations.fetch_add(1, Ordering::SeqCst) as i32 + 8;
                let reference = format_number(prefix, year, number, padding);
                Box::pin(async move { Ok(reference) })
            });

        let store = self.store.clone();
        repo.expect_insert().returning(move |quote| {
            rows(&store).push(quote.clone());
            let quote = quote.clone();
            Box::pin(async move { Ok(quote) })
        });

        let store = self.store.clone();
        repo.expect_find_by_id().returning(move |id| {
            let found = rows(&store).iter().find(|quote| quote.id == id).cloned();
            Box::pin(async move { Ok(found) })
        });

        let store = self.store.clone();
        repo.expect_update().returning(move |quote| {
            let mut guard = rows(&store);
            let slot = guard
                .iter_mut()
                .find(|stored| stored.id == quote.id)
                .expect("the quote exists");
            *slot = quote.clone();
            let quote = quote.clone();
            Box::pin(async move { Ok(quote) })
        });

        let store = self.store.clone();
        repo.expect_update_lifecycle().returning(move |quote| {
            let mut guard = rows(&store);
            let slot = guard
                .iter_mut()
                .find(|stored| stored.id == quote.id)
                .expect("the quote exists");
            let lines = slot.lines.clone();
            *slot = Quote {
                lines,
                ..quote.clone()
            };
            let saved = slot.clone();
            Box::pin(async move { Ok(saved) })
        });

        let store = self.store.clone();
        repo.expect_soft_delete().returning(move |id, _| {
            rows(&store).retain(|quote| quote.id != id);
            Box::pin(async { Ok(()) })
        });

        repo
    }
}

fn status_command(id: QuoteId, status: QuoteStatus) -> UpdateQuoteStatusCommand {
    UpdateQuoteStatusCommand::to(id, status)
}

fn update_command(from: &Quote, title: &str, status: QuoteStatus) -> UpdateQuoteCommand {
    UpdateQuoteCommand {
        id: from.id,
        title: title.to_owned(),
        customer_id: from.customer_id,
        customer_context_id: from.customer_context_id,
        deal_id: None,
        status,
        terms: QuoteTermsCommand::default(),
        lines: vec![unchanged_line()],
    }
}

fn create_command(scenario: &Scenario, lines: Vec<QuoteLineCommand>) -> CreateQuoteCommand {
    CreateQuoteCommand {
        organization_id: scenario.ids.organization,
        title: "Rénovation cuisine".to_owned(),
        customer_id: scenario.ids.customer,
        customer_context_id: scenario.ids.context,
        deal_id: None,
        terms: QuoteTermsCommand::default(),
        lines,
    }
}

async fn created(scenario: &Scenario, command: CreateQuoteCommand) -> Quote {
    let memory = Memory::empty();
    memory
        .service()
        .create_quote(command, &mut scenario.doubles().lookups(), at(7))
        .await
        .unwrap()
        .unwrap()
}

fn refusal<T: std::fmt::Debug>(outcome: Result<QuoteOutcome<T>, CoreError>) -> QuoteError {
    outcome.unwrap().unwrap_err()
}

fn invalid(field: QuoteField) -> impl Fn(&QuoteError) -> bool {
    move |error| matches!(error, QuoteError::Invalid { field: found, .. } if *found == field)
}

#[tokio::test]
async fn create_quote_calculates_total_from_lines() {
    let scenario = Scenario::new();

    let quote = created(
        &scenario,
        create_command(
            &scenario,
            vec![
                line_command(Decimal::new(25, 1), 1200),
                line_command(Decimal::new(1, 0), 500),
            ],
        ),
    )
    .await;

    assert_eq!(quote.status, QuoteStatus::Draft);
    assert_eq!(quote.reference, None);
    assert_eq!(quote.title, "Rénovation cuisine");
    assert_eq!(quote.net_cents, 3500);
    assert_eq!(quote.gross_cents, 3500);
    assert!(quote.vat_breakdown.is_empty());
    assert_eq!(quote.created_at, at(7));
}

#[tokio::test]
async fn create_quote_breaks_vat_down_per_rate_for_a_subject_organization() {
    let scenario = Scenario {
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let mut reduced = line_command(Decimal::new(1, 0), 10_000);
    reduced.vat_rate_bp = Some(550);
    let mut standard = line_command(Decimal::new(1, 0), 10_000);
    standard.vat_rate_bp = Some(2000);

    let quote = created(
        &scenario,
        create_command(&scenario, vec![reduced, standard]),
    )
    .await;

    assert_eq!(quote.net_cents, 20_000);
    assert_eq!(
        quote.vat_breakdown,
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
    assert_eq!(quote.gross_cents, 22_550);
}

#[tokio::test]
async fn a_not_subject_organization_produces_gross_equal_to_net() {
    let scenario = Scenario {
        vat: Some(VatStatus::NotSubject {
            basis: "Article 293 B du CGI".into(),
        }),
        ..Scenario::new()
    };
    let mut line = line_command(Decimal::new(1, 0), 10_000);
    line.vat_rate_bp = Some(2000);

    let quote = created(&scenario, create_command(&scenario, vec![line])).await;

    assert_eq!(quote.net_cents, quote.gross_cents);
    assert!(quote.vat_breakdown.is_empty());
}

#[tokio::test]
async fn spec_crm_4_11_the_vat_is_computed_once_per_rate_on_the_summed_base() {
    let scenario = Scenario {
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let mut line = line_command(Decimal::new(1, 0), 10_033);
    line.vat_rate_bp = Some(2000);

    let quote = created(
        &scenario,
        create_command(&scenario, vec![line.clone(), line]),
    )
    .await;

    assert_eq!(quote.net_cents, 20_066);
    assert_eq!(
        quote.vat_breakdown,
        vec![QuoteVatBreakdownLine {
            rate_bp: 2000,
            vat_cents: 4_013
        }]
    );
    assert_eq!(quote.gross_cents, 24_079);
}

#[tokio::test]
async fn three_half_cent_vat_lines_round_the_summed_base_half_away_from_zero() {
    let scenario = Scenario {
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let mut line = line_command(Decimal::new(1, 0), 105);
    line.vat_rate_bp = Some(1000);

    let quote = created(
        &scenario,
        create_command(&scenario, vec![line.clone(), line.clone(), line]),
    )
    .await;

    assert_eq!(quote.net_cents, 315);
    assert_eq!(
        quote.vat_breakdown,
        vec![QuoteVatBreakdownLine {
            rate_bp: 1000,
            vat_cents: 32
        }]
    );
    assert_eq!(quote.gross_cents, 347);
}

#[tokio::test]
async fn spec_crm_4_10_a_line_discount_applies_before_vat_and_totals_follow_it() {
    let scenario = Scenario {
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let mut discounted = line_command(Decimal::new(3, 0), 10_000);
    discounted.vat_rate_bp = Some(2000);
    discounted.discount_percent = dec("10");

    let quote = created(&scenario, create_command(&scenario, vec![discounted])).await;

    assert_eq!(quote.net_cents, 27_000);
    assert_eq!(quote.vat_breakdown[0].vat_cents, 5_400);
    assert_eq!(quote.gross_cents, 32_400);
    assert_eq!(quote.lines[0].discount_percent.get(), dec("10"));
}

#[tokio::test]
async fn spec_crm_4_12_frequency_leaves_totals_unchanged_and_the_quote_lists_recurring_totals() {
    let scenario = Scenario {
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let line = |frequency: BillingFrequency, price: i32| {
        let mut command = line_command(Decimal::new(1, 0), price);
        command.vat_rate_bp = Some(2000);
        command.billing_frequency = frequency;
        command
    };

    let one_shot = created(
        &scenario,
        create_command(
            &scenario,
            vec![
                line(BillingFrequency::OneShot, 10_000),
                line(BillingFrequency::OneShot, 2_000),
            ],
        ),
    )
    .await;
    let mixed = created(
        &scenario,
        create_command(
            &scenario,
            vec![
                line(BillingFrequency::OneShot, 10_000),
                line(BillingFrequency::Monthly, 2_000),
            ],
        ),
    )
    .await;

    assert_eq!(
        (
            one_shot.net_cents,
            &one_shot.vat_breakdown,
            one_shot.gross_cents
        ),
        (mixed.net_cents, &mixed.vat_breakdown, mixed.gross_cents)
    );
    assert!(one_shot.recurring().is_empty());
    assert_eq!(
        mixed.recurring(),
        vec![RecurringTotal {
            frequency: BillingFrequency::Monthly,
            net_cents: 2_000,
            vat_cents: 400,
            gross_cents: 2_400,
        }]
    );
}

#[tokio::test]
async fn spec_crm_4_13_a_line_stores_its_catalogue_reference_and_refuses_two() {
    let scenario = Scenario::new();
    let product_id = crate::ProductId(Uuid::new_v4());
    let mut line = line_command(Decimal::new(1, 0), 1000);
    line.product_id = Some(product_id);

    let quote = created(&scenario, create_command(&scenario, vec![line.clone()])).await;

    assert_eq!(quote.lines[0].product_id, Some(product_id));
    assert_eq!(quote.lines[0].service_rate_id, None);

    line.service_rate_id = Some(crate::ServiceRateId(Uuid::new_v4()));
    let both = Memory::empty()
        .service()
        .create_quote(
            create_command(&scenario, vec![line]),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;

    assert!(invalid(QuoteField::CatalogueReference)(&refusal(both)));
}

#[tokio::test]
async fn create_quote_stores_the_terms_and_defaults_the_billing_ids_from_the_customer() {
    let scenario = Scenario::new();
    let owner = MemberId(Uuid::new_v4());
    let mut command = create_command(&scenario, vec![line_command(Decimal::new(1, 0), 1000)]);
    command.terms = QuoteTermsCommand {
        valid_until: Some(date(20)),
        deposit_percent: dec("30.5"),
        owner_member_id: Some(owner),
        notes: Some("  Accès par la cour  ".to_owned()),
        ..QuoteTermsCommand::default()
    };

    let quote = created(&scenario, command).await;

    assert_eq!(quote.valid_until, Some(date(20)));
    assert_eq!(quote.deposit_percent.get(), dec("30.5"));
    assert_eq!(quote.owner_member_id, Some(owner));
    assert_eq!(quote.notes.as_deref(), Some("Accès par la cour"));
    assert_eq!(quote.billing_contact_id, None);
    assert_eq!(quote.billing_address_id, Some(scenario.ids.context));
}

#[tokio::test]
async fn create_quote_refuses_what_the_rules_forbid_naming_the_field() {
    let scenario = Scenario::new();
    let cases: Vec<(QuoteField, CreateQuoteCommand)> = {
        let base = || create_command(&scenario, vec![line_command(Decimal::new(1, 0), 1000)]);
        let mut empty_title = base();
        empty_title.title = " ".to_owned();
        let mut deposit = base();
        deposit.terms.deposit_percent = dec("100.5");
        let mut discount = base();
        discount.lines[0].discount_percent = dec("101");
        let mut zero_quantity = base();
        zero_quantity.lines[0].quantity = Decimal::ZERO;
        let mut vat = base();
        vat.lines[0].vat_rate_bp = Some(10_001);
        vec![
            (QuoteField::Title, empty_title),
            (QuoteField::DepositPercent, deposit),
            (QuoteField::DiscountPercent, discount),
            (QuoteField::Lines, zero_quantity),
            (QuoteField::Lines, vat),
        ]
    };

    for (field, command) in cases {
        let outcome = Memory::empty()
            .service()
            .create_quote(command, &mut scenario.doubles().lookups(), at(7))
            .await;

        assert!(invalid(field)(&refusal(outcome)), "{field}");
    }
}

#[tokio::test]
async fn create_quote_refuses_an_owner_outside_the_organization_and_foreign_billing_ids() {
    let outsider = Scenario {
        owner_in_organization: false,
        ..Scenario::new()
    };
    let mut command = create_command(&outsider, vec![line_command(Decimal::new(1, 0), 1000)]);
    command.terms.owner_member_id = Some(MemberId(Uuid::new_v4()));
    let outcome = Memory::empty()
        .service()
        .create_quote(command, &mut outsider.doubles().lookups(), at(7))
        .await;
    assert!(invalid(QuoteField::OwnerMemberId)(&refusal(outcome)));

    let foreign = Scenario {
        contact_belongs: false,
        address_belongs: false,
        ..Scenario::new()
    };
    let mut contact = create_command(&foreign, vec![line_command(Decimal::new(1, 0), 1000)]);
    contact.terms.billing_contact_id = Some(CustomerContactId(Uuid::new_v4()));
    let outcome = Memory::empty()
        .service()
        .create_quote(contact, &mut foreign.doubles().lookups(), at(7))
        .await;
    assert!(invalid(QuoteField::BillingContactId)(&refusal(outcome)));

    let mut address = create_command(&foreign, vec![line_command(Decimal::new(1, 0), 1000)]);
    address.terms.billing_address_id = Some(CustomerContextId(Uuid::new_v4()));
    let outcome = Memory::empty()
        .service()
        .create_quote(address, &mut foreign.doubles().lookups(), at(7))
        .await;
    assert!(invalid(QuoteField::BillingAddressId)(&refusal(outcome)));
}

#[tokio::test]
async fn creating_a_quote_for_an_unknown_customer_is_not_found() {
    let scenario = Scenario::new();
    let mut doubles = scenario.doubles();
    doubles.customers = MockCustomerRepository::new();
    doubles
        .customers
        .expect_find_by_id()
        .returning(|_| Box::pin(async { Ok(None) }));

    let outcome = Memory::empty()
        .service()
        .create_quote(
            create_command(&scenario, vec![line_command(Decimal::new(1, 0), 1000)]),
            &mut doubles.lookups(),
            at(7),
        )
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
}

#[tokio::test]
async fn update_quote_recalculates_total_and_stays_a_draft_without_a_number() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);
    let mut command = update_command(&existing, "Version ajustée", QuoteStatus::Draft);
    command.lines = vec![line_command(Decimal::new(3, 0), 2000)];

    let change = memory
        .service()
        .update_quote(command, &mut scenario.doubles().lookups(), at(7))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.quote.status, QuoteStatus::Draft);
    assert_eq!(change.quote.reference, None);
    assert_eq!(change.quote.title, "Version ajustée");
    assert_eq!(change.quote.net_cents, 6000);
    assert_eq!(memory.allocated(), 0);
}

#[tokio::test]
async fn spec_crm_4_1_a_quote_is_editable_only_as_a_draft() {
    let scenario = Scenario::new();

    for status in [
        QuoteStatus::Sent,
        QuoteStatus::Accepted,
        QuoteStatus::Declined,
        QuoteStatus::Cancelled,
        QuoteStatus::Superseded,
    ] {
        let existing = Quote {
            status,
            reference: Some("DEV-2026-0001".to_owned()),
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![existing.clone()]);

        let outcome = memory
            .service()
            .update_quote(
                update_command(&existing, "Autre titre", QuoteStatus::Draft),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await;

        assert_eq!(refusal(outcome), QuoteError::Locked, "{status}");
        assert_eq!(memory.stored(existing.id).title, existing.title);
    }
}

#[tokio::test]
async fn spec_crm_4_1_deleting_is_allowed_only_for_a_draft_and_uses_no_number() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);

    memory
        .service()
        .soft_delete_quote(existing.id, at(7))
        .await
        .unwrap()
        .unwrap();

    assert!(rows(&memory.store).is_empty());
    assert_eq!(memory.allocated(), 0);
    assert_eq!(memory.emitter.names(), vec!["quote.deleted"]);

    for status in [
        QuoteStatus::Sent,
        QuoteStatus::Accepted,
        QuoteStatus::Declined,
        QuoteStatus::Cancelled,
        QuoteStatus::Superseded,
    ] {
        let locked = Quote {
            status,
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![locked.clone()]);

        let outcome = memory.service().soft_delete_quote(locked.id, at(7)).await;

        assert_eq!(refusal(outcome), QuoteError::Locked, "{status}");
        assert_eq!(rows(&memory.store).len(), 1);
        assert!(memory.emitter.names().is_empty());
    }
}

#[tokio::test]
async fn spec_crm_4_14_changing_the_customer_of_a_quote_with_lines_is_refused() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);
    let mut command = update_command(&existing, &existing.title, QuoteStatus::Draft);
    command.customer_id = CustomerId(Uuid::new_v4());

    let outcome = memory
        .service()
        .update_quote(command, &mut scenario.doubles().lookups(), at(7))
        .await;

    assert!(invalid(QuoteField::CustomerId)(&refusal(outcome)));
    assert_eq!(memory.stored(existing.id).customer_id, existing.customer_id);
}

#[tokio::test]
async fn spec_crm_4_14_the_customer_can_change_while_the_quote_has_no_line() {
    let scenario = Scenario::new();
    let existing = Quote {
        lines: Vec::new(),
        net_cents: 0,
        gross_cents: 0,
        ..draft(&scenario.ids)
    };
    let memory = Memory::with(vec![existing.clone()]);
    let other = CustomerId(Uuid::new_v4());
    let mut command = update_command(&existing, &existing.title, QuoteStatus::Draft);
    command.customer_id = other;
    command.lines = Vec::new();
    let mut doubles = scenario.doubles();
    doubles.customers = MockCustomerRepository::new();
    let organization_id = scenario.ids.organization;
    doubles.customers.expect_find_by_id().returning(move |id| {
        let moved = Customer {
            id,
            organization_id,
            ..customer(&Ids {
                organization: organization_id,
                customer: id,
                context: CustomerContextId(Uuid::new_v4()),
            })
        };
        Box::pin(async move { Ok(Some(moved)) })
    });
    doubles
        .customers
        .expect_has_contact()
        .returning(|_, _| Box::pin(async { Ok(true) }));
    doubles
        .customers
        .expect_has_address()
        .returning(|_, _| Box::pin(async { Ok(true) }));

    let change = memory
        .service()
        .update_quote(command, &mut doubles.lookups(), at(7))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.quote.customer_id, other);
}

#[tokio::test]
async fn updating_with_a_status_other_than_draft_or_sent_is_refused() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);

    let outcome = memory
        .service()
        .update_quote(
            update_command(&existing, &existing.title, QuoteStatus::Accepted),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;

    assert!(invalid(QuoteField::Status)(&refusal(outcome)));
}

#[tokio::test]
async fn update_quote_sends_the_quote_in_the_same_call_and_persists_the_lifecycle() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);

    let change = memory
        .service()
        .update_quote(
            update_command(&existing, "Version envoyée", QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.from, QuoteStatus::Draft);
    assert_eq!(change.quote.status, QuoteStatus::Sent);
    assert_eq!(change.quote.reference.as_deref(), Some("DEV-2026-0008"));
    assert!(change.quote.issuer_snapshot.is_some());
    assert_eq!(memory.stored(existing.id).status, QuoteStatus::Sent);
    assert_eq!(memory.emitter.names(), vec!["quote.updated", "quote.sent"]);
}

#[tokio::test]
async fn a_refused_send_through_update_writes_nothing() {
    let scenario = with_address(|ids| Some(billing_address(ids, None, Some("Lyon"))));
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);

    let outcome = memory
        .service()
        .update_quote(
            update_command(&existing, "Nouveau titre", QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;

    assert!(invalid(QuoteField::BillingAddress)(&refusal(outcome)));
    assert_eq!(memory.stored(existing.id).title, existing.title);
    assert_eq!(memory.allocated(), 0);
    assert!(memory.emitter.names().is_empty());
}

#[tokio::test]
async fn spec_crm_4_2_sending_is_refused_by_each_rule_and_consumes_no_number() {
    let scenario = Scenario::new();
    let bad_line = {
        let mut quote = draft(&scenario.ids);
        quote.lines[0].quantity = Decimal::ZERO;
        quote
    };
    let cases: Vec<(&str, QuoteField, Quote)> = vec![
        (
            "empty title",
            QuoteField::Title,
            Quote {
                title: "  ".to_owned(),
                ..draft(&scenario.ids)
            },
        ),
        (
            "no line",
            QuoteField::Lines,
            Quote {
                lines: Vec::new(),
                ..draft(&scenario.ids)
            },
        ),
        ("invalid line", QuoteField::Lines, bad_line),
        (
            "total of zero",
            QuoteField::Total,
            Quote {
                net_cents: 0,
                gross_cents: 0,
                ..draft(&scenario.ids)
            },
        ),
        (
            "past valid_until",
            QuoteField::ValidUntil,
            Quote {
                valid_until: Some(date(6)),
                ..draft(&scenario.ids)
            },
        ),
    ];

    for (name, field, quote) in cases {
        let memory = Memory::with(vec![quote.clone()]);

        let outcome = memory
            .service()
            .update_quote_status(
                status_command(quote.id, QuoteStatus::Sent),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await;

        assert!(invalid(field)(&refusal(outcome)), "{name}");
        assert_eq!(memory.allocated(), 0, "{name}");
        assert_eq!(memory.stored(quote.id).status, QuoteStatus::Draft, "{name}");
        assert!(memory.emitter.names().is_empty(), "{name}");
    }

    let without_street = with_address(|ids| Some(billing_address(ids, None, Some("Lyon"))));
    let without_city = with_address(|ids| Some(billing_address(ids, Some("1 rue X"), None)));
    let without_address = with_address(|_| None);
    for (name, scenario) in [
        ("empty street", without_street),
        ("empty city", without_city),
        ("no billing address", without_address),
    ] {
        let quote = draft(&scenario.ids);
        let memory = Memory::with(vec![quote.clone()]);

        let outcome = memory
            .service()
            .update_quote_status(
                status_command(quote.id, QuoteStatus::Sent),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await;

        assert!(
            invalid(QuoteField::BillingAddress)(&refusal(outcome)),
            "{name}"
        );
        assert_eq!(memory.allocated(), 0, "{name}");
    }
}

#[tokio::test]
async fn spec_crm_4_2_a_valid_until_of_today_still_sends() {
    let scenario = Scenario::new();
    let quote = Quote {
        valid_until: Some(date(7)),
        ..draft(&scenario.ids)
    };
    let memory = Memory::with(vec![quote.clone()]);

    let change = memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.quote.valid_until, Some(date(7)));
}

#[tokio::test]
async fn spec_crm_4_3_sending_assigns_the_number_the_validity_and_freezes_both_parties() {
    let scenario = Scenario {
        settings: Some(settings(15, 4)),
        vat: Some(VatStatus::Subject {
            vat_number: "FR12345678901".into(),
        }),
        ..Scenario::new()
    };
    let quote = draft(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    let change = memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let sent = memory.stored(quote.id);
    assert_eq!(change.from, QuoteStatus::Draft);
    assert_eq!(sent.status, QuoteStatus::Sent);
    assert_eq!(sent.reference.as_deref(), Some("DEV-2026-0008"));
    assert_eq!(sent.valid_until, Some(date(22)));

    let issuer = sent.issuer_snapshot.expect("the issuer is frozen");
    assert_eq!(issuer.name, "Acme SARL");
    assert_eq!(
        issuer.registration_number.as_deref(),
        Some("12345678900011")
    );
    assert_eq!(issuer.vat_number.as_deref(), Some("FR12345678901"));
    assert_eq!(issuer.iban.as_deref(), Some("FR7612345"));
    assert_eq!(issuer.late_penalty, "trois fois le taux légal");
    assert_eq!(issuer.recovery_fee_cents, 4000);
    assert_eq!(issuer.invoice_footer, "Merci");

    let recipient = sent.recipient_snapshot.expect("the recipient is frozen");
    assert_eq!(recipient.name, "Duval Maçonnerie");
    assert_eq!(
        recipient.registration_number.as_deref(),
        Some("12345678901234")
    );
    assert_eq!(recipient.vat_number.as_deref(), Some("FR99123456789"));
    assert_eq!(recipient.contact_name, None);
    assert_eq!(recipient.address.line1.as_deref(), Some("12 rue des Lilas"));
    assert_eq!(recipient.address.city.as_deref(), Some("Lyon"));
}

#[tokio::test]
async fn sending_freezes_the_billing_contact_name_and_email() {
    let scenario = Scenario::new();
    let contact = billing_contact(&scenario.ids);
    let quote = Quote {
        billing_contact_id: Some(contact.id),
        ..draft(&scenario.ids)
    };
    let memory = Memory::with(vec![quote.clone()]);

    memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let recipient = memory.stored(quote.id).recipient_snapshot.unwrap();
    assert_eq!(recipient.contact_name.as_deref(), Some("Anne Duval"));
    assert_eq!(recipient.contact_email.as_deref(), Some("anne@duval.test"));
}

#[tokio::test]
async fn sending_keeps_a_valid_until_already_chosen() {
    let scenario = Scenario {
        settings: Some(settings(15, 4)),
        ..Scenario::new()
    };
    let quote = Quote {
        valid_until: Some(date(28)),
        ..draft(&scenario.ids)
    };
    let memory = Memory::with(vec![quote.clone()]);

    memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.stored(quote.id).valid_until, Some(date(28)));
}

#[tokio::test]
async fn spec_crm_1_5_a_new_padding_applies_to_the_number_allocated_next() {
    let scenario = Scenario {
        settings: Some(settings(30, 6)),
        ..Scenario::new()
    };
    let quote = draft(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        memory.stored(quote.id).reference.as_deref(),
        Some("DEV-2026-000008")
    );
}

#[tokio::test]
async fn restating_the_current_status_changes_nothing_and_emits_nothing() {
    let scenario = Scenario::new();
    let quote = sent(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    let change = memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(change.quote, quote);
    assert_eq!(change.from, QuoteStatus::Sent);
    assert_eq!(memory.allocated(), 0);
    assert!(memory.emitter.names().is_empty());
}

#[test]
fn spec_crm_4_5_a_quote_is_expired_when_sent_and_valid_until_is_before_today() {
    let ids = Ids::new();
    let yesterday = Quote {
        valid_until: Some(date(6)),
        ..sent(&ids)
    };
    let today = Quote {
        valid_until: Some(date(7)),
        ..sent(&ids)
    };
    let no_limit = Quote {
        valid_until: None,
        ..sent(&ids)
    };

    assert!(yesterday.is_expired(date(7)));
    assert!(!today.is_expired(date(7)));
    assert!(today.is_expired(date(8)));
    assert!(!no_limit.is_expired(date(30)));
    for status in [
        QuoteStatus::Draft,
        QuoteStatus::Accepted,
        QuoteStatus::Declined,
        QuoteStatus::Cancelled,
        QuoteStatus::Superseded,
    ] {
        let quote = Quote {
            status,
            ..yesterday.clone()
        };
        assert!(!quote.is_expired(date(7)), "{status}");
    }
}

#[tokio::test]
async fn spec_crm_4_6_accepting_records_the_answer_and_trims_the_signatory() {
    let scenario = Scenario::new();
    let quote = sent(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    let change = memory
        .service()
        .update_quote_status(
            UpdateQuoteStatusCommand {
                responded_by: Some("  Mme Duval  ".to_owned()),
                ..status_command(quote.id, QuoteStatus::Accepted)
            },
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let accepted = memory.stored(quote.id);
    assert_eq!(change.from, QuoteStatus::Sent);
    assert_eq!(accepted.status, QuoteStatus::Accepted);
    assert_eq!(accepted.responded_at, Some(at(7)));
    assert_eq!(accepted.responded_by.as_deref(), Some("Mme Duval"));
    assert_eq!(memory.emitter.names(), vec!["quote.accepted"]);
}

#[tokio::test]
async fn spec_crm_4_6_a_blank_signatory_is_accepted() {
    let scenario = Scenario::new();
    let quote = sent(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    memory
        .service()
        .update_quote_status(
            UpdateQuoteStatusCommand {
                responded_by: Some("   ".to_owned()),
                ..status_command(quote.id, QuoteStatus::Accepted)
            },
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let accepted = memory.stored(quote.id);
    assert_eq!(accepted.status, QuoteStatus::Accepted);
    assert_eq!(accepted.responded_by, None);
}

#[tokio::test]
async fn spec_crm_4_6_accepting_needs_a_sent_quote_that_has_not_expired() {
    let scenario = Scenario::new();

    for status in [
        QuoteStatus::Draft,
        QuoteStatus::Declined,
        QuoteStatus::Cancelled,
        QuoteStatus::Superseded,
    ] {
        let quote = Quote {
            status,
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![quote.clone()]);

        let outcome = memory
            .service()
            .update_quote_status(
                status_command(quote.id, QuoteStatus::Accepted),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await;

        assert_eq!(refusal(outcome), QuoteError::NotSent, "{status}");
    }

    let expired = Quote {
        valid_until: Some(date(6)),
        ..sent(&scenario.ids)
    };
    let memory = Memory::with(vec![expired.clone()]);
    let outcome = memory
        .service()
        .update_quote_status(
            status_command(expired.id, QuoteStatus::Accepted),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;

    assert_eq!(refusal(outcome), QuoteError::Expired);
    assert_eq!(memory.stored(expired.id).status, QuoteStatus::Sent);
    assert!(memory.emitter.names().is_empty());
}

#[tokio::test]
async fn spec_crm_4_7_declining_records_the_answer_and_needs_a_sent_quote() {
    let scenario = Scenario::new();
    let quote = sent(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    memory
        .service()
        .update_quote_status(
            UpdateQuoteStatusCommand {
                decline_reason: Some("  Trop cher  ".to_owned()),
                ..status_command(quote.id, QuoteStatus::Declined)
            },
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let declined = memory.stored(quote.id);
    assert_eq!(declined.status, QuoteStatus::Declined);
    assert_eq!(declined.responded_at, Some(at(7)));
    assert_eq!(declined.decline_reason.as_deref(), Some("Trop cher"));
    assert_eq!(memory.emitter.names(), vec!["quote.declined"]);

    let draft_quote = draft(&scenario.ids);
    let memory = Memory::with(vec![draft_quote.clone()]);
    let outcome = memory
        .service()
        .update_quote_status(
            status_command(draft_quote.id, QuoteStatus::Declined),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;
    assert_eq!(refusal(outcome), QuoteError::NotSent);
}

#[tokio::test]
async fn spec_crm_4_7_declining_needs_neither_a_reason_nor_a_valid_quote() {
    let scenario = Scenario::new();
    let expired = Quote {
        valid_until: Some(date(1)),
        ..sent(&scenario.ids)
    };
    let memory = Memory::with(vec![expired.clone()]);

    memory
        .service()
        .update_quote_status(
            status_command(expired.id, QuoteStatus::Declined),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    let declined = memory.stored(expired.id);
    assert_eq!(declined.status, QuoteStatus::Declined);
    assert_eq!(declined.decline_reason, None);
}

#[tokio::test]
async fn spec_crm_4_8_a_revision_copies_a_sent_or_declined_quote_as_a_draft() {
    let scenario = Scenario::new();

    for status in [QuoteStatus::Sent, QuoteStatus::Declined] {
        let source = Quote {
            status,
            responded_at: Some(at(3)),
            decline_reason: Some("Trop cher".to_owned()),
            notes: Some("Note".to_owned()),
            deposit_percent: DepositPercent::try_from(dec("30")).unwrap(),
            ..sent(&scenario.ids)
        };
        let memory = Memory::with(vec![source.clone()]);

        let revision = memory
            .service()
            .revise_quote(source.id, at(7))
            .await
            .unwrap()
            .unwrap();

        assert_ne!(revision.id, source.id, "{status}");
        assert_eq!(revision.status, QuoteStatus::Draft);
        assert_eq!(revision.reference, None);
        assert_eq!(revision.valid_until, None);
        assert_eq!(revision.revision_of_id, Some(source.id));
        assert_eq!(revision.responded_at, None);
        assert_eq!(revision.decline_reason, None);
        assert_eq!(revision.issuer_snapshot, None);
        assert_eq!(revision.title, source.title);
        assert_eq!(revision.notes, source.notes);
        assert_eq!(revision.deposit_percent, source.deposit_percent);
        assert_eq!(revision.net_cents, source.net_cents);
        assert_eq!(revision.lines.len(), source.lines.len());
        assert_ne!(revision.lines[0].id, source.lines[0].id);
        assert_eq!(revision.lines[0].quote_id, revision.id);
        assert_eq!(revision.lines[0].label, source.lines[0].label);
        assert_eq!(memory.stored(source.id).status, status);
        assert_eq!(memory.emitter.names(), vec!["quote.created"]);
        assert_eq!(memory.allocated(), 0);
    }
}

#[tokio::test]
async fn spec_crm_4_8_a_revision_is_refused_from_any_other_status() {
    let scenario = Scenario::new();

    for status in [
        QuoteStatus::Draft,
        QuoteStatus::Accepted,
        QuoteStatus::Cancelled,
        QuoteStatus::Superseded,
    ] {
        let source = Quote {
            status,
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![source.clone()]);

        let outcome = memory.service().revise_quote(source.id, at(7)).await;

        assert_eq!(refusal(outcome), QuoteError::NotRevisable, "{status}");
        assert_eq!(rows(&memory.store).len(), 1);
    }
}

#[tokio::test]
async fn spec_crm_4_9_sending_a_revision_supersedes_the_source_only_if_it_is_still_sent() {
    let scenario = Scenario::new();
    let source = sent(&scenario.ids);
    let memory = Memory::with(vec![source.clone()]);
    let revision = memory
        .service()
        .revise_quote(source.id, at(7))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(memory.stored(source.id).status, QuoteStatus::Sent);

    memory
        .service()
        .update_quote_status(
            status_command(revision.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.stored(source.id).status, QuoteStatus::Superseded);
    assert_eq!(memory.stored(revision.id).status, QuoteStatus::Sent);
    assert_eq!(memory.emitter.names(), vec!["quote.created", "quote.sent"]);

    let declined = Quote {
        status: QuoteStatus::Declined,
        ..sent(&scenario.ids)
    };
    let memory = Memory::with(vec![declined.clone()]);
    let revision = memory
        .service()
        .revise_quote(declined.id, at(7))
        .await
        .unwrap()
        .unwrap();

    memory
        .service()
        .update_quote_status(
            status_command(revision.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.stored(declined.id).status, QuoteStatus::Declined);
}

#[tokio::test]
async fn sending_a_revision_through_update_supersedes_the_source_too() {
    let scenario = Scenario::new();
    let source = sent(&scenario.ids);
    let memory = Memory::with(vec![source.clone()]);
    let revision = memory
        .service()
        .revise_quote(source.id, at(7))
        .await
        .unwrap()
        .unwrap();

    memory
        .service()
        .update_quote(
            update_command(&revision, &revision.title, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.stored(source.id).status, QuoteStatus::Superseded);
}

#[tokio::test]
async fn a_sent_accepted_or_declined_quote_can_be_cancelled_and_only_those() {
    let scenario = Scenario::new();

    for status in [
        QuoteStatus::Sent,
        QuoteStatus::Accepted,
        QuoteStatus::Declined,
    ] {
        let quote = Quote {
            status,
            ..sent(&scenario.ids)
        };
        let memory = Memory::with(vec![quote.clone()]);

        memory
            .service()
            .update_quote_status(
                status_command(quote.id, QuoteStatus::Cancelled),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(memory.stored(quote.id).status, QuoteStatus::Cancelled);
        assert_eq!(memory.emitter.names(), vec!["quote.cancelled"]);
    }

    for status in [QuoteStatus::Draft, QuoteStatus::Superseded] {
        let quote = Quote {
            status,
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![quote.clone()]);

        let outcome = memory
            .service()
            .update_quote_status(
                status_command(quote.id, QuoteStatus::Cancelled),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await;

        assert!(invalid(QuoteField::Status)(&refusal(outcome)), "{status}");
    }
}

#[tokio::test]
async fn a_status_cannot_move_back_to_draft_or_be_set_to_superseded_by_hand() {
    let scenario = Scenario::new();
    let quote = sent(&scenario.ids);
    let memory = Memory::with(vec![quote.clone()]);

    let back = memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Draft),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;
    assert_eq!(refusal(back), QuoteError::Locked);

    let superseded = memory
        .service()
        .update_quote_status(
            status_command(quote.id, QuoteStatus::Superseded),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;
    assert!(invalid(QuoteField::Status)(&refusal(superseded)));

    let declined = Quote {
        status: QuoteStatus::Declined,
        ..sent(&scenario.ids)
    };
    let memory = Memory::with(vec![declined.clone()]);
    let resend = memory
        .service()
        .update_quote_status(
            status_command(declined.id, QuoteStatus::Sent),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;
    assert_eq!(refusal(resend), QuoteError::Locked);
}

#[tokio::test]
async fn creating_a_quote_emits_created_and_nothing_else() {
    let scenario = Scenario::new();
    let memory = Memory::empty();

    memory
        .service()
        .create_quote(
            create_command(&scenario, vec![line_command(Decimal::new(1, 0), 5500)]),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.emitter.names(), vec!["quote.created"]);
}

#[tokio::test]
async fn a_content_change_reports_only_the_fields_that_moved() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);

    memory
        .service()
        .update_quote(
            update_command(&existing, "Nouveau titre", QuoteStatus::Draft),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(memory.emitter.names(), vec!["quote.updated"]);
    let payload = memory.emitter.only("quote.updated").payload;
    assert_eq!(payload["changed_fields"], json!(["title"]));
    assert_eq!(payload["previous"]["title"], json!(existing.title));
}

#[tokio::test]
async fn changing_a_line_or_a_term_is_reported_as_a_content_change() {
    let scenario = Scenario::new();
    let existing = draft(&scenario.ids);
    let memory = Memory::with(vec![existing.clone()]);
    let mut command = update_command(&existing, &existing.title, QuoteStatus::Draft);
    command.lines = vec![line_command(Decimal::new(2, 0), 9000)];
    command.terms.notes = Some("Nouvelle note".to_owned());

    memory
        .service()
        .update_quote(command, &mut scenario.doubles().lookups(), at(7))
        .await
        .unwrap()
        .unwrap();

    let payload = memory.emitter.only("quote.updated").payload;
    assert_eq!(payload["changed_fields"], json!(["notes", "lines"]));
    assert_eq!(payload["previous"]["net_cents"], json!(5500));
}

#[tokio::test]
async fn each_status_transition_has_its_own_event() {
    let scenario = Scenario::new();

    for (from, to, expected) in [
        (QuoteStatus::Draft, QuoteStatus::Sent, "quote.sent"),
        (QuoteStatus::Sent, QuoteStatus::Accepted, "quote.accepted"),
        (QuoteStatus::Sent, QuoteStatus::Declined, "quote.declined"),
        (QuoteStatus::Sent, QuoteStatus::Cancelled, "quote.cancelled"),
    ] {
        let quote = Quote {
            status: from,
            reference: (from != QuoteStatus::Draft).then(|| "DEV-2026-0001".to_owned()),
            ..draft(&scenario.ids)
        };
        let memory = Memory::with(vec![quote.clone()]);

        memory
            .service()
            .update_quote_status(
                status_command(quote.id, to),
                &mut scenario.doubles().lookups(),
                at(7),
            )
            .await
            .unwrap()
            .unwrap();

        assert_eq!(memory.emitter.names(), vec![expected]);
    }
}

#[tokio::test]
async fn a_write_that_fails_emits_nothing() {
    let scenario = Scenario::new();
    let emitter = RecordingEmitter::new();
    let mut repo = MockQuoteRepository::new();
    repo.expect_insert()
        .returning(|_| Box::pin(async { Err(CoreError::Conflict("nope".into())) }));
    let mut service = QuoteService::new(repo, &emitter);

    let outcome = service
        .create_quote(
            create_command(&scenario, vec![line_command(Decimal::new(1, 0), 5500)]),
            &mut scenario.doubles().lookups(),
            at(7),
        )
        .await;

    assert!(outcome.is_err());
    assert!(emitter.names().is_empty());
}

#[tokio::test]
async fn list_quotes_delegates_to_repo() {
    let scenario = Scenario::new();
    let org_id = scenario.ids.organization;
    let mut repo = MockQuoteRepository::new();
    let stored = draft(&scenario.ids);
    repo.expect_list_by_organization()
        .withf(move |org, deal, limit, offset| {
            *org == org_id && deal.is_none() && *limit == 25 && *offset == 50
        })
        .returning(move |_, _, _, _| {
            let stored = stored.clone();
            Box::pin(async move { Ok((vec![stored], 1)) })
        });

    let mut service = QuoteService::new(repo, RecordingEmitter::new());
    let (items, total) = service.list_quotes(org_id, None, 25, 50).await.unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(total, 1);
}

#[test]
fn products_landing_on_a_half_cent_round_half_away_from_zero() {
    for (quantity, unit_price_cents, expected) in [
        ("631.9", 38_875, 24_565_113),
        ("1723.1", 5_455, 9_399_511),
        ("710.1", 13_485, 9_575_699),
        ("1274.7", 13_895, 17_711_957),
        ("1024.5", 37_525, 38_444_363),
        ("1744.9", 26_385, 46_039_187),
    ] {
        let line = QuoteLine {
            quantity: dec(quantity),
            unit_price_cents,
            ..stored_line(&Ids::new(), QuoteId(Uuid::new_v4()))
        };

        assert_eq!(
            line.net_cents().expect("a total within bounds"),
            expected,
            "{quantity} x {unit_price_cents} cents"
        );
    }
}

#[test]
fn an_hourly_line_proposes_its_duration_in_minutes() {
    let mut source = draft(&Ids::new());
    source.lines[0].unit = ServiceRateUnit::Hour.as_str().to_owned();
    source.lines[0].quantity = Decimal::new(25, 1);

    let proposals = propose_tasks_from_quote(&source);

    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].quote_line_id, source.lines[0].id);
    assert_eq!(proposals[0].title, source.lines[0].label);
    assert_eq!(proposals[0].suggested_minutes, Some(150));
}

#[test]
fn a_per_unit_line_proposes_no_duration_rather_than_a_guess() {
    let mut source = draft(&Ids::new());
    source.lines[0].unit = ServiceRateUnit::Unit.as_str().to_owned();
    source.lines[0].quantity = Decimal::new(10, 0);

    assert_eq!(propose_tasks_from_quote(&source)[0].suggested_minutes, None);
}

#[test]
fn a_flat_rate_line_proposes_no_duration() {
    let mut source = draft(&Ids::new());
    source.lines[0].unit = ServiceRateUnit::FlatRate.as_str().to_owned();

    assert_eq!(propose_tasks_from_quote(&source)[0].suggested_minutes, None);
}

fn accepted_quote() -> Quote {
    Quote {
        status: QuoteStatus::Accepted,
        ..draft(&Ids::new())
    }
}

#[test]
fn a_draft_quote_is_not_plannable() {
    let err = require_quote_accepted(&draft(&Ids::new())).unwrap_err();

    assert!(matches!(err, CoreError::Conflict(_)));
}

#[test]
fn an_accepted_quote_is_plannable() {
    assert!(require_quote_accepted(&accepted_quote()).is_ok());
}

#[test]
fn a_second_project_is_refused_without_force_new() {
    let err = validate_quote_plannable(&accepted_quote(), true, false).unwrap_err();

    assert!(matches!(err, CoreError::Conflict(_)));
}

#[test]
fn a_second_project_is_accepted_with_force_new() {
    assert!(validate_quote_plannable(&accepted_quote(), true, true).is_ok());
}

#[test]
fn a_first_project_needs_no_force_new() {
    assert!(validate_quote_plannable(&accepted_quote(), false, false).is_ok());
}

#[test]
fn planning_a_non_accepted_quote_is_refused_even_with_force_new() {
    let err = validate_quote_plannable(&draft(&Ids::new()), false, true).unwrap_err();

    assert!(matches!(err, CoreError::Conflict(_)));
}
