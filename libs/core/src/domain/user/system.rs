use common::{CoreError, UserId};

use crate::User;

pub const SYSTEM_USER_SUB: &str = "system:automation";
pub const SYSTEM_USER_EMAIL: &str = "automation@system.invalid";
pub const SYSTEM_USER_USERNAME: &str = "system-automation";
pub const SYSTEM_USER_NAME: &str = "Automatisation";

pub fn is_reserved_sub(sub: &str) -> bool {
    sub == SYSTEM_USER_SUB
}

pub fn is_reserved_email(email: &str) -> bool {
    email.trim().eq_ignore_ascii_case(SYSTEM_USER_EMAIL)
}

pub fn is_system_user(user: &User) -> bool {
    is_reserved_sub(&user.sub)
}

pub fn refuse_reserved(sub: &str, email: &str) -> Result<(), CoreError> {
    if is_reserved_sub(sub) || is_reserved_email(email) {
        return Err(CoreError::Forbidden {
            reason: Some("reserved identity".to_owned()),
        });
    }
    Ok(())
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SystemUserRepository: Send {
    fn ensure_system_user(
        &mut self,
        id: UserId,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;
}

pub struct SystemUserService<R>
where
    R: SystemUserRepository,
{
    repo: R,
}

impl<R> SystemUserService<R>
where
    R: SystemUserRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn ensure(&mut self) -> Result<User, CoreError> {
        self.repo
            .ensure_system_user(UserId(common::generate_uuid_v7()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn system_user() -> User {
        let now = Utc::now();
        User {
            id: UserId(uuid::Uuid::new_v4()),
            email: SYSTEM_USER_EMAIL.into(),
            username: SYSTEM_USER_USERNAME.into(),
            name: SYSTEM_USER_NAME.into(),
            sub: SYSTEM_USER_SUB.into(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn the_reserved_sub_and_email_are_refused() {
        assert!(refuse_reserved(SYSTEM_USER_SUB, "a@b.c").is_err());
        assert!(refuse_reserved("sub-1", "Automation@System.Invalid ").is_err());
        assert!(refuse_reserved("sub-1", "a@b.c").is_ok());
    }

    #[tokio::test]
    async fn ensure_delegates_to_the_repository() {
        let mut repo = MockSystemUserRepository::new();
        repo.expect_ensure_system_user()
            .times(1)
            .returning(|_| Box::pin(async { Ok(system_user()) }));

        let user = SystemUserService::new(repo).ensure().await.unwrap();

        assert!(is_system_user(&user));
    }
}
