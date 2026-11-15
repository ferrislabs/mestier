use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use mestier_macros::transactional;

use super::commands::{
    CampaignCommand, CompletedSurveyResponses, CreateCampaignCommand, CreateSurveyCommand,
    ListCampaignSurveysCommand, ListCampaignsCommand, ListSurveyResponsesCommand,
    ListSurveysCommand, OpenSurveyInCampaignCommand, SetSurveyCampaignsCommand, SurveyCommand,
    SurveyGraphVersionCommand, SurveyResponseCommand, UpdateCampaignCommand, UpdateSurveyCommand,
};
use super::{former_graphs, guard_manage, guard_read, new_public_slug, require_graphs};
use crate::application::MestierUseCase;
use crate::domain::file_storage::{ports::FileStorage, service::FileStorageService};
use crate::domain::survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository};
use crate::domain::survey::service::{CampaignService, ResponseService, SurveyService};
use crate::domain::survey::{
    Campaign, CampaignId, Closed, Draft, Opened, ResponseRecord, Survey, SurveyError, SurveyId,
    SurveyRecord, SurveySummary, VersionedGraph, Visibility,
};

impl MestierUseCase {
    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn create_survey(
        &self,
        command: CreateSurveyCommand,
    ) -> Result<Survey<Draft>, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        let created = service
            .create(
                command.organization_id,
                SurveyId(generate_uuid_v7()),
                command.title,
                command.visibility,
            )
            .await?;
        if command.campaign_ids.is_empty() {
            return Ok(created);
        }
        match service
            .set_campaigns(command.organization_id, created.id(), command.campaign_ids)
            .await?
        {
            SurveyRecord::Draft(survey) => Ok(survey),
            SurveyRecord::Opened(_) | SurveyRecord::Closed(_) => {
                Err(SurveyError::SurveyNotDraft.into())
            }
        }
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn update_survey(
        &self,
        command: UpdateSurveyCommand,
    ) -> Result<SurveyRecord, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        let updated = service
            .update(command.organization_id, command.id, command.patch)
            .await?;
        match command.campaign_ids {
            Some(campaign_ids) => {
                service
                    .set_campaigns(command.organization_id, command.id, campaign_ids)
                    .await
            }
            None => Ok(updated),
        }
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn set_survey_campaigns(
        &self,
        command: SetSurveyCampaignsCommand,
    ) -> Result<SurveyRecord, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        service
            .set_campaigns(command.organization_id, command.id, command.campaign_ids)
            .await
    }

    #[transactional(survey, feature_flag, member, role, authz)]
    pub async fn get_survey_graph_version(
        &self,
        command: SurveyGraphVersionCommand,
    ) -> Result<VersionedGraph, CoreError> {
        let mut survey_repository = survey_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let survey = survey_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or(SurveyError::SurveyNotFound)?;
        if survey.graph_version() == Some(command.version)
            && let Some(graph) = survey.graph()
        {
            return Ok(VersionedGraph::new(command.version, graph.clone()));
        }
        let graph = survey_repository
            .find_graph_version(command.organization_id, command.id, command.version)
            .await?
            .ok_or(SurveyError::GraphVersionNotFound)?;
        Ok(VersionedGraph::new(command.version, graph))
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz)]
    pub async fn list_campaign_surveys(
        &self,
        command: ListCampaignSurveysCommand,
    ) -> Result<(Vec<SurveySummary>, u64), CoreError> {
        let mut survey_repository = survey_repository;
        let mut survey_campaign_repository = survey_campaign_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        survey_campaign_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or(SurveyError::CampaignNotFound)?;
        survey_repository
            .list_summaries_by_campaign(
                command.organization_id,
                command.id,
                command.limit,
                command.offset,
            )
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn open_survey(&self, command: SurveyCommand) -> Result<Survey<Opened>, CoreError> {
        let mut survey_repository = survey_repository;
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let public_slug = match survey_repository
            .find(command.organization_id, command.id)
            .await?
        {
            Some(SurveyRecord::Draft(survey)) if survey.visibility() == Visibility::Public => {
                Some(new_public_slug()?)
            }
            _ => None,
        };
        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        service
            .open(command.organization_id, command.id, Utc::now(), public_slug)
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn open_survey_in_campaign(
        &self,
        command: OpenSurveyInCampaignCommand,
    ) -> Result<Survey<Opened>, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        let attached = service
            .add_campaign(command.organization_id, command.id, command.campaign_id)
            .await?;
        let public_slug = match attached {
            SurveyRecord::Draft(survey) => match survey.visibility() {
                Visibility::Public => Some(new_public_slug()?),
                Visibility::Members => None,
            },
            SurveyRecord::Opened(_) | SurveyRecord::Closed(_) => {
                return Err(SurveyError::SurveyNotDraft.into());
            }
        };
        service
            .open(command.organization_id, command.id, Utc::now(), public_slug)
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn close_survey(&self, command: SurveyCommand) -> Result<Survey<Closed>, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        service
            .close(command.organization_id, command.id, Utc::now())
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn duplicate_survey(
        &self,
        command: SurveyCommand,
    ) -> Result<Survey<Draft>, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        service
            .duplicate(
                command.organization_id,
                command.id,
                SurveyId(generate_uuid_v7()),
            )
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn delete_survey(&self, command: SurveyCommand) -> Result<(), CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            SurveyService::new(survey_repository, survey_campaign_repository, emitter);
        service.delete(command.organization_id, command.id).await
    }

    #[transactional(survey, feature_flag, member, role, authz)]
    pub async fn list_surveys(
        &self,
        command: ListSurveysCommand,
    ) -> Result<(Vec<SurveyRecord>, u64), CoreError> {
        let mut survey_repository = survey_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        match command.campaign_id {
            Some(campaign_id) => {
                survey_repository
                    .list_by_campaign(
                        command.organization_id,
                        campaign_id,
                        command.limit,
                        command.offset,
                    )
                    .await
            }
            None => {
                survey_repository
                    .list(command.organization_id, command.limit, command.offset)
                    .await
            }
        }
    }

    #[transactional(survey, feature_flag, member, role, authz)]
    pub async fn get_survey(&self, command: SurveyCommand) -> Result<SurveyRecord, CoreError> {
        let mut survey_repository = survey_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        survey_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or_else(|| SurveyError::SurveyNotFound.into())
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn create_campaign(
        &self,
        command: CreateCampaignCommand,
    ) -> Result<Campaign, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            CampaignService::new(survey_campaign_repository, survey_repository, emitter);
        service
            .create(
                command.organization_id,
                CampaignId(generate_uuid_v7()),
                command.name,
                command.window,
                command.metadata,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey_campaign, feature_flag, member, role, authz)]
    pub async fn get_campaign(&self, command: CampaignCommand) -> Result<Campaign, CoreError> {
        let mut survey_campaign_repository = survey_campaign_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        survey_campaign_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or_else(|| SurveyError::CampaignNotFound.into())
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn update_campaign(
        &self,
        command: UpdateCampaignCommand,
    ) -> Result<Campaign, CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            CampaignService::new(survey_campaign_repository, survey_repository, emitter);
        service
            .update(
                command.organization_id,
                command.id,
                command.patch,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey, survey_campaign, feature_flag, member, role, authz, emitter)]
    pub async fn delete_campaign(&self, command: CampaignCommand) -> Result<(), CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service =
            CampaignService::new(survey_campaign_repository, survey_repository, emitter);
        service.delete(command.organization_id, command.id).await
    }

    #[transactional(survey_campaign, feature_flag, member, role, authz)]
    pub async fn list_campaigns(
        &self,
        command: ListCampaignsCommand,
    ) -> Result<(Vec<Campaign>, u64), CoreError> {
        let mut survey_campaign_repository = survey_campaign_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        survey_campaign_repository
            .list(command.organization_id, command.limit, command.offset)
            .await
    }

    #[transactional(survey, survey_response, feature_flag, member, role, authz)]
    pub async fn list_survey_responses(
        &self,
        command: ListSurveyResponsesCommand,
    ) -> Result<(Vec<ResponseRecord>, u64), CoreError> {
        let mut survey_repository = survey_repository;
        let mut survey_response_repository = survey_response_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let survey = survey_repository
            .find(command.organization_id, command.survey_id)
            .await?
            .ok_or(SurveyError::SurveyNotFound)?;
        let (responses, total) = survey_response_repository
            .list_for_survey(
                command.organization_id,
                command.survey_id,
                command.limit,
                command.offset,
            )
            .await?;
        require_graphs(
            &mut survey_repository,
            &survey,
            responses
                .iter()
                .filter_map(ResponseRecord::graph_version)
                .collect(),
        )
        .await?;
        Ok((responses, total))
    }

    #[transactional(survey, survey_response, feature_flag, member, role, authz)]
    pub async fn list_completed_survey_responses(
        &self,
        command: ListSurveyResponsesCommand,
    ) -> Result<CompletedSurveyResponses, CoreError> {
        let mut survey_repository = survey_repository;
        let mut survey_response_repository = survey_response_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let survey = survey_repository
            .find(command.organization_id, command.survey_id)
            .await?
            .ok_or(SurveyError::SurveyNotFound)?;
        let (responses, total) = survey_response_repository
            .list_completed_for_survey(
                command.organization_id,
                command.survey_id,
                command.limit,
                command.offset,
            )
            .await?;
        let (versions, _) = former_graphs(
            &mut survey_repository,
            &survey,
            responses
                .iter()
                .map(|response| response.graph_version())
                .collect(),
        )
        .await?;
        Ok(CompletedSurveyResponses {
            survey,
            responses,
            total,
            versions,
        })
    }

    #[transactional(survey_response, feature_flag, member, role, authz)]
    pub async fn get_survey_response(
        &self,
        command: SurveyResponseCommand,
    ) -> Result<ResponseRecord, CoreError> {
        let mut survey_response_repository = survey_response_repository;
        guard_read(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        survey_response_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or_else(|| SurveyError::ResponseNotFound.into())
    }

    #[transactional(
        survey,
        survey_campaign,
        survey_response,
        feature_flag,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn delete_survey_response<F: FileStorage>(
        &self,
        command: SurveyResponseCommand,
        storage: &FileStorageService<F>,
    ) -> Result<(), CoreError> {
        guard_manage(
            command.actor,
            command.organization_id,
            feature_flag_repository,
            member_repository,
            role_repository,
            authz,
        )
        .await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            survey_campaign_repository,
            emitter,
        );
        service
            .delete(command.organization_id, command.id, storage)
            .await
    }
}
