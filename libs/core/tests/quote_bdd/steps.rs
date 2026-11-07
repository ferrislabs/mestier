use chrono::{NaiveDate, TimeZone, Utc};
use cucumber::{gherkin::Step, given, then, when};
use mestier_core::{
    BillingFrequency, CreateQuoteCommand, CustomerContextId, CustomerId, DiscountPercent,
    OrganizationId, Quote, QuoteId, QuoteLine, QuoteLineCommand, QuoteLineId, QuoteStatus,
    QuoteTermsCommand, ServiceRateUnit, UpdateQuoteCommand, UpdateQuoteStatusCommand,
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

#[given("a customer without a billing address")]
async fn a_customer_without_a_billing_address(world: &mut QuoteWorld) {
    world.without_billing_address = true;
}

#[given(expr = "today is {string}")]
async fn today_is(world: &mut QuoteWorld, day: String) {
    let date = parse_date(&day);
    world.today =
        Some(Utc.from_utc_datetime(&date.and_hms_opt(9, 0, 0).expect("a valid time of day")));
}

#[given("a company subject to VAT")]
async fn a_company_subject_to_vat(world: &mut QuoteWorld) {
    world.subject_to_vat = true;
}

#[given(expr = "the company keeps its quotes valid {int} days")]
async fn the_company_keeps_its_quotes_valid(world: &mut QuoteWorld, days: i64) {
    world.validity_days = Some(days);
}

#[given(expr = "a company that has not issued any quote in {int}")]
async fn no_quote_issued_yet(world: &mut QuoteWorld, year: i32) {
    let prefix = format!("DEV-{year}-");
    assert!(
        repository::quotes(&world.store).iter().all(|quote| {
            !quote
                .reference
                .as_ref()
                .is_some_and(|reference| reference.starts_with(&prefix))
        }),
        "the scenario claims a clean year but the store already holds a quote for {year}"
    );
}

fn seed_quote(world: &mut QuoteWorld, title: String, status: &str, valid_until: Option<NaiveDate>) {
    let now = Utc::now();
    let organization_id = world.organization_id();
    let id = QuoteId(generate_uuid_v7());
    let status = parse_status(status);
    let quote = Quote {
        reference: (status != QuoteStatus::Draft).then(|| "DEV-2026-0001".to_owned()),
        status,
        net_cents: 87_500,
        gross_cents: 87_500,
        valid_until,
        lines: vec![QuoteLine {
            id: QuoteLineId(generate_uuid_v7()),
            organization_id,
            quote_id: id,
            service_rate_id: None,
            product_id: None,
            label: "Lay wall tiles".to_owned(),
            quantity: Decimal::new(125, 1),
            unit: ServiceRateUnit::M2.as_str().to_owned(),
            unit_price_cents: 3_800,
            vat_rate_bp: None,
            discount_percent: DiscountPercent::ZERO,
            billing_frequency: BillingFrequency::OneShot,
            notes: None,
            photo_keys: vec![],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }],
        ..Quote::draft(
            id,
            organization_id,
            world.customer_id(),
            world.customer_context_id(),
            title,
            now,
        )
    };

    repository::quotes(&world.store).push(quote);
    world.quote_id = Some(id);
}

#[given(expr = "a quote {string} in status {string}")]
async fn an_existing_quote(world: &mut QuoteWorld, title: String, status: String) {
    seed_quote(world, title, &status, None);
}

#[given(expr = "a quote {string} in status {string} valid until {string}")]
async fn an_existing_quote_valid_until(
    world: &mut QuoteWorld,
    title: String,
    status: String,
    until: String,
) {
    seed_quote(world, title, &status, Some(parse_date(&until)));
}

#[when(expr = "the artisan creates the quote {string} with the following lines:")]
async fn the_artisan_creates_a_quote(world: &mut QuoteWorld, step: &Step, title: String) {
    let command = CreateQuoteCommand {
        organization_id: world.organization_id(),
        title,
        customer_id: world.customer_id(),
        customer_context_id: world.customer_context_id(),
        deal_id: None,
        terms: QuoteTermsCommand::default(),
        lines: quote_lines(step),
    };

    let outcome = world
        .service()
        .create_quote(command, &mut world.doubles().lookups(), world.now())
        .await;

    world.record_outcome(outcome);
}

async fn change_status(world: &mut QuoteWorld, command: UpdateQuoteStatusCommand) {
    let outcome = world
        .service()
        .update_quote_status(command, &mut world.doubles().lookups(), world.now())
        .await
        .map(|accepted| accepted.map(|change| change.quote));

    world.record_outcome(outcome);
}

#[when("the customer accepts the quote")]
async fn the_customer_accepts(world: &mut QuoteWorld) {
    let command = UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Accepted);
    change_status(world, command).await;
}

#[when(expr = "the customer {string} accepts the quote")]
async fn the_customer_signs(world: &mut QuoteWorld, signatory: String) {
    let command = UpdateQuoteStatusCommand {
        responded_by: Some(signatory),
        ..UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Accepted)
    };
    change_status(world, command).await;
}

#[when(expr = "the customer declines the quote because {string}")]
async fn the_customer_declines(world: &mut QuoteWorld, reason: String) {
    let command = UpdateQuoteStatusCommand {
        decline_reason: Some(reason),
        ..UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Declined)
    };
    change_status(world, command).await;
}

#[when("the artisan sends the quote")]
async fn the_artisan_sends_the_quote(world: &mut QuoteWorld) {
    let command = UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Sent);
    change_status(world, command).await;
}

#[when("the artisan sends the quote again")]
async fn the_artisan_sends_the_quote_again(world: &mut QuoteWorld) {
    let command = UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Sent);
    change_status(world, command).await;
}

#[when(expr = "the artisan renames the quote to {string}")]
async fn the_artisan_renames_the_quote(world: &mut QuoteWorld, title: String) {
    let existing = world.current_quote();
    let command = UpdateQuoteCommand {
        id: existing.id,
        title,
        customer_id: existing.customer_id,
        customer_context_id: existing.customer_context_id,
        deal_id: None,
        status: existing.status,
        terms: QuoteTermsCommand::default(),
        lines: Vec::new(),
    };

    let outcome = world
        .service()
        .update_quote(command, &mut world.doubles().lookups(), world.now())
        .await
        .map(|updated| updated.map(|change| change.quote));

    world.record_outcome(outcome);
}

#[when("the artisan moves the quote to another customer")]
async fn the_artisan_moves_the_quote(world: &mut QuoteWorld) {
    let existing = world.current_quote();
    let command = UpdateQuoteCommand {
        id: existing.id,
        title: existing.title.clone(),
        customer_id: CustomerId(generate_uuid_v7()),
        customer_context_id: existing.customer_context_id,
        deal_id: None,
        status: QuoteStatus::Draft,
        terms: QuoteTermsCommand::default(),
        lines: existing
            .lines
            .iter()
            .map(|line| QuoteLineCommand {
                service_rate_id: None,
                product_id: None,
                label: line.label.clone(),
                quantity: line.quantity,
                unit: line.unit.clone(),
                unit_price_cents: line.unit_price_cents,
                vat_rate_bp: None,
                discount_percent: Decimal::ZERO,
                billing_frequency: BillingFrequency::OneShot,
                notes: None,
                photo_keys: vec![],
            })
            .collect(),
    };

    let outcome = world
        .service()
        .update_quote(command, &mut world.doubles().lookups(), world.now())
        .await
        .map(|updated| updated.map(|change| change.quote));

    world.record_outcome(outcome);
}

#[when("the artisan deletes the quote")]
async fn the_artisan_deletes_the_quote(world: &mut QuoteWorld) {
    let id = world.current_quote().id;

    let outcome = world.service().soft_delete_quote(id, world.now()).await;

    match outcome {
        Ok(Ok(())) => world.refusal = None,
        Ok(Err(refusal)) => world.refusal = Some(refusal),
        Err(error) => panic!("deleting failed unexpectedly: {error}"),
    }
}

#[when("the artisan revises the quote")]
async fn the_artisan_revises_the_quote(world: &mut QuoteWorld) {
    world.source_id = world.quote_id;
    let id = world.current_quote().id;

    let outcome = world.service().revise_quote(id, world.now()).await;

    world.record_outcome(outcome);
}

#[when("the artisan sends the revision")]
async fn the_artisan_sends_the_revision(world: &mut QuoteWorld) {
    let command = UpdateQuoteStatusCommand::to(world.current_quote().id, QuoteStatus::Sent);
    change_status(world, command).await;
}

#[then(expr = "the quote carries the reference {string}")]
async fn the_quote_carries_the_reference(world: &mut QuoteWorld, expected: String) {
    assert_eq!(world.current_quote().reference, Some(expected));
}

#[then("the quote has no reference yet")]
async fn the_quote_has_no_reference_yet(world: &mut QuoteWorld) {
    assert_eq!(world.current_quote().reference, None);
}

#[then("no reference was allocated")]
async fn no_reference_was_allocated(world: &mut QuoteWorld) {
    assert!(
        repository::quotes(&world.store)
            .iter()
            .filter(|quote| quote.status == QuoteStatus::Draft)
            .all(|quote| quote.reference.is_none()),
        "a draft must never carry a number"
    );
}

#[then("no quote is left")]
async fn no_quote_is_left(world: &mut QuoteWorld) {
    assert!(repository::quotes(&world.store).is_empty());
}

#[then(expr = "the quote is titled {string}")]
async fn the_quote_is_titled(world: &mut QuoteWorld, expected: String) {
    assert_eq!(world.current_quote().title, expected);
}

#[then(expr = "the total of the quote is {string}")]
async fn the_quote_total_is(world: &mut QuoteWorld, expected: String) {
    assert_eq!(format_euros(world.current_quote().net_cents), expected);
}

#[then(expr = "the VAT of the quote is {string}")]
async fn the_quote_vat_is(world: &mut QuoteWorld, expected: String) {
    let quote = world.current_quote();
    let vat: i32 = quote.vat_breakdown.iter().map(|line| line.vat_cents).sum();

    assert_eq!(format_euros(vat), expected);
}

#[then(expr = "the recurring totals are {string}")]
async fn the_recurring_totals_are(world: &mut QuoteWorld, expected: String) {
    let recurring = world
        .current_quote()
        .recurring()
        .iter()
        .map(|total| {
            format!(
                "{} {} {} {}",
                total.frequency, total.net_cents, total.vat_cents, total.gross_cents
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    assert_eq!(recurring, expected);
}

#[then(expr = "the quote is in status {string}")]
async fn the_quote_is_in_status(world: &mut QuoteWorld, expected: String) {
    assert_eq!(world.current_quote().status, parse_status(&expected));
}

#[then(expr = "the source quote is in status {string}")]
async fn the_source_quote_is_in_status(world: &mut QuoteWorld, expected: String) {
    let source = world.source_id.expect("the scenario revised a quote");

    assert_eq!(world.quote(source).status, parse_status(&expected));
}

#[then(expr = "the quote is valid until {string}")]
async fn the_quote_is_valid_until(world: &mut QuoteWorld, expected: String) {
    assert_eq!(
        world.current_quote().valid_until,
        Some(parse_date(&expected))
    );
}

#[then("the quote is expired")]
async fn the_quote_is_expired(world: &mut QuoteWorld) {
    assert!(world.current_quote().is_expired(world.now().date_naive()));
}

#[then("the quote is not expired")]
async fn the_quote_is_not_expired(world: &mut QuoteWorld) {
    assert!(!world.current_quote().is_expired(world.now().date_naive()));
}

#[then(expr = "the quote froze the issuer {string} and the recipient {string}")]
async fn the_quote_froze_both_parties(world: &mut QuoteWorld, issuer: String, recipient: String) {
    let quote = world.current_quote();

    assert_eq!(quote.issuer_snapshot.expect("a frozen issuer").name, issuer);
    assert_eq!(
        quote.recipient_snapshot.expect("a frozen recipient").name,
        recipient
    );
}

#[then(expr = "the quote was answered by {string} on {string}")]
async fn the_quote_was_answered_by(world: &mut QuoteWorld, who: String, day: String) {
    let quote = world.current_quote();

    assert_eq!(quote.responded_by.as_deref(), Some(who.as_str()));
    assert_eq!(
        quote.responded_at.map(|at| at.date_naive()),
        Some(parse_date(&day))
    );
}

#[then(expr = "the quote was declined because {string}")]
async fn the_quote_was_declined_because(world: &mut QuoteWorld, reason: String) {
    assert_eq!(
        world.current_quote().decline_reason.as_deref(),
        Some(reason.as_str())
    );
}

#[then("the revision is a draft without reference nor validity")]
async fn the_revision_is_a_clean_draft(world: &mut QuoteWorld) {
    let revision = world.current_quote();

    assert_eq!(revision.status, QuoteStatus::Draft);
    assert_eq!(revision.reference, None);
    assert_eq!(revision.valid_until, None);
    assert_eq!(revision.revision_of_id, world.source_id);
}

#[then(expr = "the revision keeps the title {string} and the lines of the source")]
async fn the_revision_keeps_the_content(world: &mut QuoteWorld, title: String) {
    let revision = world.current_quote();
    let source = world.quote(world.source_id.expect("the scenario revised a quote"));

    assert_eq!(revision.title, title);
    assert_eq!(revision.lines.len(), source.lines.len());
    assert_ne!(revision.lines[0].id, source.lines[0].id);
    assert_eq!(revision.net_cents, source.net_cents);
}

#[then(expr = "the company is notified of the event {string}")]
async fn the_company_is_notified(world: &mut QuoteWorld, expected: String) {
    let published = world.emitter.names();

    assert!(
        published.contains(&expected),
        "expected `{expected}` among the published events, got {published:?}"
    );
}

#[then(expr = "the answer is refused with the code {string}")]
async fn the_answer_is_refused_with_the_code(world: &mut QuoteWorld, code: String) {
    let refusal = world
        .refusal
        .expect("the scenario expected a refusal and got none");

    assert_eq!(refusal.code(), code, "{refusal}");
}

#[then(expr = "the answer is refused with the code {string} on {string}")]
async fn the_answer_is_refused_on_a_field(world: &mut QuoteWorld, code: String, field: String) {
    let refusal = world
        .refusal
        .expect("the scenario expected a refusal and got none");

    assert_eq!(refusal.code(), code, "{refusal}");
    assert_eq!(refusal.field().as_str(), field, "{refusal}");
}

#[then("the creation is refused")]
async fn the_creation_is_refused(world: &mut QuoteWorld) {
    assert!(
        world.refusal.is_some() || matches!(world.outcome, Some(Err(_))),
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

fn quote_lines(step: &Step) -> Vec<QuoteLineCommand> {
    let table = step
        .table
        .as_ref()
        .expect("this step is written with a table of lines");
    let header = table
        .rows
        .first()
        .expect("a table starts with its header row");
    let position = |name: &str| header.iter().position(|cell| cell.trim() == name);
    let column = |name: &str| {
        position(name).unwrap_or_else(|| panic!("the table of lines needs a `{name}` column"))
    };
    let (work, quantity, unit, price) = (
        column("work"),
        column("quantity"),
        column("unit"),
        column("unit price"),
    );
    let (discount, vat, frequency) = (position("discount"), position("vat"), position("frequency"));

    table.rows[1..]
        .iter()
        .map(|row| QuoteLineCommand {
            service_rate_id: None,
            product_id: None,
            label: row[work].trim().to_owned(),
            quantity: parse_decimal(&row[quantity]),
            unit: parse_unit(&row[unit]).as_str().to_owned(),
            unit_price_cents: parse_euros(&row[price]),
            vat_rate_bp: vat.map(|index| parse_basis_points(&row[index])),
            discount_percent: discount.map_or(Decimal::ZERO, |index| parse_decimal(&row[index])),
            billing_frequency: frequency.map_or(BillingFrequency::OneShot, |index| {
                row[index]
                    .trim()
                    .to_uppercase()
                    .parse()
                    .unwrap_or_else(|error| panic!("{error}"))
            }),
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

fn parse_basis_points(raw: &str) -> i32 {
    (parse_decimal(raw) * Decimal::from(100))
        .round()
        .try_into()
        .unwrap_or_else(|_| panic!("`{raw}` is not a VAT rate"))
}

fn parse_date(raw: &str) -> NaiveDate {
    NaiveDate::parse_from_str(raw.trim(), "%Y-%m-%d")
        .unwrap_or_else(|_| panic!("`{raw}` is not a date"))
}

fn parse_euros(raw: &str) -> i32 {
    let amount = parse_decimal(raw.trim().trim_start_matches('€'));
    let cents = (amount * Decimal::from(100)).round();

    cents
        .try_into()
        .unwrap_or_else(|_| panic!("`{raw}` is outside the range of an amount in cents"))
}

fn format_euros(cents: i32) -> String {
    format!("€{}.{:02}", cents / 100, (cents % 100).abs())
}

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
