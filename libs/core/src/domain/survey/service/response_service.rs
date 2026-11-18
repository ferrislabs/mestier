use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use super::{ResponseLocator, StartedResponse, campaigns_of};
use crate::domain::file_storage::{ports::FileStorage, service::FileStorageService};
use crate::domain::photo_library::normalize::STORED_MIME_TYPE;
use crate::domain::survey::events::{SurveyResponseCompleted, SurveyResponseSubmitted};
use crate::domain::survey::ports::{CampaignRepository, ResponseRepository, SurveyRepository};
use crate::domain::survey::{
    Answer, Answers, Campaign, Completed, InProgress, Opened, QuestionId, Response, ResponseId,
    ResponseRecord, ResumeToken, Survey, SurveyError, SurveyId, SurveyRecord, Visibility,
};
use crate::{MemberId, OrganizationId};

pub const MAX_UPLOADED_PHOTOS: u64 = 20;

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

fn unrecorded_photo(answers: &Answers, recorded: &[String]) -> Option<QuestionId> {
    answers.iter().find_map(|(id, answer)| match answer {
        Answer::Photos(photos) if photos.iter().any(|photo| !recorded.contains(photo)) => Some(*id),
        _ => None,
    })
}

fn answers_photos(answers: &Answers) -> bool {
    answers
        .values()
        .any(|answer| matches!(answer, Answer::Photos(photos) if !photos.is_empty()))
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
        visibility: Visibility,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Vec<Campaign>), CoreError> {
        let record = self.surveys.find(organization_id, survey_id).await?;
        self.accept(organization_id, record, visibility, now).await
    }

    async fn accepting_locked(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        visibility: Visibility,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Vec<Campaign>), CoreError> {
        let record = self
            .surveys
            .find_for_update(organization_id, survey_id)
            .await?;
        self.accept(organization_id, record, visibility, now).await
    }

    async fn accept(
        &mut self,
        organization_id: OrganizationId,
        record: Option<SurveyRecord>,
        visibility: Visibility,
        now: DateTime<Utc>,
    ) -> Result<(Survey<Opened>, Vec<Campaign>), CoreError> {
        let record = record.filter(|record| record.visibility() == visibility);
        let survey = match record {
            Some(SurveyRecord::Opened(survey)) => survey,
            Some(SurveyRecord::Draft(_)) => return Err(SurveyError::SurveyNotFound.into()),
            Some(SurveyRecord::Closed(_)) => return Err(SurveyError::SurveyClosed.into()),
            None => return Err(SurveyError::SurveyNotFound.into()),
        };
        let campaigns =
            campaigns_of(&mut self.campaigns, organization_id, survey.campaign_ids()).await?;
        survey.accepts(&campaigns, now)?;
        Ok((survey, campaigns))
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

    async fn require_recorded_photos(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
        answers: &Answers,
    ) -> Result<(), CoreError> {
        if !answers_photos(answers) {
            return Ok(());
        }
        let recorded = self.responses.photo_keys(organization_id, id).await?;
        match unrecorded_photo(answers, &recorded) {
            Some(question) => Err(SurveyError::AnswerForeignPhoto(question).into()),
            None => Ok(()),
        }
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
        let (survey, _) = self
            .accepting(organization_id, survey_id, Visibility::Public, now)
            .await?;
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
    ) -> Result<StartedResponse, CoreError> {
        let (survey, _) = self
            .accepting(organization_id, survey_id, Visibility::Members, now)
            .await?;
        if let Some(existing) = self
            .responses
            .find_in_progress_by_member(organization_id, survey_id, member_id)
            .await?
        {
            return Ok(StartedResponse::Resumed(in_progress_of(existing)?));
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
        Ok(StartedResponse::Created(self.insert(response).await?))
    }

    pub async fn save(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
        answers: Answers,
        now: DateTime<Utc>,
    ) -> Result<Response<InProgress>, CoreError> {
        let mut response = self.locate(organization_id, locator).await?;
        let (survey, campaigns) = self
            .accepting(
                organization_id,
                response.survey_id(),
                locator.visibility(),
                now,
            )
            .await?;
        response.save(&survey, answers)?;
        self.require_recorded_photos(organization_id, response.id(), response.answers())
            .await?;
        let saved = in_progress_of(self.responses.update(&response.into()).await?)?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyResponseSubmitted::new(&survey, &campaigns, &saved)?,
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
        let (survey, campaigns) = self
            .accepting_locked(organization_id, survey_id, locator.visibility(), now)
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
        self.require_recorded_photos(organization_id, completed.id(), completed.answers())
            .await?;
        if self.already_answered(organization_id, &completed).await? {
            return Err(SurveyError::RespondentAlreadyAnswered.into());
        }
        let saved = completed_of(self.responses.update(&completed.into()).await?)?;
        self.surveys
            .store_graph_version(
                organization_id,
                survey_id,
                saved.graph_version(),
                survey.definition(),
            )
            .await?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyResponseCompleted::new(&survey, &campaigns, &saved)?,
        )?;
        Ok(saved)
    }

    pub async fn check_photo(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
        now: DateTime<Utc>,
    ) -> Result<Response<InProgress>, CoreError> {
        let response = self.locate(organization_id, locator).await?;
        self.accepting(
            organization_id,
            response.survey_id(),
            locator.visibility(),
            now,
        )
        .await?;
        let uploaded = self
            .responses
            .count_photos_for_update(organization_id, response.id())
            .await?;
        if uploaded >= MAX_UPLOADED_PHOTOS {
            return Err(SurveyError::ResponsePhotoCapReached.into());
        }
        Ok(response)
    }

    pub async fn upload_photo<F: FileStorage>(
        &mut self,
        organization_id: OrganizationId,
        locator: &ResponseLocator,
        jpeg: Vec<u8>,
        now: DateTime<Utc>,
        storage: &FileStorageService<F>,
    ) -> Result<String, CoreError> {
        let response = self.check_photo(organization_id, locator, now).await?;
        let relative = format!("{}/{}.jpg", response.photo_directory(), generate_uuid_v7());
        let stored = storage.upload_at(&relative, STORED_MIME_TYPE, jpeg).await?;
        if let Err(error) = self
            .responses
            .insert_photo(organization_id, response.id(), &stored.key)
            .await
        {
            if let Err(cleanup) = storage.delete(&stored.key).await {
                tracing::warn!(key = %stored.key, error = %cleanup, "could not remove an unrecorded survey photo");
            }
            return Err(error);
        }
        Ok(stored.key)
    }

    pub async fn purge_abandoned<F: FileStorage>(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
        before: DateTime<Utc>,
        storage: &FileStorageService<F>,
    ) -> Result<bool, CoreError> {
        let Some(ResponseRecord::InProgress(_)) = self
            .responses
            .find_abandoned_for_update(organization_id, id, before)
            .await?
        else {
            return Ok(false);
        };
        for key in self.responses.photo_keys(organization_id, id).await? {
            storage.delete(&key).await?;
        }
        self.responses.delete(organization_id, id).await?;
        Ok(true)
    }

    pub async fn delete<F: FileStorage>(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
        storage: &FileStorageService<F>,
    ) -> Result<(), CoreError> {
        self.responses
            .find(organization_id, id)
            .await?
            .ok_or(SurveyError::ResponseNotFound)?;
        for key in self.responses.photo_keys(organization_id, id).await? {
            storage.delete(&key).await?;
        }
        self.responses.delete(organization_id, id).await
    }
}
