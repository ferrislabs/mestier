use authz::{Authorizer, Resource, Subject};
use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    Customer, CustomerError, CustomerField, CustomerId, CustomerInvoiceFact, CustomerListRow,
    CustomerOutcome, CustomerProfile, CustomerStatus, Deal, DealFilter, DealStage, DealStageName,
    LostReason, MemberId, OrganizationId, RegistrationNumber,
    application::policy,
    domain::{
        customer::{
            commands::{CreateCustomerCommand, UpdateCustomerCommand},
            directory::indicators_of,
            events::CustomerCreated,
            ports::CustomerRepository,
        },
        deal::{events::DealStageChanged, ports::DealRepository},
        member::ports::MemberRepository,
        role::ports::RoleRepository,
    },
};

/// Deals of one customer in one open stage, read when it is archived.
const OPEN_DEALS_PER_STAGE: u64 = 1_000;

pub struct CustomerService<R, D, M, Ro, A, E>
where
    R: CustomerRepository,
    D: DealRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
    E: EventEmitter,
{
    repo: R,
    deals: D,
    member_repository: M,
    role_repository: Ro,
    authz: A,
    emitter: E,
}

impl<R, D, M, Ro, A, E> CustomerService<R, D, M, Ro, A, E>
where
    R: CustomerRepository,
    D: DealRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
    E: EventEmitter,
{
    pub fn new(
        repo: R,
        deals: D,
        member_repository: M,
        role_repository: Ro,
        authz: A,
        emitter: E,
    ) -> Self {
        Self {
            repo,
            deals,
            member_repository,
            role_repository,
            authz,
            emitter,
        }
    }

    /// Read before the bulk update that loses them: that update reports a
    /// count, and each deal that moves has to be announced.
    async fn open_deals_of(&mut self, customer: &Customer) -> Result<Vec<Deal>, CoreError> {
        let mut open = Vec::new();
        for stage in DealStageName::ALL
            .into_iter()
            .filter(|stage| stage.is_open())
        {
            let filter = DealFilter {
                organization_id: customer.organization_id,
                customer_id: Some(customer.id),
                stage: Some(stage),
                query: None,
            };
            let (deals, _) = self.deals.list(&filter, OPEN_DEALS_PER_STAGE, 0).await?;
            open.extend(deals);
        }
        Ok(open)
    }

    fn emit_lost(&self, open: Vec<Deal>, now: DateTime<Utc>) -> Result<(), CoreError> {
        for mut deal in open {
            let from = deal.stage.name();
            deal.move_to(DealStage::Lost(LostReason::archived_account()), now);
            self.emitter
                .emit(deal.organization_id, &DealStageChanged { deal, from })?;
        }
        Ok(())
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

        let created = self.repo.insert(&customer).await?;

        self.emitter.emit(
            created.organization_id,
            &CustomerCreated {
                customer: created.clone(),
            },
        )?;

        Ok(Ok(created))
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

    pub async fn list_customer_rows(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
        today: NaiveDate,
    ) -> Result<(Vec<CustomerListRow>, u64), CoreError> {
        let (customers, total) = self
            .repo
            .list_by_organization(organization_id, limit, offset)
            .await?;
        let ids: Vec<CustomerId> = customers.iter().map(|customer| customer.id).collect();
        let entries = self.repo.directory(organization_id, &ids).await?;
        let facts = self.repo.invoice_facts(organization_id, &ids).await?;

        let rows = customers
            .into_iter()
            .map(|customer| {
                let entry = entries
                    .iter()
                    .find(|entry| entry.customer_id == customer.id);
                let own: Vec<CustomerInvoiceFact> = facts
                    .iter()
                    .filter(|fact| fact.customer_id == customer.id)
                    .cloned()
                    .collect();
                let indicators = indicators_of(&own, today);
                let last_quote_at = entry.and_then(|entry| entry.last_quote_at);
                CustomerListRow {
                    city: entry.and_then(|entry| entry.city.clone()),
                    primary_contact: entry.and_then(|entry| entry.primary_contact.clone()),
                    revenue_12m_cents: indicators.revenue_12m_cents,
                    overdue_cents: indicators.overdue_cents,
                    last_activity_at: indicators.last_invoice_at.max(last_quote_at),
                    customer,
                }
            })
            .collect();

        Ok((rows, total))
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
        let was_archived = customer.status == CustomerStatus::Archived;
        customer.change_status(command.status, now);
        customer.name = command.name;
        customer.registration_number = command
            .registration_number
            .map(RegistrationNumber::into_inner);
        customer.phone = command.phone;
        customer.email = command.email;
        customer.profile = profile;
        customer.updated_at = now;

        let updated = self.repo.update(&customer).await?;
        if !was_archived && updated.status == CustomerStatus::Archived {
            let open = self.open_deals_of(&updated).await?;
            self.deals
                .lose_open_of_customer(updated.id, &LostReason::archived_account(), now)
                .await?;
            self.emit_lost(open, now)?;
        }

        Ok(Ok(updated))
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
    use crate::domain::deal::ports::MockDealRepository;
    use crate::domain::member::ports::MockMemberRepository;
    use crate::domain::role::ports::MockRoleRepository;
    use authz::{Decision, MockAuthorizer};
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer(id: CustomerId) -> Customer {
        let now = Utc::now();
        Customer {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            status: crate::CustomerStatus::Prospect,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let updated = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Client,
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
        assert_eq!(updated.phone.as_deref(), Some("0102030405"));
    }

    #[tokio::test]
    async fn list_customer_rows_joins_the_directory_and_the_indicators_of_each_customer() {
        use crate::{
            CustomerDirectoryEntry, CustomerInvoiceFact, InvoiceKind, InvoiceStatus, PrimaryContact,
        };
        use chrono::TimeZone;

        let org_id = OrganizationId(Uuid::new_v4());
        let busy = CustomerId(Uuid::new_v4());
        let quiet = CustomerId(Uuid::new_v4());
        let today = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        let quote_at = Utc.with_ymd_and_hms(2026, 10, 20, 8, 0, 0).unwrap();
        let invoice_at = Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap();
        let mut repo = MockCustomerRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(10), eq(0))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![customer(busy), customer(quiet)], 2)) })
            });
        repo.expect_directory().times(1).returning(move |_, _| {
            Box::pin(async move {
                Ok(vec![CustomerDirectoryEntry {
                    customer_id: busy,
                    city: Some("Lyon".to_owned()),
                    primary_contact: Some(PrimaryContact {
                        first_name: "Marc".to_owned(),
                        last_name: "Lemoine".to_owned(),
                        role: None,
                        email: None,
                        phone: None,
                        mobile: None,
                    }),
                    last_quote_at: Some(quote_at),
                }])
            })
        });
        repo.expect_invoice_facts().times(1).returning(move |_, _| {
            Box::pin(async move {
                Ok(vec![CustomerInvoiceFact {
                    customer_id: busy,
                    kind: InvoiceKind::Standard,
                    status: InvoiceStatus::Issued,
                    gross_cents: 12_000,
                    net_cents: 10_000,
                    issued_at: invoice_at,
                    issued_on: invoice_at.date_naive(),
                    due_on: Some(NaiveDate::from_ymd_opt(2026, 10, 1).unwrap()),
                    paid_cents: 2_000,
                    credited_cents: 0,
                }])
            })
        });

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            MockMemberRepository::new(),
            MockRoleRepository::new(),
            MockAuthorizer::new(),
            RecordingEmitter::new(),
        );
        let (rows, total) = service
            .list_customer_rows(org_id, 10, 0, today)
            .await
            .unwrap();

        assert_eq!(total, 2);
        assert_eq!(rows[0].customer.id, busy);
        assert_eq!(rows[0].city.as_deref(), Some("Lyon"));
        assert_eq!(rows[0].revenue_12m_cents, 10_000);
        assert_eq!(rows[0].overdue_cents, 10_000);
        assert_eq!(rows[0].last_activity_at, Some(quote_at));
        assert_eq!(rows[1].customer.id, quiet);
        assert_eq!(rows[1].city, None);
        assert_eq!(rows[1].primary_contact, None);
        assert_eq!(rows[1].revenue_12m_cents, 0);
        assert_eq!(rows[1].overdue_cents, 0);
        assert_eq!(rows[1].last_activity_at, None);
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
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
        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );

        let err = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let created = service
            .create_customer(CreateCustomerCommand {
                actor: system_actor(),
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Client,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let updated = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Client,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let refused = service
            .update_customer(UpdateCustomerCommand {
                actor: system_actor(),
                id,
                status: crate::CustomerStatus::Prospect,
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

        let mut service = CustomerService::new(
            repo,
            MockDealRepository::new(),
            member_repository,
            role_repository,
            authz,
            RecordingEmitter::new(),
        );
        let err = service
            .update_customer(UpdateCustomerCommand {
                actor: policy::user_subject(user_id, Vec::new()),
                id,
                status: crate::CustomerStatus::Client,
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
