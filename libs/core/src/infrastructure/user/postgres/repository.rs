use common::CoreError;
use mestier_macros::repository;

use crate::{
    User,
    domain::user::ports::UserRepository,
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
            RETURNING id, email, username, display_name, sub, created_at, updated_at
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
    async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            UserRow,
            r#"
            SELECT id, email, username, display_name, sub, created_at, updated_at
            FROM users
            WHERE email = $1
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
            SELECT id, email, username, display_name, sub, created_at, updated_at
            FROM users
            WHERE sub = $1
            "#,
            sub,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }
}
