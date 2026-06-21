use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Customer, CustomerId, OrganizationId,
    domain::customer::{
        commands::{CreateCustomerCommand, UpdateCustomerCommand},
        ports::CustomerRepository,
    },
};

pub struct CustomerService<R>
where
    R: CustomerRepository,
{
    repo: R,
}

impl<R> CustomerService<R>
where
    R: CustomerRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_customer(
        &mut self,
        command: CreateCustomerCommand,
    ) -> Result<Customer, CoreError> {
        validate_customer(
            &command.last_name,
            &command.first_name,
            &command.phone,
            &command.email,
        )?;

        let now = Utc::now();
        self.repo
            .insert(&Customer {
                id: CustomerId(generate_uuid_v7()),
                organization_id: command.organization_id,
                status: command.status,
                pipeline_stage: command.pipeline_stage,
                last_name: command.last_name,
                first_name: command.first_name,
                phone: command.phone,
                email: command.email,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_customer(&mut self, id: CustomerId) -> Result<Customer, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_customers(
        &mut self,
        organization_id: OrganizationId,
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
    ) -> Result<Customer, CoreError> {
        validate_customer(
            &command.last_name,
            &command.first_name,
            &command.phone,
            &command.email,
        )?;

        let mut customer = self.get_customer(command.id).await?;
        customer.status = command.status;
        customer.pipeline_stage = command.pipeline_stage;
        customer.last_name = command.last_name;
        customer.first_name = command.first_name;
        customer.phone = command.phone;
        customer.email = command.email;
        customer.updated_at = Utc::now();

        self.repo.update(&customer).await
    }

    pub async fn soft_delete_customer(&mut self, id: CustomerId) -> Result<(), CoreError> {
        self.get_customer(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_customer(
    last_name: &str,
    first_name: &str,
    phone: &Option<String>,
    email: &Option<String>,
) -> Result<(), CoreError> {
    validate_required("customer last name", last_name)?;
    validate_required("customer first name", first_name)?;
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
    use crate::domain::customer::ports::MockCustomerRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer(id: CustomerId) -> Customer {
        let now = Utc::now();
        Customer {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            status: crate::CustomerStatus::Prospect,
            pipeline_stage: crate::CustomerPipelineStage::New,
            last_name: "Dupont".to_owned(),
            first_name: "Alice".to_owned(),
            phone: Some("+33123456789".to_owned()),
            email: Some("alice@example.com".to_owned()),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_customer_persists_via_repo() {
        let mut repo = MockCustomerRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let customer = c.clone();
            Box::pin(async move { Ok(customer) })
        });

        let mut service = CustomerService::new(repo);
        let created = service
            .create_customer(CreateCustomerCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                last_name: "Dupont".to_owned(),
                first_name: "Alice".to_owned(),
                phone: None,
                email: None,
            })
            .await
            .unwrap();

        assert_eq!(created.last_name, "Dupont");
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

        let mut service = CustomerService::new(repo);
        let updated = service
            .update_customer(UpdateCustomerCommand {
                id,
                status: crate::CustomerStatus::Client,
                pipeline_stage: crate::CustomerPipelineStage::Won,
                last_name: "Martin".to_owned(),
                first_name: "Alice".to_owned(),
                phone: Some("0102030405".to_owned()),
                email: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.last_name, "Martin");
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

        let mut service = CustomerService::new(repo);
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

        let mut service = CustomerService::new(repo);
        service.soft_delete_customer(id).await.unwrap();
    }

    #[tokio::test]
    async fn create_customer_rejects_blank_names() {
        let repo = MockCustomerRepository::new();
        let mut service = CustomerService::new(repo);

        let err = service
            .create_customer(CreateCustomerCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                status: crate::CustomerStatus::Prospect,
                pipeline_stage: crate::CustomerPipelineStage::New,
                last_name: " ".to_owned(),
                first_name: "Alice".to_owned(),
                phone: None,
                email: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
