//! Brings the field API up on a real socket, against a real database.
//!
//! `AppState` comes from `handlers::state`, the same function the binary
//! calls, so the test exercises the production wiring. Only the auth issuer is
//! pointed elsewhere, at a local JWKS server.

use std::{net::SocketAddr, sync::Arc};

use args::Args;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use clap::Parser;
use mestier_core::Permissions;
use sqlx::PgPool;
use uuid::Uuid;

use crate::issuer;

pub struct App {
    pub base_url: String,
    pub token: String,
    pub pool: PgPool,
    pub organization_id: Uuid,
    /// A customer project with a quote: one three-hour task and one expense.
    pub project_id: Uuid,
    /// A project with no customer and no quote — the case the old clocked model
    /// could not express at all.
    pub internal_project_id: Uuid,
    /// A project whose only task sits two months back, so the period must not
    /// report it.
    pub stale_project_id: Uuid,
    /// The hourly-costed person's employee id, for the test that raises their
    /// rate mid-period and checks the already-planned task did not move.
    pub employee_id: Uuid,
    /// A second member of the same organization, holding `VIEW_REPORTS` but
    /// not `VIEW_COST` (#306) — sees the same period, with every money
    /// field redacted.
    pub restricted_token: String,
    /// A third member with membership but no role assignment at all — the
    /// bare "belongs to the organization" case #306's `VIEW_REPORTS` gate
    /// now refuses outright.
    pub no_role_token: String,
    /// A fourth member holding exactly `VIEW_REPORTS | VIEW_COST` — nothing
    /// else, no `MANAGE_ORG`/`MANAGE_MEMBERS`/`MANAGE_ROLES` or any other
    /// bit — proving it is those two bits the redaction check keys off,
    /// not incidentally every bit the way `app.token`'s `Permissions::ALL`
    /// role does (#309).
    pub minimal_token: String,
    user_id: Uuid,
    other_user_id: Uuid,
    restricted_user_id: Uuid,
    no_role_user_id: Uuid,
    minimal_user_id: Uuid,
}

pub async fn start() -> App {
    let issuer_url = issuer::spawn();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set to run the field end-to-end tests");
    let redis_url = std::env::var("RATE_LIMIT_REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_owned());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect to the test database");
    let fixture = seed(&pool).await;

    let args = Arc::new(Args::parse_from(args_for(
        &database_url,
        &redis_url,
        &issuer_url,
    )));
    let state = handlers::state(args)
        .await
        .expect("build AppState for the test");

    let router = handlers_reporting::router(&state).with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the test api");
    let addr = listener.local_addr().expect("read the test api address");
    tokio::spawn(async move {
        // `ConnectInfo` the way the binary supplies it: the rate-limit
        // middleware keys on the peer address and 500s without it.
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });

    App {
        base_url: format!("http://{addr}"),
        token: issuer::mint(&fixture.sub),
        restricted_token: issuer::mint(&fixture.restricted_sub),
        no_role_token: issuer::mint(&fixture.no_role_sub),
        minimal_token: issuer::mint(&fixture.minimal_sub),
        pool,
        organization_id: fixture.organization_id,
        project_id: fixture.project_id,
        internal_project_id: fixture.internal_project_id,
        stale_project_id: fixture.stale_project_id,
        employee_id: fixture.employee_id,
        restricted_user_id: fixture.restricted_user_id,
        no_role_user_id: fixture.no_role_user_id,
        minimal_user_id: fixture.minimal_user_id,
        user_id: fixture.user_id,
        other_user_id: fixture.other_user_id,
    }
}

impl App {
    pub fn url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v1/organizations/{}/reporting{suffix}",
            self.base_url, self.organization_id
        )
    }

    /// Removes the fixture, child rows first.
    ///
    /// Explicit and ordered rather than relying on cascades: `employees`,
    /// `automation.event` and others reference `organizations` with a plain
    /// foreign key, so a single delete is refused. Errors are raised, not
    /// swallowed: a cleanup that fails quietly is how a shared development
    /// database fills up with fixtures, which is what this suite found.
    pub async fn cleanup(&self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM time_entries WHERE org_id = $1",
            "DELETE FROM day_logs WHERE org_id = $1",
            "DELETE FROM tasks WHERE org_id = $1",
            "DELETE FROM supplier_invoice_line_allocations WHERE org_id = $1",
            "DELETE FROM supplier_invoice_lines WHERE org_id = $1",
            "DELETE FROM supplier_invoices WHERE org_id = $1",
            "DELETE FROM projects WHERE org_id = $1",
            "DELETE FROM quotes WHERE org_id = $1",
            "DELETE FROM employee_cost_bases WHERE org_id = $1",
            "DELETE FROM employees WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
            "DELETE FROM customers WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            sqlx::query(statement)
                .bind(self.organization_id)
                .execute(&self.pool)
                .await
                .unwrap_or_else(|e| panic!("cleanup failed on `{statement}`: {e}"));
        }

        for user_id in [
            self.user_id,
            self.other_user_id,
            self.restricted_user_id,
            self.no_role_user_id,
            self.minimal_user_id,
        ] {
            sqlx::query("DELETE FROM users WHERE id = $1")
                .bind(user_id)
                .execute(&self.pool)
                .await
                .expect("clear the fixture user");
        }
    }
}

struct Fixture {
    sub: String,
    user_id: Uuid,
    other_user_id: Uuid,
    restricted_sub: String,
    restricted_user_id: Uuid,
    no_role_sub: String,
    no_role_user_id: Uuid,
    minimal_sub: String,
    minimal_user_id: Uuid,
    organization_id: Uuid,
    project_id: Uuid,
    internal_project_id: Uuid,
    stale_project_id: Uuid,
    employee_id: Uuid,
}

/// Three projects and two people, with nothing clocked anywhere.
///
/// Anchored on yesterday at fixed hours rather than on `Utc::now()` plus an
/// offset: the cost now comes from the task's own window, so a fixture running
/// near midnight would have its window clipped by the period and the numbers
/// below would stop being stateable.
async fn seed(pool: &PgPool) -> Fixture {
    let organization_id = Uuid::now_v7();
    let (user_id, sub, member_id, employee_id) =
        seed_person(pool, organization_id, true, CostBasis::Hourly).await;
    // Salaried, on the exact shape that read as 0,00 €: 3 500 € a month on a
    // 35 h contract, which is 23,08 € an hour.
    let (other_user_id, _, other_member_id, _) =
        seed_person(pool, organization_id, false, CostBasis::Salaried).await;

    // #306: the caller needs `VIEW_REPORTS` to reach either report at all,
    // and `VIEW_COST` on top of it to read the money rather than a redacted
    // shape. `Permissions::ALL` for the fixture's main caller (`app.token`)
    // keeps every existing assertion in this suite unchanged; a second,
    // deliberately narrower role is what the redaction test below reads
    // from `app.restricted_token`.
    let owner_role_id = seed_role(pool, organization_id, "test-owner", Permissions::ALL.0).await;
    assign_role(pool, member_id, owner_role_id).await;

    let restricted_role_id = seed_role(
        pool,
        organization_id,
        "test-restricted",
        Permissions::VIEW_REPORTS.0,
    )
    .await;
    let (restricted_user_id, restricted_sub, restricted_member_id, _) =
        seed_person(pool, organization_id, false, CostBasis::Hourly).await;
    assign_role(pool, restricted_member_id, restricted_role_id).await;

    // Membership, no role assignment at all — the bare case `VIEW_REPORTS`
    // now refuses outright.
    let (no_role_user_id, no_role_sub, _, _) =
        seed_person(pool, organization_id, false, CostBasis::Hourly).await;

    // #309: a fourth caller holding exactly `VIEW_REPORTS | VIEW_COST` and
    // nothing else — no `MANAGE_ORG`, no `MANAGE_MEMBERS`, no `MANAGE_ROLES`,
    // no chat bits. `app.token`'s `Permissions::ALL` role proves every bit
    // together reads real money; this proves the minimal pair alone does
    // too, which is what the issue actually asks for.
    let minimal_role_id = seed_role(
        pool,
        organization_id,
        "test-minimal",
        (Permissions::VIEW_REPORTS | Permissions::VIEW_COST).0,
    )
    .await;
    let (minimal_user_id, minimal_sub, minimal_member_id, _) =
        seed_person(pool, organization_id, false, CostBasis::Hourly).await;
    assign_role(pool, minimal_member_id, minimal_role_id).await;

    let customer_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(organization_id)
        .bind("Duval Masonry")
        .execute(pool)
        .await
        .expect("seed the customer");

    let yesterday = (Utc::now() - Duration::days(1))
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .expect("09:00 exists")
        .and_utc();

    let quote_id = seed_quote(pool, organization_id, customer_id).await;
    let project_id = seed_project(
        pool,
        organization_id,
        "Entretien annuel",
        Some(customer_id),
        Some(quote_id),
    )
    .await;
    let internal_project_id = seed_project(pool, organization_id, "Vie interne", None, None).await;
    let stale_project_id = seed_project(
        pool,
        organization_id,
        "Chantier terminé",
        Some(customer_id),
        None,
    )
    .await;

    // Three hours for one person, plus 45 euros of travel on the same task.
    seed_task(
        pool,
        organization_id,
        project_id,
        &[member_id],
        yesterday,
        yesterday + Duration::hours(3),
        "Taille de haie",
        4_500,
        Some("Déplacement Clermont"),
    )
    .await;

    // Two hours with two people on it: the six-person-hour meeting the clocked
    // model reported as nothing at all.
    seed_task(
        pool,
        organization_id,
        internal_project_id,
        &[member_id, other_member_id],
        yesterday + Duration::hours(5),
        yesterday + Duration::hours(7),
        "Réunion hebdo",
        0,
        None,
    )
    .await;

    // Outside the period entirely.
    seed_task(
        pool,
        organization_id,
        stale_project_id,
        &[member_id],
        yesterday - Duration::days(60),
        yesterday - Duration::days(60) + Duration::hours(4),
        "Chantier de printemps",
        0,
        None,
    )
    .await;

    // Nine hours clocked, to prove the report ignores them. Nine rather than a
    // number the plan also produces: a coincidence would make the assertion
    // pass for the wrong reason.
    seed_stray_time_entry(pool, organization_id, project_id, employee_id, yesterday).await;

    // A confirmed supplier invoice, its one line fully allocated to the
    // customer project — #338. The organization here carries no VAT status
    // (`NULL`, the same as a franchise), so the real cost is the grossed-up
    // figure, not the net one: 200,00 € net at 20 % is 240,00 €.
    seed_supplier_cost(pool, organization_id, project_id, yesterday.date_naive()).await;

    Fixture {
        sub,
        user_id,
        other_user_id,
        restricted_sub,
        restricted_user_id,
        no_role_sub,
        no_role_user_id,
        minimal_sub,
        minimal_user_id,
        organization_id,
        project_id,
        internal_project_id,
        stale_project_id,
        employee_id,
    }
}

/// A role carrying exactly the given bits — #306's redaction test needs one
/// with `VIEW_REPORTS` and not `VIEW_COST`, distinct from the fixture's
/// main caller, who gets `Permissions::ALL`.
async fn seed_role(pool: &PgPool, organization_id: Uuid, name: &str, permissions: i64) -> Uuid {
    let role_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id)
    .bind(name)
    .bind(permissions)
    .execute(pool)
    .await
    .expect("seed the role");

    role_id
}

async fn assign_role(pool: &PgPool, member_id: Uuid, role_id: Uuid) {
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(Uuid::now_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .expect("assign the role");
}

/// A user, their organization seat, and the employee profile that carries the
/// hourly rate. The field routes refuse a member without that profile, so all
/// three rows are needed for the caller to get past authorization.
/// How the seeded person is costed. Both bases are exercised because the report
/// has to agree with itself across them.
enum CostBasis {
    Hourly,
    Salaried,
}

async fn seed_person(
    pool: &PgPool,
    organization_id: Uuid,
    owns_organization: bool,
    cost_basis: CostBasis,
) -> (Uuid, String, Uuid, Uuid) {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-field-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("worker-{user_id}@example.com"))
    .bind(format!("worker-{user_id}"))
    .bind("Field Worker")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the user");

    if owns_organization {
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id)
            .bind("Test Org")
            .bind(format!("test-org-{organization_id}"))
            .bind(user_id)
            .execute(pool)
            .await
            .expect("seed the organization");
    }

    let member_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id)
    .bind(user_id)
    .bind("Worker")
    .execute(pool)
    .await
    .expect("seed the membership");

    let employee_id = Uuid::now_v7();
    let (hourly_rate_cents, is_salaried, monthly_cost_cents) = match cost_basis {
        CostBasis::Hourly => (Some(3_500_i32), false, None),
        CostBasis::Salaried => (None, true, Some(350_000_i32)),
    };
    sqlx::query(
        "INSERT INTO employees (id, org_id, member_id, hourly_rate_cents, is_salaried, monthly_cost_cents, weekly_contract_minutes)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(employee_id)
    .bind(organization_id)
    .bind(member_id)
    .bind(hourly_rate_cents)
    .bind(is_salaried)
    .bind(monthly_cost_cents)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the employee profile");

    // Profitability now costs from `employee_cost_bases`, not from these
    // columns directly (#301) — a profile with no version at all would read
    // as if nobody had entered a rate. Far enough in the past to cover every
    // task this fixture plants, including the one two months back.
    seed_cost_basis(
        pool,
        organization_id,
        employee_id,
        NaiveDate::from_ymd_opt(2020, 1, 1).expect("a date"),
        None,
        hourly_rate_cents,
        is_salaried,
        monthly_cost_cents,
    )
    .await;

    (user_id, sub, member_id, employee_id)
}

#[allow(clippy::too_many_arguments)]
async fn seed_cost_basis(
    pool: &PgPool,
    organization_id: Uuid,
    employee_id: Uuid,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    hourly_rate_cents: Option<i32>,
    is_salaried: bool,
    monthly_cost_cents: Option<i32>,
) {
    sqlx::query(
        "INSERT INTO employee_cost_bases (id, org_id, employee_id, effective_from, effective_to, hourly_rate_cents, is_salaried, monthly_cost_cents, weekly_contract_minutes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(employee_id)
    .bind(effective_from)
    .bind(effective_to)
    .bind(hourly_rate_cents)
    .bind(is_salaried)
    .bind(monthly_cost_cents)
    .bind(2100)
    .execute(pool)
    .await
    .expect("seed the cost basis version");
}

/// Closes the employee's open cost basis version today and opens a new one
/// at `hourly_rate_cents` — a raise, dated the way the application layer
/// dates one. Used by the test proving a raise entered after a task was
/// planned does not change what that task already cost.
pub async fn raise(
    pool: &PgPool,
    organization_id: Uuid,
    employee_id: Uuid,
    hourly_rate_cents: i32,
) {
    let today = Utc::now().date_naive();

    sqlx::query(
        "UPDATE employee_cost_bases SET effective_to = $1 WHERE employee_id = $2 AND effective_to IS NULL",
    )
    .bind(today)
    .bind(employee_id)
    .execute(pool)
    .await
    .expect("close the open cost basis version");

    seed_cost_basis(
        pool,
        organization_id,
        employee_id,
        today,
        None,
        Some(hourly_rate_cents),
        false,
        None,
    )
    .await;
}

async fn seed_quote(pool: &PgPool, organization_id: Uuid, customer_id: Uuid) -> Uuid {
    let context_id = Uuid::now_v7();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)")
        .bind(context_id)
        .bind(customer_id)
        .bind("Chantier principal")
        .execute(pool)
        .await
        .expect("seed the customer context");

    let quote_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, reference, title, status, net_cents, gross_cents)
         VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::quote_status, $8, $8)",
    )
    .bind(quote_id)
    .bind(organization_id)
    .bind(customer_id)
    .bind(context_id)
    .bind(format!("DEV-TEST-{quote_id}"))
    .bind("Entretien annuel")
    .bind("ACCEPTED")
    .bind(420_000_i32)
    .execute(pool)
    .await
    .expect("seed the quote");

    quote_id
}

async fn seed_project(
    pool: &PgPool,
    organization_id: Uuid,
    name: &str,
    customer_id: Option<Uuid>,
    quote_id: Option<Uuid>,
) -> Uuid {
    let project_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO projects (id, org_id, name, customer_id, quote_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(project_id)
    .bind(organization_id)
    .bind(name)
    .bind(customer_id)
    .bind(quote_id)
    .execute(pool)
    .await
    .expect("seed the project");

    project_id
}

/// A confirmed supplier invoice with one line, fully allocated to
/// `project_id` — #338. 200,00 € net at 20 % VAT, so the fixture states two
/// figures depending on the organization's own VAT status: 20 000 net, or
/// 24 000 grossed up when (as here) it cannot recover the VAT.
async fn seed_supplier_cost(
    pool: &PgPool,
    organization_id: Uuid,
    project_id: Uuid,
    issued_on: NaiveDate,
) {
    let supplier_invoice_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO supplier_invoices (
            id, org_id, supplier_name, number, issued_on, source, status,
            currency, net_cents, gross_cents
         ) VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::supplier_invoice_status, $8, $9, $10)",
    )
    .bind(supplier_invoice_id)
    .bind(organization_id)
    .bind("Point P")
    .bind(format!("F-{supplier_invoice_id}"))
    .bind(issued_on)
    .bind("MANUAL")
    .bind("CONFIRMED")
    .bind("EUR")
    .bind(20_000_i32)
    .bind(24_000_i32)
    .execute(pool)
    .await
    .expect("seed the supplier invoice");

    let line_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO supplier_invoice_lines (
            id, org_id, supplier_invoice_id, label, quantity, unit_price_cents,
            line_total_cents, vat_rate_basis_points, position
         ) VALUES ($1, $2, $3, $4, $5::numeric, $6, $7, $8, $9)",
    )
    .bind(line_id)
    .bind(organization_id)
    .bind(supplier_invoice_id)
    .bind("Plaques de plâtre")
    .bind(1_i32)
    .bind(20_000_i32)
    .bind(20_000_i32)
    .bind(2_000_i32)
    .bind(0_i32)
    .execute(pool)
    .await
    .expect("seed the supplier invoice line");

    sqlx::query(
        "INSERT INTO supplier_invoice_line_allocations (
            id, org_id, supplier_invoice_line_id, project_id, amount_cents
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(line_id)
    .bind(project_id)
    .bind(20_000_i32)
    .execute(pool)
    .await
    .expect("seed the supplier invoice line allocation");
}

/// Time clocked on the task, which nothing reads for money any more.
async fn seed_stray_time_entry(
    pool: &PgPool,
    organization_id: Uuid,
    task_id: Uuid,
    employee_id: Uuid,
    anchor: DateTime<Utc>,
) {
    let task_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM tasks WHERE org_id = $1 AND project_id = $2 LIMIT 1")
            .bind(organization_id)
            .bind(task_id)
            .fetch_optional(pool)
            .await
            .expect("look up the task to clock against");

    let Some(task_id) = task_id else {
        return;
    };

    sqlx::query(
        "INSERT INTO time_entries (id, org_id, task_id, employee_id, started_at, ended_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(organization_id)
    .bind(task_id)
    .bind(employee_id)
    .bind(anchor)
    .bind(anchor + Duration::hours(9))
    .execute(pool)
    .await
    .expect("seed the stray time entry");
}

#[allow(clippy::too_many_arguments)]
async fn seed_task(
    pool: &PgPool,
    organization_id: Uuid,
    project_id: Uuid,
    member_ids: &[Uuid],
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    title: &str,
    expenses_cents: i32,
    expenses_label: Option<&str>,
) -> Uuid {
    let task_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO tasks (id, org_id, project_id, starts_at, ends_at, all_day, status, title, expenses_cents, expenses_label)
         VALUES ($1, $2, $3, $4, $5, false, CAST($6 AS text)::task_status, $7, $8, $9)",
    )
    .bind(task_id)
    .bind(organization_id)
    .bind(project_id)
    .bind(starts_at)
    .bind(ends_at)
    .bind("PLANNED")
    .bind(title)
    .bind(expenses_cents)
    .bind(expenses_label)
    .execute(pool)
    .await
    .expect("seed the task");

    for member_id in member_ids {
        sqlx::query(
            "INSERT INTO task_assignments (id, org_id, task_id, member_id) VALUES ($1, $2, $3, $4)",
        )
        .bind(Uuid::now_v7())
        .bind(organization_id)
        .bind(task_id)
        .bind(member_id)
        .execute(pool)
        .await
        .expect("seed the assignment");
    }

    task_id
}

fn args_for(database_url: &str, redis_url: &str, issuer_url: &str) -> Vec<String> {
    let db = url::Url::parse(database_url).expect("DATABASE_URL is a url");

    vec![
        "api".to_owned(),
        "--database-host".to_owned(),
        db.host_str().unwrap_or("localhost").to_owned(),
        "--database-port".to_owned(),
        db.port().unwrap_or(5432).to_string(),
        "--database-user".to_owned(),
        db.username().to_owned(),
        "--database-password".to_owned(),
        db.password().unwrap_or_default().to_owned(),
        "--database-name".to_owned(),
        db.path().trim_start_matches('/').to_owned(),
        "--rate-limit-redis-url".to_owned(),
        redis_url.to_owned(),
        // The rate limiter keys on client IP alone, and every test in this
        // suite calls in from the same loopback address through the same
        // Redis — so the sliding window is shared across every test in a
        // run, and across a run and the one before it if run twice inside
        // the same window. The production default of 120/minute is a
        // limit on one real caller, not on an entire suite's worth of
        // fixtures; a value that low turned a second consecutive run of a
        // clean suite into a false failure.
        "--rate-limit-per-minute".to_owned(),
        "100000".to_owned(),
        "--auth-issuer".to_owned(),
        issuer_url.to_owned(),
        // None of these suites touch object storage, but `create_service`
        // creates the bucket at startup by default, which turned a reachable S3
        // into a prerequisite for running them at all. Saying no here drops
        // `rustfs` from the list — and is what lets them run in CI against two
        // service containers instead of three.
        "--file-storage-auto-create-bucket".to_owned(),
        "false".to_owned(),
    ]
}
