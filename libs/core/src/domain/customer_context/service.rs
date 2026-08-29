use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContext, CustomerContextId, CustomerId,
    application::policy,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::{
            commands::{CreateCustomerContextCommand, UpdateCustomerContextCommand},
            ports::CustomerContextRepository,
        },
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

pub struct CustomerContextService<R, C, M, Ro, A>
where
    R: CustomerContextRepository,
    C: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    repo: R,
    customer_repository: C,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

impl<R, C, M, Ro, A> CustomerContextService<R, C, M, Ro, A>
where
    R: CustomerContextRepository,
    C: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(
        repo: R,
        customer_repository: C,
        member_repository: M,
        role_repository: Ro,
        authz: A,
    ) -> Self {
        Self {
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        }
    }

    /// The parent customer's own `organization_id` — a customer context
    /// carries no organization of its own, only `customer_id`, so this is
    /// the only source of authorization context for it.
    async fn require_customer_manage(
        &mut self,
        customer_id: CustomerId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let customer = self
            .customer_repository
            .find_by_id(customer_id)
            .await?
            .ok_or(CoreError::NotFound)?;

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
        .await
    }

    pub async fn create_customer_context(
        &mut self,
        command: CreateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        validate_customer_context(
            &command.label,
            &command.address_line,
            &command.postal_code,
            &command.city,
            &command.photo_key,
        )?;

        self.require_customer_manage(command.customer_id, command.actor)
            .await?;

        let now = Utc::now();
        self.repo
            .insert(&CustomerContext {
                id: CustomerContextId(generate_uuid_v7()),
                customer_id: command.customer_id,
                label: command.label,
                address_line: command.address_line,
                postal_code: command.postal_code,
                city: command.city,
                photo_key: command.photo_key,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_customer_context(
        &mut self,
        id: CustomerContextId,
    ) -> Result<CustomerContext, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_customer_contexts(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContext>, u64), CoreError> {
        self.repo.list_by_customer(customer_id, limit, offset).await
    }

    pub async fn update_customer_context(
        &mut self,
        command: UpdateCustomerContextCommand,
    ) -> Result<CustomerContext, CoreError> {
        validate_customer_context(
            &command.label,
            &command.address_line,
            &command.postal_code,
            &command.city,
            &command.photo_key,
        )?;

        let mut customer_context = self.get_customer_context(command.id).await?;

        self.require_customer_manage(customer_context.customer_id, command.actor)
            .await?;

        customer_context.label = command.label;
        customer_context.address_line = command.address_line;
        customer_context.postal_code = command.postal_code;
        customer_context.city = command.city;
        customer_context.photo_key = command.photo_key;
        customer_context.updated_at = Utc::now();

        self.repo.update(&customer_context).await
    }

    pub async fn soft_delete_customer_context(
        &mut self,
        id: CustomerContextId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let customer_context = self.get_customer_context(id).await?;

        self.require_customer_manage(customer_context.customer_id, actor)
            .await?;

        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_customer_context(
    label: &str,
    address_line: &Option<String>,
    postal_code: &Option<String>,
    city: &Option<String>,
    photo_key: &Option<String>,
) -> Result<(), CoreError> {
    validate_required("customer_context label", label)?;
    validate_optional("customer_context address line", address_line)?;
    validate_optional("customer_context postal code", postal_code)?;
    validate_optional("customer_context city", city)?;
    validate_optional("customer_context photo key", photo_key)?;
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
    use crate::domain::customer::Customer;
    use crate::domain::customer::ports::MockCustomerRepository;
    use crate::domain::customer_context::ports::MockCustomerContextRepository;
    use crate::domain::member::ports::MockMemberRepository;
    use crate::domain::role::ports::MockRoleRepository;
    use authz::{Decision, MockAuthorizer};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer_context(id: CustomerContextId, customer_id: CustomerId) -> CustomerContext {
        let now = Utc::now();
        CustomerContext {
            id,
            customer_id,
            label: "Maison".to_owned(),
            address_line: Some("1 rue des Lilas".to_owned()),
            postal_code: Some("75001".to_owned()),
            city: Some("Paris".to_owned()),
            photo_key: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn customer(id: CustomerId) -> Customer {
        let now = Utc::now();
        Customer {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            status: crate::CustomerStatus::Prospect,
            pipeline_stage: crate::CustomerPipelineStage::New,
            name: "Alice Dupont".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
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

    fn stage_customer(customer_repository: &mut MockCustomerRepository, customer_id: CustomerId) {
        customer_repository
            .expect_find_by_id()
            .with(eq(customer_id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer(customer_id))) }));
    }

    #[tokio::test]
    async fn create_customer_context_persists_via_repo() {
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContextRepository::new();
        repo.expect_insert().times(1).returning(|p| {
            let customer_context = p.clone();
            Box::pin(async move { Ok(customer_context) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let created = service
            .create_customer_context(CreateCustomerContextCommand {
                actor: system_actor(),
                customer_id,
                label: "Maison".to_owned(),
                address_line: Some("1 rue des Lilas".to_owned()),
                postal_code: Some("75001".to_owned()),
                city: Some("Paris".to_owned()),
                photo_key: None,
            })
            .await
            .unwrap();

        assert_eq!(created.label, "Maison");
    }

    #[tokio::test]
    async fn update_customer_context_mutates_existing_customer_context() {
        let id = CustomerContextId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContextRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            Box::pin(async move { Ok(Some(customer_context(id, customer_id))) })
        });
        repo.expect_update().times(1).returning(|p| {
            let customer_context = p.clone();
            Box::pin(async move { Ok(customer_context) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let updated = service
            .update_customer_context(UpdateCustomerContextCommand {
                actor: system_actor(),
                id,
                label: "Atelier".to_owned(),
                address_line: Some("2 rue des Lilas".to_owned()),
                postal_code: Some("69001".to_owned()),
                city: Some("Lyon".to_owned()),
                photo_key: Some("uploads/customer_context.jpg".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(updated.label, "Atelier");
        assert_eq!(
            updated.photo_key.as_deref(),
            Some("uploads/customer_context.jpg")
        );
    }

    #[tokio::test]
    async fn list_customer_contexts_delegates_to_repo() {
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContextRepository::new();
        repo.expect_list_by_customer()
            .with(eq(customer_id), eq(10), eq(20))
            .returning(move |_, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![customer_context(
                            CustomerContextId(Uuid::new_v4()),
                            customer_id,
                        )],
                        1,
                    ))
                })
            });
        let customer_repository = MockCustomerRepository::new();
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();

        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let (items, total) = service
            .list_customer_contexts(customer_id, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_customer_context_checks_existence_then_deletes() {
        let id = CustomerContextId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContextRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            Box::pin(async move { Ok(Some(customer_context(id, customer_id))) })
        });
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service
            .soft_delete_customer_context(id, system_actor())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_customer_context_rejects_blank_optional_address_parts() {
        let repo = MockCustomerContextRepository::new();
        let customer_repository = MockCustomerRepository::new();
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();
        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );

        let err = service
            .create_customer_context(CreateCustomerContextCommand {
                actor: system_actor(),
                customer_id: CustomerId(Uuid::new_v4()),
                label: "Maison".to_owned(),
                address_line: Some("".to_owned()),
                postal_code: Some("75001".to_owned()),
                city: Some("Paris".to_owned()),
                photo_key: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    /// The permission gate itself: a non-system actor with no membership at
    /// all in the parent customer's organization is refused before any
    /// mutation.
    #[tokio::test]
    async fn update_customer_context_returns_forbidden_when_not_a_member() {
        let id = CustomerContextId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let user_id = crate::UserId(Uuid::new_v4());

        let mut repo = MockCustomerContextRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            Box::pin(async move { Ok(Some(customer_context(id, customer_id))) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();

        let mut service = CustomerContextService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let err = service
            .update_customer_context(UpdateCustomerContextCommand {
                actor: policy::user_subject(user_id, Vec::new()),
                id,
                label: "Atelier".to_owned(),
                address_line: None,
                postal_code: None,
                city: None,
                photo_key: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }
}
