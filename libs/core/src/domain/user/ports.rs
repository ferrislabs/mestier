use common::CoreError;

use crate::User;

#[cfg_attr(test, mockall::automock)]
pub trait UserRepository: Send {
    fn upsert_by_email(
        &mut self,
        user: &User,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;

    fn find_by_email(
        &mut self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;

    fn find_by_sub(
        &mut self,
        sub: &str,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;
}
