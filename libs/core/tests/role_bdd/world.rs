//! The state a scenario carries from one step to the next.
//!
//! It knows the two domain services and the repository doubles they run
//! against, and nothing about Postgres, Axum or the transaction machinery —
//! same discipline as `quote_bdd::world`. Two services, not one, because
//! assigning or unassigning a role is a `MemberService` operation while
//! creating, renaming and deleting a role is a `RoleService` one; both read
//! the same shared store.
//!
//! Every scenario acts as `Subject::system()`: the system subject
//! short-circuits `policy::enrich_for_organization` and is always allowed by
//! `LocalPolicyEngine` (see `authz::infrastructure::local`), so these
//! scenarios exercise the role/permission lifecycle itself rather than the
//! authorization gate in front of it — that gate already has its own
//! coverage in `domain::member::service`'s and `domain::role::service`'s
//! unit tests, and in the `handlers-member` end-to-end suite.

use std::collections::HashMap;

use authz::{LocalPolicyEngine, Subject};
use common::CoreError;
use mestier_core::{
    MemberId, MemberService, MockMemberRepository, MockRoleRepository, MockUserRepository,
    OrganizationId, Permissions, RoleId, RoleService,
};

use crate::repository::{self, Store};

pub type Members =
    MemberService<MockMemberRepository, MockRoleRepository, MockUserRepository, LocalPolicyEngine>;
pub type Roles = RoleService<MockRoleRepository, MockMemberRepository, LocalPolicyEngine>;

#[derive(Debug, Default, cucumber::World)]
pub struct RoleWorld {
    pub store: Store,
    pub organization_id: Option<OrganizationId>,
    pub member_id: Option<MemberId>,
    /// Every role a scenario named, keyed by its Gherkin name — so a later
    /// step can say "the role \"Planner\"" without carrying the id itself.
    pub roles_by_name: HashMap<String, RoleId>,
    /// The outcome of the last write a `When` step performed, kept whole so
    /// a `Then` step can assert on a refusal instead of unwinding on it.
    pub outcome: Option<Result<(), CoreError>>,
}

impl RoleWorld {
    /// A fresh `MemberService` over mocks freshly stubbed against the
    /// shared store. Short-lived by design, mirroring `QuoteWorld::service`.
    pub fn member_service(&self) -> Members {
        MemberService::new(
            repository::stubbed_members(&self.store),
            repository::stubbed_roles(&self.store),
            MockUserRepository::new(),
            authorizer(),
        )
    }

    /// A fresh `RoleService`, same rebuild-per-call discipline.
    pub fn role_service(&self) -> Roles {
        RoleService::new(
            repository::stubbed_roles(&self.store),
            repository::stubbed_members(&self.store),
            authorizer(),
        )
    }

    /// Sets the organization if a prior `Given` has not already, and
    /// returns it either way — several `Given` steps seed a member or a
    /// role directly and are written to stand on their own, with no
    /// separate "Given an organization" required first.
    pub fn ensure_organization(&mut self) -> OrganizationId {
        let organization_id = self
            .organization_id
            .get_or_insert_with(|| OrganizationId(common::generate_uuid_v7()));
        *organization_id
    }

    pub fn organization_id(&self) -> OrganizationId {
        self.organization_id
            .expect("a scenario states the organization before reading it")
    }

    pub fn member_id(&self) -> MemberId {
        self.member_id
            .expect("a scenario seeds a member before reading it")
    }

    /// The id behind a role name a prior step registered.
    pub fn role_id(&self, name: &str) -> RoleId {
        *self
            .roles_by_name
            .get(name)
            .unwrap_or_else(|| panic!("no role named `{name}` was seeded in this scenario"))
    }
}

/// The real `LocalPolicyEngine`, wired with the same `role.manage`/
/// `role.assign` -> `MANAGE_ROLES` mapping as `default_authorizer` — built
/// fresh rather than imported, since `default_authorizer` lives in
/// `application` alongside every other bounded context's action and pulls
/// in wiring this suite has no use for. Every scenario acts as
/// `Subject::system()`, which bypasses this map entirely (see the module
/// doc), so the mapping here documents the real boundary without any
/// scenario depending on its exact bits.
fn authorizer() -> LocalPolicyEngine {
    LocalPolicyEngine::builder()
        .action("role.manage", Permissions::MANAGE_ROLES.bits())
        .action("role.assign", Permissions::MANAGE_ROLES.bits())
        .build()
}

pub fn system_actor() -> Subject {
    Subject::system()
}
