use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    MemberId, OrganizationId, SchoolTutor, SchoolTutorId, SchoolTutorView,
    domain::member::format_person_name,
};

#[derive(Debug, Clone)]
pub struct SchoolTutorRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub job_title: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SchoolTutorRow> for SchoolTutor {
    fn from(row: SchoolTutorRow) -> Self {
        Self {
            id: SchoolTutorId(row.id),
            organization_id: OrganizationId(row.org_id),
            member_id: MemberId(row.member_id),
            job_title: row.job_title,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchoolTutorViewRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub member_id: Uuid,
    pub job_title: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub member_last_name: String,
    pub member_first_name: Option<String>,
    pub member_email: Option<String>,
    pub open_triad_count: i64,
}

impl From<SchoolTutorViewRow> for SchoolTutorView {
    fn from(row: SchoolTutorViewRow) -> Self {
        Self {
            member_name: format_person_name(
                &row.member_last_name,
                row.member_first_name.as_deref(),
            ),
            member_email: row.member_email,
            open_triad_count: row.open_triad_count,
            tutor: SchoolTutor {
                id: SchoolTutorId(row.id),
                organization_id: OrganizationId(row.org_id),
                member_id: MemberId(row.member_id),
                job_title: row.job_title,
                archived_at: row.archived_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
        }
    }
}
