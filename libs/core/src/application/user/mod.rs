use common::CoreError;
use mestier_macros::transactional;

use crate::{
    User, UserId,
    application::MestierUseCase,
    domain::user::{
        commands::{CreateUserCommand, UpsertUserBySubCommand},
        service::UserService,
    },
};

impl MestierUseCase {
    #[transactional(user)]
    pub async fn create_user(&self, command: CreateUserCommand) -> Result<User, CoreError> {
        let mut service = UserService::new(user_repository);
        service.create_user(command).await
    }

    #[transactional(user)]
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, CoreError> {
        let mut service = UserService::new(user_repository);
        service.find_by_email(email).await
    }

    #[transactional(user)]
    pub async fn find_user_by_sub(&self, sub: &str) -> Result<Option<User>, CoreError> {
        let mut service = UserService::new(user_repository);
        service.find_by_sub(sub).await
    }

    /// Resolves the account behind an occupied member seat
    /// (`Member::user_id`) for the item member routes — see
    /// `handlers-member`'s `MemberResponse`.
    #[transactional(user)]
    pub async fn find_user_by_id(&self, id: UserId) -> Result<Option<User>, CoreError> {
        let mut service = UserService::new(user_repository);
        service.find_by_id(id).await
    }

    /// One query for every account behind a page of member seats — the
    /// list-route counterpart of [`Self::find_user_by_id`].
    #[transactional(user)]
    pub async fn list_users_by_ids(&self, ids: &[UserId]) -> Result<Vec<User>, CoreError> {
        let mut service = UserService::new(user_repository);
        service.list_by_ids(ids).await
    }

    #[transactional(user)]
    pub async fn reconcile_user_upsert(
        &self,
        command: UpsertUserBySubCommand,
    ) -> Result<User, CoreError> {
        let mut service = UserService::new(user_repository);
        service.reconcile_upsert(command).await
    }

    #[transactional(user)]
    pub async fn reconcile_user_deletion(&self, sub: String) -> Result<(), CoreError> {
        let mut service = UserService::new(user_repository);
        service.reconcile_deletion(&sub).await
    }

    #[transactional(user)]
    pub async fn list_users(&self) -> Result<Vec<User>, CoreError> {
        let mut service = UserService::new(user_repository);
        service.list().await
    }
}
