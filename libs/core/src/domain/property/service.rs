use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomerId, Property, PropertyId,
    domain::property::{
        commands::{CreatePropertyCommand, UpdatePropertyCommand},
        ports::PropertyRepository,
    },
};

pub struct PropertyService<R>
where
    R: PropertyRepository,
{
    repo: R,
}

impl<R> PropertyService<R>
where
    R: PropertyRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_property(
        &mut self,
        command: CreatePropertyCommand,
    ) -> Result<Property, CoreError> {
        validate_property(
            &command.label,
            &command.street,
            &command.zip,
            &command.city,
            &command.photo_key,
        )?;

        let now = Utc::now();
        self.repo
            .insert(&Property {
                id: PropertyId(generate_uuid_v7()),
                customer_id: command.customer_id,
                label: command.label,
                street: command.street,
                zip: command.zip,
                city: command.city,
                photo_key: command.photo_key,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_property(&mut self, id: PropertyId) -> Result<Property, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_properties(
        &mut self,
        customer_id: CustomerId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Property>, u64), CoreError> {
        self.repo.list_by_customer(customer_id, limit, offset).await
    }

    pub async fn update_property(
        &mut self,
        command: UpdatePropertyCommand,
    ) -> Result<Property, CoreError> {
        validate_property(
            &command.label,
            &command.street,
            &command.zip,
            &command.city,
            &command.photo_key,
        )?;

        let mut property = self.get_property(command.id).await?;
        property.label = command.label;
        property.street = command.street;
        property.zip = command.zip;
        property.city = command.city;
        property.photo_key = command.photo_key;
        property.updated_at = Utc::now();

        self.repo.update(&property).await
    }

    pub async fn soft_delete_property(&mut self, id: PropertyId) -> Result<(), CoreError> {
        self.get_property(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_property(
    label: &str,
    street: &str,
    zip: &str,
    city: &str,
    photo_key: &Option<String>,
) -> Result<(), CoreError> {
    validate_required("property label", label)?;
    validate_required("property street", street)?;
    validate_required("property zip", zip)?;
    validate_required("property city", city)?;
    validate_optional("property photo key", photo_key)?;
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
    use crate::domain::property::ports::MockPropertyRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn property(id: PropertyId) -> Property {
        let now = Utc::now();
        Property {
            id,
            customer_id: CustomerId(Uuid::new_v4()),
            label: "Maison".to_owned(),
            street: "1 rue des Lilas".to_owned(),
            zip: "75001".to_owned(),
            city: "Paris".to_owned(),
            photo_key: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_property_persists_via_repo() {
        let mut repo = MockPropertyRepository::new();
        repo.expect_insert().times(1).returning(|p| {
            let property = p.clone();
            Box::pin(async move { Ok(property) })
        });

        let mut service = PropertyService::new(repo);
        let created = service
            .create_property(CreatePropertyCommand {
                customer_id: CustomerId(Uuid::new_v4()),
                label: "Maison".to_owned(),
                street: "1 rue des Lilas".to_owned(),
                zip: "75001".to_owned(),
                city: "Paris".to_owned(),
                photo_key: None,
            })
            .await
            .unwrap();

        assert_eq!(created.label, "Maison");
    }

    #[tokio::test]
    async fn update_property_mutates_existing_property() {
        let id = PropertyId(Uuid::new_v4());
        let mut repo = MockPropertyRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(property(id))) }));
        repo.expect_update().times(1).returning(|p| {
            let property = p.clone();
            Box::pin(async move { Ok(property) })
        });

        let mut service = PropertyService::new(repo);
        let updated = service
            .update_property(UpdatePropertyCommand {
                id,
                label: "Atelier".to_owned(),
                street: "2 rue des Lilas".to_owned(),
                zip: "69001".to_owned(),
                city: "Lyon".to_owned(),
                photo_key: Some("uploads/property.jpg".to_owned()),
            })
            .await
            .unwrap();

        assert_eq!(updated.label, "Atelier");
        assert_eq!(updated.photo_key.as_deref(), Some("uploads/property.jpg"));
    }

    #[tokio::test]
    async fn list_properties_delegates_to_repo() {
        let customer_id = CustomerId(Uuid::new_v4());
        let mut repo = MockPropertyRepository::new();
        repo.expect_list_by_customer()
            .with(eq(customer_id), eq(10), eq(20))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![property(PropertyId(Uuid::new_v4()))], 1)) })
            });

        let mut service = PropertyService::new(repo);
        let (items, total) = service.list_properties(customer_id, 10, 20).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_property_checks_existence_then_deletes() {
        let id = PropertyId(Uuid::new_v4());
        let mut repo = MockPropertyRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(property(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = PropertyService::new(repo);
        service.soft_delete_property(id).await.unwrap();
    }

    #[tokio::test]
    async fn create_property_rejects_blank_address_parts() {
        let repo = MockPropertyRepository::new();
        let mut service = PropertyService::new(repo);

        let err = service
            .create_property(CreatePropertyCommand {
                customer_id: CustomerId(Uuid::new_v4()),
                label: "Maison".to_owned(),
                street: "".to_owned(),
                zip: "75001".to_owned(),
                city: "Paris".to_owned(),
                photo_key: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }
}
