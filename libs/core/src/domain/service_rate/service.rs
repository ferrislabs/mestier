use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, ServiceRate, ServiceRateId,
    domain::service_rate::{
        commands::{CreateServiceRateCommand, UpdateServiceRateCommand},
        ports::ServiceRateRepository,
    },
};

pub struct ServiceRateService<R>
where
    R: ServiceRateRepository,
{
    repo: R,
}

impl<R> ServiceRateService<R>
where
    R: ServiceRateRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_service_rate(
        &mut self,
        command: CreateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
        validate_label(&command.label)?;
        validate_rate(command.rate_cents)?;

        let now = Utc::now();
        self.repo
            .insert(&ServiceRate {
                id: ServiceRateId(generate_uuid_v7()),
                organization_id: command.organization_id,
                label: command.label,
                unit: command.unit,
                rate_cents: command.rate_cents,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_service_rate(&mut self, id: ServiceRateId) -> Result<ServiceRate, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_service_rates(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ServiceRate>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_service_rate(
        &mut self,
        command: UpdateServiceRateCommand,
    ) -> Result<ServiceRate, CoreError> {
        validate_label(&command.label)?;
        validate_rate(command.rate_cents)?;

        let mut service_rate = self.get_service_rate(command.id).await?;
        service_rate.label = command.label;
        service_rate.unit = command.unit;
        service_rate.rate_cents = command.rate_cents;
        service_rate.updated_at = Utc::now();

        self.repo.update(&service_rate).await
    }

    pub async fn soft_delete_service_rate(&mut self, id: ServiceRateId) -> Result<(), CoreError> {
        self.get_service_rate(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_label(label: &str) -> Result<(), CoreError> {
    if label.trim().is_empty() {
        return Err(CoreError::Conflict(
            "service rate label cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_rate(rate_cents: i32) -> Result<(), CoreError> {
    if rate_cents < 0 {
        return Err(CoreError::Conflict(
            "service rate cannot be negative".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ServiceRateUnit, domain::service_rate::ports::MockServiceRateRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn service_rate(id: ServiceRateId) -> ServiceRate {
        let now = Utc::now();
        ServiceRate {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            label: "Taille".to_owned(),
            unit: ServiceRateUnit::Hour,
            rate_cents: 5500,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_service_rate_persists_via_repo() {
        let mut repo = MockServiceRateRepository::new();
        repo.expect_insert().times(1).returning(|r| {
            let service_rate = r.clone();
            Box::pin(async move { Ok(service_rate) })
        });

        let mut service = ServiceRateService::new(repo);
        let created = service
            .create_service_rate(CreateServiceRateCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                label: "Taille".to_owned(),
                unit: ServiceRateUnit::Hour,
                rate_cents: 5500,
            })
            .await
            .unwrap();

        assert_eq!(created.unit, ServiceRateUnit::Hour);
    }

    #[tokio::test]
    async fn update_service_rate_mutates_existing_service_rate() {
        let id = ServiceRateId(Uuid::new_v4());
        let mut repo = MockServiceRateRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(service_rate(id))) }));
        repo.expect_update().times(1).returning(|r| {
            let service_rate = r.clone();
            Box::pin(async move { Ok(service_rate) })
        });

        let mut service = ServiceRateService::new(repo);
        let updated = service
            .update_service_rate(UpdateServiceRateCommand {
                id,
                label: "Haie".to_owned(),
                unit: ServiceRateUnit::Ml,
                rate_cents: 1200,
            })
            .await
            .unwrap();

        assert_eq!(updated.label, "Haie");
        assert_eq!(updated.unit, ServiceRateUnit::Ml);
    }

    #[tokio::test]
    async fn list_service_rates_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockServiceRateRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(25), eq(50))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![service_rate(ServiceRateId(Uuid::new_v4()))], 1)) })
            });

        let mut service = ServiceRateService::new(repo);
        let (items, total) = service.list_service_rates(org_id, 25, 50).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_service_rate_checks_existence_then_deletes() {
        let id = ServiceRateId(Uuid::new_v4());
        let mut repo = MockServiceRateRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(service_rate(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = ServiceRateService::new(repo);

        service.soft_delete_service_rate(id).await.unwrap();
    }
}
