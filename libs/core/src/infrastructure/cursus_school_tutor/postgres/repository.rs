use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    MemberId, OrganizationId, SchoolTutor, SchoolTutorFilter, SchoolTutorId, SchoolTutorView,
    SearchPattern,
    domain::cursus_school_tutor::ports::CursusSchoolTutorRepository,
    infrastructure::{
        cursus_school_tutor::postgres::model::{SchoolTutorRow, SchoolTutorViewRow},
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = CursusSchoolTutor, backend = Postgres)]
pub struct PgCursusSchoolTutorRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCursusSchoolTutorRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> CursusSchoolTutorRepository for PgCursusSchoolTutorRepository<'tx> {
    async fn insert(&mut self, tutor: &SchoolTutor) -> Result<SchoolTutor, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            SchoolTutorRow,
            r#"
            INSERT INTO cursus.school_tutor
                (id, org_id, member_id, job_title, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, member_id, job_title, archived_at, created_at, updated_at
            "#,
            tutor.id.0,
            tutor.organization_id.0,
            tutor.member_id.0,
            tutor.job_title,
            tutor.archived_at,
            tutor.created_at,
            tutor.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map(Into::into)
        .map_err(map_sqlx_error)
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> Result<Option<SchoolTutorView>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            SchoolTutorViewRow,
            r#"
            SELECT s.id, s.org_id, s.member_id, s.job_title, s.archived_at, s.created_at, s.updated_at,
                   m.last_name AS member_last_name, m.first_name AS member_first_name,
                   u.email AS "member_email?",
                   (SELECT COUNT(*) FROM cursus.triad t
                     WHERE t.org_id = s.org_id
                       AND t.pedagogic_tutor_member_id = s.member_id
                       AND t.closed_at IS NULL) AS "open_triad_count!"
            FROM cursus.school_tutor s
            JOIN organization_members m ON m.id = s.member_id AND m.organization_id = s.org_id
            LEFT JOIN users u ON u.id = m.user_id
            WHERE s.org_id = $1 AND s.id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_sqlx_error)
    }

    async fn find_by_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<Option<SchoolTutor>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            SchoolTutorRow,
            r#"
            SELECT id, org_id, member_id, job_title, archived_at, created_at, updated_at
            FROM cursus.school_tutor
            WHERE org_id = $1 AND member_id = $2
            "#,
            organization_id.0,
            member_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map(|row| row.map(Into::into))
        .map_err(map_sqlx_error)
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &SchoolTutorFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<SchoolTutorView>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let archived = filter.archived.archived();
        let pattern = filter.search.as_ref().map(SearchPattern::as_str);
        let rows = sqlx::query_as!(
            SchoolTutorViewRow,
            r#"
            SELECT s.id, s.org_id, s.member_id, s.job_title, s.archived_at, s.created_at, s.updated_at,
                   m.last_name AS member_last_name, m.first_name AS member_first_name,
                   u.email AS "member_email?",
                   (SELECT COUNT(*) FROM cursus.triad t
                     WHERE t.org_id = s.org_id
                       AND t.pedagogic_tutor_member_id = s.member_id
                       AND t.closed_at IS NULL) AS "open_triad_count!"
            FROM cursus.school_tutor s
            JOIN organization_members m ON m.id = s.member_id AND m.organization_id = s.org_id
            LEFT JOIN users u ON u.id = m.user_id
            WHERE s.org_id = $1
              AND ($2::bool IS NULL OR (s.archived_at IS NOT NULL) = $2)
              AND ($3::text IS NULL
                   OR m.last_name ILIKE $3
                   OR m.first_name ILIKE $3
                   OR concat_ws(' ', m.last_name, m.first_name) ILIKE $3
                   OR concat_ws(' ', m.first_name, m.last_name) ILIKE $3
                   OR u.email ILIKE $3)
            ORDER BY m.last_name ASC, m.first_name ASC NULLS FIRST, s.created_at ASC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            archived,
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
            FROM cursus.school_tutor s
            JOIN organization_members m ON m.id = s.member_id AND m.organization_id = s.org_id
            LEFT JOIN users u ON u.id = m.user_id
            WHERE s.org_id = $1
              AND ($2::bool IS NULL OR (s.archived_at IS NOT NULL) = $2)
              AND ($3::text IS NULL
                   OR m.last_name ILIKE $3
                   OR m.first_name ILIKE $3
                   OR concat_ws(' ', m.last_name, m.first_name) ILIKE $3
                   OR concat_ws(' ', m.first_name, m.last_name) ILIKE $3
                   OR u.email ILIKE $3)
            "#,
            organization_id.0,
            archived,
            pattern,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, tutor: &SchoolTutor) -> Result<SchoolTutor, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            SchoolTutorRow,
            r#"
            UPDATE cursus.school_tutor
            SET job_title = $3, updated_at = $4
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, member_id, job_title, archived_at, created_at, updated_at
            "#,
            tutor.organization_id.0,
            tutor.id.0,
            tutor.job_title,
            tutor.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .map(Into::into)
        .ok_or(CoreError::NotFound)
    }

    async fn set_archived(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
        archived_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> Result<SchoolTutor, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            SchoolTutorRow,
            r#"
            UPDATE cursus.school_tutor
            SET archived_at = $3, updated_at = $4
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, member_id, job_title, archived_at, created_at, updated_at
            "#,
            organization_id.0,
            id.0,
            archived_at,
            updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .map(Into::into)
        .ok_or(CoreError::NotFound)
    }
}
