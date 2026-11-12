use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    CreationCounts, FeatureFlag, News, NewsIcon, NewsId, OrganizationDetail, OrganizationId,
    OrganizationSummary, PlatformStats, UserId, UserSummary, WeeklyCount,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsItemResponse {
    pub icon: NewsIcon,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsCtaResponse {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct NewsResponse {
    pub id: NewsId,
    pub title: String,
    pub image_key: Option<String>,
    pub items: Vec<NewsItemResponse>,
    pub cta: Option<NewsCtaResponse>,
    /// Absent for a draft, in the future for a scheduled news.
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<News> for NewsResponse {
    fn from(news: News) -> Self {
        let content = news.content;
        Self {
            id: news.id,
            title: content.title,
            image_key: content.image_key,
            items: content
                .items
                .into_iter()
                .map(|item| NewsItemResponse {
                    icon: item.icon,
                    title: item.title,
                    text: item.text,
                })
                .collect(),
            cta: content.cta.map(|cta| NewsCtaResponse {
                label: cta.label,
                url: cta.url,
            }),
            published_at: news.published_at,
            created_at: news.created_at,
            updated_at: news.updated_at,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminAccessResponse {
    pub can_access: bool,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct FeatureFlagResponse {
    pub key: String,
    pub label: String,
    pub description: String,
}

impl From<FeatureFlag> for FeatureFlagResponse {
    fn from(flag: FeatureFlag) -> Self {
        Self {
            key: flag.key().to_owned(),
            label: flag.label().to_owned(),
            description: flag.description().to_owned(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct WeeklyCountResponse {
    pub week_start: NaiveDate,
    pub count: u64,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct CreationCountsResponse {
    pub total: u64,
    pub last_30_days: u64,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct FlagAdoptionResponse {
    pub key: String,
    pub label: String,
    pub organizations: u64,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct PlatformStatsResponse {
    pub active_organizations: u64,
    pub active_users: u64,
    pub signups_per_week: Vec<WeeklyCountResponse>,
    pub organizations_per_week: Vec<WeeklyCountResponse>,
    pub quotes: CreationCountsResponse,
    pub invoices: CreationCountsResponse,
    pub projects: CreationCountsResponse,
    pub tasks: CreationCountsResponse,
    pub features: Vec<FlagAdoptionResponse>,
}

fn weekly(counts: Vec<WeeklyCount>) -> Vec<WeeklyCountResponse> {
    counts
        .into_iter()
        .map(|week| WeeklyCountResponse {
            week_start: week.week_start,
            count: week.count,
        })
        .collect()
}

impl From<CreationCounts> for CreationCountsResponse {
    fn from(counts: CreationCounts) -> Self {
        Self {
            total: counts.total,
            last_30_days: counts.last_30_days,
        }
    }
}

impl From<PlatformStats> for PlatformStatsResponse {
    fn from(stats: PlatformStats) -> Self {
        Self {
            active_organizations: stats.active_organizations,
            active_users: stats.active_users,
            signups_per_week: weekly(stats.signups_per_week),
            organizations_per_week: weekly(stats.organizations_per_week),
            quotes: stats.quotes.into(),
            invoices: stats.invoices.into(),
            projects: stats.projects.into(),
            tasks: stats.tasks.into(),
            features: stats
                .flag_adoption
                .into_iter()
                .map(|adoption| FlagAdoptionResponse {
                    key: adoption.flag.key().to_owned(),
                    label: adoption.flag.label().to_owned(),
                    organizations: adoption.organizations,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminOrganizationSummaryResponse {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub member_count: u64,
    pub created_at: DateTime<Utc>,
    pub features: Vec<String>,
}

impl From<OrganizationSummary> for AdminOrganizationSummaryResponse {
    fn from(summary: OrganizationSummary) -> Self {
        Self {
            id: summary.id,
            name: summary.name,
            slug: summary.slug,
            member_count: summary.member_count,
            created_at: summary.created_at,
            features: summary
                .features
                .into_iter()
                .map(|flag| flag.key().to_owned())
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminMemberResponse {
    pub name: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminOrganizationFeatureResponse {
    pub key: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
    pub enabled_at: Option<DateTime<Utc>>,
    pub enabled_by_name: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminFeatureChangeResponse {
    pub flag: String,
    pub enabled: bool,
    pub actor_name: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminOrganizationResponse {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub members: Vec<AdminMemberResponse>,
    pub features: Vec<AdminOrganizationFeatureResponse>,
    pub history: Vec<AdminFeatureChangeResponse>,
}

impl From<OrganizationDetail> for AdminOrganizationResponse {
    fn from(detail: OrganizationDetail) -> Self {
        let overview = detail.overview;
        Self {
            id: overview.id,
            name: overview.name,
            slug: overview.slug,
            created_at: overview.created_at,
            members: overview
                .members
                .into_iter()
                .map(|member| AdminMemberResponse {
                    name: member.name,
                    email: member.email,
                    roles: member.roles,
                    created_at: member.created_at,
                })
                .collect(),
            features: detail
                .features
                .into_iter()
                .map(|feature| AdminOrganizationFeatureResponse {
                    key: feature.flag.key().to_owned(),
                    label: feature.flag.label().to_owned(),
                    description: feature.flag.description().to_owned(),
                    enabled: feature.enabled(),
                    enabled_at: feature.activation.map(|a| a.enabled_at),
                    enabled_by_name: feature.enabled_by_name,
                })
                .collect(),
            history: detail
                .history
                .into_iter()
                .map(|change| AdminFeatureChangeResponse {
                    flag: change.flag,
                    enabled: change.enabled,
                    actor_name: change.actor_name,
                    occurred_at: change.occurred_at,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminFeatureToggleResponse {
    pub key: String,
    pub enabled: bool,
    pub changed: bool,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminUserOrganizationResponse {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct AdminUserResponse {
    pub id: UserId,
    pub display_name: String,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub organizations: Vec<AdminUserOrganizationResponse>,
}

impl From<UserSummary> for AdminUserResponse {
    fn from(user: UserSummary) -> Self {
        Self {
            id: user.id,
            display_name: user.display_name,
            email: user.email,
            username: user.username,
            created_at: user.created_at,
            organizations: user
                .organizations
                .into_iter()
                .map(|organization| AdminUserOrganizationResponse {
                    name: organization.name,
                    slug: organization.slug,
                })
                .collect(),
        }
    }
}
