use common::CoreError;

use crate::User;
use crate::UserId;
use crate::domain::user::commands::UpsertUserBySubCommand;

#[cfg_attr(test, mockall::automock)]
pub trait UserRepository: Send {
    fn upsert_by_email(
        &mut self,
        user: &User,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;

    /// Looks a user up by primary key. Needed wherever a caller only holds a
    /// `UserId` reference — e.g. resolving a `member` assignee's
    /// `display_name` when `PATCH /work-orders/{id}` provisions an employee
    /// record on the fly.
    fn find_by_id(
        &mut self,
        id: UserId,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;

    fn find_by_email(
        &mut self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;

    fn find_by_sub(
        &mut self,
        sub: &str,
    ) -> impl Future<Output = Result<Option<User>, CoreError>> + Send;

    fn upsert_by_sub(
        &mut self,
        command: UpsertUserBySubCommand,
    ) -> impl Future<Output = Result<User, CoreError>> + Send;

    fn soft_delete_by_sub(
        &mut self,
        sub: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_active(&mut self) -> impl Future<Output = Result<Vec<User>, CoreError>> + Send;
}
