use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use serde::Deserialize;

use crate::{
    CreationCounts, FEATURE_DISABLED, FEATURE_ENABLED, FeatureChange, FeatureFlag, FlagAdoption,
    OrganizationId, OrganizationMemberSummary, OrganizationOverview, OrganizationSummary,
    PlatformStats, RECENT_DAYS, STATS_WEEKS, UserId, UserOrganization, UserSummary, WeeklyCount,
    domain::platform::ports::PlatformAdminRepository,
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = PlatformAdmin, backend = Postgres)]
pub struct PgPlatformAdminRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgPlatformAdminRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn like_pattern(search: Option<&str>) -> Option<String> {
    let term = search?.trim();
    if term.is_empty() {
        return None;
    }
    let escaped = term
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    Some(format!("%{escaped}%"))
}

#[derive(Deserialize)]
struct UserOrganizationRow {
    name: String,
    slug: String,
}

fn count(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

impl<'tx> PgPlatformAdminRepository<'tx> {
    async fn creation_counts(
        &mut self,
        table: Table,
        now: DateTime<Utc>,
    ) -> Result<CreationCounts, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = match table {
            Table::Quotes => sqlx::query!(
                r#"
                SELECT COUNT(*) AS "total!",
                       COUNT(*) FILTER (
                           WHERE created_at > $1::timestamptz - make_interval(days => $2) AND created_at <= $1::timestamptz
                       ) AS "recent!"
                FROM quotes WHERE deleted_at IS NULL
                "#,
                now,
                RECENT_DAYS as i32,
            )
            .fetch_one(&mut ***tx)
            .await
            .map(|r| (r.total, r.recent)),
            Table::Invoices => sqlx::query!(
                r#"
                SELECT COUNT(*) AS "total!",
                       COUNT(*) FILTER (
                           WHERE created_at > $1::timestamptz - make_interval(days => $2) AND created_at <= $1::timestamptz
                       ) AS "recent!"
                FROM invoices WHERE deleted_at IS NULL
                "#,
                now,
                RECENT_DAYS as i32,
            )
            .fetch_one(&mut ***tx)
            .await
            .map(|r| (r.total, r.recent)),
            Table::Projects => sqlx::query!(
                r#"
                SELECT COUNT(*) AS "total!",
                       COUNT(*) FILTER (
                           WHERE created_at > $1::timestamptz - make_interval(days => $2) AND created_at <= $1::timestamptz
                       ) AS "recent!"
                FROM projects
                "#,
                now,
                RECENT_DAYS as i32,
            )
            .fetch_one(&mut ***tx)
            .await
            .map(|r| (r.total, r.recent)),
            Table::Tasks => sqlx::query!(
                r#"
                SELECT COUNT(*) AS "total!",
                       COUNT(*) FILTER (
                           WHERE created_at > $1::timestamptz - make_interval(days => $2) AND created_at <= $1::timestamptz
                       ) AS "recent!"
                FROM tasks WHERE deleted_at IS NULL
                "#,
                now,
                RECENT_DAYS as i32,
            )
            .fetch_one(&mut ***tx)
            .await
            .map(|r| (r.total, r.recent)),
        }
        .map_err(map_sqlx_error)?;

        Ok(CreationCounts {
            total: count(row.0),
            last_30_days: count(row.1),
        })
    }
}

#[derive(Clone, Copy)]
enum Table {
    Quotes,
    Invoices,
    Projects,
    Tasks,
}

impl<'tx> PlatformAdminRepository for PgPlatformAdminRepository<'tx> {
    async fn list_organizations(
        &mut self,
        search: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<OrganizationSummary>, u64), CoreError> {
        let pattern = like_pattern(search);
        let mut tx = self.tx.lock().await;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "total!"
            FROM organizations o
            WHERE o.deleted_at IS NULL
              AND ($1::text IS NULL OR o.name ILIKE $1 OR o.slug ILIKE $1)
            "#,
            pattern,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let rows = sqlx::query!(
            r#"
            SELECT o.id, o.name, o.slug, o.created_at,
                   (SELECT COUNT(*) FROM organization_members m
                     WHERE m.organization_id = o.id
                       AND m.deleted_at IS NULL
                       AND m.user_id IS NOT NULL) AS "member_count!",
                   ARRAY(SELECT f.flag FROM organization_feature_flags f
                          WHERE f.org_id = o.id ORDER BY f.flag) AS "features!"
            FROM organizations o
            WHERE o.deleted_at IS NULL
              AND ($1::text IS NULL OR o.name ILIKE $1 OR o.slug ILIKE $1)
            ORDER BY o.created_at DESC, o.id
            LIMIT $2 OFFSET $3
            "#,
            pattern,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let organizations = rows
            .into_iter()
            .map(|row| OrganizationSummary {
                id: OrganizationId(row.id),
                name: row.name,
                slug: row.slug,
                member_count: count(row.member_count),
                created_at: row.created_at,
                features: row
                    .features
                    .iter()
                    .filter_map(|key| FeatureFlag::try_from(key.as_str()).ok())
                    .collect(),
            })
            .collect();

        Ok((organizations, count(total)))
    }

    async fn organization_overview(
        &mut self,
        id: OrganizationId,
    ) -> Result<Option<OrganizationOverview>, CoreError> {
        let mut tx = self.tx.lock().await;

        let Some(organization) = sqlx::query!(
            r#"
            SELECT id, name, slug, created_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        else {
            return Ok(None);
        };

        let members = sqlx::query!(
            r#"
            SELECT m.last_name, m.first_name, u.email AS "email?", m.created_at,
                   ARRAY(SELECT r.name FROM member_roles mr
                           JOIN roles r ON r.id = mr.role_id
                          WHERE mr.member_id = m.id ORDER BY r.name) AS "roles!"
            FROM organization_members m
            JOIN users u ON u.id = m.user_id
            WHERE m.organization_id = $1
              AND m.deleted_at IS NULL
            ORDER BY m.created_at, m.id
            "#,
            id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| OrganizationMemberSummary {
            name: match row.first_name {
                Some(first) => format!("{} {first}", row.last_name),
                None => row.last_name,
            },
            email: row.email,
            roles: row.roles,
            created_at: row.created_at,
        })
        .collect();

        Ok(Some(OrganizationOverview {
            id: OrganizationId(organization.id),
            name: organization.name,
            slug: organization.slug,
            created_at: organization.created_at,
            members,
        }))
    }

    async fn feature_history(
        &mut self,
        id: OrganizationId,
        limit: i64,
    ) -> Result<Vec<FeatureChange>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT e.name, e.payload->>'flag' AS "flag?", e.actor_kind, e.actor_id,
                   u.display_name AS "actor_name?", e.occurred_at
            FROM automation.event e
            LEFT JOIN users u ON e.actor_kind = 'user' AND u.id = e.actor_id
            WHERE e.org_id = $1 AND e.name IN ($2, $3)
            ORDER BY e.occurred_at DESC, e.id DESC
            LIMIT $4
            "#,
            id.0,
            FEATURE_ENABLED,
            FEATURE_DISABLED,
            limit,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows
            .into_iter()
            .map(|row| FeatureChange {
                flag: row.flag.unwrap_or_default(),
                enabled: row.name == FEATURE_ENABLED,
                actor_id: row
                    .actor_id
                    .filter(|_| row.actor_kind == "user")
                    .map(UserId),
                actor_name: row.actor_name,
                occurred_at: row.occurred_at,
            })
            .collect())
    }

    async fn list_users(
        &mut self,
        search: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<UserSummary>, u64), CoreError> {
        let pattern = like_pattern(search);
        let mut tx = self.tx.lock().await;

        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "total!"
            FROM users u
            WHERE u.deleted_at IS NULL
              AND ($1::text IS NULL
                   OR u.display_name ILIKE $1 OR u.email ILIKE $1 OR u.username ILIKE $1)
            "#,
            pattern,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let rows = sqlx::query!(
            r#"
            SELECT u.id, u.display_name, u.email, u.username, u.created_at,
                   COALESCE((
                       SELECT jsonb_agg(jsonb_build_object('name', o.name, 'slug', o.slug)
                                        ORDER BY o.name, o.id)
                         FROM organization_members m
                         JOIN organizations o ON o.id = m.organization_id
                        WHERE m.user_id = u.id
                          AND m.deleted_at IS NULL
                          AND o.deleted_at IS NULL
                   ), '[]'::jsonb) AS "organizations!"
            FROM users u
            WHERE u.deleted_at IS NULL
              AND ($1::text IS NULL
                   OR u.display_name ILIKE $1 OR u.email ILIKE $1 OR u.username ILIKE $1)
            ORDER BY u.created_at DESC, u.id
            LIMIT $2 OFFSET $3
            "#,
            pattern,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let users = rows
            .into_iter()
            .map(|row| {
                let organizations: Vec<UserOrganizationRow> =
                    serde_json::from_value(row.organizations).map_err(|e| {
                        CoreError::Internal(format!("malformed user organizations: {e}"))
                    })?;
                Ok(UserSummary {
                    id: UserId(row.id),
                    display_name: row.display_name,
                    email: row.email,
                    username: row.username,
                    created_at: row.created_at,
                    organizations: organizations
                        .into_iter()
                        .map(|o| UserOrganization {
                            name: o.name,
                            slug: o.slug,
                        })
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, CoreError>>()?;

        Ok((users, count(total)))
    }

    async fn stats(&mut self, now: DateTime<Utc>) -> Result<PlatformStats, CoreError> {
        let (active_organizations, active_users, signups, organizations, adoption) = {
            let mut tx = self.tx.lock().await;
            let weeks = STATS_WEEKS as i32;

            let active_organizations = sqlx::query_scalar!(
                r#"SELECT COUNT(*) AS "total!" FROM organizations WHERE deleted_at IS NULL"#
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            let active_users = sqlx::query_scalar!(
                r#"SELECT COUNT(*) AS "total!" FROM users WHERE deleted_at IS NULL"#
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            let signups = sqlx::query!(
                r#"
                SELECT w::date AS "week_start!", COUNT(u.id) AS "total!"
                FROM generate_series(
                         date_trunc('week', $1::timestamptz AT TIME ZONE 'UTC')
                             - make_interval(weeks => $2 - 1),
                         date_trunc('week', $1::timestamptz AT TIME ZONE 'UTC'),
                         interval '1 week') AS w
                LEFT JOIN users u ON date_trunc('week', u.created_at AT TIME ZONE 'UTC') = w
                GROUP BY w
                ORDER BY w
                "#,
                now,
                weeks,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .into_iter()
            .map(|row| WeeklyCount {
                week_start: row.week_start,
                count: count(row.total),
            })
            .collect::<Vec<_>>();

            let organizations = sqlx::query!(
                r#"
                SELECT w::date AS "week_start!", COUNT(o.id) AS "total!"
                FROM generate_series(
                         date_trunc('week', $1::timestamptz AT TIME ZONE 'UTC')
                             - make_interval(weeks => $2 - 1),
                         date_trunc('week', $1::timestamptz AT TIME ZONE 'UTC'),
                         interval '1 week') AS w
                LEFT JOIN organizations o ON date_trunc('week', o.created_at AT TIME ZONE 'UTC') = w
                GROUP BY w
                ORDER BY w
                "#,
                now,
                weeks,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .into_iter()
            .map(|row| WeeklyCount {
                week_start: row.week_start,
                count: count(row.total),
            })
            .collect::<Vec<_>>();

            let adoption = sqlx::query!(
                r#"
                SELECT f.flag, COUNT(*) AS "total!"
                FROM organization_feature_flags f
                JOIN organizations o ON o.id = f.org_id AND o.deleted_at IS NULL
                GROUP BY f.flag
                "#
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            (
                active_organizations,
                active_users,
                signups,
                organizations,
                adoption,
            )
        };

        let flag_adoption = FeatureFlag::ALL
            .iter()
            .map(|flag| FlagAdoption {
                flag: *flag,
                organizations: adoption
                    .iter()
                    .find(|row| row.flag == flag.key())
                    .map(|row| count(row.total))
                    .unwrap_or(0),
            })
            .collect();

        Ok(PlatformStats {
            active_organizations: count(active_organizations),
            active_users: count(active_users),
            signups_per_week: signups,
            organizations_per_week: organizations,
            quotes: self.creation_counts(Table::Quotes, now).await?,
            invoices: self.creation_counts(Table::Invoices, now).await?,
            projects: self.creation_counts(Table::Projects, now).await?,
            tasks: self.creation_counts(Table::Tasks, now).await?,
            flag_adoption,
        })
    }
}
