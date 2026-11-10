use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentId, OrganizationId, Promotion,
    PromotionId, Training, TrainingId,
    domain::cursus_catalog::ports::CursusCatalogRepository,
    infrastructure::{
        cursus_catalog::postgres::model::{
            ClosureReasonRow, EstablishmentRow, PromotionRow, TrainingRow,
        },
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CursusCatalog, backend = Postgres)]
pub struct PgCursusCatalogRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCursusCatalogRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CursusCatalogRepository for PgCursusCatalogRepository<'tx> {
    async fn insert_establishment(
        &mut self,
        establishment: &Establishment,
    ) -> Result<Establishment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EstablishmentRow,
            r#"
            INSERT INTO cursus.establishment
                (id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at
            "#,
            establishment.id.0,
            establishment.organization_id.0,
            establishment.name,
            establishment.uai,
            establishment.siret,
            establishment.address,
            establishment.is_main_site,
            establishment.archived_at,
            establishment.created_at,
            establishment.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> Result<Option<Establishment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EstablishmentRow,
            r#"
            SELECT id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at
            FROM cursus.establishment
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_establishments(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<Establishment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            EstablishmentRow,
            r#"
            SELECT id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at
            FROM cursus.establishment
            WHERE org_id = $1 AND ($2 OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            "#,
            organization_id.0,
            include_archived,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_establishment(
        &mut self,
        establishment: &Establishment,
    ) -> Result<Establishment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EstablishmentRow,
            r#"
            UPDATE cursus.establishment
            SET name = $3, uai = $4, siret = $5, address = $6, is_main_site = $7, updated_at = $8
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at
            "#,
            establishment.organization_id.0,
            establishment.id.0,
            establishment.name,
            establishment.uai,
            establishment.siret,
            establishment.address,
            establishment.is_main_site,
            establishment.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn archive_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
        archived_at: DateTime<Utc>,
    ) -> Result<Establishment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EstablishmentRow,
            r#"
            UPDATE cursus.establishment
            SET archived_at = COALESCE(archived_at, $3), updated_at = $3
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, uai, siret, address, is_main_site, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn insert_training(&mut self, training: &Training) -> Result<Training, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TrainingRow,
            r#"
            INSERT INTO cursus.training
                (id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at
            "#,
            training.id.0,
            training.organization_id.0,
            training.establishment_id.0,
            training.name,
            training.rncp_code,
            training.rncp_label,
            training.level,
            training.degree_type,
            training.archived_at,
            training.created_at,
            training.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> Result<Option<Training>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TrainingRow,
            r#"
            SELECT id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at
            FROM cursus.training
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_trainings(
        &mut self,
        organization_id: OrganizationId,
        establishment_id: Option<EstablishmentId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Training>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let establishment_id = establishment_id.map(|id| id.0);
        let rows = sqlx::query_as!(
            TrainingRow,
            r#"
            SELECT id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at
            FROM cursus.training
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR establishment_id = $2)
              AND ($3 OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            establishment_id,
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
            FROM cursus.training
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR establishment_id = $2)
              AND ($3 OR archived_at IS NULL)
            "#,
            organization_id.0,
            establishment_id,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update_training(&mut self, training: &Training) -> Result<Training, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TrainingRow,
            r#"
            UPDATE cursus.training
            SET establishment_id = $3, name = $4, rncp_code = $5, rncp_label = $6,
                level = $7, degree_type = $8, updated_at = $9
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at
            "#,
            training.organization_id.0,
            training.id.0,
            training.establishment_id.0,
            training.name,
            training.rncp_code,
            training.rncp_label,
            training.level,
            training.degree_type,
            training.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn archive_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
        archived_at: DateTime<Utc>,
    ) -> Result<Training, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TrainingRow,
            r#"
            UPDATE cursus.training
            SET archived_at = COALESCE(archived_at, $3), updated_at = $3
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, establishment_id, name, rncp_code, rncp_label, level, degree_type, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn insert_promotion(&mut self, promotion: &Promotion) -> Result<Promotion, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PromotionRow,
            r#"
            INSERT INTO cursus.promotion
                (id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at
            "#,
            promotion.id.0,
            promotion.organization_id.0,
            promotion.training_id.0,
            promotion.label,
            promotion.start_year,
            promotion.end_year,
            promotion.archived_at,
            promotion.created_at,
            promotion.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> Result<Option<Promotion>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PromotionRow,
            r#"
            SELECT id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at
            FROM cursus.promotion
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_promotions(
        &mut self,
        organization_id: OrganizationId,
        training_id: Option<TrainingId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Promotion>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let training_id = training_id.map(|id| id.0);
        let rows = sqlx::query_as!(
            PromotionRow,
            r#"
            SELECT id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at
            FROM cursus.promotion
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR training_id = $2)
              AND ($3 OR archived_at IS NULL)
            ORDER BY start_year DESC, label ASC, created_at ASC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            training_id,
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
            FROM cursus.promotion
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR training_id = $2)
              AND ($3 OR archived_at IS NULL)
            "#,
            organization_id.0,
            training_id,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update_promotion(&mut self, promotion: &Promotion) -> Result<Promotion, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PromotionRow,
            r#"
            UPDATE cursus.promotion
            SET training_id = $3, label = $4, start_year = $5, end_year = $6, updated_at = $7
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at
            "#,
            promotion.organization_id.0,
            promotion.id.0,
            promotion.training_id.0,
            promotion.label,
            promotion.start_year,
            promotion.end_year,
            promotion.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn archive_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
        archived_at: DateTime<Utc>,
    ) -> Result<Promotion, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PromotionRow,
            r#"
            UPDATE cursus.promotion
            SET archived_at = COALESCE(archived_at, $3), updated_at = $3
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, training_id, label, start_year, end_year, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn insert_closure_reason(
        &mut self,
        closure_reason: &ClosureReason,
    ) -> Result<ClosureReason, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ClosureReasonRow,
            r#"
            INSERT INTO cursus.closure_reason (id, org_id, label, archived_at, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, org_id, label, archived_at, created_at
            "#,
            closure_reason.id.0,
            closure_reason.organization_id.0,
            closure_reason.label,
            closure_reason.archived_at,
            closure_reason.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> Result<Option<ClosureReason>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ClosureReasonRow,
            r#"
            SELECT id, org_id, label, archived_at, created_at
            FROM cursus.closure_reason
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_closure_reasons(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<ClosureReason>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ClosureReasonRow,
            r#"
            SELECT id, org_id, label, archived_at, created_at
            FROM cursus.closure_reason
            WHERE org_id = $1 AND ($2 OR archived_at IS NULL)
            ORDER BY label ASC, created_at ASC
            "#,
            organization_id.0,
            include_archived,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_closure_reason(
        &mut self,
        closure_reason: &ClosureReason,
    ) -> Result<ClosureReason, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ClosureReasonRow,
            r#"
            UPDATE cursus.closure_reason
            SET label = $3
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, label, archived_at, created_at
            "#,
            closure_reason.organization_id.0,
            closure_reason.id.0,
            closure_reason.label,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn archive_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
        archived_at: DateTime<Utc>,
    ) -> Result<ClosureReason, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ClosureReasonRow,
            r#"
            UPDATE cursus.closure_reason
            SET archived_at = COALESCE(archived_at, $3)
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, label, archived_at, created_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }
}
