use common::CoreError;
use mestier_macros::repository;

use crate::{
    Daf, DafDossier, DafId, DafSummary, ProjectId,
    domain::daf::ports::DafRepository,
    infrastructure::{
        daf::postgres::model::{DafDossierRow, DafRow, DafSummaryRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

const NUMBER_CONSTRAINT: &str = "uq_dafs_project_number";

#[repository(domain = Daf, backend = Postgres)]
pub struct PgDafRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgDafRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn to_json<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, CoreError> {
    serde_json::to_value(value)
        .map_err(|e| CoreError::Internal(format!("cannot serialize DAF content: {e}")))
}

fn map_daf_error(error: sqlx::Error) -> CoreError {
    match map_sqlx_error(error) {
        CoreError::Conflict(constraint) if constraint == NUMBER_CONSTRAINT => {
            CoreError::Conflict("this DAF number is already taken in the project".to_owned())
        }
        other => other,
    }
}

impl<'tx> DafRepository for PgDafRepository<'tx> {
    async fn find_dossier(
        &mut self,
        project_id: ProjectId,
    ) -> Result<Option<DafDossier>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DafDossierRow,
            r#"
            SELECT project_id, org_id, title, issuer, project_owner, project_manager, updated_at
            FROM daf_dossiers
            WHERE project_id = $1
            "#,
            project_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(DafDossier::from))
    }

    async fn upsert_dossier(&mut self, dossier: &DafDossier) -> Result<DafDossier, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DafDossierRow,
            r#"
            INSERT INTO daf_dossiers (project_id, org_id, title, issuer, project_owner, project_manager)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (project_id) DO UPDATE
            SET title = EXCLUDED.title, issuer = EXCLUDED.issuer,
                project_owner = EXCLUDED.project_owner,
                project_manager = EXCLUDED.project_manager,
                updated_at = now()
            WHERE daf_dossiers.org_id = EXCLUDED.org_id
            RETURNING project_id, org_id, title, issuer, project_owner, project_manager, updated_at
            "#,
            dossier.project_id.0,
            dossier.organization_id.0,
            dossier.title,
            dossier.issuer,
            dossier.project_owner,
            dossier.project_manager,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn insert(&mut self, daf: &Daf) -> Result<Daf, CoreError> {
        let supplier = daf.supplier.as_ref().map(to_json).transpose()?;
        let attachments = to_json(&daf.attachments)?;

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DafRow,
            r#"
            INSERT INTO dafs (id, org_id, project_id, number, designation, price_ref, source_product_id, supplier, installer, specs, observations, photo_key, attachments, project_owner, project_manager, status, sent_on, issued_on, returned_on, review_note, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, org_id, project_id, number, designation, price_ref, source_product_id, supplier, installer, specs, observations, photo_key, attachments, project_owner, project_manager, status, sent_on, issued_on, returned_on, review_note, created_at, updated_at
            "#,
            daf.id.0,
            daf.organization_id.0,
            daf.project_id.0,
            i32::from(daf.number.get()),
            daf.designation.as_str(),
            daf.price_ref,
            daf.source_product_id.map(|id| id.0),
            supplier,
            daf.installer,
            daf.specs,
            daf.observations,
            daf.photo_key,
            attachments,
            daf.project_owner,
            daf.project_manager,
            daf.status.as_str(),
            daf.sent_on,
            daf.issued_on,
            daf.returned_on,
            daf.review_note,
            daf.created_at,
            daf.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_daf_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: DafId) -> Result<Option<Daf>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DafRow,
            r#"
            SELECT id, org_id, project_id, number, designation, price_ref, source_product_id, supplier, installer, specs, observations, photo_key, attachments, project_owner, project_manager, status, sent_on, issued_on, returned_on, review_note, created_at, updated_at
            FROM dafs
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Daf::try_from).transpose()
    }

    async fn list_by_project(&mut self, project_id: ProjectId) -> Result<Vec<Daf>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DafRow,
            r#"
            SELECT id, org_id, project_id, number, designation, price_ref, source_product_id, supplier, installer, specs, observations, photo_key, attachments, project_owner, project_manager, status, sent_on, issued_on, returned_on, review_note, created_at, updated_at
            FROM dafs
            WHERE project_id = $1
            ORDER BY number
            "#,
            project_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Daf::try_from).collect()
    }

    async fn list_summaries(
        &mut self,
        project_id: ProjectId,
    ) -> Result<Vec<DafSummary>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            DafSummaryRow,
            r#"
            SELECT id, number, designation, price_ref, status, sent_on, issued_on, returned_on, review_note, updated_at
            FROM dafs
            WHERE project_id = $1
            ORDER BY number
            "#,
            project_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(DafSummary::try_from).collect()
    }

    async fn update(&mut self, daf: &Daf) -> Result<Daf, CoreError> {
        let supplier = daf.supplier.as_ref().map(to_json).transpose()?;
        let attachments = to_json(&daf.attachments)?;

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DafRow,
            r#"
            UPDATE dafs
            SET number = $2, designation = $3, price_ref = $4, source_product_id = $5, supplier = $6,
                installer = $7, specs = $8, observations = $9, photo_key = $10, attachments = $11,
                project_owner = $12, project_manager = $13, status = $14, sent_on = $15,
                issued_on = $16, returned_on = $17, review_note = $18, updated_at = $19
            WHERE id = $1
            RETURNING id, org_id, project_id, number, designation, price_ref, source_product_id, supplier, installer, specs, observations, photo_key, attachments, project_owner, project_manager, status, sent_on, issued_on, returned_on, review_note, created_at, updated_at
            "#,
            daf.id.0,
            i32::from(daf.number.get()),
            daf.designation.as_str(),
            daf.price_ref,
            daf.source_product_id.map(|id| id.0),
            supplier,
            daf.installer,
            daf.specs,
            daf.observations,
            daf.photo_key,
            attachments,
            daf.project_owner,
            daf.project_manager,
            daf.status.as_str(),
            daf.sent_on,
            daf.issued_on,
            daf.returned_on,
            daf.review_note,
            daf.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_daf_error)?;

        row.try_into()
    }

    async fn delete(&mut self, id: DafId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(r#"DELETE FROM dafs WHERE id = $1"#, id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
