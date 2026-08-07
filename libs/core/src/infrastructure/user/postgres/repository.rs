use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;

use crate::{
    User, UserId,
    domain::user::{commands::UpsertUserBySubCommand, ports::UserRepository},
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        user::postgres::model::UserRow,
    },
};

#[repository(domain = User, backend = Postgres)]
pub struct PgUserRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgUserRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> UserRepository for PgUserRepository<'tx> {
    #[tracing::instrument(skip(self, user), fields(db.system = "postgresql", db.operation = "upsert", db.table = "users", user.email = %user.email), err)]
    async fn upsert_by_email(&mut self, user: &User) -> Result<User, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, email, username, display_name, sub, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (email) DO UPDATE SET
                username     = EXCLUDED.username,
                display_name = EXCLUDED.display_name,
                sub          = EXCLUDED.sub,
                updated_at   = EXCLUDED.updated_at
            RETURNING id, email, username, display_name, sub, deleted_at, created_at, updated_at
            "#,
            user.id.0,
            user.email,
            user.username,
            user.name,
            user.sub,
            user.created_at,
            user.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "users"), err)]
    async fn find_by_id(&mut self, id: UserId) -> Result<Option<User>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, display_name, sub, deleted_at, created_at, updated_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "users"), err)]
    async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, display_name, sub, deleted_at, created_at, updated_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "users"), err)]
    async fn find_by_sub(&mut self, sub: &str) -> Result<Option<User>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, display_name, sub, deleted_at, created_at, updated_at
            FROM users
            WHERE sub = $1 AND deleted_at IS NULL
            "#,
            sub,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "upsert", db.table = "users", user.sub = %command.sub), err)]
    async fn upsert_by_sub(&mut self, command: UpsertUserBySubCommand) -> Result<User, CoreError> {
        let mut tx = self.tx.lock().await;
        let id = generate_uuid_v7();
        let row = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, email, username, display_name, sub, created_at, updated_at)
            VALUES ($5, $1, $2, $3, $4, now(), now())
            ON CONFLICT (sub) DO UPDATE SET
                email        = EXCLUDED.email,
                username     = EXCLUDED.username,
                display_name = EXCLUDED.display_name,
                deleted_at   = NULL,
                updated_at   = now()
            RETURNING id, email, username, display_name, sub, deleted_at, created_at, updated_at
            "#,
            command.email,
            command.username,
            command.name.unwrap_or_default(),
            command.sub,
            id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "update", db.table = "users", user.sub = %sub), err)]
    async fn soft_delete_by_sub(&mut self, sub: &str) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET deleted_at = now(), updated_at = now()
            WHERE sub = $1 AND deleted_at IS NULL
            "#,
            sub,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "users"), err)]
    async fn list_active(&mut self) -> Result<Vec<User>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, display_name, sub, deleted_at, created_at, updated_at
            FROM users
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
