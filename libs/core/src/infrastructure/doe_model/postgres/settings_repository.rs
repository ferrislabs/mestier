use common::CoreError;
use mestier_macros::repository;
use sqlx::types::Json;

use crate::{
    DoeSettings, DoeStyle, OrganizationId,
    domain::doe_model::ports::DoeSettingsRepository,
    infrastructure::{
        doe_model::postgres::model::DoeSettingsRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = DoeSettings, backend = Postgres)]
pub struct PgDoeSettingsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeSettingsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DoeSettingsRepository for PgDoeSettingsRepository<'tx> {
    async fn find(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Option<DoeSettings>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeSettingsRow,
            r#"SELECT org_id, style AS "style: Json<DoeStyle>" FROM doe_settings WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn upsert(&mut self, settings: &DoeSettings) -> Result<DoeSettings, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeSettingsRow,
            r#"
            INSERT INTO doe_settings (org_id, style, updated_at)
            VALUES ($1, $2, now())
            ON CONFLICT (org_id) DO UPDATE
            SET style = EXCLUDED.style, updated_at = now()
            RETURNING org_id, style AS "style: Json<DoeStyle>"
            "#,
            settings.organization_id.0,
            Json(&settings.style) as Json<&DoeStyle>,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }
}
