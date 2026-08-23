//! Step definitions for `features/quote.feature`.
//!
//! Steps translate the sentences of a scenario into calls on the domain service
//! and read the answer back. They hold no business rule of their own: if a total
//! looks wrong here, the fix belongs in `domain::quote::service`, never in this
//! file.

use cucumber::{gherkin::Step, given, then, when};
use mestier_core::{
    CreateQuoteCommand, CustomerContextId, CustomerId, OrganizationId, Quote, QuoteId, QuoteLine,
    QuoteLineCommand, QuoteLineId, QuoteStatus, ServiceRateUnit, UpdateQuoteStatusCommand,
};
use rust_decimal::Decimal;

use common::generate_uuid_v7;

use crate::{repository, world::QuoteWorld};

#[given("a company registered on Mestier")]
async fn a_registered_company(world: &mut QuoteWorld) {
    world.organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given(expr = "a customer {string} belonging to that company")]
async fn a_customer_of_that_company(world: &mut QuoteWorld, _name: String) {
    world.customer_id = Some(CustomerId(generate_uuid_v7()));
    world.customer_context_id = Some(CustomerContextId(generate_uuid_v7()));
}

/// States why the first reference of the scenario ends in `0001`: numbering
/// restarts every year, per company.
#[given(expr = "a company that has not issued any quote in {int}")]
async fn no_quote_issued_yet(world: &mut QuoteWorld, year: i32) {
    let prefix = format!("DEV-{year}-");
    assert!(
        repository::quotes(&world.store)
            .iter()
            .all(|quote| !quote.reference.starts_with(&prefix)),
        "the scenario claims a clean year but the store already holds a quote for {year}"
    );
}

#[given(expr = "a quote {string} in status {string}")]
async fn an_existing_quote(world: &mut QuoteWorld, title: String, status: String) {
    let now = chrono::Utc::now();
    let organization_id = world.organization_id();
    let id = QuoteId(generate_uuid_v7());
    let quote = Quote {
        id,
        organization_id,
        reference: "DEV-2026-0001".to_owned(),
        title,
        customer_id: world.customer_id(),
        customer_context_id: world.customer_context_id(),
        status: parse_status(&status),
        net_cents: 87_500,
        vat_breakdown: Vec::new(),
        gross_cents: 87_500,
        lines: vec![QuoteLine {
            id: QuoteLineId(generate_uuid_v7()),
            organization_id,
            quote_id: id,
            service_rate_id: None,
            label: "Lay wall tiles".to_owned(),
            quantity: Decimal::new(125, 1),
            unit: ServiceRateUnit::M2,
            unit_price_cents: 3_800,
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
    };

    repository::quotes(&world.store).push(quote);
    world.quote_id = Some(id);
}

#[when(expr = "the artisan creates the quote {string} with the following lines:")]
async fn the_artisan_creates_a_quote(world: &mut QuoteWorld, step: &Step, title: String) {
    let command = CreateQuoteCommand {
        organization_id: world.organization_id(),
        title,
        customer_id: world.customer_id(),
        customer_context_id: world.customer_context_id(),
        lines: quote_lines(step),
    };

    let outcome = world
        .service()
        .create_quote(command, repository::stubbed_organization())
        .await;

    world.record(outcome);
}

#[when("the customer accepts the quote")]
async fn the_customer_accepts(world: &mut QuoteWorld) {
    let command = UpdateQuoteStatusCommand {
        id: world.current_quote().id,
        status: QuoteStatus::Accepted,
    };

    let outcome = world.service().update_quote_status(command).await;

    world.record(outcome);
}

#[then(expr = "the quote carries the reference {string}")]
async fn the_quote_carries_the_reference(world: &mut QuoteWorld, expected: String) {
    assert_eq!(world.current_quote().reference, expected);
}

#[then(expr = "the total of the quote is {string}")]
async fn the_quote_total_is(world: &mut QuoteWorld, expected: String) {
    assert_eq!(format_euros(world.current_quote().net_cents), expected);
}

#[then(expr = "the quote is in status {string}")]
async fn the_quote_is_in_status(world: &mut QuoteWorld, expected: String) {
    assert_eq!(world.current_quote().status, parse_status(&expected));
}

#[then(expr = "the company is notified of the event {string}")]
async fn the_company_is_notified(world: &mut QuoteWorld, expected: String) {
    let published = world.emitter.names();

    assert!(
        published.contains(&expected),
        "expected `{expected}` among the published events, got {published:?}"
    );
}

#[then("the creation is refused")]
async fn the_creation_is_refused(world: &mut QuoteWorld) {
    let outcome = world
        .outcome
        .as_ref()
        .expect("a scenario acts before asserting on the answer");

    assert!(
        outcome.is_err(),
        "the quote was accepted when the scenario expected a refusal"
    );
}

#[then("no quote was stored")]
async fn nothing_was_stored(world: &mut QuoteWorld) {
    assert!(
        repository::quotes(&world.store).is_empty(),
        "a refused quote must leave no trace"
    );
    assert!(
        world.emitter.names().is_empty(),
        "a refused quote must publish nothing"
    );
}

/// Reads the Gherkin table into the command the domain expects.
///
/// Columns are addressed by header name, so reordering them in the `.feature`
/// stays a documentation change rather than a broken test.
fn quote_lines(step: &Step) -> Vec<QuoteLineCommand> {
    let table = step
        .table
        .as_ref()
        .expect("this step is written with a table of lines");
    let header = table
        .rows
        .first()
        .expect("a table starts with its header row");
    let column = |name: &str| {
        header
            .iter()
            .position(|cell| cell.trim() == name)
            .unwrap_or_else(|| panic!("the table of lines needs a `{name}` column"))
    };
    let (work, quantity, unit, price) = (
        column("work"),
        column("quantity"),
        column("unit"),
        column("unit price"),
    );

    table.rows[1..]
        .iter()
        .map(|row| QuoteLineCommand {
            service_rate_id: None,
            label: row[work].trim().to_owned(),
            quantity: parse_decimal(&row[quantity]),
            unit: parse_unit(&row[unit]),
            unit_price_cents: parse_euros(&row[price]),
            vat_rate_bp: None,
            notes: None,
            photo_keys: vec![],
        })
        .collect()
}

fn parse_decimal(raw: &str) -> Decimal {
    raw.trim()
        .parse()
        .unwrap_or_else(|_| panic!("`{raw}` is not a number"))
}

/// `€45.00` to the 4500 cents the domain stores. Money never travels as a
/// float, in the scenarios no more than in the code.
fn parse_euros(raw: &str) -> i32 {
    let amount = parse_decimal(raw.trim().trim_start_matches('€'));
    let cents = (amount * Decimal::from(100)).round();

    cents
        .try_into()
        .unwrap_or_else(|_| panic!("`{raw}` is outside the range of an amount in cents"))
}

/// The inverse, so a `Then` can be written the way a quote is read.
fn format_euros(cents: i32) -> String {
    format!("€{}.{:02}", cents / 100, (cents % 100).abs())
}

/// Scenarios name units and statuses in lower case, which is how a reader
/// writes them. Both enums already parse their wire form, so this only bridges
/// the case: a new variant needs no change here.
fn parse_unit(raw: &str) -> ServiceRateUnit {
    raw.trim()
        .to_uppercase()
        .parse()
        .unwrap_or_else(|error| panic!("{error}"))
}

fn parse_status(raw: &str) -> QuoteStatus {
    raw.trim()
        .to_uppercase()
        .parse()
        .unwrap_or_else(|error| panic!("{error}"))
}
