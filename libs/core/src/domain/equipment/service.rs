use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Equipment, EquipmentId, OrganizationId,
    domain::equipment::{
        commands::{CreateEquipmentCommand, UpdateEquipmentCommand},
        ports::EquipmentRepository,
    },
};

pub struct EquipmentService<R>
where
    R: EquipmentRepository,
{
    repo: R,
}

impl<R> EquipmentService<R>
where
    R: EquipmentRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_equipment(
        &mut self,
        command: CreateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
        validate_name(&command.name)?;
        validate_rate(command.hourly_rate_cents)?;

        let now = Utc::now();
        self.repo
            .insert(&Equipment {
                id: EquipmentId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                hourly_rate_cents: command.hourly_rate_cents,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_equipment(&mut self, id: EquipmentId) -> Result<Equipment, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_equipment(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Equipment>, u64), CoreError> {
        self.repo
            .list_by_organization(organization_id, limit, offset)
            .await
    }

    pub async fn update_equipment(
        &mut self,
        command: UpdateEquipmentCommand,
    ) -> Result<Equipment, CoreError> {
        validate_name(&command.name)?;
        validate_rate(command.hourly_rate_cents)?;

        let mut equipment = self.get_equipment(command.id).await?;
        equipment.name = command.name;
        equipment.hourly_rate_cents = command.hourly_rate_cents;
        equipment.updated_at = Utc::now();

        self.repo.update(&equipment).await
    }

    pub async fn soft_delete_equipment(&mut self, id: EquipmentId) -> Result<(), CoreError> {
        self.get_equipment(id).await?;
        self.repo.soft_delete(id, Utc::now()).await
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "equipment name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

fn validate_rate(rate_cents: i32) -> Result<(), CoreError> {
    if rate_cents < 0 {
        return Err(CoreError::Conflict(
            "equipment hourly rate cannot be negative".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::equipment::ports::MockEquipmentRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn equipment(id: EquipmentId) -> Equipment {
        let now = Utc::now();
        Equipment {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            name: "Truck".to_owned(),
            hourly_rate_cents: 1200,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_equipment_persists_via_repo() {
        let mut repo = MockEquipmentRepository::new();
        repo.expect_insert().times(1).returning(|e| {
            let equipment = e.clone();
            Box::pin(async move { Ok(equipment) })
        });

        let mut service = EquipmentService::new(repo);
        let created = service
            .create_equipment(CreateEquipmentCommand {
                organization_id: OrganizationId(Uuid::new_v4()),
                name: "Truck".to_owned(),
                hourly_rate_cents: 1200,
            })
            .await
            .unwrap();

        assert_eq!(created.name, "Truck");
    }

    #[tokio::test]
    async fn update_equipment_mutates_existing_equipment() {
        let id = EquipmentId(Uuid::new_v4());
        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(equipment(id))) }));
        repo.expect_update().times(1).returning(|e| {
            let equipment = e.clone();
            Box::pin(async move { Ok(equipment) })
        });

        let mut service = EquipmentService::new(repo);
        let updated = service
            .update_equipment(UpdateEquipmentCommand {
                id,
                name: "Mower".to_owned(),
                hourly_rate_cents: 900,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Mower");
        assert_eq!(updated.hourly_rate_cents, 900);
    }

    #[tokio::test]
    async fn list_equipment_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockEquipmentRepository::new();
        repo.expect_list_by_organization()
            .with(eq(org_id), eq(10), eq(20))
            .returning(move |_, _, _| {
                Box::pin(async move { Ok((vec![equipment(EquipmentId(Uuid::new_v4()))], 1)) })
            });

        let mut service = EquipmentService::new(repo);
        let (items, total) = service.list_equipment(org_id, 10, 20).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn soft_delete_equipment_checks_existence_then_deletes() {
        let id = EquipmentId(Uuid::new_v4());
        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(move |_| Box::pin(async move { Ok(Some(equipment(id))) }));
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EquipmentService::new(repo);

        service.soft_delete_equipment(id).await.unwrap();
    }
}
