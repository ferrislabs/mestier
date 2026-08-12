use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    UserId,
    domain::{
        invitation::{Invitation, InvitationId, ports::InvitationRepository},
        organization::OrganizationId,
    },
    infrastructure::{
        invitation::postgres::model::InvitationRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Invitation, backend = Postgres)]
pub struct PgInvitationRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgInvitationRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> InvitationRepository for PgInvitationRepository<'tx> {
    #[tracing::instrument(skip(self, invitation), fields(db.system = "postgresql", db.operation = "insert", db.table = "organization_invitations"), err)]
    async fn insert(&mut self, invitation: &Invitation) -> Result<Invitation, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvitationRow,
            r#"
            INSERT INTO organization_invitations
                (id, organization_id, member_id, token_hash, expires_at, consumed_at, consumed_by_user_id, created_by_user_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, member_id, token_hash, expires_at, consumed_at, consumed_by_user_id, created_by_user_id, created_at
            "#,
            invitation.id.0,
            invitation.organization_id.0,
            invitation.member_id.map(|id| id.0),
            invitation.token_hash,
            invitation.expires_at,
            invitation.consumed_at,
            invitation.consumed_by_user_id.map(|id| id.0),
            invitation.created_by_user_id.0,
            invitation.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_invitations"), err)]
    async fn find_by_id(
        &mut self,
        invitation_id: InvitationId,
    ) -> Result<Option<Invitation>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvitationRow,
            r#"
            SELECT id, organization_id, member_id, token_hash, expires_at, consumed_at, consumed_by_user_id, created_by_user_id, created_at
            FROM organization_invitations
            WHERE id = $1
            "#,
            invitation_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    /// `FOR UPDATE`: a second `accept_invitation` for the same token, racing
    /// inside its own transaction, blocks here until the first commits —
    /// without it, both could read the row as pending before either writes
    /// `consumed_at`, and the same token would be accepted twice.
    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_invitations"), err)]
    async fn find_by_token_hash(
        &mut self,
        token_hash: &[u8],
    ) -> Result<Option<Invitation>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            InvitationRow,
            r#"
            SELECT id, organization_id, member_id, token_hash, expires_at, consumed_at, consumed_by_user_id, created_by_user_id, created_at
            FROM organization_invitations
            WHERE token_hash = $1
            FOR UPDATE
            "#,
            token_hash,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_invitations"), err)]
    async fn list_pending_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Invitation>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            InvitationRow,
            r#"
            SELECT id, organization_id, member_id, token_hash, expires_at, consumed_at, consumed_by_user_id, created_by_user_id, created_at
            FROM organization_invitations
            WHERE organization_id = $1 AND consumed_at IS NULL
            ORDER BY created_at DESC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "update", db.table = "organization_invitations"), err)]
    async fn mark_consumed(
        &mut self,
        invitation_id: InvitationId,
        consumed_at: DateTime<Utc>,
        consumed_by_user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE organization_invitations
            SET consumed_at = $2, consumed_by_user_id = $3
            WHERE id = $1 AND consumed_at IS NULL
            "#,
            invitation_id.0,
            consumed_at,
            consumed_by_user_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            // Lost the race the `FOR UPDATE` lock in `find_by_token_hash`
            // was meant to prevent, or the row disappeared between the two
            // calls — either way, the second acceptance must not succeed.
            return Err(CoreError::Conflict(
                "invitation already consumed".to_owned(),
            ));
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "delete", db.table = "organization_invitations"), err)]
    async fn revoke(&mut self, invitation_id: InvitationId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            DELETE FROM organization_invitations
            WHERE id = $1 AND consumed_at IS NULL
            "#,
            invitation_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
