use common::CoreError;
use discord::domain::presence::ports::PresenceRepository;
use discord::{OrganizationId, Presence, UserId};
use mestier_macros::repository;

use super::model::PresenceRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Presence, backend = Postgres)]
pub struct PgPresenceRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgPresenceRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> PresenceRepository for PgPresenceRepository<'tx> {
    async fn upsert(&mut self, p: &Presence) -> Result<Presence, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PresenceRow,
            r#"
			INSERT INTO chat.member_presence (org_id, user_id, status, updated_at)
			VALUES ($1, $2, CAST($3 AS text)::chat.presence_status, $4)
			ON CONFLICT (org_id, user_id) DO UPDATE
				SET status = EXCLUDED.status, updated_at = EXCLUDED.updated_at
			RETURNING org_id, user_id, status::text AS "status!", updated_at
			"#,
            p.organization_id.0,
            p.user_id.0,
            p.status.as_str(),
            p.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.try_into()
    }

    async fn find(
        &mut self,
        org: OrganizationId,
        user: UserId,
    ) -> Result<Option<Presence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            PresenceRow,
            r#"
			SELECT org_id, user_id, status::text AS "status!", updated_at
			FROM chat.member_presence WHERE org_id = $1 AND user_id = $2
			"#,
            org.0,
            user.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> Result<Vec<Presence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            PresenceRow,
            r#"
			SELECT org_id, user_id, status::text AS "status!", updated_at
			FROM chat.member_presence WHERE org_id = $1 ORDER BY user_id ASC
			"#,
            org.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }
}
