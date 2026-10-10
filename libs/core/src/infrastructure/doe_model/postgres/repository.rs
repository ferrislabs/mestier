use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::types::Json;

use crate::{
    DoeModel, DoeModelId, OrganizationId,
    domain::doe_model::ports::DoeModelRepository,
    infrastructure::{
        doe_model::postgres::model::DoeModelRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = DoeModel, backend = Postgres)]
pub struct PgDoeModelRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDoeModelRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> DoeModelRepository for PgDoeModelRepository<'tx> {
    async fn insert(&mut self, model: &DoeModel) -> Result<DoeModel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeModelRow,
            r#"
            INSERT INTO doe_models (id, org_id, name, description, sections, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, name, description,
                      sections AS "sections: Json<Vec<crate::DoeModelSection>>",
                      archived_at, created_at, updated_at
            "#,
            model.id.0,
            model.organization_id.0,
            model.name,
            model.description,
            Json(&model.sections) as Json<&Vec<crate::DoeModelSection>>,
            model.archived_at,
            model.created_at,
            model.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: DoeModelId) -> Result<Option<DoeModel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeModelRow,
            r#"
            SELECT id, org_id, name, description,
                   sections AS "sections: Json<Vec<crate::DoeModelSection>>",
                   archived_at, created_at, updated_at
            FROM doe_models
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<DoeModel>, u64), CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query_as!(
            DoeModelRow,
            r#"
            SELECT id, org_id, name, description,
                   sections AS "sections: Json<Vec<crate::DoeModelSection>>",
                   archived_at, created_at, updated_at
            FROM doe_models
            WHERE org_id = $1
              AND ($2::boolean OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            include_archived,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM doe_models
            WHERE org_id = $1
              AND ($2::boolean OR archived_at IS NULL)
            "#,
            organization_id.0,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, model: &DoeModel) -> Result<DoeModel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DoeModelRow,
            r#"
            UPDATE doe_models
            SET name = $2, description = $3, sections = $4, updated_at = $5
            WHERE id = $1
            RETURNING id, org_id, name, description,
                      sections AS "sections: Json<Vec<crate::DoeModelSection>>",
                      archived_at, created_at, updated_at
            "#,
            model.id.0,
            model.name,
            model.description,
            Json(&model.sections) as Json<&Vec<crate::DoeModelSection>>,
            model.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        Ok(row.into())
    }

    async fn set_archived_at(
        &mut self,
        id: DoeModelId,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"UPDATE doe_models SET archived_at = $2 WHERE id = $1"#,
            id.0,
            archived_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
