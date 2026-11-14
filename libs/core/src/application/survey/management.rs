use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use mestier_macros::transactional;

use super::commands::{
    CampaignCommand, CreateCampaignCommand, CreateSurveyCommand, ListCampaignsCommand,
    ListSurveyResponsesCommand, ListSurveysCommand, SurveyCommand, SurveyResponseCommand,
    UpdateCampaignCommand, UpdateSurveyCommand,
};
use super::{new_public_slug, require_survey_reader, require_surveys};
use crate::application::{MestierUseCase, leave_account::require_permission};
use crate::domain::survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository};
use crate::domain::survey::service::{CampaignService, ResponseService, SurveyService};
use crate::domain::survey::{
    Campaign, CampaignId, Closed, Draft, Opened, ResponseRecord, Survey, SurveyError, SurveyId,
    SurveyRecord, Visibility,
};

impl MestierUseCase {
    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn create_survey(
        &self,
        command: CreateSurveyCommand,
    ) -> Result<Survey<Draft>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        service
            .create(
                command.organization_id,
                SurveyId(generate_uuid_v7()),
                command.title,
                command.visibility,
            )
            .await
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn update_survey(
        &self,
        command: UpdateSurveyCommand,
    ) -> Result<Survey<Draft>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        let updated = service
            .update_draft(command.organization_id, command.id, command.patch)
            .await?;
        match command.campaign_id {
            Some(campaign_id) => {
                service
                    .attach_campaign(command.organization_id, command.id, campaign_id)
                    .await
            }
            None => Ok(updated),
        }
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn open_survey(&self, command: SurveyCommand) -> Result<Survey<Opened>, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
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
        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        service
            .open(command.organization_id, command.id, Utc::now(), public_slug)
            .await
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn close_survey(&self, command: SurveyCommand) -> Result<Survey<Closed>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        service
            .close(command.organization_id, command.id, Utc::now())
            .await
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn duplicate_survey(
        &self,
        command: SurveyCommand,
    ) -> Result<Survey<Draft>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        service
            .duplicate(
                command.organization_id,
                command.id,
                SurveyId(generate_uuid_v7()),
            )
            .await
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn delete_survey(&self, command: SurveyCommand) -> Result<(), CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = SurveyService::new(survey_repository, campaign_repository, emitter);
        service.delete(command.organization_id, command.id).await
    }

    #[transactional(survey, feature_flag, member, role, authz)]
    pub async fn list_surveys(
        &self,
        command: ListSurveysCommand,
    ) -> Result<(Vec<SurveyRecord>, u64), CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_survey_reader(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        survey_repository
            .list(command.organization_id, command.limit, command.offset)
            .await
    }

    #[transactional(survey, feature_flag, member, role, authz)]
    pub async fn get_survey(&self, command: SurveyCommand) -> Result<SurveyRecord, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_survey_reader(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        survey_repository
            .find(command.organization_id, command.id)
            .await?
            .ok_or_else(|| SurveyError::SurveyNotFound.into())
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn create_campaign(
        &self,
        command: CreateCampaignCommand,
    ) -> Result<Campaign, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = CampaignService::new(campaign_repository, survey_repository, emitter);
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

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn update_campaign(
        &self,
        command: UpdateCampaignCommand,
    ) -> Result<Campaign, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = CampaignService::new(campaign_repository, survey_repository, emitter);
        service
            .update(
                command.organization_id,
                command.id,
                command.patch,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey, campaign, feature_flag, member, role, authz, emitter)]
    pub async fn delete_campaign(&self, command: CampaignCommand) -> Result<(), CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = CampaignService::new(campaign_repository, survey_repository, emitter);
        service.delete(command.organization_id, command.id).await
    }

    #[transactional(campaign, feature_flag, member, role, authz)]
    pub async fn list_campaigns(
        &self,
        command: ListCampaignsCommand,
    ) -> Result<(Vec<Campaign>, u64), CoreError> {
        let mut campaign_repository = campaign_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_survey_reader(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        campaign_repository
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
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_survey_reader(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        survey_repository
            .find(command.organization_id, command.survey_id)
            .await?
            .ok_or(SurveyError::SurveyNotFound)?;
        survey_response_repository
            .list_for_survey(
                command.organization_id,
                command.survey_id,
                command.limit,
                command.offset,
            )
            .await
    }

    #[transactional(survey_response, feature_flag, member, role, authz)]
    pub async fn get_survey_response(
        &self,
        command: SurveyResponseCommand,
    ) -> Result<ResponseRecord, CoreError> {
        let mut survey_response_repository = survey_response_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_survey_reader(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
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
        campaign,
        survey_response,
        feature_flag,
        member,
        role,
        authz,
        emitter
    )]
    pub async fn delete_survey_response(
        &self,
        command: SurveyResponseCommand,
    ) -> Result<(), CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_permission(
            command.actor,
            command.organization_id,
            "survey.manage",
            &mut member_repository,
            &mut role_repository,
            authz,
        )
        .await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        service.delete(command.organization_id, command.id).await
    }
}
