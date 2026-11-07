use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Customer, CustomerError, CustomerField, CustomerId, CustomerOutcome, CustomerProfile,
    CustomerStatus, MemberId, OrganizationId, RegistrationNumber,
    application::policy,
    domain::{
        customer::{
            commands::{CreateCustomerCommand, UpdateCustomerCommand},
            ports::CustomerRepository,
        },
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

pub struct CustomerService<R, M, Ro, A>
where
    R: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    repo: R,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

impl<R, M, Ro, A> CustomerService<R, M, Ro, A>
where
    R: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(repo: R, member_repository: M, role_repository: Ro, authz: A) -> Self {
        Self {
            repo,
            member_repository,
            role_repository,
            authz,
        }
    }

    async fn refusal_of(
        &mut self,
        organization_id: OrganizationId,
        customer_id: CustomerId,
        current_owner: Option<MemberId>,
        profile: &CustomerProfile,
    ) -> Result<Option<CustomerError>, CoreError> {
        if let Some(owner) = profile
            .owner_member_id
            .filter(|owner| Some(*owner) != current_owner)
        {
            let same_organization = self
                .member_repository
                .find_by_id(owner)
                .await?
                .is_some_and(|member| member.organization_id == organization_id);
            if !same_organization {
                return Ok(Some(CustomerError {
                    field: CustomerField::OwnerMemberId,
                    rule: "must be a member of the organization",
                }));
            }
        }

        if let Some(contact) = profile.billing_contact_id
            && !self.repo.has_contact(customer_id, contact).await?
        {
            return Ok(Some(CustomerError {
                field: CustomerField::BillingContactId,
                rule: "must be a contact of this customer",
            }));
        }

        if let Some(address) = profile.billing_address_id
            && !self.repo.has_address(customer_id, address).await?
        {
            return Ok(Some(CustomerError {
                field: CustomerField::BillingAddressId,
                rule: "must be an address of this customer",
            }));
        }

        Ok(None)
    }

    pub async fn create_customer(
        &mut self,
        command: CreateCustomerCommand,
    ) -> Result<CustomerOutcome<Customer>, CoreError> {
        validate_customer(&command.name, &command.phone, &command.email)?;

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "customer.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        let id = CustomerId(generate_uuid_v7());
        let profile = command.profile.normalized();
        if let Some(refusal) = self
            .refusal_of(command.organization_id, id, None, &profile)
            .await?
        {
            return Ok(Err(refusal));
        }

        let now = Utc::now();
        let mut customer = Customer {
            id,
            organization_id: command.organization_id,
            status: CustomerStatus::Prospect,
            pipeline_stage: command.pipeline_stage,
            name: command.name,
            registration_number: command
                .registration_number
                .map(RegistrationNumber::into_inner),
            phone: command.phone,
            email: command.email,
            profile,
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        customer.change_status(command.status, now);

        self.repo.insert(&customer).await.map(Ok)
    }

    pub async fn get_customer(&mut self, id: CustomerId) -> Result<Customer, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_customers(
        &mut self,
        organization_id: crate::OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Customer>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_customer(
        &mut self,
        command: UpdateCustomerCommand,
    ) -> Result<CustomerOutcome<Customer>, CoreError> {
        validate_customer(&command.name, &command.phone, &command.email)?;

        // Load first — the update carries no organization_id of its own,
        // so the row's own is the only source of authorization context.
        let mut customer = self.get_customer(command.id).await?;

        let actor = policy::enrich_for_organization(
            command.actor,
            customer.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "customer.manage",
            Resource::new("organization", customer.organization_id.0.to_string()),
        )
        .await?;

        let profile = command.profile.normalized();
        if let Some(refusal) = self
            .refusal_of(
                customer.organization_id,
                customer.id,
                customer.profile.owner_member_id,
                &profile,
            )
            .await?
        {
            return Ok(Err(refusal));
        }

        let now = Utc::now();
        customer.change_status(command.status, now);
        customer.pipeline_stage = command.pipeline_stage;
        customer.name = command.name;
        customer.registration_number = command
            .registration_number
            .map(RegistrationNumber::into_inner);
        customer.phone = command.phone;
        customer.email = command.email;
        customer.profile = profile;
        customer.updated_at = now;

        self.repo.update(&customer).await.map(Ok)
    }

    pub async fn soft_delete_customer(
        &mut self,
        id: CustomerId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let customer = self.get_customer(id).await?;

        let actor = policy::enrich_for_organization(
            actor,
            customer.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "customer.manage",
            Resource::new("organization", customer.organization_id.0.to_string()),
        )
        .await?;

        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_customer(
    name: &str,
    phone: &Option<String>,
    email: &Option<String>,
) -> Result<(), CoreError> {
    validate_required("customer name", name)?;
    validate_optional("customer phone", phone)?;
    validate_optional("customer email", email)?;
    Ok(())
}

fn validate_required(label: &str, value: &str) -> Result<(), CoreError> {
    if value.trim().is_empty() {
        return Err(CoreError::Conflict(format!("{label} cannot be empty")));
    }

    Ok(())
}

fn validate_optional(label: &str, value: &Option<String>) -> Result<(), CoreError> {
    if value.as_deref().is_some_and(|v| v.trim().is_empty()) {
        return Err(CoreError::Conflict(format!("{label} cannot be empty")));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OrganizationId;
    use crate::domain::customer::ports::MockCustomerRepository;
    use crate::domain::member::ports::MockMemberRepository;
    use crate::domain::role::ports::MockRoleRepository;
    use authz::{Decision, MockAuthorizer};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer(id: CustomerId) -> Customer {
        let now = Utc::now();
        Customer {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            status: crate::CustomerStatus::Prospect,
            pipeline_stage: crate::CustomerPipelineStage::New,
            name: "Alice Dupont".to_owned(),
            registration_number: None,
            phone: Some("+33123456789".to_owned()),
            email: Some("alice@example.com".to_owned()),
            profile: CustomerProfile::default(),
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// System subjects short-circuit `policy::enrich_for_organization` (no
    /// member/role DB load), which is the cheapest fixture default for tests
    /// that are not themselves about authorization.
    fn system_actor() -> Subject {
        Subject::system()
    }

    fn allow_once(authz: &mut MockAuthorizer) {
        authz
            .expect_evaluate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    }

    #[tokio::test]
    async fn create_customer_persists_via_repo() {
        let mut repo = MockCustomerRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Alice Dupont".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(created.name, "Alice Dupont");
    }

    #[tokio::test]
    async fn update_customer_mutates_existing_customer() {
        let id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer(id))) }));
        repo.expect_update().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let updated = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Client,
                pipeline_stage: crate::CustomerPipelineStage::Won,
                name: "Syndic Martin".to_owned(),
                registration_number: None,
                phone: Some("0102030405".to_owned()),
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.name, "Syndic Martin");
        assert_eq!(updated.status, crate::CustomerStatus::Client);
        assert_eq!(updated.pipeline_stage, crate::CustomerPipelineStage::Won);
        assert_eq!(updated.phone.as_deref(), Some("0102030405"));
    }

    #[tokio::test]
    async fn list_customers_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockCustomerRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(10), eq(20))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![customer(CustomerId(Uuid::new_v4()))], 1)) })
            });
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let (items, total) = service.list_customers(org_id, 10, 20).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_customer_checks_existence_then_deletes() {
        let id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        service
            .soft_delete_customer(id, system_actor())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_customer_rejects_blank_names() {
        let repo = MockCustomerRepository::new();
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();
        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);

        let err = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: " ".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_customer_persists_a_registration_number() {
        let mut repo = MockCustomerRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Alice Dupont".to_owned(),
                registration_number: Some(RegistrationNumber::try_from("123 456 789").unwrap()),
                phone: None,
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(created.registration_number.as_deref(), Some("123456789"));
    }

    #[tokio::test]
    async fn create_customer_with_the_status_client_sets_converted_at() {
        let mut repo = MockCustomerRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Client,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Alice Dupont".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap()
            .unwrap();

        assert!(created.converted_at.is_some());
    }

    #[tokio::test]
    async fn update_customer_keeps_an_owner_who_left_the_organization() {
        let id = CustomerId(Uuid::new_v4());
        let departed = crate::MemberId(Uuid::new_v4());
        let mut repo = MockCustomerRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let mut stored = customer(id);
            stored.profile.owner_member_id = Some(departed);
            Box::pin(async move { Ok(Some(stored)) })
        });
        repo.expect_update().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let updated = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Client,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Alice Dupont".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile {
                    owner_member_id: Some(departed),
                    ..CustomerProfile::default()
                },
            })
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.profile.owner_member_id, Some(departed));
        assert_eq!(updated.status, crate::CustomerStatus::Client);
    }

    #[tokio::test]
    async fn update_customer_refuses_a_new_owner_who_is_not_a_member() {
        let id = CustomerId(Uuid::new_v4());
        let stranger = crate::MemberId(Uuid::new_v4());
        let mut repo = MockCustomerRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer(id))) }));
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let refused = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                name: "Alice Dupont".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile {
                    owner_member_id: Some(stranger),
                    ..CustomerProfile::default()
                },
            })
            .await
            .unwrap()
            .unwrap_err();

        assert_eq!(refused.field, CustomerField::OwnerMemberId);
    }

    /// The permission gate itself: a non-system actor with no membership at
    /// all in the target organization is refused before any mutation.
    #[tokio::test]
    async fn update_customer_returns_forbidden_when_not_a_member() {
        let id = CustomerId(Uuid::new_v4());
        let user_id = crate::UserId(Uuid::new_v4());

        let mut repo = MockCustomerRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer(id))) }));
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();

        let mut service = CustomerService::new(repo, member_repository, role_repository, authz);
        let err = service
            .update_customer(UpdateCustomerCommand {
                actor: policy::user_subject(user_id, Vec::new()),
                id,
                status: crate::CustomerStatus::Client,
                pipeline_stage: crate::CustomerPipelineStage::Won,
                name: "Syndic Martin".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                profile: CustomerProfile::default(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }
}
