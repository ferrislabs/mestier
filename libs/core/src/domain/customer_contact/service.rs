use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContact, CustomerContactId, CustomerId,
    application::policy,
    domain::{
        customer::ports::CustomerRepository,
        customer_contact::{
            commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
            ports::CustomerContactRepository,
        },
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

pub struct CustomerContactService<R, C, M, Ro, A>
where
    R: CustomerContactRepository,
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

impl<R, C, M, Ro, A> CustomerContactService<R, C, M, Ro, A>
where
    R: CustomerContactRepository,
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

    /// The parent customer's own `organization_id` — a customer contact
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

    pub async fn create_customer_contact(
        &mut self,
        command: CreateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        validate_customer_contact(
            &command.first_name,
            &command.last_name,
            &command.role,
            &command.phone,
            &command.email,
        )?;

        self.require_customer_manage(command.customer_id, command.actor)
            .await?;

        let now = Utc::now();
        self.repo
            .insert(&CustomerContact {
                id: CustomerContactId(generate_uuid_v7()),
                customer_id: command.customer_id,
                first_name: command.first_name,
                last_name: command.last_name,
                role: command.role,
                phone: command.phone,
                email: command.email,
                is_primary: command.is_primary,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_customer_contact(
        &mut self,
        id: CustomerContactId,
    ) -> Result<CustomerContact, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_customer_contacts(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<CustomerContact>, u64), CoreError> {
        self.repo.list_by_customer(customer_id, limit, offset).await
    }

    pub async fn update_customer_contact(
        &mut self,
        command: UpdateCustomerContactCommand,
    ) -> Result<CustomerContact, CoreError> {
        validate_customer_contact(
            &command.first_name,
            &command.last_name,
            &command.role,
            &command.phone,
            &command.email,
        )?;

        let mut customer_contact = self.get_customer_contact(command.id).await?;

        self.require_customer_manage(customer_contact.customer_id, command.actor)
            .await?;

        customer_contact.first_name = command.first_name;
        customer_contact.last_name = command.last_name;
        customer_contact.role = command.role;
        customer_contact.phone = command.phone;
        customer_contact.email = command.email;
        customer_contact.is_primary = command.is_primary;
        customer_contact.updated_at = Utc::now();

        self.repo.update(&customer_contact).await
    }

    pub async fn soft_delete_customer_contact(
        &mut self,
        id: CustomerContactId,
        actor: Subject,
    ) -> Result<(), CoreError> {
        let customer_contact = self.get_customer_contact(id).await?;

        self.require_customer_manage(customer_contact.customer_id, actor)
            .await?;

        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_customer_contact(
    first_name: &str,
    last_name: &str,
    role: &Option<String>,
    phone: &Option<String>,
    email: &Option<String>,
) -> Result<(), CoreError> {
    validate_required("customer_contact first name", first_name)?;
    validate_required("customer_contact last name", last_name)?;
    validate_optional("customer_contact role", role)?;
    validate_optional("customer_contact phone", phone)?;
    validate_optional("customer_contact email", email)?;
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
    use crate::domain::customer_contact::ports::MockCustomerContactRepository;
    use crate::domain::member::ports::MockMemberRepository;
    use crate::domain::role::ports::MockRoleRepository;
    use authz::{Decision, MockAuthorizer};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer_contact(id: CustomerContactId, customer_id: CustomerId) -> CustomerContact {
        let now = Utc::now();
        CustomerContact {
            id,
            customer_id,
            first_name: "Alice".to_owned(),
            last_name: "Martin".to_owned(),
            role: Some("Acheteuse".to_owned()),
            phone: None,
            email: Some("alice@example.com".to_owned()),
            is_primary: true,
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
    async fn create_customer_contact_persists_via_repo() {
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContactRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let contact = c.clone();
            Box::pin(async move { Ok(contact) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerContactService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let created = service
            .create_customer_contact(CreateCustomerContactCommand {
                actor: system_actor(),
                customer_id,
                first_name: "Alice".to_owned(),
                last_name: "Martin".to_owned(),
                role: None,
                phone: None,
                email: None,
                is_primary: false,
            })
            .await
            .unwrap();

        assert_eq!(created.first_name, "Alice");
    }

    #[tokio::test]
    async fn update_customer_contact_mutates_existing_contact() {
        let id = CustomerContactId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContactRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            Box::pin(async move { Ok(Some(customer_contact(id, customer_id))) })
        });
        repo.expect_update().times(1).returning(|c| {
            let contact = c.clone();
            Box::pin(async move { Ok(contact) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let mut authz = MockAuthorizer::new();
        allow_once(&mut authz);

        let mut service = CustomerContactService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let updated = service
            .update_customer_contact(UpdateCustomerContactCommand {
                actor: system_actor(),
                id,
                first_name: "Nadia".to_owned(),
                last_name: "Martin".to_owned(),
                role: Some("Direction".to_owned()),
                phone: Some("0102030405".to_owned()),
                email: None,
                is_primary: false,
            })
            .await
            .unwrap();

        assert_eq!(updated.first_name, "Nadia");
        assert!(!updated.is_primary);
    }

    #[tokio::test]
    async fn list_customer_contacts_delegates_to_repo() {
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockCustomerContactRepository::new();
        repo.expect_list_by_customer()
            .with(eq(customer_id), eq(10), eq(20))
            .returning(move |_, _, _| {
                Box::pin(async move {
                    Ok((
                        vec![customer_contact(
                            CustomerContactId(Uuid::new_v4()),
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

        let mut service = CustomerContactService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let (items, total) = service
            .list_customer_contacts(customer_id, 10, 20)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn create_customer_contact_rejects_blank_name() {
        let repo = MockCustomerContactRepository::new();
        let customer_repository = MockCustomerRepository::new();
        let member_repository = MockMemberRepository::new();
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();
        let mut service = CustomerContactService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );

        let err = service
            .create_customer_contact(CreateCustomerContactCommand {
                actor: system_actor(),
                customer_id: CustomerId(Uuid::new_v4()),
                first_name: " ".to_owned(),
                last_name: "Martin".to_owned(),
                role: None,
                phone: None,
                email: None,
                is_primary: false,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    /// The permission gate itself: a non-system actor with no membership at
    /// all in the parent customer's organization is refused before any
    /// mutation.
    #[tokio::test]
    async fn soft_delete_customer_contact_returns_forbidden_when_not_a_member() {
        let id = CustomerContactId(Uuid::new_v4());
        let customer_id = CustomerId(Uuid::new_v4());
        let user_id = crate::UserId(Uuid::new_v4());

        let mut repo = MockCustomerContactRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            Box::pin(async move { Ok(Some(customer_contact(id, customer_id))) })
        });
        let mut customer_repository = MockCustomerRepository::new();
        stage_customer(&mut customer_repository, customer_id);
        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_find_by_org_and_user()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let role_repository = MockRoleRepository::new();
        let authz = MockAuthorizer::new();

        let mut service = CustomerContactService::new(
            repo,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let err = service
            .soft_delete_customer_contact(id, policy::user_subject(user_id, Vec::new()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }
}
