use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    CustomUnit, CustomUnitId, OrganizationId, ServiceRateUnit,
    domain::custom_unit::{commands::CreateCustomUnitCommand, ports::CustomUnitRepository},
};

pub struct CustomUnitService<R>
where
    R: CustomUnitRepository,
{
    repo: R,
}

impl<R> CustomUnitService<R>
where
    R: CustomUnitRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_custom_unit(
        &mut self,
        command: CreateCustomUnitCommand,
    ) -> Result<CustomUnit, CoreError> {
        let code = command.code.trim().to_owned();
        validate_code(&code)?;

        self.repo
            .insert(&CustomUnit {
                id: CustomUnitId(generate_uuid_v7()),
                organization_id: command.organization_id,
                code,
                created_at: Utc::now(),
            })
            .await
    }

    pub async fn list_custom_units(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<CustomUnit>, CoreError> {
        self.repo.list_by_organization(organization_id).await
    }
}

fn validate_code(code: &str) -> Result<(), CoreError> {
    if code.is_empty() {
        return Err(CoreError::Conflict(
            "custom unit code cannot be empty".to_owned(),
        ));
    }

    if ServiceRateUnit::ALL
        .iter()
        .any(|unit| unit.as_str().eq_ignore_ascii_case(code))
    {
        return Err(CoreError::Conflict(
            "custom unit code collides with a built-in unit".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::custom_unit::ports::MockCustomUnitRepository;
    use uuid::Uuid;

    #[tokio::test]
    async fn create_custom_unit_persists_via_repo() {
        let mut repo = MockCustomUnitRepository::new();
        repo.expect_insert().times(1).returning(|u| {
            let unit = u.clone();
            Box::pin(async move { Ok(unit) })
        });

        let mut service = CustomUnitService::new(repo);
        let created = service
            .create_custom_unit(CreateCustomUnitCommand {
                actor: authz::Subject::system(),
                organization_id: OrganizationId(Uuid::new_v4()),
                code: " sac ".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(created.code, "sac");
    }

    #[tokio::test]
    async fn create_custom_unit_rejects_a_blank_code() {
        let repo = MockCustomUnitRepository::new();
        let mut service = CustomUnitService::new(repo);

        let result = service
            .create_custom_unit(CreateCustomUnitCommand {
                actor: authz::Subject::system(),
                organization_id: OrganizationId(Uuid::new_v4()),
                code: "   ".to_owned(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_custom_unit_rejects_a_code_colliding_with_a_built_in_unit() {
        let repo = MockCustomUnitRepository::new();
        let mut service = CustomUnitService::new(repo);

        let result = service
            .create_custom_unit(CreateCustomUnitCommand {
                actor: authz::Subject::system(),
                organization_id: OrganizationId(Uuid::new_v4()),
                code: "hour".to_owned(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_custom_units_delegates_to_repo() {
        let org_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockCustomUnitRepository::new();
        repo.expect_list_by_organization()
            .withf(move |id| *id == org_id)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(vec![CustomUnit {
                        id: CustomUnitId(Uuid::new_v4()),
                        organization_id: org_id,
                        code: "sac".to_owned(),
                        created_at: Utc::now(),
                    }])
                })
            });

        let mut service = CustomUnitService::new(repo);
        let units = service.list_custom_units(org_id).await.unwrap();

        assert_eq!(units.len(), 1);
    }
}
