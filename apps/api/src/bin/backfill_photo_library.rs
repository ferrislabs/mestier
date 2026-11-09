//! Copies the photos of the existing DOE of one organization into its photo
//! library. Safe to run again: a photo already in the library is left alone.
//!
//! ```text
//! PHOTO_BACKFILL_ORGANIZATION=<uuid> cargo run -p api --bin backfill_photo_library
//! ```
//!
//! It reads the same configuration as the API (database, storage).

use std::{error::Error, sync::Arc};

use args::Args;
use clap::Parser;
use dotenvy::dotenv;
use handlers::state;
use mestier_core::OrganizationId;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let organization: Uuid = std::env::var("PHOTO_BACKFILL_ORGANIZATION")
        .map_err(|_| "set PHOTO_BACKFILL_ORGANIZATION to the organization id")?
        .parse()?;

    let args = Arc::new(Args::parse());
    let app_state = state(args).await?;

    let report = app_state
        .usecase
        .backfill_organization_photos(OrganizationId(organization), &app_state.file_storage)
        .await?;

    println!(
        "imported {}, already in the library {}, not an image {}, failed {}",
        report.imported, report.already_present, report.skipped, report.failed
    );
    if report.failed > 0 {
        return Err("some photos could not be copied, run it again".into());
    }
    Ok(())
}
