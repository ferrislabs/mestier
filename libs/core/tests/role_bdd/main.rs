//! Executable specification for the role/permission lifecycle: creating a
//! role with a chosen subset of permissions, assigning it to a member
//! (possibly several roles at once, whose permissions aggregate by bitwise
//! OR), unassigning it, and the two protections around deletion — a seeded
//! role's fixed name, and the refusal to delete a role still held.
//!
//! Feature files live under `tests/features-role/`, deliberately not under
//! `tests/features/` alongside `quote.feature`: `cucumber::Cucumber::run`
//! glob-walks its target directory recursively for every `*.feature` file
//! regardless of which binary is running, so sharing a directory with
//! `quote_bdd` would make each binary try — and fail, under
//! `fail_on_skipped()` — to match the other suite's steps. `quote_bdd`'s own
//! files are frozen, so the split lives on this side.
//!
//! Run with `cargo test -p mestier-core --test role_bdd`, or plain `cargo
//! test --workspace`. No database needed: both repository ports are
//! `mockall` doubles over an in-memory store.

mod repository;
mod steps;
mod world;

use cucumber::World;

use crate::world::RoleWorld;

#[tokio::main]
async fn main() {
    RoleWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features-role")
        .await;
}
