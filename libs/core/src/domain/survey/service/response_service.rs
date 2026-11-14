use chrono::{DateTime, Utc};
use common::CoreError;
use events::EventEmitter;

use super::{ResponseLocator, campaign_of};
use crate::domain::survey::events::{SurveyResponseCompleted, SurveyResponseSubmitted};
use crate::domain::survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository};
use crate::domain::survey::{
    Answers, Campaign, Completed, InProgress, Opened, Response, ResponseId, ResponseRecord,
    ResumeToken, Survey, SurveyError, SurveyId, SurveyRecord,
};
use crate::{MemberId, OrganizationId};

pub struct ResponseService<R, S, C, E>
where
    R: ResponseRepository,
    S: SurveyRepository,
    C: CampaignRepository,
    E: EventEmitter,
{
    responses: R,
    surveys: S,
    campaigns: C,
    emitter: E,
}

fn in_progress_of(record: ResponseRecord) -> Result<Response<InProgress>, SurveyError> {
    match record {
        ResponseRecord::InProgress(response) => Ok(response),
        ResponseRecord::Completed(_) => Err(SurveyError::ResponseAlreadyCompleted),
    }
}

fn completed_of(record: ResponseRecord) -> Result<Response<Completed>, SurveyError> {
    match record {
        ResponseRecord::Completed(response) => Ok(response),
        ResponseRecord::InProgress(_) => Err(SurveyError::ResponseInconsistentState),
    }
}

fn answered_by(record: &ResponseRecord, member_id: MemberId) -> bool {
    let owner = match record {
        ResponseRecord::InProgress(response) => response.member_id(),
        ResponseRecord::Completed(response) => response.member_id(),
    };
    owner == Some(member_id)
}

impl<R, S, C, E> ResponseService<R, S, C, E>
where
    R: ResponseRepository,
    S: SurveyRepository,
    C: CampaignRepository,
    E: EventEmitter,
{
    pub fn new(responses: R, surveys: S, campaigns: C, emitter: E) -> Self {
        Self {
            responses,
            surveys,
            campaigns,
            emitter,
        }
    }

    async fn accepting(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Option<Campaign>), CoreError> {
        let record = self.surveys.find(organization_id, survey_id).await?;
        self.accept(organization_id, record, now).await
    }

    async fn accepting_locked(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Option<Campaign>), CoreError> {
        let record = self
            .surveys
            .find_for_update(organization_id, survey_id)
            .await?;
        self.accept(organization_id, record, now).await
    }

    async fn accept(
        &mut self,
        organization_id: OrganizationId,
        record: Option<SurveyRecord>,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Option<Campaign>), CoreError> {
        let survey = match record {
            Some(SurveyRecord::Opened(survey)) => survey,
            Some(SurveyRecord::Draft(_)) => return Err(SurveyError::SurveyNotOpened.into()),
            Some(SurveyRecord::Closed(_)) => return Err(SurveyError::SurveyClosed.into()),
            None => return Err(SurveyError::SurveyNotFound.into()),
        };
        let campaign =
            campaign_of(&mut self.campaigns, organization_id, survey.campaign_id()).await?;
        survey.accepts(campaign.as_ref(), now)?;
        Ok((survey, campaign))
    }

    async fn locate(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
    ) -> Result<Response<InProgress>, CoreError> {
        let record = match locator {
            ResponseLocator::Member { id, member_id } => self
                .responses
                .find(organization_id, *id)
                .await?
                .filter(|record| answered_by(record, *member_id)),
            ResponseLocator::Public { survey_id, token } => {
                self.responses
                    .find_by_resume_token(organization_id, *survey_id, token)
                    .await?
            }
        };
        let record = record.ok_or(SurveyError::ResponseNotFound)?;
        Ok(in_progress_of(record)?)
    }

    async fn insert(
        &mut self,
        response: Response<InProgress>,
    ) -> Result<Response<InProgress>, CoreError> {
        let saved = self.responses.insert(&response.into()).await?;
        Ok(in_progress_of(saved)?)
    }

    pub async fn start_public(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        id: ResponseId,
        resume_token: ResumeToken,
        now: DateTime<Utc>,
    ) -> Result<Response<InProgress>, CoreError> {
        let (survey, _) = self.accepting(organization_id, survey_id, now).await?;
        if let Some(existing) = self
            .responses
            .find_by_resume_token(organization_id, survey_id, &resume_token)
            .await?
        {
            return Ok(in_progress_of(existing)?);
        }
        let response = Response::start(id, &survey, None, Some(resume_token), now)?;
        self.insert(response).await
    }

    pub async fn start_member(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        id: ResponseId,
        member_id: MemberId,
        now: DateTime<Utc>,
    ) -> Result<Response<InProgress>, CoreError> {
        let (survey, _) = self.accepting(organization_id, survey_id, now).await?;
        if let Some(existing) = self
            .responses
            .find_in_progress_by_member(organization_id, survey_id, member_id)
            .await?
        {
            return Ok(in_progress_of(existing)?);
        }
        if !survey.allow_multiple_responses()
            && self
                .responses
                .find_completed_by_member(organization_id, survey_id, member_id)
                .await?
                .is_some()
        {
            return Err(SurveyError::RespondentAlreadyAnswered.into());
        }
        let response = Response::start(id, &survey, Some(member_id), None, now)?;
        self.insert(response).await
    }

    pub async fn save(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
        answers: Answers,
        now: DateTime<Utc>,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut response = self.locate(organization_id, locator).await?;
        let (survey, campaign) = self
            .accepting(organization_id, response.survey_id(), now)
            .await?;
        response.save(&survey, answers)?;
        let saved = in_progress_of(self.responses.update(&response.into()).await?)?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyResponseSubmitted::new(&survey, campaign.as_ref(), &saved)?,
        )?;
        Ok(saved)
    }

    async fn already_answered(
        &mut self,
        organization_id: OrganizationId,
        response: &Response<Completed>,
    ) -> Result<bool, CoreError> {
        if !response.unique_respondent() {
            return Ok(false);
        }
        let survey_id = response.survey_id();
        let existing = match (response.member_id(), response.identifier_value()) {
            (Some(member_id), _) => {
                self.responses
                    .find_completed_by_member(organization_id, survey_id, member_id)
                    .await?
            }
            (None, Some(value)) => {
                self.responses
                    .find_completed_by_identifier(organization_id, survey_id, value)
                    .await?
            }
            (None, None) => None,
        };
        Ok(existing.is_some_and(|record| record.id() != response.id()))
    }

    pub async fn complete(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
        now: DateTime<Utc>,
    ) -> Result<Response<Completed>, CoreError> {
        let response = self.locate(organization_id, locator).await?;
        let survey_id = response.survey_id();
        let (survey, campaign) = self
            .accepting_locked(organization_id, survey_id, now)
            .await?;
        if let Some(cap) = survey.response_cap()
            && self
                .responses
                .count_completed(organization_id, survey_id)
                .await?
                >= u64::from(cap.get())
        {
            return Err(SurveyError::ResponseCapReached.into());
        }
        let completed = response.complete(&survey, now)?;
        if self.already_answered(organization_id, &completed).await? {
            return Err(SurveyError::RespondentAlreadyAnswered.into());
        }
        let saved = completed_of(self.responses.update(&completed.into()).await?)?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyResponseCompleted::new(&survey, campaign.as_ref(), &saved)?,
        )?;
        Ok(saved)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> Result<(), CoreError> {
        self.responses
            .find(organization_id, id)
            .await?
            .ok_or(SurveyError::ResponseNotFound)?;
        self.responses.delete(organization_id, id).await
    }
}
