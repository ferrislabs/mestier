//! Executable specification for the quote lifecycle.
//!
//! The scenarios in `features/` state what a quote does in plain sentences,
//! for readers who arbitrate a business rule without reading Rust. Everything
//! below is the translation layer between those sentences and the domain.
//!
//! Run with `cargo test -p mestier-core --test quote_bdd`, or plain
//! `cargo test --workspace`. No database needed: the repository port is a
//! `mockall` double over an in-memory store.

mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::QuoteWorld;

#[tokio::main]
async fn main() {
    QuoteWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
