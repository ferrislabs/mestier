use chrono::Utc;
use common::generate_uuid_v7;
use cucumber::{given, then, when};
use mestier_core::{
    AddressKind, CreateCustomerCommand, CreateCustomerContactCommand, CreateCustomerContextCommand,
    Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerError, CustomerId, CustomerKind, CustomerProfile, CustomerStatus, LeadSource, Member,
    MemberId, OrganizationId, RegistrationNumber, UpdateCustomerCommand, payment_terms_of,
};

use crate::{repository, world::CustomerWorld};

fn status(name: &str) -> CustomerStatus {
    name.parse()
        .unwrap_or_else(|_| panic!("the scenario names an unknown status `{name}`"))
}

fn stored(world: &CustomerWorld) -> Customer {
    stored_by_id(world, world.customer_id())
}

fn stored_by_id(world: &CustomerWorld, id: CustomerId) -> Customer {
    repository::state(&world.store)
        .customers
        .get(&id)
        .cloned()
        .expect("the customer was stored")
}

fn customer_named(world: &CustomerWorld, name: &str) -> CustomerId {
    *world
        .customers
        .get(name)
        .unwrap_or_else(|| panic!("the scenario never created the customer `{name}`"))
}

fn member(world: &mut CustomerWorld, name: &str, organization_id: OrganizationId) {
    let id = MemberId(generate_uuid_v7());
    repository::state(&world.store).members.insert(
        id,
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: name.to_owned(),
            first_name: None,
            joined_at: None,
            created_at: Utc::now(),
            deleted_at: None,
        },
    );
    world.members.insert(name.to_owned(), id);
}

async fn create(
    world: &mut CustomerWorld,
    name: &str,
    status: CustomerStatus,
    profile: CustomerProfile,
) -> Result<Customer, CustomerError> {
    let command = CreateCustomerCommand {
        actor: world.actor(),
        organization_id: world.organization_id(),
        status,
        name: name.to_owned(),
        registration_number: None,
        phone: None,
        email: None,
        profile,
    };
    let created = world
        .customers()
        .create_customer(command)
        .await
        .expect("no infrastructure failure here");
    if let Ok(customer) = &created {
        world.customers.insert(name.to_owned(), customer.id);
        world.current = Some(customer.id);
    }
    created
}

async fn update(
    world: &mut CustomerWorld,
    id: CustomerId,
    status: Option<CustomerStatus>,
    profile: impl FnOnce(&mut CustomerProfile),
) -> Result<Customer, CustomerError> {
    let current = stored_by_id(world, id);
    let mut next = current.profile.clone();
    profile(&mut next);
    let command = UpdateCustomerCommand {
        actor: world.actor(),
        id,
        status: status.unwrap_or(current.status),
        name: current.name,
        registration_number: current
            .registration_number
            .map(|number| RegistrationNumber::try_from(number).expect("stored values are valid")),
        phone: current.phone,
        email: current.email,
        profile: next,
    };
    world
        .customers()
        .update_customer(command)
        .await
        .expect("no infrastructure failure here")
}

fn contact(customer_id: CustomerId, first_name: &str) -> CustomerContact {
    let now = Utc::now();
    CustomerContact {
        id: CustomerContactId(generate_uuid_v7()),
        customer_id,
        first_name: first_name.to_owned(),
        last_name: "Test".to_owned(),
        role: None,
        phone: None,
        mobile: None,
        email: None,
        is_primary: false,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn address(customer_id: CustomerId, label: &str) -> CustomerContext {
    let now = Utc::now();
    CustomerContext {
        id: CustomerContextId(generate_uuid_v7()),
        customer_id,
        label: label.to_owned(),
        kind: AddressKind::Site,
        address_line: None,
        address_line2: None,
        postal_code: None,
        city: None,
        country: "FR".to_owned(),
        photo_key: None,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn contact_named(world: &CustomerWorld, first_name: &str) -> CustomerContact {
    repository::state(&world.store)
        .contacts
        .iter()
        .find(|found| found.first_name == first_name)
        .cloned()
        .unwrap_or_else(|| panic!("no contact named `{first_name}`"))
}

fn address_named(world: &CustomerWorld, label: &str) -> CustomerContext {
    repository::state(&world.store)
        .contexts
        .iter()
        .find(|found| found.label == label)
        .cloned()
        .unwrap_or_else(|| panic!("no address labelled `{label}`"))
}

#[given("a company registered on Mestier")]
async fn a_registered_company(world: &mut CustomerWorld) {
    world.organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given(expr = "a team member {string}")]
async fn a_team_member(world: &mut CustomerWorld, name: String) {
    let organization_id = world.organization_id();
    member(world, &name, organization_id);
}

#[given(expr = "a customer {string} that is a {word}")]
async fn a_customer_with_status(world: &mut CustomerWorld, name: String, wanted: String) {
    create(world, &name, status(&wanted), CustomerProfile::default())
        .await
        .expect("a customer without references is accepted");
}

#[given(expr = "a customer {string} with a contact {string} and an address {string}")]
async fn a_customer_with_contact_and_address(
    world: &mut CustomerWorld,
    name: String,
    first_name: String,
    label: String,
) {
    let created = create(
        world,
        &name,
        CustomerStatus::Prospect,
        CustomerProfile::default(),
    )
    .await
    .expect("a customer without references is accepted");
    let mut guard = repository::state(&world.store);
    guard.contacts.push(contact(created.id, &first_name));
    guard.contexts.push(address(created.id, &label));
}

#[when(
    expr = "the artisan creates the {word} customer {string} with website {string}, VAT number {string}, owner {string}, source {string}, notes {string} and payment terms of {int} days"
)]
#[allow(clippy::too_many_arguments)]
async fn the_artisan_creates(
    world: &mut CustomerWorld,
    kind: String,
    name: String,
    website: String,
    vat_number: String,
    owner: String,
    source: String,
    notes: String,
    days: i64,
) {
    let profile = CustomerProfile {
        kind: kind.parse().expect("a known kind"),
        website: Some(website),
        vat_number: Some(vat_number),
        owner_member_id: Some(world.members[&owner]),
        lead_source: Some(source.parse().expect("a known source")),
        notes: Some(notes),
        payment_terms: payment_terms_of(Some(days)).expect("within bounds"),
        ..CustomerProfile::default()
    };
    create(world, &name, CustomerStatus::Prospect, profile)
        .await
        .expect("a complete profile is accepted");
}

#[when(expr = "the artisan designates the contact {string} and the address {string} for billing")]
async fn the_artisan_designates(world: &mut CustomerWorld, contact: String, label: String) {
    let contact_id = contact_named(world, &contact).id;
    let address_id = address_named(world, &label).id;
    let id = world.customer_id();
    update(world, id, None, |profile| {
        profile.billing_contact_id = Some(contact_id);
        profile.billing_address_id = Some(address_id);
    })
    .await
    .expect("a contact and an address of the customer are accepted");
}

#[when(expr = "the artisan designates the contact {string} of {string} for billing on {string}")]
async fn designates_a_foreign_contact(
    world: &mut CustomerWorld,
    contact: String,
    _owner: String,
    target: String,
) {
    let contact_id = contact_named(world, &contact).id;
    let id = customer_named(world, &target);
    world.refusal = update(world, id, None, |profile| {
        profile.billing_contact_id = Some(contact_id);
    })
    .await
    .err();
}

#[when(expr = "the artisan designates the address {string} of {string} for billing on {string}")]
async fn designates_a_foreign_address(
    world: &mut CustomerWorld,
    label: String,
    _owner: String,
    target: String,
) {
    let address_id = address_named(world, &label).id;
    let id = customer_named(world, &target);
    world.refusal = update(world, id, None, |profile| {
        profile.billing_address_id = Some(address_id);
    })
    .await
    .err();
}

#[when("the artisan creates a customer owned by a member of another company")]
async fn owned_by_a_foreign_member(world: &mut CustomerWorld) {
    member(world, "Stranger", OrganizationId(generate_uuid_v7()));
    let profile = CustomerProfile {
        owner_member_id: Some(world.members["Stranger"]),
        ..CustomerProfile::default()
    };
    world.refusal = create(
        world,
        "Boulangerie Lemoine",
        CustomerStatus::Prospect,
        profile,
    )
    .await
    .err();
}

#[when(expr = "the artisan sets the status to {word}")]
async fn the_artisan_sets_the_status(world: &mut CustomerWorld, wanted: String) {
    let id = world.customer_id();
    if let Some(converted_at) = stored(world).converted_at {
        world.first_conversion.get_or_insert(converted_at);
    }
    update(world, id, Some(status(&wanted)), |_| {})
        .await
        .expect("a status change is accepted");
    if world.first_conversion.is_none() {
        world.first_conversion = stored(world).converted_at;
    }
}

#[when(expr = "the artisan enters the registration number {string}")]
async fn enters_a_registration_number(world: &mut CustomerWorld, entered: String) {
    world.registration = Some(RegistrationNumber::try_from(entered.as_str()));
}

#[when(expr = "the artisan enters payment terms of {int} days")]
async fn enters_payment_terms(world: &mut CustomerWorld, days: i64) {
    world.payment_terms = Some(payment_terms_of(Some(days)));
}

#[when(expr = "the artisan adds the contact {string} with mobile {string} and phone {string}")]
async fn adds_a_contact(
    world: &mut CustomerWorld,
    first_name: String,
    mobile: String,
    phone: String,
) {
    let command = CreateCustomerContactCommand {
        actor: world.actor(),
        customer_id: world.customer_id(),
        first_name,
        last_name: "Test".to_owned(),
        role: None,
        phone: Some(phone),
        mobile: Some(mobile),
        email: None,
        is_primary: false,
    };
    world
        .contacts()
        .create_customer_contact(command)
        .await
        .expect("a contact with a mobile is stored");
}

#[when(expr = "the artisan adds the {word} address {string} in {string} with second line {string}")]
async fn adds_an_address(
    world: &mut CustomerWorld,
    kind: String,
    label: String,
    country: String,
    line2: String,
) {
    let command = CreateCustomerContextCommand {
        actor: world.actor(),
        customer_id: world.customer_id(),
        label,
        kind: kind.parse().expect("a known address kind"),
        address_line: Some("1 quai du Port".to_owned()),
        address_line2: Some(line2),
        postal_code: Some("44000".to_owned()),
        city: Some("Nantes".to_owned()),
        country,
        photo_key: None,
    };
    world
        .contexts()
        .create_customer_context(command)
        .await
        .expect("an address with a kind and a country is stored");
}

#[then(expr = "the customer is a {word}")]
async fn the_customer_kind(world: &mut CustomerWorld, kind: String) {
    assert_eq!(
        stored(world).profile.kind,
        kind.parse::<CustomerKind>().expect("a known kind")
    );
}

#[then(expr = "the customer website is {string}")]
async fn the_customer_website(world: &mut CustomerWorld, website: String) {
    assert_eq!(
        stored(world).profile.website.as_deref(),
        Some(website.as_str())
    );
}

#[then(expr = "the customer VAT number is {string}")]
async fn the_customer_vat_number(world: &mut CustomerWorld, vat_number: String) {
    assert_eq!(
        stored(world).profile.vat_number.as_deref(),
        Some(vat_number.as_str())
    );
}

#[then(expr = "the customer owner is {string}")]
async fn the_customer_owner(world: &mut CustomerWorld, owner: String) {
    assert_eq!(
        stored(world).profile.owner_member_id,
        Some(world.members[&owner])
    );
}

#[then(expr = "the customer source is {string}")]
async fn the_customer_source(world: &mut CustomerWorld, source: String) {
    assert_eq!(
        stored(world).profile.lead_source,
        Some(source.parse::<LeadSource>().expect("a known source"))
    );
}

#[then(expr = "the customer notes are {string}")]
async fn the_customer_notes(world: &mut CustomerWorld, notes: String) {
    assert_eq!(stored(world).profile.notes.as_deref(), Some(notes.as_str()));
}

#[then(expr = "the customer payment terms are {int} days")]
async fn the_customer_payment_terms(world: &mut CustomerWorld, days: i32) {
    assert_eq!(
        stored(world).profile.payment_terms.map(|terms| terms.get()),
        Some(days)
    );
}

#[then(expr = "the billing contact is {string}")]
async fn the_billing_contact(world: &mut CustomerWorld, first_name: String) {
    assert_eq!(
        stored(world).profile.billing_contact_id,
        Some(contact_named(world, &first_name).id)
    );
}

#[then(expr = "the billing address is {string}")]
async fn the_billing_address(world: &mut CustomerWorld, label: String) {
    assert_eq!(
        stored(world).profile.billing_address_id,
        Some(address_named(world, &label).id)
    );
}

#[then(expr = "the customer status is {word}")]
async fn the_customer_status(world: &mut CustomerWorld, wanted: String) {
    assert_eq!(stored(world).status, status(&wanted));
}

#[then("converted_at is set")]
async fn converted_at_is_set(world: &mut CustomerWorld) {
    assert!(stored(world).converted_at.is_some());
}

#[then("converted_at is empty")]
async fn converted_at_is_empty(world: &mut CustomerWorld) {
    assert_eq!(stored(world).converted_at, None);
}

#[then("archived_at is set")]
async fn archived_at_is_set(world: &mut CustomerWorld) {
    assert!(stored(world).archived_at.is_some());
}

#[then("archived_at is empty")]
async fn archived_at_is_empty(world: &mut CustomerWorld) {
    assert_eq!(stored(world).archived_at, None);
}

#[then("converted_at is the same as after the first conversion")]
async fn converted_at_is_kept(world: &mut CustomerWorld) {
    assert_eq!(stored(world).converted_at, world.first_conversion);
}

#[then(expr = "the registration number is stored as {string}")]
async fn the_registration_number_is_stored(world: &mut CustomerWorld, expected: String) {
    let number = world
        .registration
        .take()
        .expect("the scenario entered a number")
        .expect("the number was accepted");
    assert_eq!(number.as_str(), expected);
}

#[then(expr = "the registration number is refused on the field {string}")]
async fn the_registration_number_is_refused(world: &mut CustomerWorld, field: String) {
    let refusal = world
        .registration
        .take()
        .expect("the scenario entered a number")
        .expect_err("the number should have been refused");
    assert_eq!(refusal.field.as_str(), field);
}

#[then(expr = "the payment terms are refused on the field {string}")]
async fn the_payment_terms_are_refused(world: &mut CustomerWorld, field: String) {
    let refusal = world
        .payment_terms
        .take()
        .expect("the scenario entered payment terms")
        .expect_err("the terms should have been refused");
    assert_eq!(refusal.field.as_str(), field);
}

#[then(expr = "the designation is refused on the field {string}")]
async fn the_designation_is_refused(world: &mut CustomerWorld, field: String) {
    let refusal = world
        .refusal
        .expect("the designation should have been refused");
    assert_eq!(refusal.field.as_str(), field);
}

#[then(expr = "the customer {string} has no billing contact")]
async fn no_billing_contact(world: &mut CustomerWorld, name: String) {
    let id = customer_named(world, &name);
    assert_eq!(stored_by_id(world, id).profile.billing_contact_id, None);
}

#[then(expr = "the customer {string} has no billing address")]
async fn no_billing_address(world: &mut CustomerWorld, name: String) {
    let id = customer_named(world, &name);
    assert_eq!(stored_by_id(world, id).profile.billing_address_id, None);
}

#[then(expr = "the contact {string} has the mobile {string} and the phone {string}")]
async fn the_contact_numbers(
    world: &mut CustomerWorld,
    first_name: String,
    mobile: String,
    phone: String,
) {
    let found = contact_named(world, &first_name);
    assert_eq!(found.mobile.as_deref(), Some(mobile.as_str()));
    assert_eq!(found.phone.as_deref(), Some(phone.as_str()));
}

#[then(expr = "the address {string} is of kind {word} in {string} with second line {string}")]
async fn the_address_facts(
    world: &mut CustomerWorld,
    label: String,
    kind: String,
    country: String,
    line2: String,
) {
    let found = address_named(world, &label);
    assert_eq!(
        found.kind,
        kind.parse::<AddressKind>().expect("a known kind")
    );
    assert_eq!(found.country, country);
    assert_eq!(found.address_line2.as_deref(), Some(line2.as_str()));
}

#[then(expr = "the company is notified of the event {string}")]
async fn the_company_is_notified(world: &mut CustomerWorld, expected: String) {
    let published = world.emitter.names();

    assert!(
        published.contains(&expected),
        "expected `{expected}` among the published events, got {published:?}"
    );
}

#[then(expr = "the company is notified of {int} events {string}")]
async fn the_company_is_notified_n_times(
    world: &mut CustomerWorld,
    count: usize,
    expected: String,
) {
    let published = world.emitter.names();
    let seen = published.iter().filter(|name| **name == expected).count();

    assert_eq!(seen, count, "`{expected}` among {published:?}");
}

#[then("the company is notified of no event")]
async fn the_company_is_notified_of_nothing(world: &mut CustomerWorld) {
    assert_eq!(world.emitter.names(), Vec::<String>::new());
}

#[then(expr = "the event {string} carries {string} equal to {string}")]
async fn the_event_carries(
    world: &mut CustomerWorld,
    name: String,
    pointer: String,
    expected: String,
) {
    let payload = world.emitter.only(&name).payload;
    let found = payload
        .pointer(&pointer)
        .unwrap_or_else(|| panic!("no `{pointer}` in {payload}"));
    let found = found
        .as_str()
        .map_or_else(|| found.to_string(), str::to_owned);

    assert_eq!(found, expected, "`{pointer}` of {payload}");
}
