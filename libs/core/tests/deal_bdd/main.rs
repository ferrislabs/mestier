mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::DealWorld;

#[tokio::main]
async fn main() {
    DealWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features-deal")
        .await;
}
