use common::CoreError;
use mestier_macros::repository;
use serde_json::Value;
use sqlx::types::Json;

use crate::{
    DisplayFields, OrganizationId, SupportSettingsRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = SupportSettings, backend = Postgres)]
pub struct PgSupportSettingsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgSupportSettingsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn stored_display_fields(
    organization_id: OrganizationId,
    stored: Value,
) -> Result<DisplayFields, CoreError> {
    serde_json::from_value::<Vec<String>>(stored)
        .map_err(|error| error.to_string())
        .and_then(|keys| DisplayFields::try_from(keys).map_err(|error| error.to_string()))
        .map_err(|error| {
            CoreError::Internal(format!(
                "stored support display fields of organization {} are invalid: {error}",
                organization_id.0
            ))
        })
}

impl<'tx> SupportSettingsRepository for PgSupportSettingsRepository<'tx> {
    async fn display_fields(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<DisplayFields, CoreError> {
        let stored = {
            let mut tx = self.tx.lock().await;
            sqlx::query_scalar!(
                r#"SELECT display_fields FROM support_settings WHERE org_id = $1"#,
                organization_id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        stored.map_or_else(
            || Ok(DisplayFields::default()),
            |stored| stored_display_fields(organization_id, stored),
        )
    }

    async fn replace_display_fields(
        &mut self,
        organization_id: OrganizationId,
        fields: &DisplayFields,
    ) -> Result<DisplayFields, CoreError> {
        let stored = {
            let mut tx = self.tx.lock().await;
            sqlx::query_scalar!(
                r#"
                INSERT INTO support_settings (org_id, display_fields)
                VALUES ($1, $2)
                ON CONFLICT (org_id) DO UPDATE
                SET display_fields = EXCLUDED.display_fields, updated_at = now()
                RETURNING display_fields
                "#,
                organization_id.0,
                Json(fields) as Json<&DisplayFields>,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        stored_display_fields(organization_id, stored)
    }
}
