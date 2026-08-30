//! The `RoleRepository` and `MemberRepository` doubles the scenarios run
//! against.
//!
//! `mockall::automock` derives both types from their ports, so there is no
//! hand-written adapter to keep in step with the trait. What it does not
//! derive is state: a mock is moved into whichever service it is handed to
//! and cannot be read back, so roles, members and their assignments live in
//! a store the `World` owns and every stub closes over — same shape as
//! `quote_bdd::repository`, just carrying two aggregates instead of one,
//! because a role assignment touches both.
//!
//! Only the methods these scenarios exercise are stubbed. Reaching an
//! unstubbed one fails the step with mockall's own message, which names the
//! method, and adding it here is then a handful of lines.

use std::sync::{Arc, Mutex, MutexGuard};

use mestier_core::{Member, MemberId, MockMemberRepository, MockRoleRepository, Role, RoleId};

/// Shared between the `World` and every mock built from it.
#[derive(Debug, Default)]
pub struct StoreData {
    pub roles: Vec<Role>,
    pub members: Vec<Member>,
    /// One entry per (member, role) assignment — mirrors the `member_roles`
    /// join table, `ON CONFLICT DO NOTHING` on assign and idempotent on
    /// unassign, same as `PgMemberRepository`.
    pub member_roles: Vec<(MemberId, RoleId)>,
}

pub type Store = Arc<Mutex<StoreData>>;

/// Locked for the length of a synchronous block and never across an
/// `await`, which is what keeps the stubbed futures `Send`. A poisoned
/// mutex means a step already panicked and the run is over.
pub fn lock(store: &Store) -> MutexGuard<'_, StoreData> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A fresh mock over `store`'s roles, rebuilt on every step because the
/// previous one was consumed by the service it was handed to.
pub fn stubbed_roles(store: &Store) -> MockRoleRepository {
    let mut repo = MockRoleRepository::new();

    let s = store.clone();
    repo.expect_insert().returning(move |role| {
        lock(&s).roles.push(role.clone());
        let role = role.clone();
        Box::pin(async move { Ok(role) })
    });

    let s = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = lock(&s).roles.iter().find(|r| r.id == id).cloned();
        Box::pin(async move { Ok(found) })
    });

    let s = store.clone();
    repo.expect_list_by_organization().returning(move |org_id| {
        let roles: Vec<Role> = lock(&s)
            .roles
            .iter()
            .filter(|r| r.organization_id == org_id)
            .cloned()
            .collect();
        Box::pin(async move { Ok(roles) })
    });

    let s = store.clone();
    repo.expect_update().returning(move |role| {
        let mut guard = lock(&s);
        if let Some(existing) = guard.roles.iter_mut().find(|r| r.id == role.id) {
            *existing = role.clone();
        }
        let role = role.clone();
        Box::pin(async move { Ok(role) })
    });

    let s = store.clone();
    repo.expect_delete().returning(move |id| {
        lock(&s).roles.retain(|r| r.id != id);
        Box::pin(async move { Ok(()) })
    });

    let s = store.clone();
    repo.expect_count_assigned_members().returning(move |id| {
        let count = lock(&s)
            .member_roles
            .iter()
            .filter(|(_, role_id)| *role_id == id)
            .count();
        Box::pin(async move { Ok(count as i64) })
    });

    repo
}

/// A fresh mock over `store`'s members and assignments, rebuilt on every
/// step for the same reason as [`stubbed_roles`].
pub fn stubbed_members(store: &Store) -> MockMemberRepository {
    let mut repo = MockMemberRepository::new();

    let s = store.clone();
    repo.expect_find_by_id().returning(move |id| {
        let found = lock(&s).members.iter().find(|m| m.id == id).cloned();
        Box::pin(async move { Ok(found) })
    });

    let s = store.clone();
    repo.expect_assign_role()
        .returning(move |member_id, role_id| {
            let mut guard = lock(&s);
            if !guard
                .member_roles
                .iter()
                .any(|(m, r)| *m == member_id && *r == role_id)
            {
                guard.member_roles.push((member_id, role_id));
            }
            Box::pin(async move { Ok(()) })
        });

    let s = store.clone();
    repo.expect_unassign_role()
        .returning(move |member_id, role_id| {
            lock(&s)
                .member_roles
                .retain(|(m, r)| !(*m == member_id && *r == role_id));
            Box::pin(async move { Ok(()) })
        });

    let s = store.clone();
    repo.expect_list_role_ids().returning(move |member_id| {
        let ids: Vec<RoleId> = lock(&s)
            .member_roles
            .iter()
            .filter(|(m, _)| *m == member_id)
            .map(|(_, r)| *r)
            .collect();
        Box::pin(async move { Ok(ids) })
    });

    repo
}
