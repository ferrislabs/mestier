use std::collections::HashSet;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    Equipment, EquipmentId, OrganizationId, TaskId,
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

    /// Replaces the complete set of equipment attached to `task_id` — never
    /// a delta, mirroring `TaskLabelService::replace_task_labels`'s own
    /// contract: idempotence, and a single path for both attaching and
    /// detaching a piece of equipment. Every id in `equipment_ids` must name
    /// equipment of `organization_id`; an unknown id, or one belonging to a
    /// different organization, is rejected as `NotFound` before anything is
    /// written. Repeated ids collapse to one.
    pub async fn replace_task_equipment(
        &mut self,
        organization_id: OrganizationId,
        task_id: TaskId,
        equipment_ids: Vec<EquipmentId>,
    ) -> Result<(), CoreError> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::with_capacity(equipment_ids.len());

        for equipment_id in equipment_ids {
            if !seen.insert(equipment_id) {
                continue;
            }

            let equipment = self
                .repo
                .find_by_id(equipment_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            if equipment.organization_id != organization_id {
                return Err(CoreError::NotFound);
            }

            deduped.push(equipment_id);
        }

        self.repo.replace_task_links(task_id, &deduped).await
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

    // -- replace_task_equipment ---------------------------------------------

    fn equipment_in(id: EquipmentId, organization_id: OrganizationId) -> Equipment {
        Equipment {
            organization_id,
            ..equipment(id)
        }
    }

    #[tokio::test]
    async fn replace_task_equipment_replaces_links_with_the_validated_set() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let equipment_a = EquipmentId(Uuid::new_v4());
        let equipment_b = EquipmentId(Uuid::new_v4());

        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(equipment_a))
            .returning(move |id| {
                let e = equipment_in(id, organization_id);
                Box::pin(async move { Ok(Some(e)) })
            });
        repo.expect_find_by_id()
            .with(eq(equipment_b))
            .returning(move |id| {
                let e = equipment_in(id, organization_id);
                Box::pin(async move { Ok(Some(e)) })
            });
        repo.expect_replace_task_links()
            .withf(move |t, ids| *t == task_id && ids == [equipment_a, equipment_b])
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EquipmentService::new(repo);
        service
            .replace_task_equipment(organization_id, task_id, vec![equipment_a, equipment_b])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_equipment_dedupes_repeated_ids() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let equipment_a = EquipmentId(Uuid::new_v4());

        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(equipment_a))
            .returning(move |id| {
                let e = equipment_in(id, organization_id);
                Box::pin(async move { Ok(Some(e)) })
            });
        repo.expect_replace_task_links()
            .withf(move |t, ids| *t == task_id && ids == [equipment_a])
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EquipmentService::new(repo);
        service
            .replace_task_equipment(organization_id, task_id, vec![equipment_a, equipment_a])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_equipment_clears_every_link_for_an_empty_list() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());

        let mut repo = MockEquipmentRepository::new();
        repo.expect_replace_task_links()
            .withf(move |t, ids| *t == task_id && ids.is_empty())
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = EquipmentService::new(repo);
        service
            .replace_task_equipment(organization_id, task_id, Vec::new())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_equipment_rejects_an_unknown_equipment_id() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let unknown = EquipmentId(Uuid::new_v4());

        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(unknown))
            .returning(|_| Box::pin(async { Ok(None) }));
        // No `expect_replace_task_links`: nothing must be written once one id
        // in the list fails validation.

        let mut service = EquipmentService::new(repo);
        let err = service
            .replace_task_equipment(organization_id, task_id, vec![unknown])
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn replace_task_equipment_rejects_equipment_from_another_organization() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let foreign_equipment = EquipmentId(Uuid::new_v4());

        let mut repo = MockEquipmentRepository::new();
        repo.expect_find_by_id()
            .with(eq(foreign_equipment))
            .returning(move |id| {
                let e = equipment_in(id, other_org_id);
                Box::pin(async move { Ok(Some(e)) })
            });

        let mut service = EquipmentService::new(repo);
        let err = service
            .replace_task_equipment(organization_id, task_id, vec![foreign_equipment])
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }
}
