use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    ContactEmail, MemberId, MemberProfile, OrganizationId, PhoneNumber, PostalAddress, PostalCode,
};

#[derive(Debug, Clone)]
pub struct MemberProfileRow {
    pub member_id: Uuid,
    pub org_id: Uuid,
    pub work_email: Option<String>,
    pub personal_email: Option<String>,
    pub mobile: Option<String>,
    pub landline: Option<String>,
    pub street: String,
    pub complement: String,
    pub postal_code: Option<String>,
    pub city: String,
    pub country: String,
    pub job_title: Option<String>,
    pub manager_member_id: Option<Uuid>,
    pub work_permit_end_date: Option<NaiveDate>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn stored<T, E>(
    column: &str,
    value: Option<String>,
    parse: impl for<'a> FnOnce(&'a str) -> Result<T, E>,
) -> Result<Option<T>, CoreError> {
    value
        .map(|text| {
            parse(&text).map_err(|_| {
                CoreError::Internal(format!("member_hr_profiles.{column} holds `{text}`"))
            })
        })
        .transpose()
}

impl TryFrom<MemberProfileRow> for MemberProfile {
    type Error = CoreError;

    fn try_from(row: MemberProfileRow) -> Result<Self, Self::Error> {
        Ok(Self {
            member_id: MemberId(row.member_id),
            organization_id: OrganizationId(row.org_id),
            work_email: stored("work_email", row.work_email, ContactEmail::parse)?,
            personal_email: stored("personal_email", row.personal_email, ContactEmail::parse)?,
            mobile: stored("mobile", row.mobile, PhoneNumber::parse)?,
            landline: stored("landline", row.landline, PhoneNumber::parse)?,
            address: PostalAddress {
                street: row.street,
                complement: row.complement,
                postal_code: stored("postal_code", row.postal_code, |text| {
                    PostalCode::try_from(text)
                })?,
                city: row.city,
                country: row.country,
            },
            job_title: row.job_title,
            manager_member_id: row.manager_member_id.map(MemberId),
            work_permit_end_date: row.work_permit_end_date,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
