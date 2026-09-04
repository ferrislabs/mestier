use authz::Subject;
use chrono::{Duration, Utc};
use common::{CoreError, UserId};
use iam::{
    IamCreateUser, IamProvider,
    infrastructure::ferriskey::{FerriskeyConfig, FerriskeyIamProvider},
};
use mestier_core::{
    AssigneeRef, CreateCustomerCommand, CreateCustomerContextCommand, CreateOrganizationCommand,
    CreateProjectCommand, CreateQuoteCommand, CreateTaskCommand, CreateUserCommand,
    CustomerPipelineStage, CustomerStatus, EventHub, MestierUseCase, PatchTaskCommand,
    QuoteLineCommand, ServiceRateUnit, default_authorizer,
};
use rust_decimal::Decimal;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{config::SeedArgs, error::SeedError};

/// Seeds a demo organization plus a handful of realistic records (customers,
/// a project, a few planning tasks, a quote) so a reviewer opening a preview
/// URL sees a populated app.
///
/// Idempotent: if an organization named `args.org_slug` already exists this
/// returns `Ok(())` immediately without touching anything else. Called on
/// every ArgoCD sync of a preview `Application`, so this check has to run
/// before any other side effect (creating a FerrisKey user included).
pub async fn run(args: &SeedArgs) -> Result<(), SeedError> {
    let pool = PgPoolOptions::new().connect(&db_url(args)).await?;

    // The idempotency gate. A plain runtime query (not `query!`) so this
    // compiles without a live database or `sqlx prepare` — see CLAUDE.md on
    // offline builds. Checked before any other side effect: a Job that runs
    // on every sync must not mint a fresh FerrisKey user each time it finds
    // its own organization already sitting there.
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM organizations WHERE slug = $1 AND deleted_at IS NULL")
            .bind(&args.org_slug)
            .fetch_optional(&pool)
            .await?;

    if existing.is_some() {
        info!(org_slug = %args.org_slug, "demo organization already exists, nothing to do");
        return Ok(());
    }

    // Minimal composition: just the pool and the in-process policy engine —
    // no Redis, no S3, no automation worker. Those back `create_service`'s
    // rate limiter, file storage and webhook engine, none of which any use
    // case called below touches.
    let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());

    let owner = provision_owner(args, &usecase).await?;

    let org = match usecase
        .acting_as(owner.local_id)
        .create_organization(CreateOrganizationCommand {
            name: args.org_name.clone(),
            slug: args.org_slug.clone(),
            owner_id: owner.sub_as_user_id,
        })
        .await
    {
        Ok(org) => org,
        // Race with another concurrent run against the same slug (unlikely
        // for a PostSync hook, which ArgoCD runs to completion before the
        // next sync, but cheap to make safe): same outcome as the upfront
        // check finding it, so the same no-op.
        Err(CoreError::Conflict(reason)) => {
            warn!(
                org_slug = %args.org_slug,
                reason,
                "organization appeared concurrently, nothing more to do"
            );
            return Ok(());
        }
        Err(err) => return Err(err.into()),
    };

    info!(org_id = %org.id.0, org_slug = %org.slug, "demo organization created");

    // Every call from here attributes its event to the demo owner rather
    // than the system actor `MestierUseCase::new` defaults to.
    let usecase = usecase.acting_as(owner.local_id);
    let actor = Subject::system();

    seed_demo_data(&usecase, actor, org.id, owner.local_id).await?;

    info!(org_slug = %args.org_slug, "demo data seeded");
    Ok(())
}

/// Same assembly `create_service` uses to build its own connection string —
/// see `libs/core/src/application/mod.rs` — kept identical so the two
/// binaries connect to the same database the same way.
fn db_url(args: &SeedArgs) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        args.db.user, args.db.password, args.db.host, args.db.port, args.db.name,
    )
}

/// The demo owner's deterministic identity, derived from the org slug alone:
/// email and FerrisKey username, plus a display name from the org's own.
/// Deterministic on purpose — a second run against the same slug (the
/// idempotency key) must look the owner up under the exact same identity
/// rather than minting a new one.
fn owner_identity(org_slug: &str, org_name: &str) -> (String, String, String) {
    (
        format!("demo-owner+{org_slug}@mestier.preview"),
        format!("demo-owner-{org_slug}"),
        format!("{org_name} (Demo)"),
    )
}

struct Owner {
    /// The local `users.id` — what `MestierUseCase::acting_as` and every
    /// event's `actor_id` need.
    local_id: UserId,
    /// The FerrisKey `sub`, reinterpreted as a `UserId` the way
    /// `IdentityExt::user_id` does on the real request path.
    /// `CreateOrganizationCommand::owner_id` is deliberately this value, not
    /// `local_id` — see `MestierUseCase::create_organization`'s own doc.
    sub_as_user_id: UserId,
}

/// Ensures a FerrisKey user and a matching local `users` row exist for the
/// demo organization's owner, creating whichever is missing.
///
/// Both lookups are by a slug-derived deterministic email/sub, so a retry
/// after a partial failure (FerrisKey user created, local row not yet)
/// reuses what is already there instead of erroring on a duplicate.
async fn provision_owner(args: &SeedArgs, usecase: &MestierUseCase) -> Result<Owner, SeedError> {
    let (email, username, display_name) = owner_identity(&args.org_slug, &args.org_name);

    let iam = FerriskeyIamProvider::new(FerriskeyConfig::new(
        args.auth.issuer.clone(),
        args.auth.client_id.clone(),
        args.auth.client_secret.clone(),
    ));

    let iam_user = match iam.find_user_by_email(&email).await? {
        Some(user) => user,
        None => {
            iam.create_user(IamCreateUser {
                email: email.clone(),
                username: username.clone(),
                name: Some(display_name.clone()),
                send_invite_email: false,
            })
            .await?
        }
    };
    let sub = iam_user.id.0;

    let sub_as_user_id = sub
        .parse::<Uuid>()
        .map(UserId)
        .map_err(|_| SeedError::NonUuidSubject(sub.clone()))?;

    let local_user = match usecase.find_user_by_sub(&sub).await? {
        Some(user) => user,
        None => {
            usecase
                .create_user(CreateUserCommand {
                    name: display_name,
                    username,
                    email,
                    sub,
                })
                .await?
        }
    };

    Ok(Owner {
        local_id: local_user.id,
        sub_as_user_id,
    })
}

/// A handful of realistic records exercising planning, quotes and
/// customers — not exhaustive fixture data. Every use case below already
/// exists in `libs/core`; nothing new is added to reach this.
async fn seed_demo_data(
    usecase: &MestierUseCase,
    actor: Subject,
    organization_id: common::OrganizationId,
    owner_local_id: UserId,
) -> Result<(), SeedError> {
    let customer = usecase
        .create_customer(CreateCustomerCommand {
            actor: actor.clone(),
            organization_id,
            status: CustomerStatus::Client,
            pipeline_stage: CustomerPipelineStage::Won,
            name: "Menuiserie Bertrand".to_owned(),
            registration_number: None,
            phone: Some("+33 6 12 34 56 78".to_owned()),
            email: Some("contact@menuiserie-bertrand.example".to_owned()),
        })
        .await?;

    usecase
        .create_customer(CreateCustomerCommand {
            actor: actor.clone(),
            organization_id,
            status: CustomerStatus::Prospect,
            pipeline_stage: CustomerPipelineStage::QuoteSent,
            name: "Rénovation Lefèvre".to_owned(),
            registration_number: None,
            phone: Some("+33 6 98 76 54 32".to_owned()),
            email: Some("m.lefevre@example.com".to_owned()),
        })
        .await?;

    usecase
        .create_customer(CreateCustomerCommand {
            actor: actor.clone(),
            organization_id,
            status: CustomerStatus::Prospect,
            pipeline_stage: CustomerPipelineStage::New,
            name: "Toiture Morel".to_owned(),
            registration_number: None,
            phone: None,
            email: Some("j.morel@example.com".to_owned()),
        })
        .await?;

    let context = usecase
        .create_customer_context(CreateCustomerContextCommand {
            actor: actor.clone(),
            customer_id: customer.id,
            label: "Chantier principal".to_owned(),
            address_line: Some("14 rue des Artisans".to_owned()),
            postal_code: Some("69003".to_owned()),
            city: Some("Lyon".to_owned()),
            photo_key: None,
        })
        .await?;

    let project = usecase
        .create_project(CreateProjectCommand {
            actor: actor.clone(),
            organization_id,
            name: "Rénovation cuisine — Bertrand".to_owned(),
            customer_id: Some(customer.id),
            customer_context_id: Some(context.id),
            quote_id: None,
        })
        .await?;

    let labels = usecase.list_task_labels(organization_id).await?;
    let label_id = |name: &str| labels.iter().find(|l| l.name == name).map(|l| l.id);

    let now = Utc::now();

    let installation = usecase
        .create_task(CreateTaskCommand {
            actor: actor.clone(),
            organization_id,
            parent_task_id: None,
            title: "Pose de la cuisine".to_owned(),
            description: Some("Installation des meubles et plan de travail".to_owned()),
            starts_at: Some(now + Duration::days(1)),
            ends_at: Some(now + Duration::days(1) + Duration::hours(7)),
            all_day: false,
            blocks_availability: true,
            customer_id: Some(customer.id),
            customer_context_id: Some(context.id),
            quote_id: None,
            project_id: Some(project.id),
            expenses_cents: 0,
            expenses_label: None,
        })
        .await?;

    let site_meeting = usecase
        .create_task(CreateTaskCommand {
            actor: actor.clone(),
            organization_id,
            parent_task_id: None,
            title: "Réunion de chantier".to_owned(),
            description: Some("Point d'avancement avec le client".to_owned()),
            starts_at: Some(now + Duration::days(2)),
            ends_at: Some(now + Duration::days(2) + Duration::hours(1)),
            all_day: false,
            blocks_availability: true,
            customer_id: Some(customer.id),
            customer_context_id: Some(context.id),
            quote_id: None,
            project_id: Some(project.id),
            expenses_cents: 0,
            expenses_label: None,
        })
        .await?;

    let supplier_trip = usecase
        .create_task(CreateTaskCommand {
            actor: actor.clone(),
            organization_id,
            parent_task_id: None,
            title: "Déplacement fournisseur".to_owned(),
            description: Some("Récupération du plan de travail sur mesure".to_owned()),
            starts_at: Some(now + Duration::days(3)),
            ends_at: Some(now + Duration::days(3) + Duration::hours(2)),
            all_day: false,
            blocks_availability: false,
            customer_id: None,
            customer_context_id: None,
            quote_id: None,
            project_id: None,
            expenses_cents: 4500,
            expenses_label: Some("Carburant".to_owned()),
        })
        .await?;

    // Owner as assignee on every task — a solo-artisan demo org has no one
    // else to assign — plus the matching preset label where one fits. Both
    // exercise the calendar's assignee-avatar and label rendering rather
    // than leaving every task bare.
    let (members, _) = usecase
        .list_members(actor.clone(), organization_id, 10, 0)
        .await?;
    let owner_member_id = members
        .into_iter()
        .find(|m| m.account.as_ref().is_some_and(|u| u.id == owner_local_id))
        .map(|m| m.member.id);

    if let Some(member_id) = owner_member_id {
        for (task_id, label) in [
            (installation.id, None),
            (site_meeting.id, label_id("Réunion")),
            (supplier_trip.id, label_id("Déplacement")),
        ] {
            let mut patch = PatchTaskCommand::new(task_id, actor.clone());
            patch.assignees = Some(vec![AssigneeRef(member_id)]);
            patch.label_ids = label.map(|id| vec![id]);
            usecase.patch_task(patch).await?;
        }
    }

    usecase
        .create_quote(
            CreateQuoteCommand {
                organization_id,
                title: "Devis rénovation cuisine".to_owned(),
                customer_id: customer.id,
                customer_context_id: context.id,
                lines: vec![
                    QuoteLineCommand {
                        service_rate_id: None,
                        label: "Fourniture et pose des meubles de cuisine".to_owned(),
                        quantity: Decimal::from(1),
                        unit: ServiceRateUnit::FlatRate,
                        unit_price_cents: 850_000,
                        vat_rate_bp: Some(2000),
                        notes: None,
                        photo_keys: Vec::new(),
                    },
                    QuoteLineCommand {
                        service_rate_id: None,
                        label: "Main d'oeuvre pose".to_owned(),
                        quantity: Decimal::from(3),
                        unit: ServiceRateUnit::Day,
                        unit_price_cents: 35_000,
                        vat_rate_bp: Some(2000),
                        notes: None,
                        photo_keys: Vec::new(),
                    },
                ],
            },
            actor,
        )
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // These two are the only pieces of this module that are pure functions —
    // everything else is a straight-line sequence of use-case calls against
    // a live database and FerrisKey, which is what an `#[ignore]`d
    // integration test would exercise. Not added here: building one that
    // cleans up correctly after itself would mean re-deriving the full FK
    // order across every table `seed_demo_data` writes to (organizations,
    // roles, members, users, customers, contexts, projects, tasks, task
    // labels, task assignments, quotes, quote lines) with no way to run it
    // in this environment to check that cleanup is actually right — a
    // broken cleanup left running against a shared dev database is worse
    // than no test. `create_organization`'s own conflict handling (the
    // behaviour this binary's idempotency rests on) already has coverage in
    // `libs/core`.

    #[test]
    fn db_url_composes_the_same_way_create_service_does() {
        let args = SeedArgs {
            org_slug: "pr-42".to_owned(),
            org_name: "Preview PR 42".to_owned(),
            log: args::log::LogArgs::default(),
            db: args::database::DatabaseArgs {
                host: "db.example.com".to_owned(),
                name: "mestier".to_owned(),
                password: "s3cret".to_owned(),
                port: 5433,
                user: "alice".to_owned(),
            },
            auth: args::auth::AuthArgs {
                issuer: "https://iam.example.com/realms/mestier".to_owned(),
                client_id: "mestier".to_owned(),
                client_secret: "unused".to_owned(),
            },
        };

        assert_eq!(
            db_url(&args),
            "postgres://alice:s3cret@db.example.com:5433/mestier"
        );
    }

    #[test]
    fn owner_identity_is_deterministic_and_slug_derived() {
        let (email, username, display_name) = owner_identity("pr-42", "Preview PR 42");

        assert_eq!(email, "demo-owner+pr-42@mestier.preview");
        assert_eq!(username, "demo-owner-pr-42");
        assert_eq!(display_name, "Preview PR 42 (Demo)");

        // Same slug, same identity — this is the idempotency the whole
        // binary depends on: a re-run must look the owner up under the
        // exact identity the first run created, not mint a new one.
        assert_eq!(
            owner_identity("pr-42", "A completely different display name"),
            (
                email,
                username,
                "A completely different display name (Demo)".to_owned()
            ),
        );
    }
}
