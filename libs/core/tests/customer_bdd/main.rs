mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::CustomerWorld;

#[tokio::main]
async fn main() {
    CustomerWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features-customer")
        .await;
}
