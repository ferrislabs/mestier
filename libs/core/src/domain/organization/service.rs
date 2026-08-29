use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

/// Translates a Postgres unique-violation into a business-friendly message.
/// The infra layer surfaces `CoreError::Conflict(constraint_name)`; we map the
/// stable constraint identifier to a message safe to expose at the API.
fn map_organization_conflict(err: CoreError) -> CoreError {
    match err {
        CoreError::Conflict(constraint) => {
            let message = match constraint.as_str() {
                "organizations_slug_key" => "slug already taken",
                _ => "organization conflict",
            };
            CoreError::Conflict(message.to_owned())
        }
        other => other,
    }
}

use authz::{Authorizer, Resource};

use crate::{
    UserId,
    application::policy,
    domain::{
        member::{Member, MemberId, ports::MemberRepository},
        organization::{
            Organization, OrganizationId,
            commands::{
                CreateOrganizationCommand, UpdateLegalIdentityCommand, UpdateOrganizationCommand,
            },
            ports::OrganizationRepository,
        },
        role::{
            ADMIN_ROLE_NAME, MEMBER_ROLE_NAME, OWNER_ROLE_NAME, Permissions, Role, RoleId,
            default_admin_business_permissions, default_member_business_permissions,
            ports::RoleRepository,
        },
        task_label::{PRESET_TASK_LABELS, TaskLabel, TaskLabelId, ports::TaskLabelRepository},
        user::ports::UserRepository,
    },
};

pub struct OrganizationService<O, R, M, U, A>
where
    O: OrganizationRepository,
    R: RoleRepository,
    M: MemberRepository,
    U: UserRepository,
    A: Authorizer,
{
    organization_repository: O,
    role_repository: R,
    member_repository: M,
    user_repository: U,
    authz: A,
}

impl<O, R, M, U, A> OrganizationService<O, R, M, U, A>
where
    O: OrganizationRepository,
    R: RoleRepository,
    M: MemberRepository,
    U: UserRepository,
    A: Authorizer,
{
    pub fn new(
        organization_repository: O,
        role_repository: R,
        member_repository: M,
        user_repository: U,
        authz: A,
    ) -> Self {
        Self {
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        }
    }

    #[tracing::instrument(skip(self), fields(organization_id = %id.0), err)]
    pub async fn get_organization(
        &mut self,
        id: OrganizationId,
    ) -> Result<Organization, CoreError> {
        self.organization_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(sub = %sub), err)]
    pub async fn list_organizations_for_user(
        &mut self,
        sub: &str,
    ) -> Result<Vec<Organization>, CoreError> {
        let user = self
            .user_repository
            .find_by_sub(sub)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.organization_repository.list_for_user(user.id).await
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn list_organizations(
        &mut self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Organization>, u64), CoreError> {
        self.organization_repository
            .list_paginated(limit, offset)
            .await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %command.id.0, organization.slug = %command.slug), err)]
    pub async fn update_organization(
        &mut self,
        command: UpdateOrganizationCommand,
    ) -> Result<Organization, CoreError> {
        // 1. Load — required to authorize against actual org context.
        let mut organization = self
            .organization_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        // 2. Authorize — enrich the actor with the org membership /
        //    aggregated permission bitfield, then ask the policy engine.
        let actor = policy::enrich_for_organization(
            command.actor,
            organization.id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "organization.update",
            Resource::new("organization", organization.id.0.to_string())
                .with_property("mestier.slug", organization.slug.clone()),
        )
        .await?;

        // 3. Mutate.
        organization.name = command.name;
        organization.slug = command.slug;
        organization.field_clock_enabled = command.field_clock_enabled;
        organization.updated_at = Utc::now();

        self.organization_repository
            .update(&organization)
            .await
            .map_err(map_organization_conflict)
    }

    /// Replaces the legal-identity block in full — see
    /// `UpdateLegalIdentityCommand` for why a `None` field clears rather
    /// than skips. Gated by the same `organization.update` permission as
    /// name/slug: both are organization-level administration.
    #[tracing::instrument(skip(self), fields(organization_id = %command.id.0), err)]
    pub async fn update_legal_identity(
        &mut self,
        command: UpdateLegalIdentityCommand,
    ) -> Result<Organization, CoreError> {
        let mut organization = self
            .organization_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let actor = policy::enrich_for_organization(
            command.actor,
            organization.id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "organization.update",
            Resource::new("organization", organization.id.0.to_string())
                .with_property("mestier.slug", organization.slug.clone()),
        )
        .await?;

        organization.legal_name = command.legal_name;
        organization.legal_form = command.legal_form;
        organization.registration_number = command.registration_number;
        organization.vat_status = command.vat_status;
        organization.share_capital_cents = command.share_capital_cents;
        organization.address_line1 = command.address_line1;
        organization.address_line2 = command.address_line2;
        organization.address_postal_code = command.address_postal_code;
        organization.address_city = command.address_city;
        organization.address_country = command.address_country;
        organization.contact_email = command.contact_email;
        organization.contact_phone = command.contact_phone;
        organization.insurance_mention = command.insurance_mention;
        organization.vat_on_debits = command.vat_on_debits;
        organization.updated_at = Utc::now();

        self.organization_repository.update(&organization).await
    }

    #[tracing::instrument(skip(self), fields(organization_id = %id.0), err)]
    pub async fn soft_delete_organization(&mut self, id: OrganizationId) -> Result<(), CoreError> {
        self.organization_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.organization_repository
            .soft_delete(id, Utc::now())
            .await
    }

    /// Creates an organization together with its default roles, the
    /// owner's membership, and — the piece this workstream adds — its three
    /// preset task labels ("Réunion", "Déplacement", "Formation"). All in
    /// the same transaction: see `#[transactional(organization, role,
    /// member, user, authz, task_label)]` on
    /// `MestierUseCase::create_organization`. `task_label_repository` is a
    /// method-scoped generic rather than a field on `OrganizationService`
    /// itself, so every other use case on this service (get/list/update/
    /// soft_delete/leave) stays entirely unaware of the label aggregate.
    #[tracing::instrument(
        skip(self, task_label_repository),
        fields(organization.slug = %command.slug, owner_id = %command.owner_id.0),
        err
    )]
    pub async fn create_organization<TL>(
        &mut self,
        command: CreateOrganizationCommand,
        mut task_label_repository: TL,
    ) -> Result<Organization, CoreError>
    where
        TL: TaskLabelRepository,
    {
        let now = Utc::now();
        let owner_id = command.owner_id;

        let user = self
            .user_repository
            .find_by_sub(owner_id.to_string().as_str())
            .await?
            .ok_or(CoreError::NotFound)?;

        let organization = self
            .organization_repository
            .insert(&Organization {
                id: OrganizationId(generate_uuid_v7()),
                name: command.name,
                slug: command.slug,
                owner_id: user.id,
                legal_name: None,
                legal_form: None,
                registration_number: None,
                vat_status: None,
                share_capital_cents: None,
                address_line1: None,
                address_line2: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                contact_email: None,
                contact_phone: None,
                insurance_mention: None,
                quote_number_prefix: "DEV".to_owned(),
                invoice_number_prefix: "FAC".to_owned(),
                field_clock_enabled: false,
                vat_on_debits: false,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .map_err(map_organization_conflict)?;

        let owner_role = self
            .role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: OWNER_ROLE_NAME.into(),
                permissions: Permissions::ALL,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: ADMIN_ROLE_NAME.into(),
                permissions: Permissions::MANAGE_MEMBERS | default_admin_business_permissions(),
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.role_repository
            .insert(&Role {
                id: RoleId(generate_uuid_v7()),
                organization_id: organization.id,
                name: MEMBER_ROLE_NAME.into(),
                permissions: default_member_business_permissions(),
                created_at: now,
                updated_at: now,
            })
            .await?;

        let member = self
            .member_repository
            .insert(&Member {
                id: MemberId(generate_uuid_v7()),
                organization_id: organization.id,
                user_id: Some(user.id),
                last_name: user.name,
                first_name: None,
                joined_at: Some(now),
                created_at: now,
                deleted_at: None,
            })
            .await?;

        self.member_repository
            .assign_role(member.id, owner_role.id)
            .await?;

        // Presets, not privileges: nothing distinguishes these rows from a
        // label created by hand afterwards — the user may rename or delete
        // any of them (see the planning module design doc).
        for (name, color) in PRESET_TASK_LABELS {
            task_label_repository
                .insert(&TaskLabel {
                    id: TaskLabelId(generate_uuid_v7()),
                    organization_id: organization.id,
                    name: name.to_owned(),
                    color: color.to_owned(),
                    created_at: now,
                    updated_at: now,
                })
                .await?;
        }

        Ok(organization)
    }

    #[tracing::instrument(skip(self), fields(organization_id = %organization_id.0, user_id = %user_id.0), err)]
    pub async fn leave_organization(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let organization = self.get_organization(organization_id).await?;

        if organization.owner_id == user_id {
            return self.soft_delete_organization(organization_id).await;
        }

        let member = self
            .member_repository
            .find_by_org_and_user(organization_id, user_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        self.member_repository
            .soft_delete(member.id, Utc::now())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        User, UserId,
        application::policy,
        domain::{
            member::ports::MockMemberRepository, organization::ports::MockOrganizationRepository,
            role::ports::MockRoleRepository, task_label::ports::MockTaskLabelRepository,
            user::ports::MockUserRepository,
        },
    };
    use authz::{Decision, MockAuthorizer};
    use mockall::predicate::eq;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    fn fixture(id: OrganizationId) -> Organization {
        let now = Utc::now();
        Organization {
            id,
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
            legal_name: None,
            legal_form: None,
            registration_number: None,
            vat_status: None,
            share_capital_cents: None,
            address_line1: None,
            address_line2: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            quote_number_prefix: "DEV".to_owned(),
            invoice_number_prefix: "FAC".to_owned(),
            field_clock_enabled: false,
            vat_on_debits: false,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Subject built the way the API handler will build it: `mestier.user_id`
    /// is set, no IAM roles, no org context yet (the service enriches it).
    fn actor_for(user_id: UserId) -> authz::Subject {
        policy::user_subject(user_id, Vec::new())
    }

    /// Stages the calls `policy::enrich_for_organization` makes: lookup
    /// the member, list its role ids, list the org's roles. The roles
    /// list is left empty so aggregated permissions = 0; tests that rely
    /// on the engine answering `allow`/`deny` go through MockAuthorizer.
    fn stage_org_membership(
        members: &mut MockMemberRepository,
        roles: &mut MockRoleRepository,
        org_id: OrganizationId,
        user_id: UserId,
        member_id: MemberId,
    ) {
        members
            .expect_find_by_org_and_user()
            .with(eq(org_id), eq(user_id))
            .times(1)
            .returning(move |organization_id, user_id| {
                let m = Member {
                    id: member_id,
                    organization_id,
                    user_id: Some(user_id),
                    last_name: "Member".to_owned(),
                    first_name: None,
                    joined_at: Some(Utc::now()),
                    created_at: Utc::now(),
                    deleted_at: None,
                };
                Box::pin(async move { Ok(Some(m)) })
            });
        members
            .expect_list_role_ids()
            .with(eq(member_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        roles
            .expect_list_by_organization()
            .with(eq(org_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
    }

    #[tokio::test]
    async fn get_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );
        let err = service.get_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_organization_returns_entity_when_found() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let org = service.get_organization(id).await.unwrap();

        assert_eq!(org.id, id);
    }

    #[tokio::test]
    async fn update_organization_mutates_and_saves() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            id,
            user_id,
            member_id,
        );
        organization_repository
            .expect_update()
            .times(1)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );

        let updated = service
            .update_organization(UpdateOrganizationCommand {
                actor: actor_for(user_id),
                id,
                name: "Acme Inc.".into(),
                slug: "acme-inc".into(),
                field_clock_enabled: false,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Acme Inc.");
        assert_eq!(updated.slug, "acme-inc");
    }

    /// `fixture()` starts with the clock off; the command flips it on and
    /// the saved entity must carry that through, same as `name`/`slug`.
    #[tokio::test]
    async fn update_organization_flips_field_clock_enabled() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            id,
            user_id,
            member_id,
        );
        organization_repository
            .expect_update()
            .times(1)
            .withf(|o| o.field_clock_enabled)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );

        let updated = service
            .update_organization(UpdateOrganizationCommand {
                actor: actor_for(user_id),
                id,
                name: "Acme".into(),
                slug: "acme".into(),
                field_clock_enabled: true,
            })
            .await
            .unwrap();

        assert!(updated.field_clock_enabled);
    }

    #[tokio::test]
    async fn update_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let err = service
            .update_organization(UpdateOrganizationCommand {
                actor: actor_for(UserId(Uuid::new_v4())),
                id,
                name: "Whatever".into(),
                slug: "whatever".into(),
                field_clock_enabled: false,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn update_organization_returns_forbidden_when_not_a_member() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        member_repository
            .expect_find_by_org_and_user()
            .with(eq(id), eq(user_id))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let err = service
            .update_organization(UpdateOrganizationCommand {
                actor: actor_for(user_id),
                id,
                name: "Acme Inc.".into(),
                slug: "acme-inc".into(),
                field_clock_enabled: false,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn update_organization_returns_forbidden_when_authz_denies() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            id,
            user_id,
            member_id,
        );
        // No `expect_update` — the call must short-circuit before mutation.

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .withf(move |req| {
                req.action.name == "organization.update"
                    && req.resource.r#type == "organization"
                    && req.resource.id == id.0.to_string()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::deny()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );

        let err = service
            .update_organization(UpdateOrganizationCommand {
                actor: actor_for(user_id),
                id,
                name: "Acme Inc.".into(),
                slug: "acme-inc".into(),
                field_clock_enabled: false,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    fn legal_identity_command(
        id: OrganizationId,
        actor: authz::Subject,
    ) -> UpdateLegalIdentityCommand {
        UpdateLegalIdentityCommand {
            actor,
            id,
            legal_name: Some("Acme SARL".into()),
            legal_form: Some("SARL".into()),
            registration_number: Some("123 456 789 00012".into()),
            vat_status: Some(
                crate::domain::organization::legal_identity::VatStatus::Subject {
                    vat_number: "FR12345678901".into(),
                },
            ),
            share_capital_cents: Some(1_000_000),
            address_line1: Some("12 rue des Artisans".into()),
            address_line2: None,
            address_postal_code: Some("75001".into()),
            address_city: Some("Paris".into()),
            address_country: Some("FR".into()),
            contact_email: Some("contact@acme.fr".into()),
            contact_phone: None,
            insurance_mention: Some("RC Pro n°123456 - MAAF Assurances".into()),
            vat_on_debits: false,
        }
    }

    #[tokio::test]
    async fn update_legal_identity_mutates_and_saves() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            id,
            user_id,
            member_id,
        );
        organization_repository
            .expect_update()
            .times(1)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );

        let updated = service
            .update_legal_identity(legal_identity_command(id, actor_for(user_id)))
            .await
            .unwrap();

        assert_eq!(updated.legal_name.as_deref(), Some("Acme SARL"));
        assert_eq!(
            updated.vat_status,
            Some(
                crate::domain::organization::legal_identity::VatStatus::Subject {
                    vat_number: "FR12345678901".into()
                }
            )
        );
        assert_eq!(updated.address_city.as_deref(), Some("Paris"));
    }

    #[tokio::test]
    async fn update_legal_identity_clears_fields_left_none() {
        let id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let mut org = fixture(id);
                org.legal_name = Some("Old Name".into());
                Box::pin(async move { Ok(Some(org)) })
            });
        stage_org_membership(
            &mut member_repository,
            &mut role_repository,
            id,
            user_id,
            member_id,
        );
        organization_repository
            .expect_update()
            .times(1)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut authz = MockAuthorizer::new();
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            authz,
        );

        let mut command = legal_identity_command(id, actor_for(user_id));
        command.legal_name = None;

        let updated = service.update_legal_identity(command).await.unwrap();

        assert_eq!(updated.legal_name, None);
    }

    #[tokio::test]
    async fn soft_delete_organization_calls_repo() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |id| {
                let org = fixture(id);
                Box::pin(async move { Ok(Some(org)) })
            });

        organization_repository
            .expect_soft_delete()
            .withf(move |i, _| *i == id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        service.soft_delete_organization(id).await.unwrap();
    }

    #[tokio::test]
    async fn soft_delete_organization_returns_not_found_when_missing() {
        let id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let err = service.soft_delete_organization(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_organizations_for_user_resolves_sub_then_delegates_to_repo() {
        let user_id = UserId(Uuid::new_v4());
        let sub = "sub-abc-123";

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let mut user_repository = MockUserRepository::new();

        user_repository
            .expect_find_by_sub()
            .with(eq(sub))
            .times(1)
            .returning(move |s| {
                let now = Utc::now();
                let user = User {
                    id: user_id,
                    email: "user@example.com".into(),
                    username: "user".into(),
                    name: "User".into(),
                    sub: s.to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });

        organization_repository
            .expect_list_for_user()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async { Ok(vec![]) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let orgs = service.list_organizations_for_user(sub).await.unwrap();

        assert!(orgs.is_empty());
    }

    fn create_cmd() -> CreateOrganizationCommand {
        CreateOrganizationCommand {
            name: "Acme".into(),
            slug: "acme".into(),
            owner_id: UserId(Uuid::new_v4()),
        }
    }

    #[tokio::test]
    async fn create_organization_seeds_roles_and_owner_membership() {
        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let mut user_repository = MockUserRepository::new();

        user_repository
            .expect_find_by_sub()
            .times(1)
            .returning(|s| {
                let now = Utc::now();
                let user = User {
                    id: UserId(Uuid::new_v4()),
                    email: "owner@example.com".into(),
                    username: "owner".into(),
                    name: "Owner".into(),
                    sub: s.to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });

        organization_repository
            .expect_insert()
            .times(1)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });

        role_repository.expect_insert().times(3).returning(|r| {
            let cloned = Role {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                permissions: r.permissions,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });

        member_repository.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });

        member_repository
            .expect_assign_role()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut task_label_repository = MockTaskLabelRepository::new();
        task_label_repository
            .expect_insert()
            .times(3)
            .returning(|l| {
                let cloned = l.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let org = service
            .create_organization(create_cmd(), task_label_repository)
            .await
            .unwrap();

        assert_eq!(org.name, "Acme");
        assert_eq!(org.slug, "acme");
        assert!(org.deleted_at.is_none());
        assert!(
            !org.field_clock_enabled,
            "a freshly created organization starts with the clock off"
        );
    }

    /// The preset names/colors themselves — `create_organization_seeds_roles_and_owner_membership`
    /// only asserts the insert count, not which labels landed.
    #[tokio::test]
    async fn create_organization_seeds_the_three_documented_preset_labels() {
        let mut organization_repository = MockOrganizationRepository::new();
        let mut role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let mut user_repository = MockUserRepository::new();

        user_repository
            .expect_find_by_sub()
            .times(1)
            .returning(|s| {
                let now = Utc::now();
                let user = User {
                    id: UserId(Uuid::new_v4()),
                    email: "owner@example.com".into(),
                    username: "owner".into(),
                    name: "Owner".into(),
                    sub: s.to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });
        organization_repository
            .expect_insert()
            .times(1)
            .returning(|o| {
                let cloned = o.clone();
                Box::pin(async move { Ok(cloned) })
            });
        role_repository.expect_insert().times(3).returning(|r| {
            let cloned = Role {
                id: r.id,
                organization_id: r.organization_id,
                name: r.name.clone(),
                permissions: r.permissions,
                created_at: r.created_at,
                updated_at: r.updated_at,
            };
            Box::pin(async move { Ok(cloned) })
        });
        member_repository.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });
        member_repository
            .expect_assign_role()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let inserted: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let inserted_handle = Arc::clone(&inserted);
        let mut task_label_repository = MockTaskLabelRepository::new();
        task_label_repository
            .expect_insert()
            .times(3)
            .returning(move |l| {
                inserted_handle
                    .lock()
                    .unwrap()
                    .push((l.name.clone(), l.color.clone()));
                let cloned = l.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        service
            .create_organization(create_cmd(), task_label_repository)
            .await
            .unwrap();

        let inserted = inserted.lock().unwrap();
        assert_eq!(inserted.len(), 3);
        assert!(inserted.iter().any(|(name, _)| name == "Réunion"));
        assert!(inserted.iter().any(|(name, _)| name == "Déplacement"));
        assert!(inserted.iter().any(|(name, _)| name == "Formation"));
        assert!(
            inserted
                .iter()
                .all(|(_, color)| color.starts_with('#') && color.len() == 7),
            "every preset must carry a well-formed hex color"
        );
    }

    #[tokio::test]
    async fn create_organization_translates_slug_unique_violation_to_business_error() {
        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let mut user_repository = MockUserRepository::new();

        user_repository
            .expect_find_by_sub()
            .times(1)
            .returning(|s| {
                let now = Utc::now();
                let user = User {
                    id: UserId(Uuid::new_v4()),
                    email: "owner@example.com".into(),
                    username: "owner".into(),
                    name: "Owner".into(),
                    sub: s.to_owned(),
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(user)) })
            });

        // Infra-style payload: constraint name only.
        organization_repository
            .expect_insert()
            .times(1)
            .returning(|_| {
                Box::pin(async { Err(CoreError::Conflict("organizations_slug_key".into())) })
            });

        // No `expect_insert` on this mock: creation must fail on the
        // organization insert itself, before any preset label is ever
        // attempted.
        let task_label_repository = MockTaskLabelRepository::new();

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        let err = service
            .create_organization(create_cmd(), task_label_repository)
            .await
            .unwrap_err();

        match err {
            CoreError::Conflict(msg) => assert_eq!(msg, "slug already taken"),
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn leave_organization_soft_deletes_when_owner_leaves() {
        let owner_id = UserId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(org_id))
            .times(2)
            .returning(move |id| {
                let now = Utc::now();
                let org = Organization {
                    id,
                    name: "Acme".into(),
                    slug: "acme".into(),
                    owner_id,
                    legal_name: None,
                    legal_form: None,
                    registration_number: None,
                    vat_status: None,
                    share_capital_cents: None,
                    address_line1: None,
                    address_line2: None,
                    address_postal_code: None,
                    address_city: None,
                    address_country: None,
                    contact_email: None,
                    contact_phone: None,
                    insurance_mention: None,
                    quote_number_prefix: "DEV".to_owned(),
                    invoice_number_prefix: "FAC".to_owned(),
                    field_clock_enabled: false,
                    vat_on_debits: false,
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(org)) })
            });

        organization_repository
            .expect_soft_delete()
            .withf(move |i, _| *i == org_id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        service.leave_organization(org_id, owner_id).await.unwrap();
    }

    #[tokio::test]
    async fn leave_organization_soft_deletes_membership_when_non_owner_leaves() {
        let owner_id = UserId(Uuid::new_v4());
        let leaver_id = UserId(Uuid::new_v4());
        let org_id = OrganizationId(Uuid::new_v4());
        let member_id = MemberId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut member_repository = MockMemberRepository::new();
        let user_repository = MockUserRepository::new();

        organization_repository
            .expect_find_by_id()
            .with(eq(org_id))
            .times(1)
            .returning(move |id| {
                let now = Utc::now();
                let org = Organization {
                    id,
                    name: "Acme".into(),
                    slug: "acme".into(),
                    owner_id,
                    legal_name: None,
                    legal_form: None,
                    registration_number: None,
                    vat_status: None,
                    share_capital_cents: None,
                    address_line1: None,
                    address_line2: None,
                    address_postal_code: None,
                    address_city: None,
                    address_country: None,
                    contact_email: None,
                    contact_phone: None,
                    insurance_mention: None,
                    quote_number_prefix: "DEV".to_owned(),
                    invoice_number_prefix: "FAC".to_owned(),
                    field_clock_enabled: false,
                    vat_on_debits: false,
                    deleted_at: None,
                    created_at: now,
                    updated_at: now,
                };
                Box::pin(async move { Ok(Some(org)) })
            });

        member_repository
            .expect_find_by_org_and_user()
            .with(eq(org_id), eq(leaver_id))
            .times(1)
            .returning(move |organization_id, user_id| {
                let m = Member {
                    id: member_id,
                    organization_id,
                    user_id: Some(user_id),
                    last_name: "Member".to_owned(),
                    first_name: None,
                    joined_at: Some(Utc::now()),
                    created_at: Utc::now(),
                    deleted_at: None,
                };
                Box::pin(async move { Ok(Some(m)) })
            });

        member_repository
            .expect_soft_delete()
            .withf(move |id, _| *id == member_id)
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = OrganizationService::new(
            organization_repository,
            role_repository,
            member_repository,
            user_repository,
            MockAuthorizer::new(),
        );

        service.leave_organization(org_id, leaver_id).await.unwrap();
    }
}
