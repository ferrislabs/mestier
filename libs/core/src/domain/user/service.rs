use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    User, UserId, domain::user::commands::CreateUserCommand, domain::user::ports::UserRepository,
};

pub struct UserService<U>
where
    U: UserRepository,
{
    repo: U,
}

impl<U> UserService<U>
where
    U: UserRepository,
{
    pub fn new(repo: U) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip(self), fields(user.email = %command.email, user.username = %command.username), err)]
    pub async fn create_user(&mut self, command: CreateUserCommand) -> Result<User, CoreError> {
        let now = Utc::now();
        let user = User {
            id: UserId(generate_uuid_v7()),
            name: command.name,
            username: command.username,
            email: command.email,
            sub: command.sub,
            created_at: now,
            updated_at: now,
        };

        self.repo.upsert_by_email(&user).await
    }

    pub async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
        self.repo.find_by_email(email).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user::ports::MockUserRepository;

    fn cmd() -> CreateUserCommand {
        CreateUserCommand {
            name: "Alice".into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            sub: "sub-1".into(),
        }
    }

    #[tokio::test]
    async fn create_user_persists_via_repo() {
        let mut repo = MockUserRepository::new();
        repo.expect_upsert_by_email().times(1).returning(|u| {
            let cloned = User {
                id: u.id,
                email: u.email.clone(),
                username: u.username.clone(),
                name: u.name.clone(),
                sub: u.sub.clone(),
                created_at: u.created_at,
                updated_at: u.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = UserService::new(repo);
        let user = service.create_user(cmd()).await.unwrap();

        assert_eq!(user.email, "alice@example.com");
        assert_eq!(user.username, "alice");
    }

    #[tokio::test]
    async fn create_user_propagates_repo_error() {
        let mut repo = MockUserRepository::new();
        repo.expect_upsert_by_email()
            .returning(|_| Box::pin(async { Err(CoreError::Database("boom".into())) }));

        let mut service = UserService::new(repo);

        let err = service.create_user(cmd()).await.unwrap_err();
        assert!(matches!(err, CoreError::Database(_)));
    }
}
