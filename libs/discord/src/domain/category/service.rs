use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Category, CategoryId, OrganizationId,
    domain::category::{
        commands::{CreateCategoryCommand, UpdateCategoryCommand},
        ports::CategoryRepository,
    },
    events::{DomainEvent, EventPublisher},
};

pub struct CategoryService<R, E> {
    repo: R,
    events: E,
}

impl<R, E> CategoryService<R, E>
where
    R: CategoryRepository,
    E: EventPublisher,
{
    pub fn new(repo: R, events: E) -> Self {
        Self { repo, events }
    }

    pub async fn create_category(
        &mut self,
        cmd: CreateCategoryCommand,
    ) -> Result<Category, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "category name cannot be blank".to_owned(),
            ));
        }
        let now = Utc::now();
        let category = Category {
            id: CategoryId(generate_uuid_v7()),
            organization_id: cmd.organization_id,
            name: cmd.name,
            position: cmd.position,
            created_at: now,
            updated_at: now,
        };
        let saved = self.repo.insert(&category).await?;
        self.events
            .publish(DomainEvent::CategoryCreated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn update_category(
        &mut self,
        cmd: UpdateCategoryCommand,
    ) -> Result<Category, CoreError> {
        if cmd.name.trim().is_empty() {
            return Err(CoreError::Conflict(
                "category name cannot be blank".to_owned(),
            ));
        }
        let existing = self
            .repo
            .find_by_id(cmd.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let updated = Category {
            id: existing.id,
            organization_id: existing.organization_id,
            name: cmd.name,
            position: cmd.position,
            created_at: existing.created_at,
            updated_at: Utc::now(),
        };
        let saved = self.repo.update(&updated).await?;
        self.events
            .publish(DomainEvent::CategoryUpdated(saved.clone()))
            .await?;
        Ok(saved)
    }

    pub async fn delete_category(&mut self, id: CategoryId) -> Result<(), CoreError> {
        let existing = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        self.repo.delete(id).await?;
        self.events
            .publish(DomainEvent::CategoryDeleted {
                organization_id: existing.organization_id,
                category_id: id,
            })
            .await?;
        Ok(())
    }

    pub async fn list_categories(
        &mut self,
        org: OrganizationId,
    ) -> Result<Vec<Category>, CoreError> {
        self.repo.list_by_organization(org).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CategoryId,
        domain::category::ports::MockCategoryRepository,
        events::{DomainEvent, MockEventPublisher},
    };
    use common::OrganizationId;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn make_category(id: CategoryId, org: OrganizationId) -> Category {
        use chrono::Utc;
        Category {
            id,
            organization_id: org,
            name: "General".to_owned(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_category_persists_and_publishes_created_event() {
        let org = OrganizationId(Uuid::new_v4());
        let mut repo = MockCategoryRepository::new();
        repo.expect_insert().times(1).returning(move |c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::CategoryCreated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = CategoryService::new(repo, events);
        let result = svc
            .create_category(CreateCategoryCommand {
                organization_id: org,
                name: "General".to_owned(),
                position: 0,
            })
            .await
            .unwrap();

        assert_eq!(result.organization_id, org);
        assert_eq!(result.name, "General");
    }

    #[tokio::test]
    async fn create_category_rejects_blank_name() {
        let repo = MockCategoryRepository::new();
        let events = MockEventPublisher::new();
        let mut svc = CategoryService::new(repo, events);

        let result = svc
            .create_category(CreateCategoryCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                name: "   ".to_owned(),
                position: 0,
            })
            .await;

        assert!(matches!(result, Err(common::CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_category_publishes_deleted_event() {
        let org = OrganizationId(Uuid::new_v4());
        let id = CategoryId(Uuid::new_v4());

        let mut repo = MockCategoryRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(make_category(id, org))) }));
        repo.expect_delete()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::CategoryDeleted { .. }))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = CategoryService::new(repo, events);
        svc.delete_category(id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_category_returns_not_found_when_missing() {
        let id = CategoryId(Uuid::new_v4());

        let mut repo = MockCategoryRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let events = MockEventPublisher::new();
        let mut svc = CategoryService::new(repo, events);

        let result = svc.delete_category(id).await;
        assert!(matches!(result, Err(common::CoreError::NotFound)));
    }

    #[tokio::test]
    async fn update_category_publishes_updated_event() {
        let org = OrganizationId(Uuid::new_v4());
        let id = CategoryId(Uuid::new_v4());

        let mut repo = MockCategoryRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(make_category(id, org))) }));
        repo.expect_update().times(1).returning(|c| {
            let c = c.clone();
            Box::pin(async move { Ok(c) })
        });

        let mut events = MockEventPublisher::new();
        events
            .expect_publish()
            .withf(|e| matches!(e, DomainEvent::CategoryUpdated(_)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut svc = CategoryService::new(repo, events);
        svc.update_category(UpdateCategoryCommand {
            id,
            name: "Renamed".to_owned(),
            position: 1,
        })
        .await
        .unwrap();
    }
}
