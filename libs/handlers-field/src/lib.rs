use auth::Identity;
use axum::{Router, middleware::from_fn_with_state};
use axum_extra::routing::RouterExt;
use handlers::{
    ApiError, AppState, auth::auth_middleware, rate_limit::rate_limit_middleware, resolve_user_id,
};
use mestier_core::{EmployeeId, MemberId, OrganizationId};

pub mod field;
pub mod paths;
pub mod response;

pub const TAG: &str = "field";

/// Who the caller is, on the field app.
///
/// Two identities, both needed: jobs are assigned to a **member**, while a
/// time entry and its hourly cost belong to an **employee**. `member_id` is
/// the join key between them.
pub struct FieldActor {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub employee_id: EmployeeId,
}

/// Resolves the connected account into the employee it stands for.
///
/// This is stricter than the membership check the other modules do. There,
/// belonging to the organization is enough; here the caller must also *be* an
/// employee, because clocking writes time against an hourly rate. A member
/// with no employee profile is refused rather than given a blank one, since
/// inventing a profile would silently create a person who costs nothing.
///
/// Every field route resolves the actor itself and acts only on it. No route
/// takes an employee id from the request, which is what stops one employee
/// clocking on behalf of another.
pub async fn resolve_field_actor(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
) -> Result<FieldActor, ApiError> {
    let user_id = resolve_user_id(state, identity).await?;
    let member = state
        .usecase
        .find_membership(organization_id, user_id)
        .await?
        .ok_or(ApiError::Forbidden)?;
    let employee = state
        .usecase
        .get_employee_by_member(member.id)
        .await
        .map_err(|_| ApiError::Forbidden)?;
    Ok(FieldActor {
        organization_id,
        member_id: member.id,
        employee_id: employee.id,
    })
}

pub fn router(state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(field::my_tasks::handler)
        .typed_get(field::current::handler)
        .typed_post(field::start::handler)
        .typed_post(field::stop::handler)
        .typed_post(field::recover::handler)
        .typed_post(field::attach_photo::handler)
        .typed_post(field::end_day::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
