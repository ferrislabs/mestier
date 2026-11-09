use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    CompanyTutorId, Contract, ContractId, ContractTutor, ContractView, OrganizationId,
    ParticipantId, PromotionId, Triad, TriadDetail, TriadId, TriadSummary, TutorTriadLink,
    domain::cursus_triad::ports::CursusTriadRepository,
    infrastructure::{
        cursus_triad::postgres::model::{
            ContractRow, ContractTutorRow, ContractTutorViewRow, ContractViewRow, TriadDetailRow,
            TriadRow, TriadSummaryRow, TutorLinkRow, assemble_contract_views,
        },
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CursusTriad, backend = Postgres)]
pub struct PgCursusTriadRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCursusTriadRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }

    async fn tutor_view_rows(
        &mut self,
        organization_id: OrganizationId,
        contract_ids: &[Uuid],
    ) -> Result<Vec<ContractTutorViewRow>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ContractTutorViewRow,
            r#"
            SELECT ct.contract_id, ct.company_tutor_id, t.participant_id,
                   p.last_name, p.first_name, ct.is_main, ct.archived_at
            FROM cursus.contract_tutor ct
            JOIN cursus.company_tutor t ON t.org_id = ct.org_id AND t.id = ct.company_tutor_id
            JOIN cursus.participant p ON p.org_id = t.org_id AND p.id = t.participant_id
            WHERE ct.org_id = $1 AND ct.contract_id = ANY($2)
            ORDER BY ct.position ASC
            "#,
            organization_id.0,
            contract_ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }
}

impl<'tx> CursusTriadRepository for PgCursusTriadRepository<'tx> {
    async fn insert_triad(&mut self, triad: &Triad) -> Result<Triad, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TriadRow,
            r#"
            INSERT INTO cursus.triad
                (id, org_id, promotion_id, apprentice_id, pedagogic_tutor_member_id, closed_at, closure_reason_id, closure_note, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, promotion_id, apprentice_id, pedagogic_tutor_member_id, closed_at, closure_reason_id, closure_note, created_at, updated_at
            "#,
            triad.id.0,
            triad.organization_id.0,
            triad.promotion_id.0,
            triad.apprentice_id.0,
            triad.pedagogic_tutor_member_id.map(|id| id.0),
            triad.closure.as_ref().map(|closure| closure.closed_at),
            triad.closure.as_ref().map(|closure| closure.reason_id.0),
            triad.closure.as_ref().and_then(|closure| closure.note.as_deref()),
            triad.created_at,
            triad.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_triad(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> Result<Option<Triad>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TriadRow,
            r#"
            SELECT id, org_id, promotion_id, apprentice_id, pedagogic_tutor_member_id, closed_at, closure_reason_id, closure_note, created_at, updated_at
            FROM cursus.triad
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

    async fn save_triad(&mut self, triad: &Triad) -> Result<Triad, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TriadRow,
            r#"
            UPDATE cursus.triad
            SET pedagogic_tutor_member_id = $3, closed_at = $4, closure_reason_id = $5, closure_note = $6, updated_at = $7
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, promotion_id, apprentice_id, pedagogic_tutor_member_id, closed_at, closure_reason_id, closure_note, created_at, updated_at
            "#,
            triad.organization_id.0,
            triad.id.0,
            triad.pedagogic_tutor_member_id.map(|id| id.0),
            triad.closure.as_ref().map(|closure| closure.closed_at),
            triad.closure.as_ref().map(|closure| closure.reason_id.0),
            triad.closure.as_ref().and_then(|closure| closure.note.as_deref()),
            triad.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn find_triad_detail(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> Result<Option<TriadDetail>, CoreError> {
        let row = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                TriadDetailRow,
                r#"
                SELECT t.id, t.org_id, t.promotion_id, t.apprentice_id, t.pedagogic_tutor_member_id,
                       t.closed_at, t.closure_reason_id, t.closure_note, t.created_at, t.updated_at,
                       p.training_id, p.label AS promotion_label,
                       p.start_year AS promotion_start_year, p.end_year AS promotion_end_year,
                       p.archived_at AS promotion_archived_at,
                       p.created_at AS promotion_created_at, p.updated_at AS promotion_updated_at,
                       tr.name AS training_name,
                       ap.last_name AS apprentice_last_name, ap.first_name AS apprentice_first_name,
                       ap.email AS apprentice_email,
                       m.last_name AS "tutor_last_name?", m.first_name AS "tutor_first_name?",
                       cr.label AS "closure_reason_label?"
                FROM cursus.triad t
                JOIN cursus.promotion p ON p.org_id = t.org_id AND p.id = t.promotion_id
                JOIN cursus.training tr ON tr.org_id = p.org_id AND tr.id = p.training_id
                JOIN cursus.apprentice a ON a.org_id = t.org_id AND a.id = t.apprentice_id
                JOIN cursus.participant ap ON ap.org_id = a.org_id AND ap.id = a.participant_id
                LEFT JOIN organization_members m
                       ON m.id = t.pedagogic_tutor_member_id AND m.organization_id = t.org_id
                LEFT JOIN cursus.closure_reason cr
                       ON cr.org_id = t.org_id AND cr.id = t.closure_reason_id
                WHERE t.org_id = $1 AND t.id = $2
                "#,
                organization_id.0,
                id.0,
            )
            .fetch_optional(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        match row {
            Some(row) => {
                let contracts = self.list_contract_views(organization_id, id).await?;
                Ok(Some(row.into_detail(contracts)))
            }
            None => Ok(None),
        }
    }

    async fn list_triads(
        &mut self,
        organization_id: OrganizationId,
        promotion_id: Option<PromotionId>,
        closed: Option<bool>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TriadSummary>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let promotion_id = promotion_id.map(|id| id.0);
        let rows = sqlx::query_as!(
            TriadSummaryRow,
            r#"
            SELECT t.id, t.org_id, t.promotion_id, t.apprentice_id, t.pedagogic_tutor_member_id,
                   t.closed_at, t.closure_reason_id, t.closure_note, t.created_at, t.updated_at,
                   p.label AS promotion_label, tr.name AS training_name,
                   ap.last_name AS apprentice_last_name, ap.first_name AS apprentice_first_name,
                   m.last_name AS "tutor_last_name?", m.first_name AS "tutor_first_name?",
                   (SELECT h.name
                      FROM cursus.contract c
                      JOIN cursus.host_company h ON h.org_id = c.org_id AND h.id = c.host_company_id
                     WHERE c.org_id = t.org_id AND c.triad_id = t.id AND c.closed_at IS NULL
                     ORDER BY c.start_date DESC
                     LIMIT 1) AS "current_host_company_name?"
            FROM cursus.triad t
            JOIN cursus.promotion p ON p.org_id = t.org_id AND p.id = t.promotion_id
            JOIN cursus.training tr ON tr.org_id = p.org_id AND tr.id = p.training_id
            JOIN cursus.apprentice a ON a.org_id = t.org_id AND a.id = t.apprentice_id
            JOIN cursus.participant ap ON ap.org_id = a.org_id AND ap.id = a.participant_id
            LEFT JOIN organization_members m
                   ON m.id = t.pedagogic_tutor_member_id AND m.organization_id = t.org_id
            WHERE t.org_id = $1
              AND ($2::uuid IS NULL OR t.promotion_id = $2)
              AND ($3::bool IS NULL OR (t.closed_at IS NOT NULL) = $3)
            ORDER BY lower(ap.last_name) ASC, lower(coalesce(ap.first_name, '')) ASC, t.created_at ASC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            promotion_id,
            closed,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM cursus.triad t
            WHERE t.org_id = $1
              AND ($2::uuid IS NULL OR t.promotion_id = $2)
              AND ($3::bool IS NULL OR (t.closed_at IS NOT NULL) = $3)
            "#,
            organization_id.0,
            promotion_id,
            closed,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn list_triad_summaries(
        &mut self,
        organization_id: OrganizationId,
        ids: &[TriadId],
    ) -> Result<Vec<TriadSummary>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as!(
            TriadSummaryRow,
            r#"
            SELECT t.id, t.org_id, t.promotion_id, t.apprentice_id, t.pedagogic_tutor_member_id,
                   t.closed_at, t.closure_reason_id, t.closure_note, t.created_at, t.updated_at,
                   p.label AS promotion_label, tr.name AS training_name,
                   ap.last_name AS apprentice_last_name, ap.first_name AS apprentice_first_name,
                   m.last_name AS "tutor_last_name?", m.first_name AS "tutor_first_name?",
                   (SELECT h.name
                      FROM cursus.contract c
                      JOIN cursus.host_company h ON h.org_id = c.org_id AND h.id = c.host_company_id
                     WHERE c.org_id = t.org_id AND c.triad_id = t.id AND c.closed_at IS NULL
                     ORDER BY c.start_date DESC
                     LIMIT 1) AS "current_host_company_name?"
            FROM cursus.triad t
            JOIN cursus.promotion p ON p.org_id = t.org_id AND p.id = t.promotion_id
            JOIN cursus.training tr ON tr.org_id = p.org_id AND tr.id = p.training_id
            JOIN cursus.apprentice a ON a.org_id = t.org_id AND a.id = t.apprentice_id
            JOIN cursus.participant ap ON ap.org_id = a.org_id AND ap.id = a.participant_id
            LEFT JOIN organization_members m
                   ON m.id = t.pedagogic_tutor_member_id AND m.organization_id = t.org_id
            WHERE t.org_id = $1 AND t.id = ANY($2)
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn insert_contract(&mut self, contract: &Contract) -> Result<Contract, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ContractRow,
            r#"
            INSERT INTO cursus.contract
                (id, org_id, triad_id, host_company_id, kind, number, start_date, end_date, closed_at, closure_reason_id, closure_note, comment, mission_sheet_file_key, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, org_id, triad_id, host_company_id, kind, number, start_date, end_date, closed_at, closure_reason_id, closure_note, comment, mission_sheet_file_key, created_at, updated_at
            "#,
            contract.id.0,
            contract.organization_id.0,
            contract.triad_id.0,
            contract.host_company_id.0,
            contract.kind.as_str(),
            contract.number,
            contract.period.start(),
            contract.period.end(),
            contract.closure.as_ref().map(|closure| closure.closed_at),
            contract.closure.as_ref().map(|closure| closure.reason_id.0),
            contract.closure.as_ref().and_then(|closure| closure.note.as_deref()),
            contract.comment,
            contract.mission_sheet_file_key.as_ref().map(|key| key.as_str()),
            contract.created_at,
            contract.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_contract(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> Result<Option<Contract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ContractRow,
            r#"
            SELECT id, org_id, triad_id, host_company_id, kind, number, start_date, end_date, closed_at, closure_reason_id, closure_note, comment, mission_sheet_file_key, created_at, updated_at
            FROM cursus.contract
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Contract::try_from).transpose()
    }

    async fn save_contract(&mut self, contract: &Contract) -> Result<Contract, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ContractRow,
            r#"
            UPDATE cursus.contract
            SET host_company_id = $3, kind = $4, number = $5, start_date = $6, end_date = $7,
                closed_at = $8, closure_reason_id = $9, closure_note = $10,
                comment = $11, mission_sheet_file_key = $12, updated_at = $13
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, triad_id, host_company_id, kind, number, start_date, end_date, closed_at, closure_reason_id, closure_note, comment, mission_sheet_file_key, created_at, updated_at
            "#,
            contract.organization_id.0,
            contract.id.0,
            contract.host_company_id.0,
            contract.kind.as_str(),
            contract.number,
            contract.period.start(),
            contract.period.end(),
            contract.closure.as_ref().map(|closure| closure.closed_at),
            contract.closure.as_ref().map(|closure| closure.reason_id.0),
            contract.closure.as_ref().and_then(|closure| closure.note.as_deref()),
            contract.comment,
            contract.mission_sheet_file_key.as_ref().map(|key| key.as_str()),
            contract.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.ok_or(CoreError::NotFound)?.try_into()
    }

    async fn find_contract_view(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> Result<Option<ContractView>, CoreError> {
        let rows = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractViewRow,
                r#"
                SELECT c.id, c.org_id, c.triad_id, c.host_company_id, c.kind, c.number, c.start_date, c.end_date,
                       c.closed_at, c.closure_reason_id, c.closure_note, c.comment, c.mission_sheet_file_key,
                       c.created_at, c.updated_at,
                       h.name AS host_company_name, cr.label AS "closure_reason_label?"
                FROM cursus.contract c
                JOIN cursus.host_company h ON h.org_id = c.org_id AND h.id = c.host_company_id
                LEFT JOIN cursus.closure_reason cr ON cr.org_id = c.org_id AND cr.id = c.closure_reason_id
                WHERE c.org_id = $1 AND c.id = $2
                "#,
                organization_id.0,
                id.0,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        let tutors = self.tutor_view_rows(organization_id, &[id.0]).await?;
        Ok(assemble_contract_views(rows, tutors)?.into_iter().next())
    }

    async fn list_contracts(
        &mut self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> Result<Vec<Contract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ContractRow,
            r#"
            SELECT id, org_id, triad_id, host_company_id, kind, number, start_date, end_date, closed_at, closure_reason_id, closure_note, comment, mission_sheet_file_key, created_at, updated_at
            FROM cursus.contract
            WHERE org_id = $1 AND triad_id = $2
            ORDER BY start_date ASC, created_at ASC
            "#,
            organization_id.0,
            triad_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Contract::try_from).collect()
    }

    async fn list_contract_views(
        &mut self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> Result<Vec<ContractView>, CoreError> {
        let rows = {
            let mut tx = self.tx.lock().await;
            sqlx::query_as!(
                ContractViewRow,
                r#"
                SELECT c.id, c.org_id, c.triad_id, c.host_company_id, c.kind, c.number, c.start_date, c.end_date,
                       c.closed_at, c.closure_reason_id, c.closure_note, c.comment, c.mission_sheet_file_key,
                       c.created_at, c.updated_at,
                       h.name AS host_company_name, cr.label AS "closure_reason_label?"
                FROM cursus.contract c
                JOIN cursus.host_company h ON h.org_id = c.org_id AND h.id = c.host_company_id
                LEFT JOIN cursus.closure_reason cr ON cr.org_id = c.org_id AND cr.id = c.closure_reason_id
                WHERE c.org_id = $1 AND c.triad_id = $2
                ORDER BY c.start_date ASC, c.created_at ASC
                "#,
                organization_id.0,
                triad_id.0,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        let contract_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let tutors = self.tutor_view_rows(organization_id, &contract_ids).await?;
        assemble_contract_views(rows, tutors)
    }

    async fn list_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
    ) -> Result<Vec<ContractTutor>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ContractTutorRow,
            r#"
            SELECT contract_id, company_tutor_id, org_id, is_main, position, archived_at
            FROM cursus.contract_tutor
            WHERE org_id = $1 AND contract_id = $2
            ORDER BY position ASC
            "#,
            organization_id.0,
            contract_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(ContractTutor::from).collect())
    }

    async fn replace_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        tutors: &[ContractTutor],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            DELETE FROM cursus.contract_tutor
            WHERE org_id = $1 AND contract_id = $2
            "#,
            organization_id.0,
            contract_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for tutor in tutors {
            sqlx::query!(
                r#"
                INSERT INTO cursus.contract_tutor (contract_id, company_tutor_id, org_id, is_main, position, archived_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                tutor.contract_id.0,
                tutor.company_tutor_id.0,
                tutor.organization_id.0,
                tutor.is_main,
                tutor.position,
                tutor.archived_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    async fn archive_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        archived_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE cursus.contract_tutor
            SET archived_at = $3
            WHERE org_id = $1 AND contract_id = $2 AND archived_at IS NULL
            "#,
            organization_id.0,
            contract_id.0,
            archived_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn company_tutor_is_linked(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            r#"
            SELECT EXISTS (
                SELECT 1 FROM cursus.contract_tutor WHERE org_id = $1 AND company_tutor_id = $2
            ) AS "linked!"
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }

    async fn list_apprentice_triad_ids(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> Result<Vec<TriadId>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids = sqlx::query_scalar!(
            r#"
            SELECT t.id
            FROM cursus.triad t
            JOIN cursus.apprentice a ON a.org_id = t.org_id AND a.id = t.apprentice_id
            WHERE t.org_id = $1 AND a.participant_id = $2
            "#,
            organization_id.0,
            participant_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(ids.into_iter().map(TriadId).collect())
    }

    async fn list_tutor_links(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> Result<Vec<TutorTriadLink>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TutorLinkRow,
            r#"
            SELECT c.triad_id, ct.is_main, ct.archived_at
            FROM cursus.contract_tutor ct
            JOIN cursus.contract c ON c.org_id = ct.org_id AND c.id = ct.contract_id
            JOIN cursus.company_tutor t ON t.org_id = ct.org_id AND t.id = ct.company_tutor_id
            WHERE ct.org_id = $1 AND t.participant_id = $2
            "#,
            organization_id.0,
            participant_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(TutorTriadLink::from).collect())
    }
}
