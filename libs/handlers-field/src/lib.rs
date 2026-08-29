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
        .typed_post(field::declare::handler)
        .typed_post(field::stop::handler)
        .typed_post(field::recover::handler)
        .typed_post(field::attach_photo::handler)
        .typed_post(field::end_day::handler)
        .typed_post(field::report_assignment::handler)
        .typed_patch(field::amend_report::handler)
        .typed_delete(field::withdraw_report::handler)
        .typed_get(field::list_reports::handler)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}

#[cfg(test)]
mod tests {
    /// #305's own scope note: "handlers-field: reads and writes the
    /// caller's own data only, so it needs no new bit. Assert that rather
    /// than assume it." A route here refuses by *resolving no actor at
    /// all* (`resolve_field_actor` returns `Forbidden` unless the caller
    /// is themselves the employee), not by checking a business
    /// permission — there is nothing to grant or withhold, and adding a
    /// bit would be a second, redundant guard next to a check that
    /// already reads no wider than "you are you". This is a structural
    /// regression guard, not a comment promising it: a future route added
    /// under `field/` that skips `resolve_field_actor` — and so might
    /// read or write on someone else's behalf — fails this test rather
    /// than silently regressing the invariant.
    #[test]
    fn every_field_handler_resolves_its_own_actor() {
        let field_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/field");
        let mut checked = 0;

        for entry in std::fs::read_dir(&field_dir).expect("read src/field") {
            let entry = entry.expect("read dir entry");
            let path = entry.path();
            if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }

            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            assert!(
                source.contains("resolve_field_actor"),
                "{} does not call resolve_field_actor — a field route must resolve \
                 its own caller, never act on an id taken from the request",
                path.display()
            );
            checked += 1;
        }

        assert!(checked > 0, "no field handler files were found to check");
    }
}
