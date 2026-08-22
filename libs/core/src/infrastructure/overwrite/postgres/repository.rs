use common::{CoreError, generate_uuid_v7};
use discord::domain::overwrite::ports::OverwriteRepository;
use discord::{ChannelId, ChannelPermissionOverwrite, OverwriteTarget, UpsertChannelOverwrite};
use mestier_macros::repository;
use uuid::Uuid;

use super::model::ChannelPermissionOverwriteRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Overwrite, backend = Postgres)]
pub struct PgOverwriteRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgOverwriteRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> OverwriteRepository for PgOverwriteRepository<'tx> {
    async fn upsert(
        &self,
        command: UpsertChannelOverwrite,
    ) -> Result<ChannelPermissionOverwrite, CoreError> {
        let new_id = generate_uuid_v7();
        let mut tx = self.tx.lock().await;

        let (target_type, target_id): (&str, Option<Uuid>) = match &command.target {
            OverwriteTarget::Everyone => ("EVERYONE", None),
            OverwriteTarget::Role(r) => ("ROLE", Some(r.0)),
            OverwriteTarget::Member(u) => ("MEMBER", Some(u.0)),
        };

        let row = if target_id.is_none() {
            // EVERYONE branch: partial index ON (channel_id) WHERE target_type='EVERYONE'
            sqlx::query_as!(
                ChannelPermissionOverwriteRow,
                r#"
                INSERT INTO chat.channel_permission_overwrite
                    (id, channel_id, org_id, target_type, target_id, allow, deny, created_at, updated_at)
                VALUES ($1, $2, $3, $4, NULL, $5, $6, now(), now())
                ON CONFLICT (channel_id) WHERE target_type = 'EVERYONE'
                DO UPDATE SET allow = EXCLUDED.allow,
                              deny  = EXCLUDED.deny,
                              updated_at = now()
                RETURNING id, channel_id, org_id, target_type, target_id, allow, deny, created_at, updated_at
                "#,
                new_id,
                command.channel_id.0,
                command.organization_id.0,
                target_type,
                command.allow,
                command.deny,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        } else {
            // ROLE/MEMBER branch: partial index ON (channel_id, target_type, target_id) WHERE target_type IN ('ROLE','MEMBER')
            sqlx::query_as!(
                ChannelPermissionOverwriteRow,
                r#"
                INSERT INTO chat.channel_permission_overwrite
                    (id, channel_id, org_id, target_type, target_id, allow, deny, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, now(), now())
                ON CONFLICT (channel_id, target_type, target_id) WHERE target_type IN ('ROLE','MEMBER')
                DO UPDATE SET allow = EXCLUDED.allow,
                              deny  = EXCLUDED.deny,
                              updated_at = now()
                RETURNING id, channel_id, org_id, target_type, target_id, allow, deny, created_at, updated_at
                "#,
                new_id,
                command.channel_id.0,
                command.organization_id.0,
                target_type,
                target_id,
                command.allow,
                command.deny,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        row.try_into()
    }

    async fn delete(
        &self,
        channel_id: ChannelId,
        target: OverwriteTarget,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        match target {
            OverwriteTarget::Everyone => {
                sqlx::query!(
                    r#"
                    DELETE FROM chat.channel_permission_overwrite
                    WHERE channel_id = $1 AND target_type = 'EVERYONE'
                    "#,
                    channel_id.0,
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
            OverwriteTarget::Role(role_id) => {
                sqlx::query!(
                    r#"
                    DELETE FROM chat.channel_permission_overwrite
                    WHERE channel_id = $1 AND target_type = 'ROLE' AND target_id = $2
                    "#,
                    channel_id.0,
                    role_id.0,
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
            OverwriteTarget::Member(user_id) => {
                sqlx::query!(
                    r#"
                    DELETE FROM chat.channel_permission_overwrite
                    WHERE channel_id = $1 AND target_type = 'MEMBER' AND target_id = $2
                    "#,
                    channel_id.0,
                    user_id.0,
                )
                .execute(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;
            }
        }
        Ok(())
    }

    async fn list_for_channel(
        &self,
        channel_id: ChannelId,
    ) -> Result<Vec<ChannelPermissionOverwrite>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ChannelPermissionOverwriteRow,
            r#"
            SELECT id, channel_id, org_id, target_type, target_id, allow, deny, created_at, updated_at
            FROM chat.channel_permission_overwrite
            WHERE channel_id = $1
            ORDER BY target_type ASC, target_id ASC NULLS FIRST
            "#,
            channel_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::test_support::dev_pool;
    use common::{RoleId, generate_uuid_v7};
    use discord::{OrganizationId, UserId};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        dev_pool().await
    }

    async fn seed_org_user_channel(
        tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> (OrganizationId, UserId, ChannelId) {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("test-{}@example.com", user_id),
            format!("user-{}", user_id),
            "Test User",
            format!("sub-{}", user_id),
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{}", org_id),
            user_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        let channel_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO chat.channels (id, org_id, channel_type, name, position, archived, created_at, updated_at)
               VALUES ($1, $2, 'TEXT'::chat.channel_type, $3, 0, false, now(), now())"#,
            channel_id,
            org_id,
            format!("test-channel-{}", channel_id),
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        (
            OrganizationId(org_id),
            UserId(user_id),
            ChannelId(channel_id),
        )
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn overwrite_upsert_list_delete_and_private_channel_resolve() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, user_id, channel_id) = {
                let mut guard = tx.lock().await;
                seed_org_user_channel(*guard).await
            };

            let role_id = RoleId(generate_uuid_v7());
            let repo = PgOverwriteRepository::new(&tx);

            // ── Upsert EVERYONE overwrite (deny VIEW_CHANNEL = 32) ────────────
            let everyone_cmd = UpsertChannelOverwrite {
                channel_id,
                organization_id: org_id,
                target: OverwriteTarget::Everyone,
                allow: 0,
                deny: 32, // VIEW_CHANNEL bit
            };
            let everyone_ow = repo.upsert(everyone_cmd).await?;
            assert_eq!(everyone_ow.deny, 32);
            assert!(matches!(everyone_ow.target, OverwriteTarget::Everyone));

            // ── Upsert ROLE overwrite (allow VIEW_CHANNEL = 32) ───────────────
            let role_cmd = UpsertChannelOverwrite {
                channel_id,
                organization_id: org_id,
                target: OverwriteTarget::Role(role_id),
                allow: 32,
                deny: 0,
            };
            let role_ow = repo.upsert(role_cmd).await?;
            assert_eq!(role_ow.allow, 32);
            assert!(matches!(role_ow.target, OverwriteTarget::Role(_)));

            // ── Upsert MEMBER overwrite (deny VIEW_CHANNEL = 32) ─────────────
            let member_cmd = UpsertChannelOverwrite {
                channel_id,
                organization_id: org_id,
                target: OverwriteTarget::Member(user_id),
                allow: 0,
                deny: 32,
            };
            let member_ow = repo.upsert(member_cmd).await?;
            assert_eq!(member_ow.deny, 32);
            assert!(matches!(member_ow.target, OverwriteTarget::Member(_)));

            // ── list_for_channel returns all three ────────────────────────────
            let list = repo.list_for_channel(channel_id).await?;
            assert_eq!(list.len(), 3, "expected 3 overwrites, got {}", list.len());

            // ── Re-upsert EVERYONE with updated allow bits — idempotent ───────
            let updated_cmd = UpsertChannelOverwrite {
                channel_id,
                organization_id: org_id,
                target: OverwriteTarget::Everyone,
                allow: 64, // SEND_MESSAGES
                deny: 32,
            };
            let updated_ow = repo.upsert(updated_cmd).await?;
            assert_eq!(updated_ow.allow, 64);
            assert_eq!(updated_ow.deny, 32);

            // Still 3 rows (no duplicate EVERYONE) ─────────────────────────────
            let list2 = repo.list_for_channel(channel_id).await?;
            assert_eq!(
                list2.len(),
                3,
                "re-upsert must not insert a second EVERYONE row"
            );

            // ── Delete MEMBER overwrite ────────────────────────────────────────
            repo.delete(channel_id, OverwriteTarget::Member(user_id))
                .await?;
            let list3 = repo.list_for_channel(channel_id).await?;
            assert_eq!(list3.len(), 2, "after deleting MEMBER, expected 2 rows");
            assert!(
                list3
                    .iter()
                    .all(|o| !matches!(o.target, OverwriteTarget::Member(_))),
                "MEMBER overwrite must be deleted"
            );

            // ── Private channel resolution end-to-end ─────────────────────────
            // base = VIEW_CHANNEL | SEND_MESSAGES = 32 | 64 = 96
            // EVERYONE deny 32 → perms = 64
            // ROLE allow 32 → perms = 96  (role member can view)
            // No MEMBER overwrite remains
            let all_ows = repo.list_for_channel(channel_id).await?;
            let everyone_ow = all_ows
                .iter()
                .find(|o| matches!(o.target, OverwriteTarget::Everyone));
            let role_ows: Vec<&ChannelPermissionOverwrite> = all_ows
                .iter()
                .filter(|o| matches!(o.target, OverwriteTarget::Role(r) if r == role_id))
                .collect();

            let base_bits: i64 = 96; // VIEW_CHANNEL | SEND_MESSAGES
            let effective_with_role = discord::resolve_channel_permissions(
                base_bits,
                everyone_ow,
                &role_ows.into_iter().cloned().collect::<Vec<_>>(),
                None,
            );
            // EVERYONE denies VIEW → base 96 & !32 = 64; ROLE allows 32 → 96
            assert_eq!(
                effective_with_role & 32,
                32,
                "role member must have VIEW_CHANNEL"
            );

            let effective_without_role =
                discord::resolve_channel_permissions(base_bits, everyone_ow, &[], None);
            // No role overwrite: EVERYONE deny wins → 64 (no VIEW_CHANNEL)
            assert_eq!(
                effective_without_role & 32,
                0,
                "non-role member must lack VIEW_CHANNEL"
            );

            Err::<(), _>(CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
