use args::{auth::AuthArgs, database::DatabaseArgs, log::LogArgs};
use clap::Parser;

/// CLI contract for the preview-environment demo seeder.
///
/// Invoked as `seed-demo --org-slug <slug> --org-name <name>`, plus whatever
/// DB/auth flags the `api` binary already exposes via `libs/args` — reused
/// here as-is rather than re-invented, so a preview `Application`'s Helm
/// values configure both binaries the same way.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "seed-demo",
    about = "Seeds a demo organization and a small set of realistic data into a preview environment"
)]
pub struct SeedArgs {
    /// Slug of the demo organization to create. Doubles as the idempotency
    /// key: a second run against the same slug is a no-op.
    #[arg(
        long = "org-slug",
        env = "SEED_ORG_SLUG",
        name = "SEED_ORG_SLUG",
        long_help = "Slug of the demo organization to create (e.g. \"pr-42\")"
    )]
    pub org_slug: String,

    /// Display name of the demo organization to create.
    #[arg(
        long = "org-name",
        env = "SEED_ORG_NAME",
        name = "SEED_ORG_NAME",
        long_help = "Display name of the demo organization to create (e.g. \"Preview PR 42\")"
    )]
    pub org_name: String,

    #[command(flatten)]
    pub log: LogArgs,

    #[command(flatten)]
    pub db: DatabaseArgs,

    #[command(flatten)]
    pub auth: AuthArgs,
}
