mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::BillingWorld;

#[tokio::main]
async fn main() {
    BillingWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features-billing")
        .await;
}
