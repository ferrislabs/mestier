use common::CoreError;
use mestier_macros::repository;

use crate::{
    AbsenceType, AbsenceTypeId, OrganizationId,
    domain::absence_type::ports::AbsenceTypeRepository,
    infrastructure::{
        absence_type::postgres::model::AbsenceTypeRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = AbsenceType, backend = Postgres)]
pub struct PgAbsenceTypeRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgAbsenceTypeRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> AbsenceTypeRepository for PgAbsenceTypeRepository<'tx> {
    async fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended('absence_types:' || $1::text, 0))",
            organization_id.0.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<AbsenceType>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            AbsenceTypeRow,
            r#"
            SELECT id, org_id, name, hue, count_mode, account_id, paid, needs_approval,
                   needs_proof, kind::text AS "kind!", archived_at, created_at
            FROM absence_types
            WHERE org_id = $1
            ORDER BY created_at, id
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(AbsenceType::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceTypeId,
    ) -> Result<Option<AbsenceType>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceTypeRow,
            r#"
            SELECT id, org_id, name, hue, count_mode, account_id, paid, needs_approval,
                   needs_proof, kind::text AS "kind!", archived_at, created_at
            FROM absence_types
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(AbsenceType::try_from).transpose()
    }

    async fn insert(&mut self, absence_type: &AbsenceType) -> Result<AbsenceType, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceTypeRow,
            r#"
            INSERT INTO absence_types (
                id, org_id, name, hue, count_mode, account_id, paid, needs_approval,
                needs_proof, kind, archived_at, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CAST($10 AS text)::absence_kind, $11, $12)
            RETURNING id, org_id, name, hue, count_mode, account_id, paid, needs_approval,
                      needs_proof, kind::text AS "kind!", archived_at, created_at
            "#,
            absence_type.id.0,
            absence_type.organization_id.0,
            absence_type.name.as_str(),
            absence_type.hue.get() as i16,
            absence_type.count_mode.as_str(),
            absence_type.account_id,
            absence_type.paid,
            absence_type.needs_approval,
            absence_type.needs_proof,
            absence_type.kind.as_str(),
            absence_type.archived_at,
            absence_type.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        AbsenceType::try_from(row)
    }

    async fn update(&mut self, absence_type: &AbsenceType) -> Result<AbsenceType, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            AbsenceTypeRow,
            r#"
            UPDATE absence_types
            SET name = $3, hue = $4, count_mode = $5, account_id = $6, paid = $7,
                needs_approval = $8, needs_proof = $9, kind = CAST($10 AS text)::absence_kind,
                archived_at = $11
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, name, hue, count_mode, account_id, paid, needs_approval,
                      needs_proof, kind::text AS "kind!", archived_at, created_at
            "#,
            absence_type.organization_id.0,
            absence_type.id.0,
            absence_type.name.as_str(),
            absence_type.hue.get() as i16,
            absence_type.count_mode.as_str(),
            absence_type.account_id,
            absence_type.paid,
            absence_type.needs_approval,
            absence_type.needs_proof,
            absence_type.kind.as_str(),
            absence_type.archived_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound)
            .and_then(AbsenceType::try_from)
    }
}
