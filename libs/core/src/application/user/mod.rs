use common::CoreError;
use mestier_macros::transactional;

use crate::{
    User,
    application::MestierUseCase,
    domain::user::{commands::CreateUserCommand, service::UserService},
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
}
