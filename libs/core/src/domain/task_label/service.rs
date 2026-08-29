use std::collections::HashSet;

use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, TaskId, TaskLabel, TaskLabelId,
    domain::task_label::{
        commands::{CreateTaskLabelCommand, UpdateTaskLabelCommand},
        ports::TaskLabelRepository,
    },
};

/// Translates the DB's unique-violation constraint name into a
/// business-friendly message — mirrors
/// `organization::service::map_organization_conflict`.
fn map_task_label_conflict(err: CoreError) -> CoreError {
    match err {
        CoreError::Conflict(constraint) if constraint == "uq_task_labels_org_id_name" => {
            CoreError::Conflict(
                "a label with this name already exists in this organization".to_owned(),
            )
        }
        other => other,
    }
}

fn validate_name(name: &str) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "task label name cannot be blank".to_owned(),
        ));
    }

    Ok(())
}

/// Mirrors `chk_task_labels_color_format`: a strict `#RRGGBB` hex value.
fn validate_color(color: &str) -> Result<(), CoreError> {
    let is_valid = color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|c| c.is_ascii_hexdigit());

    if !is_valid {
        return Err(CoreError::Conflict(
            "task label color must be a #RRGGBB hex value".to_owned(),
        ));
    }

    Ok(())
}

pub struct TaskLabelService<R>
where
    R: TaskLabelRepository,
{
    repo: R,
}

impl<R> TaskLabelService<R>
where
    R: TaskLabelRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_task_label(
        &mut self,
        command: CreateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        validate_name(&command.name)?;
        validate_color(&command.color)?;

        let now = Utc::now();
        self.repo
            .insert(&TaskLabel {
                id: TaskLabelId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name,
                color: command.color,
                created_at: now,
                updated_at: now,
            })
            .await
            .map_err(map_task_label_conflict)
    }

    pub async fn get_task_label(&mut self, id: TaskLabelId) -> Result<TaskLabel, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_task_labels(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskLabel>, CoreError> {
        self.repo.list_by_organization(organization_id).await
    }

    pub async fn update_task_label(
        &mut self,
        command: UpdateTaskLabelCommand,
    ) -> Result<TaskLabel, CoreError> {
        validate_name(&command.name)?;
        validate_color(&command.color)?;

        let mut label = self.get_task_label(command.id).await?;
        label.name = command.name;
        label.color = command.color;
        label.updated_at = Utc::now();

        self.repo
            .update(&label)
            .await
            .map_err(map_task_label_conflict)
    }

    pub async fn delete_task_label(&mut self, id: TaskLabelId) -> Result<(), CoreError> {
        self.get_task_label(id).await?;
        self.repo.delete(id).await
    }

    /// Replaces the complete set of labels attached to `task_id` — never a
    /// delta, mirroring the `PATCH` contract for `assignees`: idempotence,
    /// and a single path for both adding and removing a label. Every id in
    /// `label_ids` must name a label of `organization_id`; an unknown id, or
    /// one belonging to a different organization, is rejected as
    /// `NotFound` before anything is written. Repeated ids collapse to one.
    pub async fn replace_task_labels(
        &mut self,
        organization_id: OrganizationId,
        task_id: TaskId,
        label_ids: Vec<TaskLabelId>,
    ) -> Result<(), CoreError> {
        let mut seen = HashSet::new();
        let mut deduped = Vec::with_capacity(label_ids.len());

        for label_id in label_ids {
            if !seen.insert(label_id) {
                continue;
            }

            let label = self
                .repo
                .find_by_id(label_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            if label.organization_id != organization_id {
                return Err(CoreError::NotFound);
            }

            deduped.push(label_id);
        }

        self.repo.replace_links(task_id, &deduped).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task_label::ports::MockTaskLabelRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn label(id: TaskLabelId, organization_id: OrganizationId) -> TaskLabel {
        let now = Utc::now();
        TaskLabel {
            id,
            organization_id,
            name: "Réunion".to_owned(),
            color: "#2563EB".to_owned(),
            created_at: now,
            updated_at: now,
        }
    }

    fn create_command(organization_id: OrganizationId) -> CreateTaskLabelCommand {
        CreateTaskLabelCommand {
            actor: authz::Subject::system(),
            organization_id,
            name: "Réunion".to_owned(),
            color: "#2563EB".to_owned(),
        }
    }

    // -- create_task_label ---------------------------------------------------

    #[tokio::test]
    async fn create_task_label_persists_via_repo() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_insert().times(1).returning(|l| {
            let cloned = l.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = TaskLabelService::new(repo);
        let created = service
            .create_task_label(create_command(organization_id))
            .await
            .unwrap();

        assert_eq!(created.name, "Réunion");
        assert_eq!(created.color, "#2563EB");
        assert_eq!(created.organization_id, organization_id);
    }

    #[tokio::test]
    async fn create_task_label_rejects_a_blank_name() {
        let mut service = TaskLabelService::new(MockTaskLabelRepository::new());

        let mut command = create_command(OrganizationId(Uuid::new_v4()));
        command.name = "   ".to_owned();

        let err = service.create_task_label(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_label_rejects_a_malformed_color() {
        let mut service = TaskLabelService::new(MockTaskLabelRepository::new());

        let mut command = create_command(OrganizationId(Uuid::new_v4()));
        command.color = "blue".to_owned();

        let err = service.create_task_label(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_label_rejects_a_color_missing_the_hash() {
        let mut service = TaskLabelService::new(MockTaskLabelRepository::new());

        let mut command = create_command(OrganizationId(Uuid::new_v4()));
        command.color = "2563EB".to_owned();

        let err = service.create_task_label(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_label_translates_a_duplicate_name_into_a_friendly_conflict() {
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_insert().times(1).returning(|_| {
            Box::pin(async { Err(CoreError::Conflict("uq_task_labels_org_id_name".into())) })
        });

        let mut service = TaskLabelService::new(repo);
        let err = service
            .create_task_label(create_command(OrganizationId(Uuid::new_v4())))
            .await
            .unwrap_err();

        match err {
            CoreError::Conflict(msg) => {
                assert_eq!(
                    msg,
                    "a label with this name already exists in this organization"
                )
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    // -- get_task_label -------------------------------------------------------

    #[tokio::test]
    async fn get_task_label_returns_not_found_when_missing() {
        let id = TaskLabelId(Uuid::new_v4());
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = TaskLabelService::new(repo);
        let err = service.get_task_label(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_task_label_returns_entity_when_found() {
        let id = TaskLabelId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let l = label(id, organization_id);
            Box::pin(async move { Ok(Some(l)) })
        });

        let mut service = TaskLabelService::new(repo);
        let found = service.get_task_label(id).await.unwrap();

        assert_eq!(found.id, id);
    }

    // -- list_task_labels -------------------------------------------------------

    #[tokio::test]
    async fn list_task_labels_delegates_to_repo() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_list_by_organization()
            .with(eq(organization_id))
            .returning(move |org_id| {
                Box::pin(async move { Ok(vec![label(TaskLabelId(Uuid::new_v4()), org_id)]) })
            });

        let mut service = TaskLabelService::new(repo);
        let labels = service.list_task_labels(organization_id).await.unwrap();

        assert_eq!(labels.len(), 1);
    }

    // -- update_task_label ------------------------------------------------------

    #[tokio::test]
    async fn update_task_label_mutates_existing_label() {
        let id = TaskLabelId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = label(id, organization_id);

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let l = existing.clone();
            Box::pin(async move { Ok(Some(l)) })
        });
        repo.expect_update()
            .withf(|l| l.name == "Chantier extérieur" && l.color == "#059669")
            .returning(|l| {
                let cloned = l.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = TaskLabelService::new(repo);
        let updated = service
            .update_task_label(UpdateTaskLabelCommand {
                actor: authz::Subject::system(),
                id,
                name: "Chantier extérieur".to_owned(),
                color: "#059669".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Chantier extérieur");
        assert_eq!(updated.color, "#059669");
    }

    #[tokio::test]
    async fn update_task_label_rejects_a_blank_name() {
        let mut service = TaskLabelService::new(MockTaskLabelRepository::new());

        let err = service
            .update_task_label(UpdateTaskLabelCommand {
                actor: authz::Subject::system(),
                id: TaskLabelId(Uuid::new_v4()),
                name: "  ".to_owned(),
                color: "#2563EB".to_owned(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_task_label_translates_a_duplicate_name_into_a_friendly_conflict() {
        let id = TaskLabelId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = label(id, organization_id);

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let l = existing.clone();
            Box::pin(async move { Ok(Some(l)) })
        });
        repo.expect_update().returning(|_| {
            Box::pin(async { Err(CoreError::Conflict("uq_task_labels_org_id_name".into())) })
        });

        let mut service = TaskLabelService::new(repo);
        let err = service
            .update_task_label(UpdateTaskLabelCommand {
                actor: authz::Subject::system(),
                id,
                name: "Déplacement".to_owned(),
                color: "#059669".to_owned(),
            })
            .await
            .unwrap_err();

        match err {
            CoreError::Conflict(msg) => {
                assert_eq!(
                    msg,
                    "a label with this name already exists in this organization"
                )
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    // -- delete_task_label --------------------------------------------------

    #[tokio::test]
    async fn delete_task_label_checks_existence_then_deletes() {
        let id = TaskLabelId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let existing = label(id, organization_id);

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let l = existing.clone();
            Box::pin(async move { Ok(Some(l)) })
        });
        repo.expect_delete()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(()) }));

        let mut service = TaskLabelService::new(repo);
        service.delete_task_label(id).await.unwrap();
    }

    #[tokio::test]
    async fn delete_task_label_returns_not_found_when_missing() {
        let id = TaskLabelId(Uuid::new_v4());
        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));
        // No `expect_delete`: a missing label must be rejected before any
        // delete is attempted.

        let mut service = TaskLabelService::new(repo);
        let err = service.delete_task_label(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    // -- replace_task_labels -----------------------------------------------

    #[tokio::test]
    async fn replace_task_labels_replaces_links_with_the_validated_set() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let label_a = TaskLabelId(Uuid::new_v4());
        let label_b = TaskLabelId(Uuid::new_v4());

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(label_a))
            .returning(move |id| {
                let l = label(id, organization_id);
                Box::pin(async move { Ok(Some(l)) })
            });
        repo.expect_find_by_id()
            .with(eq(label_b))
            .returning(move |id| {
                let l = label(id, organization_id);
                Box::pin(async move { Ok(Some(l)) })
            });
        repo.expect_replace_links()
            .withf(move |t, ids| *t == task_id && ids == [label_a, label_b])
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = TaskLabelService::new(repo);
        service
            .replace_task_labels(organization_id, task_id, vec![label_a, label_b])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_labels_dedupes_repeated_ids() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let label_a = TaskLabelId(Uuid::new_v4());

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(label_a))
            .returning(move |id| {
                let l = label(id, organization_id);
                Box::pin(async move { Ok(Some(l)) })
            });
        repo.expect_replace_links()
            .withf(move |t, ids| *t == task_id && ids == [label_a])
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = TaskLabelService::new(repo);
        service
            .replace_task_labels(organization_id, task_id, vec![label_a, label_a])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_labels_clears_every_link_for_an_empty_list() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_replace_links()
            .withf(move |t, ids| *t == task_id && ids.is_empty())
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = TaskLabelService::new(repo);
        service
            .replace_task_labels(organization_id, task_id, Vec::new())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn replace_task_labels_rejects_an_unknown_label_id() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let unknown = TaskLabelId(Uuid::new_v4());

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(unknown))
            .returning(|_| Box::pin(async { Ok(None) }));
        // No `expect_replace_links`: nothing must be written once one id in
        // the list fails validation.

        let mut service = TaskLabelService::new(repo);
        let err = service
            .replace_task_labels(organization_id, task_id, vec![unknown])
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn replace_task_labels_rejects_a_label_from_another_organization() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let other_org_id = OrganizationId(Uuid::new_v4());
        let task_id = TaskId(Uuid::new_v4());
        let foreign_label = TaskLabelId(Uuid::new_v4());

        let mut repo = MockTaskLabelRepository::new();
        repo.expect_find_by_id()
            .with(eq(foreign_label))
            .returning(move |id| {
                let l = label(id, other_org_id);
                Box::pin(async move { Ok(Some(l)) })
            });

        let mut service = TaskLabelService::new(repo);
        let err = service
            .replace_task_labels(organization_id, task_id, vec![foreign_label])
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }
}
