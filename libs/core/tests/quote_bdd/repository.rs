use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{TimeZone, Utc};
use common::CoreError;
use mestier_core::{
    AddressKind, BillingSettings, BillingSettingsInput, Customer, CustomerContact,
    CustomerContactId, CustomerContext, CustomerContextId, CustomerId, CustomerProfile,
    CustomerStatus, MockCustomerContactRepository, MockCustomerContextRepository,
    MockCustomerRepository, MockMemberRepository, MockOrganizationRepository, MockQuoteRepository,
    Organization, OrganizationId, Quote, QuoteLookups, UserId, VatStatus,
};
use uuid::Uuid;

pub type Store = Arc<Mutex<Vec<Quote>>>;

pub fn quotes(store: &Store) -> MutexGuard<'_, Vec<Quote>> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn stubbed(store: &Store) -> MockQuoteRepository {
    let mut repo = MockQuoteRepository::new();

    let store_ref = store.clone();
    repo.expect_allocate_number()
        .returning(move |org, prefix, year, padding| {
            let needle = format!("{prefix}-{year}-");
            let count = quotes(&store_ref)
                .iter()
                .filter(|quote| {
                    quote.organization_id == org
                        && quote
                            .reference
                            .as_ref()
                            .is_some_and(|reference| reference.starts_with(&needle))
                })
                .count();
            let reference = format!("{needle}{:0width$}", count + 1, width = padding.get());

            Box::pin(async move { Ok(reference) })
        });

    let store_ref = store.clone();
    repo.expect_insert().returning(move |quote| {
        quotes(&store_ref).push(quote.clone());
        let quote = quote.clone();

        Box::pin(async move { Ok(quote) })
    });

    let store_ref = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = quotes(&store_ref)
            .iter()
            .find(|quote| quote.id == id)
            .cloned();

        Box::pin(async move { Ok(found) })
    });

    let store_ref = store.clone();
    repo.expect_update().returning(move |quote| {
        let outcome = match quotes(&store_ref)
            .iter_mut()
            .find(|stored| stored.id == quote.id)
        {
            Some(stored) => {
                *stored = quote.clone();
                Ok(stored.clone())
            }
            None => Err(CoreError::NotFound),
        };

        Box::pin(async move { outcome })
    });

    let store_ref = store.clone();
    repo.expect_update_lifecycle().returning(move |quote| {
        let outcome = match quotes(&store_ref)
            .iter_mut()
            .find(|stored| stored.id == quote.id)
        {
            Some(stored) => {
                let lines = stored.lines.clone();
                *stored = Quote {
                    lines,
                    ..quote.clone()
                };
                Ok(stored.clone())
            }
            None => Err(CoreError::NotFound),
        };

        Box::pin(async move { outcome })
    });

    let store_ref = store.clone();
    repo.expect_soft_delete().returning(move |id, _| {
        quotes(&store_ref).retain(|quote| quote.id != id);

        Box::pin(async { Ok(()) })
    });

    repo
}

pub struct Company {
    pub id: OrganizationId,
    pub subject_to_vat: bool,
    pub validity_days: i64,
}

pub struct Client {
    pub id: CustomerId,
    pub address_id: CustomerContextId,
    pub has_billing_address: bool,
}

pub struct Doubles {
    pub organizations: MockOrganizationRepository,
    pub customers: MockCustomerRepository,
    pub contacts: MockCustomerContactRepository,
    pub addresses: MockCustomerContextRepository,
    pub members: MockMemberRepository,
}

impl Doubles {
    pub fn lookups(
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

pub fn doubles(company: &Company, client: &Client) -> Doubles {
    let now = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

    let mut organizations = MockOrganizationRepository::new();
    let settings = BillingSettings::try_from(BillingSettingsInput {
        payment_terms_days: 30,
        quote_validity_days: company.validity_days,
        default_vat_rate_bp: 2000,
        late_penalty: "trois fois le taux légal".to_owned(),
        recovery_fee_cents: 4000,
        invoice_footer: String::new(),
        number_padding: 4,
        iban: None,
        bic: None,
    })
    .expect("scenario settings are valid");
    organizations
        .expect_find_billing_settings()
        .returning(move |_| {
            let settings = settings.clone();
            Box::pin(async move { Ok(Some(settings)) })
        });
    let subject_to_vat = company.subject_to_vat;
    organizations.expect_find_by_id().returning(move |id| {
        let organization = Organization {
            id,
            name: "Scenario company".into(),
            slug: "scenario-company".into(),
            owner_id: UserId(Uuid::new_v4()),
            legal_name: Some("Scenario SARL".into()),
            legal_form: None,
            registration_number: None,
            vat_status: subject_to_vat.then(|| VatStatus::Subject {
                vat_number: "FR12345678901".into(),
            }),
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
        };
        Box::pin(async move { Ok(Some(organization)) })
    });

    let mut customers = MockCustomerRepository::new();
    let organization_id = company.id;
    let customer_id = client.id;
    let address_id = client.address_id;
    customers.expect_find_by_id().returning(move |id| {
        let customer = Customer {
            id,
            organization_id,
            status: CustomerStatus::Prospect,
            name: "Duval Masonry".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
            profile: CustomerProfile {
                billing_address_id: Some(address_id),
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
    customers
        .expect_has_contact()
        .returning(|_, _| Box::pin(async { Ok(true) }));
    customers
        .expect_has_address()
        .returning(|_, _| Box::pin(async { Ok(true) }));

    let mut contacts = MockCustomerContactRepository::new();
    contacts
        .expect_find_by_id()
        .returning(move |id: CustomerContactId| {
            let contact = CustomerContact {
                id,
                customer_id,
                first_name: "Anne".to_owned(),
                last_name: "Duval".to_owned(),
                role: None,
                phone: None,
                mobile: None,
                email: None,
                is_primary: true,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            Box::pin(async move { Ok(Some(contact)) })
        });

    let mut addresses = MockCustomerContextRepository::new();
    let has_address = client.has_billing_address;
    addresses.expect_find_by_id().returning(move |id| {
        let address = has_address.then(|| CustomerContext {
            id,
            customer_id,
            label: "Head office".to_owned(),
            kind: AddressKind::Billing,
            address_line: Some("12 rue des Lilas".to_owned()),
            address_line2: None,
            postal_code: Some("69003".to_owned()),
            city: Some("Lyon".to_owned()),
            country: "FR".to_owned(),
            photo_key: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        });
        Box::pin(async move { Ok(address) })
    });

    Doubles {
        organizations,
        customers,
        contacts,
        addresses,
        members: MockMemberRepository::new(),
    }
}
