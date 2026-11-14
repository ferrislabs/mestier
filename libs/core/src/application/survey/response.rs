use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use mestier_macros::transactional;

use super::commands::{
    SaveMemberResponseCommand, StartMemberResponseCommand, SurveyCommand, SurveyResponseCommand,
};
use super::{new_resume_token, public_survey, require_member, require_surveys};
use crate::application::MestierUseCase;
use crate::domain::survey::ports::SurveyRepository;
use crate::domain::survey::service::{ResponseLocator, ResponseService};
use crate::domain::survey::{
    Answers, Completed, InProgress, PublicSlug, RespondentSurvey, Response, ResponseId,
    ResumeToken, SurveyError, SurveyRecord, Visibility,
};

impl MestierUseCase {
    #[transactional(survey, campaign, survey_response, feature_flag, member, role, emitter)]
    pub async fn start_member_response(
        &self,
        command: StartMemberResponseCommand,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        let member_id = require_member(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        service
            .start_member(
                command.organization_id,
                command.survey_id,
                ResponseId(generate_uuid_v7()),
                member_id,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey, campaign, survey_response, feature_flag, member, role, emitter)]
    pub async fn save_member_response(
        &self,
        command: SaveMemberResponseCommand,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        let member_id = require_member(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Member {
            id: command.id,
            member_id,
        };
        service
            .save(
                command.organization_id,
                &locator,
                command.answers,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey, campaign, survey_response, feature_flag, member, role, emitter)]
    pub async fn complete_member_response(
        &self,
        command: SurveyResponseCommand,
    ) -> Result<Response<Completed>, CoreError> {
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        let member_id = require_member(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Member {
            id: command.id,
            member_id,
        };
        service
            .complete(command.organization_id, &locator, Utc::now())
            .await
    }

    #[transactional(survey, feature_flag, member, role)]
    pub async fn open_member_survey(
        &self,
        command: SurveyCommand,
    ) -> Result<RespondentSurvey, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_surveys(&mut feature_flag_repository, command.organization_id).await?;
        require_member(
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        match survey_repository
            .find(command.organization_id, command.id)
            .await?
        {
            Some(SurveyRecord::Opened(survey)) if survey.visibility() == Visibility::Members => {
                Ok(survey.respondent_view())
            }
            Some(SurveyRecord::Closed(survey)) if survey.visibility() == Visibility::Members => {
                Err(SurveyError::SurveyClosed.into())
            }
            _ => Err(SurveyError::SurveyNotFound.into()),
        }
    }

    #[transactional(survey, feature_flag)]
    pub async fn open_public_survey(
        &self,
        slug: &PublicSlug,
    ) -> Result<RespondentSurvey, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        match public_survey(&mut survey_repository, &mut feature_flag_repository, slug).await? {
            SurveyRecord::Opened(survey) => Ok(survey.respondent_view()),
            SurveyRecord::Closed(_) => Err(SurveyError::SurveyClosed.into()),
            SurveyRecord::Draft(_) => Err(SurveyError::SurveyNotFound.into()),
        }
    }

    #[transactional(survey, campaign, survey_response, feature_flag, emitter)]
    pub async fn start_public_response(
        &self,
        slug: &PublicSlug,
        resume_token: Option<ResumeToken>,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let survey =
            public_survey(&mut survey_repository, &mut feature_flag_repository, slug).await?;
        let resume_token = match resume_token {
            Some(token) => token,
            None => new_resume_token()?,
        };

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        service
            .start_public(
                survey.organization_id(),
                survey.id(),
                ResponseId(generate_uuid_v7()),
                resume_token,
                Utc::now(),
            )
            .await
    }

    #[transactional(survey, campaign, survey_response, feature_flag, emitter)]
    pub async fn save_public_response(
        &self,
        slug: &PublicSlug,
        resume_token: ResumeToken,
        answers: Answers,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let survey =
            public_survey(&mut survey_repository, &mut feature_flag_repository, slug).await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Public {
            survey_id: survey.id(),
            token: resume_token,
        };
        service
            .save(survey.organization_id(), &locator, answers, Utc::now())
            .await
    }

    #[transactional(survey, campaign, survey_response, feature_flag, emitter)]
    pub async fn complete_public_response(
        &self,
        slug: &PublicSlug,
        resume_token: ResumeToken,
    ) -> Result<Response<Completed>, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let survey =
            public_survey(&mut survey_repository, &mut feature_flag_repository, slug).await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Public {
            survey_id: survey.id(),
            token: resume_token,
        };
        service
            .complete(survey.organization_id(), &locator, Utc::now())
            .await
    }
}
