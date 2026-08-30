use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    UserId,
    domain::{
        member::{Member, MemberId, ports::MemberRepository},
        organization::OrganizationId,
        role::RoleId,
    },
    infrastructure::{
        member::postgres::model::MemberRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Member, backend = Postgres)]
pub struct PgMemberRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMemberRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> MemberRepository for PgMemberRepository<'tx> {
    #[tracing::instrument(skip(self, member), fields(db.system = "postgresql", db.operation = "insert", db.table = "organization_members"), err)]
    async fn insert(&mut self, member: &Member) -> Result<Member, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberRow,
            r#"
            INSERT INTO organization_members (id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            "#,
            member.id.0,
            member.organization_id.0,
            member.user_id.map(|id| id.0),
            member.last_name,
            member.first_name,
            member.joined_at,
            member.created_at,
            member.deleted_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_members"), err)]
    async fn find_by_id(&mut self, member_id: MemberId) -> Result<Option<Member>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberRow,
            r#"
            SELECT id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            FROM organization_members
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            member_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_members"), err)]
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Member>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberRow,
            r#"
            SELECT id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            FROM organization_members
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY last_name ASC, first_name ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM organization_members WHERE organization_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_members"), err)]
    async fn list_active_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Member>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberRow,
            r#"
            SELECT id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            FROM organization_members
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY last_name ASC, first_name ASC, created_at ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "insert", db.table = "member_roles"), err)]
    async fn assign_role(&mut self, member_id: MemberId, role_id: RoleId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO member_roles (id, member_id, role_id)
            VALUES (gen_random_uuid(), $1, $2)
            ON CONFLICT (member_id, role_id) DO NOTHING
            "#,
            member_id.0,
            role_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "delete", db.table = "member_roles"), err)]
    async fn unassign_role(
        &mut self,
        member_id: MemberId,
        role_id: RoleId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            DELETE FROM member_roles
            WHERE member_id = $1 AND role_id = $2
            "#,
            member_id.0,
            role_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organization_members"), err)]
    async fn find_by_org_and_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<Option<Member>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberRow,
            r#"
            SELECT id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            FROM organization_members
            WHERE organization_id = $1 AND user_id = $2 AND deleted_at IS NULL
            "#,
            organization_id.0,
            user_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self, member), fields(db.system = "postgresql", db.operation = "update", db.table = "organization_members"), err)]
    async fn update(&mut self, member: &Member) -> Result<Member, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberRow,
            r#"
            UPDATE organization_members
            SET user_id = $2, last_name = $3, first_name = $4, joined_at = $5
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
            "#,
            member.id.0,
            member.user_id.map(|id| id.0),
            member.last_name,
            member.first_name,
            member.joined_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "update", db.table = "organization_members"), err)]
    async fn soft_delete(
        &mut self,
        member_id: MemberId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE organization_members
            SET deleted_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            member_id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "member_roles"), err)]
    async fn list_role_ids(&mut self, member_id: MemberId) -> Result<Vec<RoleId>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT role_id
            FROM member_roles
            WHERE member_id = $1
            "#,
            member_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| RoleId(r.role_id)).collect())
    }
}
