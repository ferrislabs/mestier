use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    ClosureReason, ClosureReasonId, Establishment, EstablishmentId, OrganizationId, Promotion,
    PromotionId, Training, TrainingId,
    domain::cursus_catalog::{
        commands::{
            CreateClosureReasonCommand, CreateEstablishmentCommand, CreatePromotionCommand,
            CreateTrainingCommand, UpdateClosureReasonCommand, UpdateEstablishmentCommand,
            UpdatePromotionCommand, UpdateTrainingCommand,
        },
        ports::CursusCatalogRepository,
    },
};

pub struct CursusCatalogService<R>
where
    R: CursusCatalogRepository,
{
    repo: R,
}

impl<R> CursusCatalogService<R>
where
    R: CursusCatalogRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_establishment(
        &mut self,
        command: CreateEstablishmentCommand,
    ) -> Result<Establishment, CoreError> {
        let name = required("establishment name", &command.name)?;

        let now = Utc::now();
        self.repo
            .insert_establishment(&Establishment {
                id: EstablishmentId(generate_uuid_v7()),
                organization_id: command.organization_id,
                name,
                uai: optional(command.uai),
                siret: optional(command.siret),
                address: optional(command.address),
                is_main_site: command.is_main_site,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> Result<Establishment, CoreError> {
        self.repo
            .find_establishment(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_establishments(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<Establishment>, CoreError> {
        self.repo
            .list_establishments(organization_id, include_archived)
            .await
    }

    pub async fn update_establishment(
        &mut self,
        command: UpdateEstablishmentCommand,
    ) -> Result<Establishment, CoreError> {
        let name = required("establishment name", &command.name)?;

        let mut establishment = self
            .get_establishment(command.organization_id, command.id)
            .await?;
        establishment.name = name;
        establishment.uai = optional(command.uai);
        establishment.siret = optional(command.siret);
        establishment.address = optional(command.address);
        establishment.is_main_site = command.is_main_site;
        establishment.updated_at = Utc::now();

        self.repo.update_establishment(&establishment).await
    }

    pub async fn archive_establishment(
        &mut self,
        organization_id: OrganizationId,
        id: EstablishmentId,
    ) -> Result<Establishment, CoreError> {
        self.get_establishment(organization_id, id).await?;
        self.repo
            .archive_establishment(organization_id, id, Utc::now())
            .await
    }

    pub async fn create_training(
        &mut self,
        command: CreateTrainingCommand,
    ) -> Result<Training, CoreError> {
        let name = required("training name", &command.name)?;
        if let Some(establishment_id) = command.establishment_id {
            self.get_establishment(command.organization_id, establishment_id)
                .await?;
        }

        let now = Utc::now();
        self.repo
            .insert_training(&Training {
                id: TrainingId(generate_uuid_v7()),
                organization_id: command.organization_id,
                establishment_id: command.establishment_id,
                name,
                rncp_code: optional(command.rncp_code),
                rncp_label: optional(command.rncp_label),
                level: optional(command.level),
                degree_type: optional(command.degree_type),
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> Result<Training, CoreError> {
        self.repo
            .find_training(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_trainings(
        &mut self,
        organization_id: OrganizationId,
        establishment_id: Option<EstablishmentId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Training>, u64), CoreError> {
        self.repo
            .list_trainings(
                organization_id,
                establishment_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    pub async fn update_training(
        &mut self,
        command: UpdateTrainingCommand,
    ) -> Result<Training, CoreError> {
        let name = required("training name", &command.name)?;

        let mut training = self
            .get_training(command.organization_id, command.id)
            .await?;
        if let Some(establishment_id) = command.establishment_id
            && training.establishment_id != command.establishment_id
        {
            self.get_establishment(command.organization_id, establishment_id)
                .await?;
        }
        training.establishment_id = command.establishment_id;
        training.name = name;
        training.rncp_code = optional(command.rncp_code);
        training.rncp_label = optional(command.rncp_label);
        training.level = optional(command.level);
        training.degree_type = optional(command.degree_type);
        training.updated_at = Utc::now();

        self.repo.update_training(&training).await
    }

    pub async fn archive_training(
        &mut self,
        organization_id: OrganizationId,
        id: TrainingId,
    ) -> Result<Training, CoreError> {
        self.get_training(organization_id, id).await?;
        self.repo
            .archive_training(organization_id, id, Utc::now())
            .await
    }

    pub async fn create_promotion(
        &mut self,
        command: CreatePromotionCommand,
    ) -> Result<Promotion, CoreError> {
        let label = required("promotion label", &command.label)?;
        validate_years(command.start_year, command.end_year)?;
        self.get_training(command.organization_id, command.training_id)
            .await?;

        let now = Utc::now();
        self.repo
            .insert_promotion(&Promotion {
                id: PromotionId(generate_uuid_v7()),
                organization_id: command.organization_id,
                training_id: command.training_id,
                label,
                start_year: command.start_year,
                end_year: command.end_year,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> Result<Promotion, CoreError> {
        self.repo
            .find_promotion(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_promotions(
        &mut self,
        organization_id: OrganizationId,
        training_id: Option<TrainingId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Promotion>, u64), CoreError> {
        self.repo
            .list_promotions(
                organization_id,
                training_id,
                include_archived,
                limit,
                offset,
            )
            .await
    }

    pub async fn update_promotion(
        &mut self,
        command: UpdatePromotionCommand,
    ) -> Result<Promotion, CoreError> {
        let label = required("promotion label", &command.label)?;
        validate_years(command.start_year, command.end_year)?;

        let mut promotion = self
            .get_promotion(command.organization_id, command.id)
            .await?;
        if promotion.training_id != command.training_id {
            self.get_training(command.organization_id, command.training_id)
                .await?;
        }
        promotion.training_id = command.training_id;
        promotion.label = label;
        promotion.start_year = command.start_year;
        promotion.end_year = command.end_year;
        promotion.updated_at = Utc::now();

        self.repo.update_promotion(&promotion).await
    }

    pub async fn archive_promotion(
        &mut self,
        organization_id: OrganizationId,
        id: PromotionId,
    ) -> Result<Promotion, CoreError> {
        self.get_promotion(organization_id, id).await?;
        self.repo
            .archive_promotion(organization_id, id, Utc::now())
            .await
    }

    pub async fn create_closure_reason(
        &mut self,
        command: CreateClosureReasonCommand,
    ) -> Result<ClosureReason, CoreError> {
        let label = required("closure reason label", &command.label)?;

        self.repo
            .insert_closure_reason(&ClosureReason {
                id: ClosureReasonId(generate_uuid_v7()),
                organization_id: command.organization_id,
                label,
                archived_at: None,
                created_at: Utc::now(),
            })
            .await
    }

    pub async fn get_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> Result<ClosureReason, CoreError> {
        self.repo
            .find_closure_reason(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_closure_reasons(
        &mut self,
        organization_id: OrganizationId,
        include_archived: bool,
    ) -> Result<Vec<ClosureReason>, CoreError> {
        self.repo
            .list_closure_reasons(organization_id, include_archived)
            .await
    }

    pub async fn update_closure_reason(
        &mut self,
        command: UpdateClosureReasonCommand,
    ) -> Result<ClosureReason, CoreError> {
        let label = required("closure reason label", &command.label)?;

        let mut closure_reason = self
            .get_closure_reason(command.organization_id, command.id)
            .await?;
        closure_reason.label = label;

        self.repo.update_closure_reason(&closure_reason).await
    }

    pub async fn archive_closure_reason(
        &mut self,
        organization_id: OrganizationId,
        id: ClosureReasonId,
    ) -> Result<ClosureReason, CoreError> {
        self.get_closure_reason(organization_id, id).await?;
        self.repo
            .archive_closure_reason(organization_id, id, Utc::now())
            .await
    }
}

fn required(field: &str, value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Conflict(format!("{field} cannot be empty")));
    }

    Ok(trimmed.to_owned())
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn validate_years(start_year: i32, end_year: i32) -> Result<(), CoreError> {
    if end_year < start_year {
        return Err(CoreError::Conflict(
            "promotion end year cannot precede its start year".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::cursus_catalog::ports::MockCursusCatalogRepository;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn establishment_in(organization_id: OrganizationId, id: EstablishmentId) -> Establishment {
        let now = Utc::now();
        Establishment {
            id,
            organization_id,
            name: "Campus".to_owned(),
            uai: None,
            siret: None,
            address: None,
            is_main_site: false,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn training_in(organization_id: OrganizationId, id: TrainingId) -> Training {
        let now = Utc::now();
        Training {
            id,
            organization_id,
            establishment_id: Some(EstablishmentId(Uuid::new_v4())),
            name: "BTS".to_owned(),
            rncp_code: None,
            rncp_label: None,
            level: None,
            degree_type: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn promotion_in(organization_id: OrganizationId, id: PromotionId) -> Promotion {
        let now = Utc::now();
        Promotion {
            id,
            organization_id,
            training_id: TrainingId(Uuid::new_v4()),
            label: "2026-2028".to_owned(),
            start_year: 2026,
            end_year: 2028,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn reason_in(organization_id: OrganizationId, id: ClosureReasonId) -> ClosureReason {
        ClosureReason {
            id,
            organization_id,
            label: "Rupture".to_owned(),
            archived_at: None,
            created_at: Utc::now(),
        }
    }

    fn create_establishment(
        organization_id: OrganizationId,
        name: &str,
    ) -> CreateEstablishmentCommand {
        CreateEstablishmentCommand {
            actor: authz::Subject::system(),
            organization_id,
            name: name.to_owned(),
            uai: Some("  ".to_owned()),
            siret: Some(" 123 ".to_owned()),
            address: None,
            is_main_site: true,
        }
    }

    fn create_training(
        organization_id: OrganizationId,
        establishment_id: Option<EstablishmentId>,
    ) -> CreateTrainingCommand {
        CreateTrainingCommand {
            actor: authz::Subject::system(),
            organization_id,
            establishment_id,
            name: "BTS".to_owned(),
            rncp_code: None,
            rncp_label: None,
            level: None,
            degree_type: None,
        }
    }

    fn create_promotion(
        organization_id: OrganizationId,
        training_id: TrainingId,
        start_year: i32,
        end_year: i32,
    ) -> CreatePromotionCommand {
        CreatePromotionCommand {
            actor: authz::Subject::system(),
            organization_id,
            training_id,
            label: "2026-2028".to_owned(),
            start_year,
            end_year,
        }
    }

    #[tokio::test]
    async fn create_establishment_trims_text_and_drops_blank_options() {
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_insert_establishment().times(1).returning(|e| {
            let establishment = e.clone();
            Box::pin(async move { Ok(establishment) })
        });

        let created = CursusCatalogService::new(repo)
            .create_establishment(create_establishment(org(), "  Campus  "))
            .await
            .unwrap();

        assert_eq!(created.name, "Campus");
        assert_eq!(created.uai, None);
        assert_eq!(created.siret.as_deref(), Some("123"));
        assert!(created.is_main_site);
        assert_eq!(created.archived_at, None);
    }

    #[tokio::test]
    async fn create_establishment_rejects_a_blank_name_without_writing() {
        let repo = MockCursusCatalogRepository::new();

        let err = CursusCatalogService::new(repo)
            .create_establishment(create_establishment(org(), "   "))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_establishment_mutates_the_loaded_row() {
        let organization_id = org();
        let id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment()
            .with(eq(organization_id), eq(id))
            .returning(move |_, _| {
                Box::pin(async move { Ok(Some(establishment_in(organization_id, id))) })
            });
        repo.expect_update_establishment().times(1).returning(|e| {
            let establishment = e.clone();
            Box::pin(async move { Ok(establishment) })
        });

        let updated = CursusCatalogService::new(repo)
            .update_establishment(UpdateEstablishmentCommand {
                actor: authz::Subject::system(),
                organization_id,
                id,
                name: "Annexe".to_owned(),
                uai: Some("0123456A".to_owned()),
                siret: None,
                address: Some("1 rue de la Paix".to_owned()),
                is_main_site: true,
            })
            .await
            .unwrap();

        assert_eq!(updated.id, id);
        assert_eq!(updated.name, "Annexe");
        assert_eq!(updated.uai.as_deref(), Some("0123456A"));
        assert!(updated.is_main_site);
    }

    #[tokio::test]
    async fn get_establishment_rejects_a_row_from_another_organization() {
        let organization_id = org();
        let id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment()
            .returning(move |_, _| Box::pin(async move { Ok(Some(establishment_in(org(), id))) }));

        let err = CursusCatalogService::new(repo)
            .get_establishment(organization_id, id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn archive_establishment_checks_existence_then_archives() {
        let organization_id = org();
        let id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment().returning(move |_, _| {
            Box::pin(async move { Ok(Some(establishment_in(organization_id, id))) })
        });
        repo.expect_archive_establishment()
            .withf(move |o, i, _| *o == organization_id && *i == id)
            .times(1)
            .returning(move |o, i, at| {
                Box::pin(async move {
                    Ok(Establishment {
                        archived_at: Some(at),
                        ..establishment_in(o, i)
                    })
                })
            });

        let archived = CursusCatalogService::new(repo)
            .archive_establishment(organization_id, id)
            .await
            .unwrap();

        assert!(archived.archived_at.is_some());
    }

    #[tokio::test]
    async fn archive_establishment_of_an_unknown_id_is_not_found() {
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let err = CursusCatalogService::new(repo)
            .archive_establishment(org(), EstablishmentId(Uuid::new_v4()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_establishments_delegates_the_archived_flag() {
        let organization_id = org();
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_list_establishments()
            .with(eq(organization_id), eq(true))
            .returning(move |o, _| {
                Box::pin(
                    async move { Ok(vec![establishment_in(o, EstablishmentId(Uuid::new_v4()))]) },
                )
            });

        let items = CursusCatalogService::new(repo)
            .list_establishments(organization_id, true)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn create_training_persists_under_a_known_establishment() {
        let organization_id = org();
        let establishment_id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment()
            .with(eq(organization_id), eq(establishment_id))
            .returning(move |_, _| {
                Box::pin(
                    async move { Ok(Some(establishment_in(organization_id, establishment_id))) },
                )
            });
        repo.expect_insert_training().times(1).returning(|t| {
            let training = t.clone();
            Box::pin(async move { Ok(training) })
        });

        let created = CursusCatalogService::new(repo)
            .create_training(create_training(organization_id, Some(establishment_id)))
            .await
            .unwrap();

        assert_eq!(created.establishment_id, Some(establishment_id));
        assert_eq!(created.organization_id, organization_id);
    }

    #[tokio::test]
    async fn create_training_rejects_an_unknown_establishment_without_writing() {
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let err = CursusCatalogService::new(repo)
            .create_training(create_training(
                org(),
                Some(EstablishmentId(Uuid::new_v4())),
            ))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn create_training_rejects_an_establishment_of_another_organization() {
        let establishment_id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment().returning(move |_, _| {
            Box::pin(async move { Ok(Some(establishment_in(org(), establishment_id))) })
        });

        let err = CursusCatalogService::new(repo)
            .create_training(create_training(org(), Some(establishment_id)))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn create_training_without_an_establishment_skips_the_lookup() {
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_establishment().never();
        repo.expect_insert_training().times(1).returning(|t| {
            let training = t.clone();
            Box::pin(async move { Ok(training) })
        });

        let created = CursusCatalogService::new(repo)
            .create_training(create_training(org(), None))
            .await
            .unwrap();

        assert_eq!(created.establishment_id, None);
    }

    #[tokio::test]
    async fn update_training_moving_to_a_foreign_establishment_is_not_found() {
        let organization_id = org();
        let id = TrainingId(Uuid::new_v4());
        let target = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_training().returning(move |_, _| {
            Box::pin(async move { Ok(Some(training_in(organization_id, id))) })
        });
        repo.expect_find_establishment().returning(move |_, _| {
            Box::pin(async move { Ok(Some(establishment_in(org(), target))) })
        });

        let err = CursusCatalogService::new(repo)
            .update_training(UpdateTrainingCommand {
                actor: authz::Subject::system(),
                organization_id,
                id,
                establishment_id: Some(target),
                name: "BTS".to_owned(),
                rncp_code: None,
                rncp_label: None,
                level: None,
                degree_type: None,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn update_training_keeping_its_establishment_skips_the_lookup() {
        let organization_id = org();
        let id = TrainingId(Uuid::new_v4());
        let existing = training_in(organization_id, id);
        let establishment_id = existing.establishment_id;
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_training().returning(move |_, _| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });
        repo.expect_update_training().times(1).returning(|t| {
            let training = t.clone();
            Box::pin(async move { Ok(training) })
        });

        let updated = CursusCatalogService::new(repo)
            .update_training(UpdateTrainingCommand {
                actor: authz::Subject::system(),
                organization_id,
                id,
                establishment_id,
                name: "Licence".to_owned(),
                rncp_code: Some("RNCP1".to_owned()),
                rncp_label: None,
                level: Some("6".to_owned()),
                degree_type: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "Licence");
        assert_eq!(updated.rncp_code.as_deref(), Some("RNCP1"));
    }

    #[tokio::test]
    async fn archive_training_delegates_after_the_existence_check() {
        let organization_id = org();
        let id = TrainingId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_training().returning(move |_, _| {
            Box::pin(async move { Ok(Some(training_in(organization_id, id))) })
        });
        repo.expect_archive_training()
            .times(1)
            .returning(|o, i, at| {
                Box::pin(async move {
                    Ok(Training {
                        archived_at: Some(at),
                        ..training_in(o, i)
                    })
                })
            });

        let archived = CursusCatalogService::new(repo)
            .archive_training(organization_id, id)
            .await
            .unwrap();

        assert!(archived.archived_at.is_some());
    }

    #[tokio::test]
    async fn list_trainings_delegates_filter_and_paging() {
        let organization_id = org();
        let establishment_id = EstablishmentId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_list_trainings()
            .with(
                eq(organization_id),
                eq(Some(establishment_id)),
                eq(false),
                eq(10),
                eq(20),
            )
            .returning(|_, _, _, _, _| Box::pin(async { Ok((Vec::new(), 0)) }));

        let (items, total) = CursusCatalogService::new(repo)
            .list_trainings(organization_id, Some(establishment_id), false, 10, 20)
            .await
            .unwrap();

        assert!(items.is_empty());
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn create_promotion_accepts_a_single_year_span() {
        let organization_id = org();
        let training_id = TrainingId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_training().returning(move |_, _| {
            Box::pin(async move { Ok(Some(training_in(organization_id, training_id))) })
        });
        repo.expect_insert_promotion().times(1).returning(|p| {
            let promotion = p.clone();
            Box::pin(async move { Ok(promotion) })
        });

        let created = CursusCatalogService::new(repo)
            .create_promotion(create_promotion(organization_id, training_id, 2026, 2026))
            .await
            .unwrap();

        assert_eq!((created.start_year, created.end_year), (2026, 2026));
    }

    #[tokio::test]
    async fn create_promotion_rejects_an_end_year_before_the_start_year() {
        let repo = MockCursusCatalogRepository::new();

        let err = CursusCatalogService::new(repo)
            .create_promotion(create_promotion(
                org(),
                TrainingId(Uuid::new_v4()),
                2028,
                2026,
            ))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_promotion_rejects_a_training_of_another_organization() {
        let training_id = TrainingId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_training().returning(move |_, _| {
            Box::pin(async move { Ok(Some(training_in(org(), training_id))) })
        });

        let err = CursusCatalogService::new(repo)
            .create_promotion(create_promotion(org(), training_id, 2026, 2028))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn update_promotion_rejects_an_end_year_before_the_start_year() {
        let repo = MockCursusCatalogRepository::new();

        let err = CursusCatalogService::new(repo)
            .update_promotion(UpdatePromotionCommand {
                actor: authz::Subject::system(),
                organization_id: org(),
                id: PromotionId(Uuid::new_v4()),
                training_id: TrainingId(Uuid::new_v4()),
                label: "2026-2028".to_owned(),
                start_year: 2028,
                end_year: 2027,
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn update_promotion_mutates_the_loaded_row() {
        let organization_id = org();
        let id = PromotionId(Uuid::new_v4());
        let existing = promotion_in(organization_id, id);
        let training_id = existing.training_id;
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_promotion().returning(move |_, _| {
            let existing = existing.clone();
            Box::pin(async move { Ok(Some(existing)) })
        });
        repo.expect_update_promotion().times(1).returning(|p| {
            let promotion = p.clone();
            Box::pin(async move { Ok(promotion) })
        });

        let updated = CursusCatalogService::new(repo)
            .update_promotion(UpdatePromotionCommand {
                actor: authz::Subject::system(),
                organization_id,
                id,
                training_id,
                label: "2027-2029".to_owned(),
                start_year: 2027,
                end_year: 2029,
            })
            .await
            .unwrap();

        assert_eq!(updated.label, "2027-2029");
        assert_eq!(updated.end_year, 2029);
    }

    #[tokio::test]
    async fn archive_promotion_delegates_after_the_existence_check() {
        let organization_id = org();
        let id = PromotionId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_promotion().returning(move |_, _| {
            Box::pin(async move { Ok(Some(promotion_in(organization_id, id))) })
        });
        repo.expect_archive_promotion()
            .times(1)
            .returning(|o, i, at| {
                Box::pin(async move {
                    Ok(Promotion {
                        archived_at: Some(at),
                        ..promotion_in(o, i)
                    })
                })
            });

        let archived = CursusCatalogService::new(repo)
            .archive_promotion(organization_id, id)
            .await
            .unwrap();

        assert!(archived.archived_at.is_some());
    }

    #[tokio::test]
    async fn create_closure_reason_rejects_a_blank_label() {
        let repo = MockCursusCatalogRepository::new();

        let err = CursusCatalogService::new(repo)
            .create_closure_reason(CreateClosureReasonCommand {
                actor: authz::Subject::system(),
                organization_id: org(),
                label: " ".to_owned(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_closure_reason_persists_a_trimmed_label() {
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_insert_closure_reason().times(1).returning(|r| {
            let reason = r.clone();
            Box::pin(async move { Ok(reason) })
        });

        let created = CursusCatalogService::new(repo)
            .create_closure_reason(CreateClosureReasonCommand {
                actor: authz::Subject::system(),
                organization_id: org(),
                label: " Abandon ".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(created.label, "Abandon");
    }

    #[tokio::test]
    async fn update_closure_reason_renames_the_loaded_row() {
        let organization_id = org();
        let id = ClosureReasonId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_closure_reason().returning(move |_, _| {
            Box::pin(async move { Ok(Some(reason_in(organization_id, id))) })
        });
        repo.expect_update_closure_reason().times(1).returning(|r| {
            let reason = r.clone();
            Box::pin(async move { Ok(reason) })
        });

        let updated = CursusCatalogService::new(repo)
            .update_closure_reason(UpdateClosureReasonCommand {
                actor: authz::Subject::system(),
                organization_id,
                id,
                label: "Fin anticipée".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(updated.label, "Fin anticipée");
    }

    #[tokio::test]
    async fn archive_closure_reason_of_another_organization_is_not_found() {
        let id = ClosureReasonId(Uuid::new_v4());
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_find_closure_reason()
            .returning(move |_, _| Box::pin(async move { Ok(Some(reason_in(org(), id))) }));

        let err = CursusCatalogService::new(repo)
            .archive_closure_reason(org(), id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn list_closure_reasons_delegates_the_archived_flag() {
        let organization_id = org();
        let mut repo = MockCursusCatalogRepository::new();
        repo.expect_list_closure_reasons()
            .with(eq(organization_id), eq(false))
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));

        let items = CursusCatalogService::new(repo)
            .list_closure_reasons(organization_id, false)
            .await
            .unwrap();

        assert!(items.is_empty());
    }
}
