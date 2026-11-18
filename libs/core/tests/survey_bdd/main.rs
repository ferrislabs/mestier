mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::SurveyWorld;

#[tokio::main]
async fn main() {
    SurveyWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features-survey")
        .await;
}
