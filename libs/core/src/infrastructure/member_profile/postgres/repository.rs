use common::CoreError;
use mestier_macros::repository;

use crate::{
    MemberId, MemberProfile,
    domain::member_profile::ports::MemberProfileRepository,
    infrastructure::{
        member_profile::postgres::model::MemberProfileRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = MemberProfile, backend = Postgres)]
pub struct PgMemberProfileRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMemberProfileRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> MemberProfileRepository for PgMemberProfileRepository<'tx> {
    async fn find_by_member(
        &mut self,
        member_id: MemberId,
    ) -> Result<Option<MemberProfile>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberProfileRow,
            r#"
            SELECT member_id, org_id, work_email, personal_email, mobile, landline, street,
                   complement, postal_code, city, country, job_title, manager_member_id,
                   work_permit_end_date, archived_at, created_at, updated_at
            FROM member_hr_profiles
            WHERE member_id = $1
            "#,
            member_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(MemberProfile::try_from).transpose()
    }

    async fn upsert(&mut self, profile: &MemberProfile) -> Result<MemberProfile, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberProfileRow,
            r#"
            INSERT INTO member_hr_profiles (
                member_id, org_id, work_email, personal_email, mobile, landline, street,
                complement, postal_code, city, country, job_title, manager_member_id,
                work_permit_end_date, archived_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (member_id) DO UPDATE SET
                work_email = EXCLUDED.work_email,
                personal_email = EXCLUDED.personal_email,
                mobile = EXCLUDED.mobile,
                landline = EXCLUDED.landline,
                street = EXCLUDED.street,
                complement = EXCLUDED.complement,
                postal_code = EXCLUDED.postal_code,
                city = EXCLUDED.city,
                country = EXCLUDED.country,
                job_title = EXCLUDED.job_title,
                manager_member_id = EXCLUDED.manager_member_id,
                work_permit_end_date = EXCLUDED.work_permit_end_date,
                archived_at = EXCLUDED.archived_at,
                updated_at = EXCLUDED.updated_at
            RETURNING member_id, org_id, work_email, personal_email, mobile, landline, street,
                      complement, postal_code, city, country, job_title, manager_member_id,
                      work_permit_end_date, archived_at, created_at, updated_at
            "#,
            profile.member_id.0,
            profile.organization_id.0,
            profile.work_email.as_ref().map(|v| v.as_str()),
            profile.personal_email.as_ref().map(|v| v.as_str()),
            profile.mobile.as_ref().map(|v| v.as_str()),
            profile.landline.as_ref().map(|v| v.as_str()),
            profile.address.street,
            profile.address.complement,
            profile.address.postal_code.as_ref().map(|v| v.as_str()),
            profile.address.city,
            profile.address.country,
            profile.job_title,
            profile.manager_member_id.map(|id| id.0),
            profile.work_permit_end_date,
            profile.archived_at,
            profile.created_at,
            profile.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        MemberProfile::try_from(row)
    }
}
