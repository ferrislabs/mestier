use chrono::{Days, Utc};
use common::generate_uuid_v7;
use cucumber::{given, then, when};
use mestier_core::{
    CreateCustomerCommand, CreateDealCommand, Customer, CustomerId, CustomerProfile,
    CustomerStatus, Deal, DealAmount, DealId, DealStage, DealStageName, DealTitle, LostReason,
    MoveDealCommand, OrganizationId, UpdateCustomerCommand,
};

use crate::{
    repository,
    world::{DealWorld, Refusal},
};

fn customer_status(name: &str) -> CustomerStatus {
    name.parse()
        .unwrap_or_else(|_| panic!("the scenario names an unknown status `{name}`"))
}

fn stage_name(name: &str) -> DealStageName {
    name.parse()
        .unwrap_or_else(|_| panic!("the scenario names an unknown stage `{name}`"))
}

fn customer_named(world: &DealWorld, name: &str) -> CustomerId {
    *world
        .customers
        .get(name)
        .unwrap_or_else(|| panic!("the scenario never created the customer `{name}`"))
}

fn deal_named(world: &DealWorld, title: &str) -> DealId {
    *world
        .deals
        .get(title)
        .unwrap_or_else(|| panic!("the scenario never created the deal `{title}`"))
}

fn stored_deal(world: &DealWorld, title: &str) -> Deal {
    let id = deal_named(world, title);
    repository::state(&world.store)
        .deals
        .get(&id)
        .cloned()
        .expect("the deal was stored")
}

fn stored_customer(world: &DealWorld, name: &str) -> Customer {
    let id = customer_named(world, name);
    repository::state(&world.store)
        .customers
        .get(&id)
        .cloned()
        .expect("the customer was stored")
}

fn today() -> chrono::NaiveDate {
    Utc::now().date_naive()
}

fn seed_deal(
    world: &mut DealWorld,
    title: &str,
    customer: CustomerId,
    stage: DealStage,
    closed_days_ago: Option<u64>,
) {
    let now = Utc::now();
    let closed_at = closed_days_ago
        .map(|days| now.checked_sub_days(Days::new(days)).expect("in range"))
        .or_else(|| (!stage.is_open()).then_some(now));
    let id = DealId(generate_uuid_v7());
    let deal = Deal {
        id,
        organization_id: world.organization_id(),
        customer_id: customer,
        title: DealTitle::try_from(title).expect("a titled deal"),
        amount: DealAmount::try_from(100_000).expect("a positive amount"),
        stage,
        expected_close_date: None,
        owner_member_id: None,
        source: None,
        notes: None,
        created_at: now,
        stage_changed_at: closed_at.unwrap_or(now),
        closed_at,
        quote_count: 0,
    };
    repository::state(&world.store).deals.insert(id, deal);
    world.deals.insert(title.to_owned(), id);
}

fn first_customer(world: &DealWorld) -> CustomerId {
    customer_named(world, "Boulangerie Lemoine")
}

#[given("a company registered on Mestier")]
async fn a_registered_company(world: &mut DealWorld) {
    world.organization_id = Some(OrganizationId(generate_uuid_v7()));
}

#[given(expr = "a customer {string} that is a {word}")]
async fn a_customer_with_status(world: &mut DealWorld, name: String, wanted: String) {
    let command = CreateCustomerCommand {
        actor: world.actor(),
        organization_id: world.organization_id(),
        status: customer_status(&wanted),
        name: name.clone(),
        registration_number: None,
        phone: None,
        email: None,
        profile: CustomerProfile::default(),
    };
    let created = world
        .customer_service()
        .create_customer(command)
        .await
        .expect("no infrastructure failure here")
        .expect("a customer without references is accepted");
    world.customers.insert(name, created.id);
}

#[given(expr = "a customer {string} converted {int} days ago")]
async fn a_customer_converted(world: &mut DealWorld, name: String, days: u64) {
    a_customer_with_status(world, name.clone(), "CLIENT".to_owned()).await;
    let id = customer_named(world, &name);
    let converted_at = Utc::now()
        .checked_sub_days(Days::new(days))
        .expect("in range");
    let mut guard = repository::state(&world.store);
    let customer = guard.customers.get_mut(&id).expect("just created");
    customer.converted_at = Some(converted_at);
}

#[given(expr = "a deal {string} in stage {word} for {string}")]
async fn a_deal_in_stage(world: &mut DealWorld, title: String, stage: String, customer: String) {
    let customer = customer_named(world, &customer);
    let stage = DealStage::try_from((stage_name(&stage), None)).expect("not a lost stage");
    seed_deal(world, &title, customer, stage, None);
}

#[given(expr = "a deal {string} lost with the reason {string} for {string}")]
async fn a_lost_deal(world: &mut DealWorld, title: String, reason: String, customer: String) {
    let customer = customer_named(world, &customer);
    let stage = DealStage::Lost(LostReason::try_from(reason).expect("a reason"));
    seed_deal(world, &title, customer, stage, None);
}

#[given(expr = "the deal {string} has a quote")]
async fn the_deal_has_a_quote(world: &mut DealWorld, title: String) {
    let id = deal_named(world, &title);
    let mut guard = repository::state(&world.store);
    guard
        .deals
        .get_mut(&id)
        .expect("the deal exists")
        .quote_count = 1;
}

#[given(expr = "{int} deals won within the last 90 days")]
async fn deals_won_recently(world: &mut DealWorld, count: usize) {
    let customer = first_customer(world);
    for index in 0..count {
        seed_deal(
            world,
            &format!("won {index}"),
            customer,
            DealStage::Won,
            Some(10),
        );
    }
}

#[given(expr = "{int} deals lost within the last 90 days")]
async fn deals_lost_recently(world: &mut DealWorld, count: usize) {
    let customer = first_customer(world);
    for index in 0..count {
        let stage = DealStage::Lost(LostReason::try_from("Pas de réponse").expect("a reason"));
        seed_deal(world, &format!("lost {index}"), customer, stage, Some(10));
    }
}

#[given(expr = "a deal won {int} days ago")]
async fn a_deal_won_long_ago(world: &mut DealWorld, days: u64) {
    let customer = first_customer(world);
    seed_deal(
        world,
        &format!("won {days} days ago"),
        customer,
        DealStage::Won,
        Some(days),
    );
}

#[given(expr = "a deal lost {int} days ago")]
async fn a_deal_lost_long_ago(world: &mut DealWorld, days: u64) {
    let customer = first_customer(world);
    let stage = DealStage::Lost(LostReason::try_from("Pas de réponse").expect("a reason"));
    seed_deal(
        world,
        &format!("lost {days} days ago"),
        customer,
        stage,
        Some(days),
    );
}

#[when(expr = "the artisan creates the deal {string} of {int} cents for {string}")]
async fn the_artisan_creates_a_deal(
    world: &mut DealWorld,
    title: String,
    amount: i64,
    customer: String,
) {
    let customer_id = world
        .customers
        .get(&customer)
        .copied()
        .unwrap_or(CustomerId(generate_uuid_v7()));
    let command = CreateDealCommand {
        actor: world.actor(),
        organization_id: world.organization_id(),
        customer_id,
        title: title.clone(),
        amount_cents: amount,
        expected_close_date: None,
        owner_member_id: None,
        source: None,
        notes: None,
    };
    match world.deals().create_deal(command).await {
        Ok(Ok(deal)) => {
            world.deals.insert(title, deal.id);
        }
        Ok(Err(rule)) => world.refusal = Some(Refusal::Rule(rule)),
        Err(error) => world.refusal = Some(Refusal::Core(error)),
    }
}

async fn move_deal(world: &mut DealWorld, title: &str, stage: &str, reason: Option<String>) {
    let command = MoveDealCommand {
        actor: world.actor(),
        id: deal_named(world, title),
        stage: stage_name(stage),
        lost_reason: reason,
    };
    match world.deals().move_deal(command).await {
        Ok(Ok(_)) => {}
        Ok(Err(rule)) => world.refusal = Some(Refusal::Rule(rule)),
        Err(error) => world.refusal = Some(Refusal::Core(error)),
    }
}

#[when(expr = "the artisan moves the deal {string} to LOST without a reason")]
async fn moves_to_lost_without_a_reason(world: &mut DealWorld, title: String) {
    move_deal(world, &title, "LOST", None).await;
}

#[when(expr = "the artisan moves the deal {string} to LOST with the reason {string}")]
async fn moves_to_lost_with_a_reason(world: &mut DealWorld, title: String, reason: String) {
    move_deal(world, &title, "LOST", Some(reason)).await;
}

#[when(expr = "the artisan moves the deal {string} to {word}")]
async fn moves_to_a_stage(world: &mut DealWorld, title: String, stage: String) {
    move_deal(world, &title, &stage, None).await;
}

#[when(expr = "the artisan deletes the deal {string}")]
async fn the_artisan_deletes(world: &mut DealWorld, title: String) {
    let id = deal_named(world, &title);
    match world.deals().delete_deal(id, world.actor()).await {
        Ok(Ok(())) => {}
        Ok(Err(rule)) => world.refusal = Some(Refusal::Rule(rule)),
        Err(error) => world.refusal = Some(Refusal::Core(error)),
    }
}

#[when(expr = "the artisan archives the customer {string}")]
async fn the_artisan_archives(world: &mut DealWorld, name: String) {
    let current = stored_customer(world, &name);
    let command = UpdateCustomerCommand {
        actor: world.actor(),
        id: current.id,
        status: CustomerStatus::Archived,
        name: current.name,
        registration_number: None,
        phone: current.phone,
        email: current.email,
        profile: current.profile,
    };
    world
        .customer_service()
        .update_customer(command)
        .await
        .expect("no infrastructure failure here")
        .expect("archiving is accepted");
}

#[when("the artisan reads the pipeline summary")]
async fn reads_the_summary(world: &mut DealWorld) {
    let organization_id = world.organization_id();
    world.summary = Some(
        world
            .deals()
            .summary(organization_id, Utc::now())
            .await
            .expect("no infrastructure failure here"),
    );
}

#[then(expr = "the deal {string} is in stage {word}")]
async fn the_deal_is_in_stage(world: &mut DealWorld, title: String, stage: String) {
    assert_eq!(stored_deal(world, &title).stage.name(), stage_name(&stage));
}

#[then(expr = "the deal {string} is open")]
async fn the_deal_is_open(world: &mut DealWorld, title: String) {
    let deal = stored_deal(world, &title);
    assert!(deal.stage.is_open());
    assert_eq!(deal.closed_at, None);
}

#[then(expr = "the deal {string} is worth {int} cents")]
async fn the_deal_is_worth(world: &mut DealWorld, title: String, cents: i64) {
    assert_eq!(stored_deal(world, &title).amount.get(), cents);
}

#[then(expr = "the deal {string} is closed today")]
async fn the_deal_is_closed_today(world: &mut DealWorld, title: String) {
    let deal = stored_deal(world, &title);
    assert_eq!(deal.closed_at.map(|at| at.date_naive()), Some(today()));
}

#[then(expr = "the deal {string} has the lost reason {string}")]
async fn the_deal_has_the_lost_reason(world: &mut DealWorld, title: String, reason: String) {
    let deal = stored_deal(world, &title);
    assert_eq!(
        deal.stage.lost_reason().map(LostReason::as_str),
        Some(reason.as_str())
    );
}

#[then(expr = "the deal {string} has no lost reason")]
async fn the_deal_has_no_lost_reason(world: &mut DealWorld, title: String) {
    assert_eq!(stored_deal(world, &title).stage.lost_reason(), None);
}

#[then(expr = "the deal is refused with {string}")]
async fn the_deal_is_refused(world: &mut DealWorld, expected: String) {
    let refusal = world.refusal.as_ref().expect("the request was refused");
    assert_eq!(refusal.describe(), expected);
}

#[then("no deal is stored")]
async fn no_deal_is_stored(world: &mut DealWorld) {
    assert!(repository::state(&world.store).deals.is_empty());
}

#[then(expr = "the deal {string} still exists")]
async fn the_deal_still_exists(world: &mut DealWorld, title: String) {
    let id = deal_named(world, &title);
    assert!(repository::state(&world.store).deals.contains_key(&id));
}

#[then(expr = "the deal {string} no longer exists")]
async fn the_deal_no_longer_exists(world: &mut DealWorld, title: String) {
    let id = deal_named(world, &title);
    assert!(!repository::state(&world.store).deals.contains_key(&id));
}

#[then(expr = "the customer {string} is a {word}")]
async fn the_customer_is(world: &mut DealWorld, name: String, status: String) {
    assert_eq!(
        stored_customer(world, &name).status,
        customer_status(&status)
    );
}

#[then(expr = "the customer {string} was converted today")]
async fn the_customer_was_converted_today(world: &mut DealWorld, name: String) {
    let converted = stored_customer(world, &name).converted_at;
    assert_eq!(converted.map(|at| at.date_naive()), Some(today()));
}

#[then(expr = "the customer {string} was converted {int} days ago")]
async fn the_customer_was_converted_days_ago(world: &mut DealWorld, name: String, days: u64) {
    let expected = Utc::now()
        .checked_sub_days(Days::new(days))
        .expect("in range")
        .date_naive();
    let converted = stored_customer(world, &name).converted_at;
    assert_eq!(converted.map(|at| at.date_naive()), Some(expected));
}

#[then(expr = "the summary counts {int} won and {int} lost deals")]
async fn the_summary_counts(world: &mut DealWorld, won: i64, lost: i64) {
    let summary = world.summary.as_ref().expect("the summary was read");
    assert_eq!((summary.won_count, summary.lost_count), (won, lost));
}

#[then(expr = "the win rate is {word}")]
async fn the_win_rate_is(world: &mut DealWorld, rate: String) {
    let summary = world.summary.as_ref().expect("the summary was read");
    let expected = (rate != "none").then(|| rate.parse::<u8>().expect("a percentage"));
    assert_eq!(summary.win_rate, expected);
}

#[then(expr = "the open pipeline holds {int} deals")]
async fn the_open_pipeline_holds(world: &mut DealWorld, count: i64) {
    let summary = world.summary.as_ref().expect("the summary was read");
    assert_eq!(summary.open_count, count);
}

#[then(expr = "the summary counts {int} conversions")]
async fn the_summary_counts_conversions(world: &mut DealWorld, count: i64) {
    let summary = world.summary.as_ref().expect("the summary was read");
    assert_eq!(summary.conversions, count);
}
