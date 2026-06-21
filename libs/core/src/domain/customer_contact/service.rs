use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerContact, CustomerContactId, CustomerId,
    domain::customer_contact::{
        commands::{CreateCustomerContactCommand, UpdateCustomerContactCommand},
        ports::CustomerContactRepository,
    },
};

pub struct CustomerContactService<R>
where
    R: CustomerContactRepository,
{
    repo: R,
}

impl<R> CustomerContactService<R>
where
    R: CustomerContactRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
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
    ) -> Result<(), CoreError> {
        self.get_customer_contact(id).await?;
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
    use crate::domain::customer_contact::ports::MockCustomerContactRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn customer_contact(id: CustomerContactId) -> CustomerContact {
        let now = Utc::now();
        CustomerContact {
            id,
            customer_id: CustomerId(Uuid::new_v4()),
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

    #[tokio::test]
    async fn create_customer_contact_persists_via_repo() {
        let mut repo = MockCustomerContactRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let contact = c.clone();
            Box::pin(async move { Ok(contact) })
        });

        let mut service = CustomerContactService::new(repo);
        let created = service
            .create_customer_contact(CreateCustomerContactCommand {
                customer_id: CustomerId(Uuid::new_v4()),
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
        let mut repo = MockCustomerContactRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(customer_contact(id))) }));
        repo.expect_update().times(1).returning(|c| {
            let contact = c.clone();
            Box::pin(async move { Ok(contact) })
        });

        let mut service = CustomerContactService::new(repo);
        let updated = service
            .update_customer_contact(UpdateCustomerContactCommand {
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
                    Ok((vec![customer_contact(CustomerContactId(Uuid::new_v4()))], 1))
                })
            });

        let mut service = CustomerContactService::new(repo);
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
        let mut service = CustomerContactService::new(repo);

        let err = service
            .create_customer_contact(CreateCustomerContactCommand {
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
}
