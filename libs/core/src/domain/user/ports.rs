use common::CoreError;

use crate::User;
use crate::UserId;
use crate::domain::user::commands::UpsertUserBySubCommand;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
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

    /// Every active user among `ids`, in one query. Added additively for
    /// the planning read model, which resolves the `display_name` of
    /// member-only resources without one lookup per member (see the
    /// planning module design doc's N+1 warning).
    fn list_by_ids(
        &mut self,
        ids: &[UserId],
    ) -> impl Future<Output = Result<Vec<User>, CoreError>> + Send;
}
