use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    Apprentice, ApprenticeId, CompanyTutor, CompanyTutorId, CompanyTutorRecord, OrganizationId,
    Participant, ParticipantFilter, ParticipantId, ParticipantKind, ParticipantStatus,
    SearchPattern, UserId,
    domain::cursus_participant::ports::CursusParticipantRepository,
    infrastructure::{
        cursus_participant::postgres::model::{
            ApprenticeRow, CompanyTutorRecordRow, CompanyTutorRow, ParticipantRow,
        },
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CursusParticipant, backend = Postgres)]
pub struct PgCursusParticipantRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCursusParticipantRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CursusParticipantRepository for PgCursusParticipantRepository<'tx> {
    async fn insert_participant(
        &mut self,
        participant: &Participant,
    ) -> Result<Participant, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            INSERT INTO cursus.participant
                (id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            "#,
            participant.id.0,
            participant.organization_id.0,
            participant.user_id.map(|id| id.0),
            participant.last_name,
            participant.first_name,
            participant.email,
            participant.phone,
            participant.invitation_token_hash,
            participant.invitation_expires_at,
            participant.deactivated_at,
            participant.created_at,
            participant.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_participant(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Option<Participant>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant
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

    async fn find_participant_by_email(
        &mut self,
        organization_id: OrganizationId,
        email: &str,
    ) -> Result<Option<Participant>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant
            WHERE org_id = $1 AND lower(email) = lower($2)
            "#,
            organization_id.0,
            email,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_participant_by_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Participant>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant
            WHERE org_id = $1 AND user_id = $2
            "#,
            organization_id.0,
            user_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_participant_by_token_hash(
        &mut self,
        token_hash: &[u8],
    ) -> Result<Option<Participant>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant
            WHERE invitation_token_hash = $1
            "#,
            token_hash,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_active_participants_by_user(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<Participant>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant
            WHERE user_id = $1 AND deactivated_at IS NULL
            ORDER BY created_at ASC
            "#,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_participants(
        &mut self,
        organization_id: OrganizationId,
        filter: &ParticipantFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Participant>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let kind = filter.kind.map(ParticipantKind::as_str);
        let status = filter.status.map(ParticipantStatus::as_str);
        let host_company_id = filter.host_company_id.map(|id| id.0);
        let pattern = filter.search.as_ref().map(SearchPattern::as_str);
        let rows = sqlx::query_as!(
            ParticipantRow,
            r#"
            SELECT id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            FROM cursus.participant p
            WHERE p.org_id = $1
              AND ($2::text IS NULL
                   OR ($2 = 'apprentice' AND EXISTS (
                           SELECT 1 FROM cursus.apprentice a WHERE a.participant_id = p.id))
                   OR ($2 = 'company_tutor' AND EXISTS (
                           SELECT 1 FROM cursus.company_tutor t WHERE t.participant_id = p.id)))
              AND ($3::text IS NULL
                   OR $3 = CASE
                       WHEN p.deactivated_at IS NOT NULL THEN 'inactive'
                       WHEN p.user_id IS NULL THEN 'pending'
                       ELSE 'active'
                   END)
              AND ($4::uuid IS NULL
                   OR EXISTS (
                       SELECT 1 FROM cursus.company_tutor h
                       WHERE h.org_id = p.org_id AND h.participant_id = p.id AND h.host_company_id = $4))
              AND ($5::text IS NULL
                   OR concat_ws(' ', p.first_name, p.last_name) ILIKE $5
                   OR concat_ws(' ', p.last_name, p.first_name) ILIKE $5
                   OR p.email ILIKE $5)
            ORDER BY lower(p.last_name) ASC, lower(coalesce(p.first_name, '')) ASC, p.created_at ASC
            LIMIT $6 OFFSET $7
            "#,
            organization_id.0,
            kind,
            status,
            host_company_id,
            pattern,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM cursus.participant p
            WHERE p.org_id = $1
              AND ($2::text IS NULL
                   OR ($2 = 'apprentice' AND EXISTS (
                           SELECT 1 FROM cursus.apprentice a WHERE a.participant_id = p.id))
                   OR ($2 = 'company_tutor' AND EXISTS (
                           SELECT 1 FROM cursus.company_tutor t WHERE t.participant_id = p.id)))
              AND ($3::text IS NULL
                   OR $3 = CASE
                       WHEN p.deactivated_at IS NOT NULL THEN 'inactive'
                       WHEN p.user_id IS NULL THEN 'pending'
                       ELSE 'active'
                   END)
              AND ($4::uuid IS NULL
                   OR EXISTS (
                       SELECT 1 FROM cursus.company_tutor h
                       WHERE h.org_id = p.org_id AND h.participant_id = p.id AND h.host_company_id = $4))
              AND ($5::text IS NULL
                   OR concat_ws(' ', p.first_name, p.last_name) ILIKE $5
                   OR concat_ws(' ', p.last_name, p.first_name) ILIKE $5
                   OR p.email ILIKE $5)
            "#,
            organization_id.0,
            kind,
            status,
            host_company_id,
            pattern,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn list_apprentice_ids(
        &mut self,
        organization_id: OrganizationId,
        participant_ids: &[ParticipantId],
    ) -> Result<Vec<(ParticipantId, ApprenticeId)>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = participant_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT participant_id, id
            FROM cursus.apprentice
            WHERE org_id = $1 AND participant_id = ANY($2)
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| (ParticipantId(row.participant_id), ApprenticeId(row.id)))
            .collect())
    }

    async fn list_company_tutors_of(
        &mut self,
        organization_id: OrganizationId,
        participant_ids: &[ParticipantId],
    ) -> Result<Vec<CompanyTutorRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = participant_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query_as!(
            CompanyTutorRecordRow,
            r#"
            SELECT t.id, t.org_id, t.participant_id, t.host_company_id, t.job_title,
                   t.created_at, t.updated_at, h.name AS host_company_name
            FROM cursus.company_tutor t
            JOIN cursus.host_company h ON h.org_id = t.org_id AND h.id = t.host_company_id
            WHERE t.org_id = $1 AND t.participant_id = ANY($2)
            ORDER BY h.name ASC, t.created_at ASC
            "#,
            organization_id.0,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_participant(
        &mut self,
        participant: &Participant,
    ) -> Result<Participant, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            UPDATE cursus.participant
            SET last_name = $3, first_name = $4, email = $5, phone = $6, updated_at = $7
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            "#,
            participant.organization_id.0,
            participant.id.0,
            participant.last_name,
            participant.first_name,
            participant.email,
            participant.phone,
            participant.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn set_invitation(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        token_hash: &[u8],
        expires_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Participant, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            UPDATE cursus.participant
            SET invitation_token_hash = $3, invitation_expires_at = $4, updated_at = $5
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            token_hash,
            expires_at,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn link_user(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        user_id: UserId,
        updated_at: DateTime<Utc>,
    ) -> Result<Participant, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            UPDATE cursus.participant
            SET user_id = $3, invitation_token_hash = NULL, invitation_expires_at = NULL, updated_at = $4
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            user_id.0,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn set_deactivated_at(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        deactivated_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> Result<Participant, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ParticipantRow,
            r#"
            UPDATE cursus.participant
            SET deactivated_at = $3, updated_at = $4
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, user_id, last_name, first_name, email, phone, invitation_token_hash, invitation_expires_at, deactivated_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            deactivated_at,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn upsert_apprentice(
        &mut self,
        apprentice: &Apprentice,
    ) -> Result<Apprentice, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ApprenticeRow,
            r#"
            INSERT INTO cursus.apprentice
                (id, org_id, participant_id, birth_date, ine_code, address, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (participant_id) DO UPDATE
            SET birth_date = EXCLUDED.birth_date,
                ine_code = EXCLUDED.ine_code,
                address = EXCLUDED.address,
                updated_at = EXCLUDED.updated_at
            RETURNING id, org_id, participant_id, birth_date, ine_code, address, created_at, updated_at
            "#,
            apprentice.id.0,
            apprentice.organization_id.0,
            apprentice.participant_id.0,
            apprentice.birth_date,
            apprentice.ine_code,
            apprentice.address,
            apprentice.created_at,
            apprentice.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_apprentice(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> Result<Option<Apprentice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ApprenticeRow,
            r#"
            SELECT id, org_id, participant_id, birth_date, ine_code, address, created_at, updated_at
            FROM cursus.apprentice
            WHERE org_id = $1 AND participant_id = $2
            "#,
            organization_id.0,
            participant_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_apprentice_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: ApprenticeId,
    ) -> Result<Option<Apprentice>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ApprenticeRow,
            r#"
            SELECT id, org_id, participant_id, birth_date, ine_code, address, created_at, updated_at
            FROM cursus.apprentice
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

    async fn find_company_tutor(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> Result<Option<CompanyTutor>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CompanyTutorRow,
            r#"
            SELECT id, org_id, participant_id, host_company_id, job_title, created_at, updated_at
            FROM cursus.company_tutor
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

    async fn insert_company_tutor(
        &mut self,
        tutor: &CompanyTutor,
    ) -> Result<CompanyTutor, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CompanyTutorRow,
            r#"
            INSERT INTO cursus.company_tutor
                (id, org_id, participant_id, host_company_id, job_title, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, participant_id, host_company_id, job_title, created_at, updated_at
            "#,
            tutor.id.0,
            tutor.organization_id.0,
            tutor.participant_id.0,
            tutor.host_company_id.0,
            tutor.job_title,
            tutor.created_at,
            tutor.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn list_company_tutors(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> Result<Vec<CompanyTutorRecord>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CompanyTutorRecordRow,
            r#"
            SELECT t.id, t.org_id, t.participant_id, t.host_company_id, t.job_title,
                   t.created_at, t.updated_at, h.name AS host_company_name
            FROM cursus.company_tutor t
            JOIN cursus.host_company h ON h.org_id = t.org_id AND h.id = t.host_company_id
            WHERE t.org_id = $1 AND t.participant_id = $2
            ORDER BY h.name ASC, t.created_at ASC
            "#,
            organization_id.0,
            participant_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_company_tutor(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let deleted = sqlx::query!(
            r#"
            DELETE FROM cursus.company_tutor
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if deleted.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
