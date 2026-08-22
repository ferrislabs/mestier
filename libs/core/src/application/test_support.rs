//! Shared helpers for the live-postgres integration tests under `application/`.
//!
//! One reason this exists: every fixture used to end with a `cleanup` function
//! whose every statement swallowed its error with `.ok()`. A cleanup that fails
//! quietly is how a shared development database fills up with fixtures — 5 128
//! stray organizations and 10 128 stray customers, when this was written, in a
//! 28 MB database whose real content is one organization. Worse, the residue
//! then breaks other suites: the automation engine claims due runs system-wide,
//! so a backlog of abandoned runs makes its tests fail depending on what was
//! left behind.
//!
//! So a failed cleanup panics here, loudly, naming the statement. A test that
//! cannot tidy up after itself has found a real problem, not a nuisance.

use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// Runs one cleanup statement bound to a single id, and panics if it fails.
///
/// Statements are passed as text rather than through `query!` on purpose: the
/// nine fixtures that call this each delete a different set of tables in their
/// own order, and a macro cannot take that list as a parameter. The trade is a
/// table name checked at run time instead of compile time — acceptable now that
/// a mistake fails the test on the next run instead of being swallowed.
pub async fn purge(pool: &PgPool, statement: &str, id: Uuid) {
    sqlx::query(statement)
        .bind(id)
        .execute(pool)
        .await
        .unwrap_or_else(|error| panic!("cleanup failed on `{statement}`: {error}"));
}

/// Splits `DATABASE_URL` into its server prefix and its database name.
pub fn split_database_url(url: &str) -> (String, String) {
    let idx = url
        .rfind('/')
        .expect("DATABASE_URL must contain a `/` separating the server from the database name");

    (url[..idx].to_owned(), url[idx + 1..].to_owned())
}

/// A database of its own for a suite that cannot share one.
///
/// Created on first use, migrated from the embedded migrations — no `sqlx` CLI
/// on `PATH` required, unlike the older scratch-database test in
/// `application/task/tests.rs`.
///
/// There is no suite-teardown hook in Rust, so the database outlives the run.
/// Instead, every startup drops the ones earlier runs left behind, which bounds
/// the leak to a single database rather than one per run. That also means two
/// concurrent test processes would fight over it: these suites are serialised
/// (`--test-threads=1`) and CI is ephemeral, so the trade is worth the
/// simplicity.
pub async fn scratch_pool(prefix: &'static str, cell: &'static OnceCell<String>) -> PgPool {
    let url = cell
        .get_or_init(|| async {
            let url = std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set to run the integration tests");
            let (base_url, _) = split_database_url(&url);
            let admin = PgPool::connect(&format!("{base_url}/postgres"))
                .await
                .expect("connecting to the `postgres` database must succeed");

            let stale: Vec<String> =
                sqlx::query_scalar("SELECT datname FROM pg_database WHERE datname LIKE $1")
                    .bind(format!("{prefix}%"))
                    .fetch_all(&admin)
                    .await
                    .expect("listing stale scratch databases must succeed");
            for name in stale {
                // `FORCE` because a previous run may have been killed with its
                // connections still open, and a stale database nobody can drop is
                // exactly the leak this is here to prevent.
                sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#))
                    .execute(&admin)
                    .await
                    .unwrap_or_else(|error| panic!("dropping stale `{name}` failed: {error}"));
            }

            let name = format!("{prefix}{}", Uuid::new_v4().simple());
            sqlx::query(&format!(r#"CREATE DATABASE "{name}""#))
                .execute(&admin)
                .await
                .unwrap_or_else(|error| panic!("creating `{name}` failed: {error}"));

            let scratch_url = format!("{base_url}/{name}");
            let pool = PgPool::connect(&scratch_url)
                .await
                .unwrap_or_else(|error| panic!("connecting to `{name}` failed: {error}"));
            sqlx::migrate!("../../migrations")
                .run(&pool)
                .await
                .unwrap_or_else(|error| panic!("migrating `{name}` failed: {error}"));
            pool.close().await;

            scratch_url
        })
        .await;

    // A pool per caller, against the shared scratch database. One pool shared by
    // the whole suite runs out of connections part-way through — the tests used
    // to open their own against the development database, and only *which*
    // database changes here.
    PgPool::connect(url)
        .await
        .unwrap_or_else(|error| panic!("connecting to the scratch database failed: {error}"))
}

/// The automation aggregate's own database, shared by its five test modules.
///
/// One database for the whole aggregate rather than one per module: they are
/// serialised anyway, and five databases would be five things to prune. What
/// matters is that no worker outside this process claims their runs — see
/// [`scratch_pool`].
static AUTOMATION_SCRATCH: OnceCell<String> = OnceCell::const_new();

pub async fn automation_pool() -> PgPool {
    let pool = scratch_pool("mestier_automation_test_", &AUTOMATION_SCRATCH).await;

    // Every automation test starts on an empty queue.
    //
    // `run_engine_pass` claims every *due* run in the database, not just the one
    // the test just started, so runs left behind by earlier tests — a failed one
    // waiting on its backoff, a deliberately unfinished one — get claimed instead
    // and the test's own run is still `pending` when it asserts. `RUN_CLAIM_LOCK`
    // serialises the tests against each other; it does not empty the queue
    // between them.
    //
    // Wiping it here rather than cleaning up in each of the twenty tests puts the
    // invariant in one place, and this database belongs to no one else.
    for statement in [
        "DELETE FROM automation.run_step",
        "DELETE FROM automation.run",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .unwrap_or_else(|error| panic!("clearing the run queue failed: {error}"));
    }

    pool
}
