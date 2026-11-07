use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use uuid::Uuid;

use super::repository::PgUserRepository;
use crate::{
    User,
    domain::user::system::{
        SYSTEM_USER_EMAIL, SYSTEM_USER_NAME, SYSTEM_USER_SUB, SYSTEM_USER_USERNAME,
        SystemUserRepository,
    },
    infrastructure::{postgres::error::map_sqlx_error, user::postgres::model::UserRow},
};

type SystemUserColumns = (
    Uuid,
    String,
    String,
    String,
    String,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
    DateTime<Utc>,
);

impl<'tx> SystemUserRepository for PgUserRepository<'tx> {
    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "upsert", db.table = "users"), err)]
    async fn ensure_system_user(&mut self, id: UserId) -> Result<User, CoreError> {
        let mut tx = self.tx().lock().await;
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) \
             VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        )
        .bind(id.0)
        .bind(SYSTEM_USER_EMAIL)
        .bind(SYSTEM_USER_USERNAME)
        .bind(SYSTEM_USER_NAME)
        .bind(SYSTEM_USER_SUB)
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let columns = sqlx::query_as::<_, SystemUserColumns>(
            "SELECT id, email, username, display_name, sub, deleted_at, created_at, updated_at \
             FROM users WHERE sub = $1",
        )
        .bind(SYSTEM_USER_SUB)
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or_else(|| CoreError::Internal("system user could not be created".to_owned()))?;

        Ok(UserRow {
            id: columns.0,
            email: columns.1,
            username: columns.2,
            display_name: columns.3,
            sub: columns.4,
            deleted_at: columns.5,
            created_at: columns.6,
            updated_at: columns.7,
        }
        .into())
    }
}
