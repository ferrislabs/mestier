use common::generate_uuid_v7;
use cucumber::{given, then, when};
use mestier_core::{
    BillingSettings, BillingSettingsInput, NumberSeries, NumberingError, OrganizationId,
    ReplaceBillingSettingsCommand, SetNextNumberCommand,
};

use crate::{repository, world::BillingWorld};

fn valid_input() -> BillingSettingsInput {
    BillingSettingsInput {
        payment_terms_days: 30,
        quote_validity_days: 30,
        default_vat_rate_bp: 2000,
        late_penalty: "trois fois le taux d’intérêt légal".to_owned(),
        recovery_fee_cents: 4000,
        invoice_footer: String::new(),
        number_padding: 4,
        iban: None,
        bic: None,
    }
}

fn input_with(field: &str, value: i64) -> BillingSettingsInput {
    let base = valid_input();
    match field {
        "payment_terms_days" => BillingSettingsInput {
            payment_terms_days: value,
            ..base
        },
        "quote_validity_days" => BillingSettingsInput {
            quote_validity_days: value,
            ..base
        },
        "default_vat_rate_bp" => BillingSettingsInput {
            default_vat_rate_bp: value,
            ..base
        },
        "recovery_fee_cents" => BillingSettingsInput {
            recovery_fee_cents: value,
            ..base
        },
        "number_padding" => BillingSettingsInput {
            number_padding: value,
            ..base
        },
        other => panic!("the scenario names an unknown field `{other}`"),
    }
}

fn series(name: &str) -> NumberSeries {
    name.parse()
        .unwrap_or_else(|_| panic!("the scenario names an unknown series `{name}`"))
}

#[given("a company registered on Mestier")]
async fn a_registered_company(world: &mut BillingWorld) {
    world.organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given(expr = "the {word} series has assigned number {int} in {int}")]
async fn a_series_with_a_last_number(world: &mut BillingWorld, name: String, last: i32, year: i32) {
    repository::state(&world.store)
        .counters
        .insert((series(&name), year), last);
}

#[given(expr = "the reference {string} was assigned before")]
async fn a_reference_was_assigned(world: &mut BillingWorld, reference: String) {
    world.assigned_references.push(reference);
}

#[when("the artisan reads the billing settings")]
async fn the_artisan_reads(world: &mut BillingWorld) {
    let id = world.organization_id();
    world.read = Some(
        world
            .service()
            .get_billing_settings(id)
            .await
            .expect("reading never fails for a known company"),
    );
}

#[when(expr = "the artisan saves the billing settings with {word} set to {int}")]
async fn the_artisan_saves(world: &mut BillingWorld, field: String, value: i64) {
    match BillingSettings::try_from(input_with(&field, value)) {
        Err(refusal) => world.save_refusal = Some(refusal),
        Ok(settings) => {
            let command = ReplaceBillingSettingsCommand {
                actor: world.actor(),
                id: world.organization_id(),
                settings,
            };
            world
                .service()
                .replace_billing_settings(command)
                .await
                .expect("a valid save is stored");
        }
    }
}

#[when(expr = "the artisan sets the next {word} number to {int} for {int}")]
async fn the_artisan_sets_the_next_number(
    world: &mut BillingWorld,
    name: String,
    requested: i64,
    year: i32,
) {
    let command = SetNextNumberCommand {
        actor: world.actor(),
        id: world.organization_id(),
        series: series(&name),
        year,
        next_number: requested,
    };
    match world
        .service()
        .set_next_number(command)
        .await
        .expect("no infrastructure failure here")
    {
        Ok(numbering) => world.numbering = Some(numbering),
        Err(refusal) => world.numbering_refusal = Some(refusal),
    }
}

#[then(expr = "the payment terms are {int} days")]
async fn payment_terms(world: &mut BillingWorld, days: i32) {
    assert_eq!(read(world).payment_terms_days.get(), days);
}

#[then(expr = "the quote validity is {int} days")]
async fn quote_validity(world: &mut BillingWorld, days: i32) {
    assert_eq!(read(world).quote_validity_days.get(), days);
}

#[then(expr = "the default VAT rate is {int} basis points")]
async fn default_vat_rate(world: &mut BillingWorld, basis_points: i32) {
    assert_eq!(read(world).default_vat_rate.basis_points(), basis_points);
}

#[then(expr = "the recovery fee is {int} cents")]
async fn recovery_fee(world: &mut BillingWorld, cents: i32) {
    assert_eq!(read(world).recovery_fee.get(), cents);
}

#[then(expr = "the number padding is {int}")]
async fn number_padding(world: &mut BillingWorld, digits: usize) {
    assert_eq!(read(world).number_padding.get(), digits);
}

#[then("no IBAN is set")]
async fn no_iban(world: &mut BillingWorld) {
    assert_eq!(read(world).iban, None);
}

#[then(expr = "the save is refused on the field {string}")]
async fn the_save_is_refused(world: &mut BillingWorld, field: String) {
    let refusal = world
        .save_refusal
        .expect("the save should have been refused");
    assert_eq!(refusal.field.as_str(), field);
}

#[then("no billing settings were stored")]
async fn nothing_stored(world: &mut BillingWorld) {
    let guard = repository::state(&world.store);
    assert_eq!(guard.writes, 0);
    assert!(guard.settings.is_none());
}

#[then(expr = "the change is refused as not forward, the last assigned number being {int}")]
async fn the_change_is_refused(world: &mut BillingWorld, last: i32) {
    assert_eq!(
        world.numbering_refusal,
        Some(NumberingError::NotForward {
            last_assigned: last
        })
    );
}

#[then(expr = "the {word} series still has {int} as last assigned number in {int}")]
async fn the_series_is_unchanged(world: &mut BillingWorld, name: String, last: i32, year: i32) {
    assert_eq!(repository::counter(&world.store, series(&name), year), last);
}

#[then(expr = "the {word} series has {int} as last assigned number in {int}")]
async fn the_series_has(world: &mut BillingWorld, name: String, last: i32, year: i32) {
    assert_eq!(repository::counter(&world.store, series(&name), year), last);
}

#[then(expr = "the next {word} reference of {int} is {string}")]
async fn the_next_reference(world: &mut BillingWorld, name: String, year: i32, reference: String) {
    let wanted = series(&name);
    let id = world.organization_id();
    let numbering = world
        .service()
        .get_numbering(id, year)
        .await
        .expect("reading the numbering never fails for a known company");
    let entry = numbering
        .iter()
        .find(|entry| entry.series == wanted)
        .expect("every series is listed");
    assert_eq!(entry.next_reference(), reference);
}

#[then(expr = "the reference {string} is unchanged")]
async fn the_reference_is_unchanged(world: &mut BillingWorld, reference: String) {
    assert!(world.assigned_references.contains(&reference));
}

fn read(world: &BillingWorld) -> &BillingSettings {
    world
        .read
        .as_ref()
        .expect("the scenario reads the settings first")
}
