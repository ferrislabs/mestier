use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    DoeModel, DoeModelId, DoeModelSection, DoeSettings, DoeStyle, OrganizationId,
    domain::doe_model::{
        commands::{
            CreateDoeModelCommand, CreateDoeModelFromStarterCommand, UpdateDoeModelCommand,
            UpdateDoeSettingsCommand,
        },
        ports::{DoeModelRepository, DoeSettingsRepository},
    },
};

pub const MAX_SECTIONS: usize = 30;

fn blank_to_none(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_owned()).filter(|v| !v.is_empty())
}

fn validate(name: &str, sections: &[DoeModelSection]) -> Result<(), CoreError> {
    if name.trim().is_empty() {
        return Err(CoreError::Conflict(
            "DOE model name cannot be empty".to_owned(),
        ));
    }
    if sections.is_empty() {
        return Err(CoreError::Conflict(
            "a DOE model needs at least one section".to_owned(),
        ));
    }
    if sections.len() > MAX_SECTIONS {
        return Err(CoreError::Conflict(format!(
            "a DOE model has at most {MAX_SECTIONS} sections"
        )));
    }
    if sections.iter().any(|s| s.title.trim().is_empty()) {
        return Err(CoreError::Conflict(
            "every DOE model section needs a title".to_owned(),
        ));
    }
    Ok(())
}

pub struct DoeModelService<M, S>
where
    M: DoeModelRepository,
    S: DoeSettingsRepository,
{
    model_repository: M,
    settings_repository: S,
}

impl<M, S> DoeModelService<M, S>
where
    M: DoeModelRepository,
    S: DoeSettingsRepository,
{
    pub fn new(model_repository: M, settings_repository: S) -> Self {
        Self {
            model_repository,
            settings_repository,
        }
    }

    pub async fn create_model(
        &mut self,
        command: CreateDoeModelCommand,
    ) -> Result<DoeModel, CoreError> {
        validate(&command.name, &command.sections)?;

        let now = Utc::now();
        self.model_repository
            .insert(&DoeModel {
                id: DoeModelId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.name.trim().to_owned(),
                description: blank_to_none(command.description),
                sections: command.sections,
                style: command.style.map(DoeStyle::cleaned),
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn create_from_starter(
        &mut self,
        command: CreateDoeModelFromStarterCommand,
    ) -> Result<DoeModel, CoreError> {
        let now = Utc::now();
        self.model_repository
            .insert(&DoeModel {
                id: DoeModelId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name: command.starter.name().to_owned(),
                description: command.starter.description().map(str::to_owned),
                sections: command.starter.sections(),
                style: None,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_model(&mut self, id: DoeModelId) -> Result<DoeModel, CoreError> {
        self.model_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_models(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<DoeModel>, u64), CoreError> {
        self.model_repository
            .list_by_organization(organization_id, include_archived, limit, offset)
            .await
    }

    pub async fn update_model(
        &mut self,
        command: UpdateDoeModelCommand,
    ) -> Result<DoeModel, CoreError> {
        validate(&command.name, &command.sections)?;

        let mut model = self.get_model(command.id).await?;
        if model.is_archived() {
            return Err(CoreError::Conflict(
                "an archived DOE model cannot be edited".to_owned(),
            ));
        }
        model.name = command.name.trim().to_owned();
        model.description = blank_to_none(command.description);
        model.sections = command.sections;
        model.style = command.style.map(DoeStyle::cleaned);
        model.updated_at = Utc::now();

        self.model_repository.update(&model).await
    }

    pub async fn archive_model(&mut self, id: DoeModelId) -> Result<(), CoreError> {
        self.get_model(id).await?;
        self.model_repository
            .set_archived_at(id, Some(Utc::now()))
            .await
    }

    pub async fn restore_model(&mut self, id: DoeModelId) -> Result<(), CoreError> {
        self.get_model(id).await?;
        self.model_repository.set_archived_at(id, None).await
    }

    pub async fn get_settings(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<DoeSettings, CoreError> {
        Ok(self
            .settings_repository
            .find(organization_id)
            .await?
            .unwrap_or_else(|| DoeSettings::default_for(organization_id)))
    }

    pub async fn update_settings(
        &mut self,
        command: UpdateDoeSettingsCommand,
    ) -> Result<DoeSettings, CoreError> {
        self.settings_repository
            .upsert(&DoeSettings {
                organization_id: command.organization_id,
                style: command.style.cleaned(),
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DoeModelSectionKind, HexColor, MockDoeModelRepository, MockDoeSettingsRepository,
        StarterModel,
    };

    fn org() -> OrganizationId {
        OrganizationId(generate_uuid_v7())
    }

    fn plans(title: &str) -> DoeModelSection {
        DoeModelSection {
            title: title.to_owned(),
            enabled: true,
            kind: DoeModelSectionKind::Plans,
        }
    }

    fn service(
        models: MockDoeModelRepository,
        settings: MockDoeSettingsRepository,
    ) -> DoeModelService<MockDoeModelRepository, MockDoeSettingsRepository> {
        DoeModelService::new(models, settings)
    }

    fn create(name: &str, sections: Vec<DoeModelSection>) -> CreateDoeModelCommand {
        CreateDoeModelCommand {
            actor: authz::Subject::system(),
            organization_id: org(),
            name: name.to_owned(),
            description: Some("  ".to_owned()),
            sections,
            style: None,
        }
    }

    fn stored(archived: bool) -> DoeModel {
        let now = Utc::now();
        DoeModel {
            id: DoeModelId(generate_uuid_v7()),
            organization_id: org(),
            name: "M".to_owned(),
            description: None,
            sections: vec![plans("P")],
            style: None,
            archived_at: archived.then_some(now),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn create_rejects_invalid_input() {
        let many: Vec<_> = (0..31).map(|i| plans(&format!("S{i}"))).collect();
        let cases = [
            create("  ", vec![plans("A")]),
            create("M", vec![]),
            create("M", vec![plans("  ")]),
            create("M", many),
        ];
        for command in cases {
            let mut svc = service(
                MockDoeModelRepository::new(),
                MockDoeSettingsRepository::new(),
            );
            assert!(matches!(
                svc.create_model(command).await,
                Err(CoreError::Conflict(_))
            ));
        }
    }

    #[tokio::test]
    async fn create_blanks_description_and_trims_name() {
        let mut models = MockDoeModelRepository::new();
        models.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let mut svc = service(models, MockDoeSettingsRepository::new());

        let model = svc
            .create_model(create(" Mon DOE ", vec![plans("A")]))
            .await
            .unwrap();

        assert_eq!(model.name, "Mon DOE");
        assert_eq!(model.description, None);
    }

    #[tokio::test]
    async fn create_from_starter_copies_the_starter() {
        let mut models = MockDoeModelRepository::new();
        models.expect_insert().times(1).returning(|m| {
            let cloned = m.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let mut svc = service(models, MockDoeSettingsRepository::new());
        let organization_id = org();

        let model = svc
            .create_from_starter(CreateDoeModelFromStarterCommand {
                actor: authz::Subject::system(),
                organization_id,
                starter: StarterModel::Landscaping,
            })
            .await
            .unwrap();

        assert_eq!(model.organization_id, organization_id);
        assert_eq!(model.name, "Paysagiste");
        assert_eq!(model.sections, StarterModel::Landscaping.sections());
    }

    #[tokio::test]
    async fn update_refuses_an_archived_model() {
        let model = stored(true);
        let id = model.id;
        let mut models = MockDoeModelRepository::new();
        models.expect_find_by_id().returning(move |_| {
            let cloned = model.clone();
            Box::pin(async move { Ok(Some(cloned)) })
        });
        let mut svc = service(models, MockDoeSettingsRepository::new());

        let result = svc
            .update_model(UpdateDoeModelCommand {
                actor: authz::Subject::system(),
                id,
                name: "N".to_owned(),
                description: None,
                sections: vec![plans("A")],
                style: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn get_unknown_model_is_not_found() {
        let mut models = MockDoeModelRepository::new();
        models
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut svc = service(models, MockDoeSettingsRepository::new());

        assert!(matches!(
            svc.get_model(DoeModelId(generate_uuid_v7())).await,
            Err(CoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn settings_default_when_absent() {
        let mut settings = MockDoeSettingsRepository::new();
        settings
            .expect_find()
            .returning(|_| Box::pin(async { Ok(None) }));
        let mut svc = service(MockDoeModelRepository::new(), settings);
        let organization_id = org();

        let got = svc.get_settings(organization_id).await.unwrap();

        assert_eq!(got, DoeSettings::default_for(organization_id));
    }

    #[tokio::test]
    async fn update_settings_blanks_logo_key() {
        let mut settings = MockDoeSettingsRepository::new();
        settings.expect_upsert().returning(|s| {
            let cloned = s.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let mut svc = service(MockDoeModelRepository::new(), settings);

        let got = svc
            .update_settings(UpdateDoeSettingsCommand {
                actor: authz::Subject::system(),
                organization_id: org(),
                style: DoeStyle {
                    primary: HexColor::try_from("#112233".to_owned()).unwrap(),
                    logo_key: Some(" ".to_owned()),
                    ..DoeStyle::default()
                },
            })
            .await
            .unwrap();

        assert_eq!(got.style.logo_key, None);
        assert_eq!(got.style.primary.as_str(), "#112233");
    }
}
