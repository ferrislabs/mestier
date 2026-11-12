use chrono::{DateTime, NaiveDate, Utc};

use crate::{FeatureActivation, FeatureFlag, OrganizationId, UserId};

pub mod admin;
pub mod ports;

pub use admin::PlatformAdmin;

pub const STATS_WEEKS: usize = 12;
pub const RECENT_DAYS: i64 = 30;
pub const HISTORY_LIMIT: i64 = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationSummary {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub member_count: u64,
    pub created_at: DateTime<Utc>,
    pub features: Vec<FeatureFlag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationMemberSummary {
    pub name: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationOverview {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub members: Vec<OrganizationMemberSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOrganization {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserSummary {
    pub id: UserId,
    pub display_name: String,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub organizations: Vec<UserOrganization>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FeatureChange {
    pub flag: String,
    pub enabled: bool,
    pub actor_id: Option<UserId>,
    pub actor_name: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationFeature {
    pub flag: FeatureFlag,
    pub activation: Option<FeatureActivation>,
    pub enabled_by_name: Option<String>,
}

impl OrganizationFeature {
    pub fn enabled(&self) -> bool {
        self.activation.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrganizationDetail {
    pub overview: OrganizationOverview,
    pub features: Vec<OrganizationFeature>,
    pub history: Vec<FeatureChange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeeklyCount {
    pub week_start: NaiveDate,
    pub count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationCounts {
    pub total: u64,
    pub last_30_days: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagAdoption {
    pub flag: FeatureFlag,
    pub organizations: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStats {
    pub active_organizations: u64,
    pub active_users: u64,
    pub signups_per_week: Vec<WeeklyCount>,
    pub organizations_per_week: Vec<WeeklyCount>,
    pub quotes: CreationCounts,
    pub invoices: CreationCounts,
    pub projects: CreationCounts,
    pub tasks: CreationCounts,
    pub flag_adoption: Vec<FlagAdoption>,
}
