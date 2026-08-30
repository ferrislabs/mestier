//! Step definitions for `features-role/role.feature`.
//!
//! Steps translate the sentences of a scenario into calls on the real
//! `MemberService`/`RoleService` and read the answer back. They hold no
//! business rule of their own: if a permission looks wrong here, the fix
//! belongs in `domain::role::service` or `domain::member::service`, never in
//! this file.

use chrono::Utc;
use common::generate_uuid_v7;
use cucumber::{given, then, when};
use mestier_core::{
    AssignRoleCommand, CreateRoleCommand, Member, MemberId, Permissions, Role, RoleId,
    UnassignRoleCommand, UpdateRoleCommand, policy,
};

use crate::{
    repository,
    world::{RoleWorld, system_actor},
};

#[given("an organization")]
async fn an_organization(world: &mut RoleWorld) {
    world.ensure_organization();
}

#[given("a member with no role")]
async fn a_member_with_no_role(world: &mut RoleWorld) {
    let member = seed_member(world);
    world.member_id = Some(member.id);
}

#[given(expr = "a role {string} with permissions {string}")]
async fn a_role_with_permissions(world: &mut RoleWorld, name: String, permissions: String) {
    let permissions = parse_permissions(&permissions);
    seed_role(world, &name, permissions, false);
}

#[given(expr = "a member holding the roles {string} and {string}")]
async fn a_member_holding_two_roles(world: &mut RoleWorld, first: String, second: String) {
    let member_id = seed_member(world).id;
    world.member_id = Some(member_id);

    for name in [first, second] {
        let permissions = fixture_permissions(&name);
        let role = seed_role(world, &name, permissions, false);
        repository::lock(&world.store)
            .member_roles
            .push((member_id, role.id));
    }
}

#[given(expr = "a member holding the role {string}")]
async fn a_member_holding_one_role(world: &mut RoleWorld, name: String) {
    let member_id = seed_member(world).id;
    world.member_id = Some(member_id);

    let permissions = fixture_permissions(&name);
    let role = seed_role(world, &name, permissions, false);
    repository::lock(&world.store)
        .member_roles
        .push((member_id, role.id));
}

#[given(expr = "the seeded role {string}")]
async fn the_seeded_role(world: &mut RoleWorld, name: String) {
    seed_role(world, &name, Permissions::MANAGE_MEMBERS, true);
}

#[when(expr = "a role {string} is created with permissions {string}")]
async fn a_role_is_created(world: &mut RoleWorld, name: String, permissions: String) {
    let organization_id = world.organization_id();
    let permissions = parse_permissions(&permissions);

    let outcome = world
        .role_service()
        .create_role(CreateRoleCommand {
            actor: system_actor(),
            organization_id,
            name: name.clone(),
            permissions,
        })
        .await;

    match outcome {
        Ok(role) => {
            world.roles_by_name.insert(name, role.id);
            world.outcome = Some(Ok(()));
        }
        Err(error) => world.outcome = Some(Err(error)),
    }
}

#[when(expr = "the member is assigned the role {string}")]
async fn the_member_is_assigned_the_role(world: &mut RoleWorld, name: String) {
    let member_id = world.member_id();
    let role_id = world.role_id(&name);

    let outcome = world
        .member_service()
        .assign_role(AssignRoleCommand {
            actor: system_actor(),
            member_id,
            role_id,
        })
        .await;

    world.outcome = Some(outcome);
}

#[when(expr = "the role {string} is unassigned from the member")]
async fn the_role_is_unassigned(world: &mut RoleWorld, name: String) {
    let member_id = world.member_id();
    let role_id = world.role_id(&name);

    let outcome = world
        .member_service()
        .unassign_role(UnassignRoleCommand {
            actor: system_actor(),
            member_id,
            role_id,
        })
        .await;

    world.outcome = Some(outcome);
}

#[when(expr = "renaming the role {string} to {string} is attempted")]
async fn renaming_the_role_is_attempted(world: &mut RoleWorld, name: String, new_name: String) {
    let role_id = world.role_id(&name);
    let permissions = repository::lock(&world.store)
        .roles
        .iter()
        .find(|role| role.id == role_id)
        .map(|role| role.permissions)
        .expect("the role the scenario just seeded is in the store");

    let outcome = world
        .role_service()
        .update_role(UpdateRoleCommand {
            actor: system_actor(),
            role_id,
            name: new_name,
            permissions,
        })
        .await;

    world.outcome = Some(outcome.map(|_| ()));
}

#[when(expr = "deleting the role {string} is attempted")]
async fn deleting_the_role_is_attempted(world: &mut RoleWorld, name: String) {
    let role_id = world.role_id(&name);

    let outcome = world
        .role_service()
        .delete_role(role_id, system_actor())
        .await;

    world.outcome = Some(outcome);
}

#[then(expr = "the role {string} holds exactly the permissions {string}")]
async fn the_role_holds_exactly(world: &mut RoleWorld, name: String, expected: String) {
    let role_id = world.role_id(&name);
    let role = repository::lock(&world.store)
        .roles
        .iter()
        .find(|role| role.id == role_id)
        .cloned()
        .expect("the role the scenario just created is in the store");

    assert_eq!(
        sorted_names(role.permissions),
        parse_expected_names(&expected)
    );
}

#[then(expr = "the member's aggregated permissions are exactly {string}")]
async fn aggregated_permissions_are_exactly(world: &mut RoleWorld, expected: String) {
    let member_id = world.member_id();
    let organization_id = world.organization_id();

    let role_ids = world
        .member_service()
        .list_role_ids(member_id, system_actor())
        .await
        .expect("listing the member's role ids succeeds");
    let roles = world
        .role_service()
        .list_roles(organization_id, system_actor())
        .await
        .expect("listing the organization's roles succeeds");
    let member = repository::lock(&world.store)
        .members
        .iter()
        .find(|member| member.id == member_id)
        .cloned()
        .expect("the member the scenario seeded is in the store");

    let aggregated = policy::resolve_org_permissions(&member, &role_ids, &roles);

    assert_eq!(sorted_names(aggregated), parse_expected_names(&expected));
}

#[then("the attempt is refused")]
async fn the_attempt_is_refused(world: &mut RoleWorld) {
    let outcome = world
        .outcome
        .as_ref()
        .expect("a scenario acts before asserting on the answer");

    assert!(
        outcome.is_err(),
        "the attempt succeeded when the scenario expected a refusal"
    );
}

/// Seeds a member directly into the store — a `Given`, not a service call,
/// same convention `quote_bdd::steps::an_existing_quote` uses for state a
/// scenario states as already true rather than produces.
fn seed_member(world: &mut RoleWorld) -> Member {
    let organization_id = world.ensure_organization();
    let now = Utc::now();
    let member = Member {
        id: MemberId(generate_uuid_v7()),
        organization_id,
        user_id: None,
        last_name: "Fixture".to_owned(),
        first_name: None,
        joined_at: None,
        created_at: now,
        deleted_at: None,
    };

    repository::lock(&world.store).members.push(member.clone());
    member
}

/// Seeds a role directly into the store and registers it under `name` so a
/// later step can refer back to it — same convention as [`seed_member`].
fn seed_role(world: &mut RoleWorld, name: &str, permissions: Permissions, is_seeded: bool) -> Role {
    let organization_id = world.ensure_organization();
    let now = Utc::now();
    let role = Role {
        id: RoleId(generate_uuid_v7()),
        organization_id,
        name: name.to_owned(),
        permissions,
        is_seeded,
        created_at: now,
        updated_at: now,
    };

    repository::lock(&world.store).roles.push(role.clone());
    world.roles_by_name.insert(name.to_owned(), role.id);
    role
}

/// The fixed permission set behind a fixture role name reused across
/// scenarios without a fresh `Given a role "X" with permissions "..."` —
/// keeps `Planner`/`Accountant` meaning one thing everywhere they appear.
fn fixture_permissions(role_name: &str) -> Permissions {
    match role_name {
        "Planner" => Permissions::VIEW_PLANNING | Permissions::MANAGE_PLANNING,
        "Accountant" => Permissions::VIEW_COST | Permissions::VIEW_REPORTS,
        other => panic!("no fixture permissions defined for role `{other}`"),
    }
}

/// Reads a comma-separated list of permission bit names, e.g.
/// `"VIEW_COST, VIEW_REPORTS"`. Scenarios name bits in upper case, which is
/// how `Permissions::NAMED` itself spells them.
fn parse_permissions(raw: &str) -> Permissions {
    let names = permission_names(raw);
    Permissions::from_names(&names).unwrap_or_else(|error| panic!("{error}"))
}

/// Every named bit `permissions` carries, sorted so a `Then` step can
/// compare against a hand-written list without caring about
/// `Permissions::NAMED`'s declared order.
fn sorted_names(permissions: Permissions) -> Vec<String> {
    let mut names: Vec<String> = permissions
        .granted_names()
        .into_iter()
        .map(str::to_owned)
        .collect();
    names.sort_unstable();
    names
}

/// The inverse of [`sorted_names`], read off a scenario's own expectation
/// string — including the empty string, which names no permission at all.
fn parse_expected_names(raw: &str) -> Vec<String> {
    let mut names = permission_names(raw);
    names.sort_unstable();
    names
}

fn permission_names(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .collect()
}
