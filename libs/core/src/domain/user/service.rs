use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    User, UserId,
    domain::user::commands::{CreateUserCommand, UpsertUserBySubCommand},
    domain::user::ports::UserRepository,
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
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.repo.upsert_by_email(&user).await
    }

    pub async fn find_by_email(&mut self, email: &str) -> Result<Option<User>, CoreError> {
        self.repo.find_by_email(email).await
    }

    pub async fn find_by_sub(&mut self, sub: &str) -> Result<Option<User>, CoreError> {
        self.repo.find_by_sub(sub).await
    }

    /// Looks a user up by primary key — used to resolve the account behind
    /// an occupied organization seat (`member.user_id`) on the item member
    /// routes. Never surfaces `id`/`sub` itself; callers project only
    /// `email`/`name` into the HTTP response.
    pub async fn find_by_id(&mut self, id: UserId) -> Result<Option<User>, CoreError> {
        self.repo.find_by_id(id).await
    }

    /// Every active user among `ids`, in one query — the list-route
    /// counterpart of [`Self::find_by_id`], avoiding one lookup per member.
    pub async fn list_by_ids(&mut self, ids: &[UserId]) -> Result<Vec<User>, CoreError> {
        self.repo.list_by_ids(ids).await
    }

    #[tracing::instrument(skip(self), fields(user.sub = %command.sub, user.email = %command.email), err)]
    pub async fn reconcile_upsert(
        &mut self,
        command: UpsertUserBySubCommand,
    ) -> Result<User, CoreError> {
        self.repo.upsert_by_sub(command).await
    }

    #[tracing::instrument(skip(self), fields(user.sub = %sub), err)]
    pub async fn reconcile_deletion(&mut self, sub: &str) -> Result<(), CoreError> {
        self.repo.soft_delete_by_sub(sub).await
    }

    pub async fn list(&mut self) -> Result<Vec<User>, CoreError> {
        self.repo.list_active().await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

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

    fn make_user() -> User {
        let now = Utc::now();
        User {
            id: UserId(Uuid::new_v4()),
            email: "alice@example.com".into(),
            username: "alice".into(),
            name: "Alice".into(),
            sub: "sub-1".into(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn upsert_cmd() -> UpsertUserBySubCommand {
        UpsertUserBySubCommand {
            sub: "sub-1".into(),
            email: "alice@example.com".into(),
            username: "alice".into(),
            name: Some("Alice".into()),
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
                deleted_at: u.deleted_at,
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

    #[tokio::test]
    async fn find_by_sub_delegates_to_repo() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_sub()
            .with(mockall::predicate::eq("sub-1"))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = UserService::new(repo);
        let user = service.find_by_sub("sub-1").await.unwrap();

        assert!(user.is_none());
    }

    #[tokio::test]
    async fn find_by_id_delegates_to_repo() {
        let id = UserId(Uuid::new_v4());
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_id()
            .with(mockall::predicate::eq(id))
            .times(1)
            .returning(move |_| {
                let mut user = make_user();
                user.id = id;
                Box::pin(async move { Ok(Some(user)) })
            });

        let mut service = UserService::new(repo);
        let user = service.find_by_id(id).await.unwrap();

        assert_eq!(user.unwrap().id, id);
    }

    #[tokio::test]
    async fn list_by_ids_delegates_to_repo() {
        let ids = vec![UserId(Uuid::new_v4()), UserId(Uuid::new_v4())];
        let mut repo = MockUserRepository::new();
        repo.expect_list_by_ids()
            .withf({
                let ids = ids.clone();
                move |arg| arg == ids.as_slice()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![make_user()]) }));

        let mut service = UserService::new(repo);
        let users = service.list_by_ids(&ids).await.unwrap();

        assert_eq!(users.len(), 1);
    }

    #[tokio::test]
    async fn reconcile_upsert_delegates_to_repo() {
        let mut repo = MockUserRepository::new();
        repo.expect_upsert_by_sub()
            .times(1)
            .returning(|_| Box::pin(async { Ok(make_user()) }));

        let mut service = UserService::new(repo);
        let user = service.reconcile_upsert(upsert_cmd()).await.unwrap();

        assert_eq!(user.sub, "sub-1");
        assert!(user.deleted_at.is_none());
    }

    #[tokio::test]
    async fn reconcile_deletion_delegates_to_repo() {
        let mut repo = MockUserRepository::new();
        repo.expect_soft_delete_by_sub()
            .with(mockall::predicate::eq("sub-1"))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = UserService::new(repo);
        service.reconcile_deletion("sub-1").await.unwrap();
    }

    #[tokio::test]
    async fn reconcile_deletion_propagates_not_found() {
        let mut repo = MockUserRepository::new();
        repo.expect_soft_delete_by_sub()
            .returning(|_| Box::pin(async { Err(CoreError::NotFound) }));

        let mut service = UserService::new(repo);
        let err = service.reconcile_deletion("sub-ghost").await.unwrap_err();
        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_delegates_to_repo() {
        let mut repo = MockUserRepository::new();
        repo.expect_list_active()
            .times(1)
            .returning(|| Box::pin(async { Ok(vec![make_user()]) }));

        let mut service = UserService::new(repo);
        let users = service.list().await.unwrap();

        assert_eq!(users.len(), 1);
        assert_eq!(users[0].sub, "sub-1");
    }
}
