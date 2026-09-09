use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomUnit, OrganizationId,
    domain::custom_unit::ports::CustomUnitRepository,
    infrastructure::{
        custom_unit::postgres::model::CustomUnitRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CustomUnit, backend = Postgres)]
pub struct PgCustomUnitRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCustomUnitRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CustomUnitRepository for PgCustomUnitRepository<'tx> {
    async fn insert(&mut self, custom_unit: &CustomUnit) -> Result<CustomUnit, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CustomUnitRow,
            r#"
            INSERT INTO custom_units (id, org_id, code, created_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, org_id, code, created_at
            "#,
            custom_unit.id.0,
            custom_unit.organization_id.0,
            custom_unit.code,
            custom_unit.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomUnit>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CustomUnitRow,
            r#"
            SELECT id, org_id, code, created_at
            FROM custom_units
            WHERE org_id = $1
            ORDER BY code ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}
