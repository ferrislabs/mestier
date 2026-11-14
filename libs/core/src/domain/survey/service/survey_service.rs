use chrono::{DateTime, Utc};
use common::CoreError;
use events::EventEmitter;

use super::{SurveyPatch, campaign_of};
use crate::OrganizationId;
use crate::domain::survey::aggregate::Stage;
use crate::domain::survey::events::SurveyOpened;
use crate::domain::survey::ports::{CampaignRepository, SurveyRepository};
use crate::domain::survey::{
    CampaignId, Closed, Draft, Opened, PublicSlug, Survey, SurveyError, SurveyId, SurveyRecord,
    Visibility,
};

pub struct SurveyService<S, C, E>
where
    S: SurveyRepository,
    C: CampaignRepository,
    E: EventEmitter,
{
    surveys: S,
    campaigns: C,
    emitter: E,
}

fn draft_of(record: SurveyRecord) -> Result<Survey<Draft>, SurveyError> {
    match record {
        SurveyRecord::Draft(survey) => Ok(survey),
        SurveyRecord::Opened(_) | SurveyRecord::Closed(_) => Err(SurveyError::SurveyNotDraft),
    }
}

fn opened_of(record: SurveyRecord) -> Result<Survey<Opened>, SurveyError> {
    match record {
        SurveyRecord::Opened(survey) => Ok(survey),
        SurveyRecord::Draft(_) => Err(SurveyError::SurveyNotOpened),
        SurveyRecord::Closed(_) => Err(SurveyError::SurveyClosed),
    }
}

fn closed_of(record: SurveyRecord) -> Result<Survey<Closed>, SurveyError> {
    match record {
        SurveyRecord::Closed(survey) => Ok(survey),
        SurveyRecord::Draft(_) | SurveyRecord::Opened(_) => {
            Err(SurveyError::SurveyInconsistentState)
        }
    }
}

fn copy_of<St: Stage>(survey: &Survey<St>, id: SurveyId) -> Result<Survey<Draft>, SurveyError> {
    let mut copy = Survey::new(
        id,
        survey.organization_id(),
        survey.title().to_owned(),
        survey.visibility(),
    )?;
    copy.edit_description(survey.description().map(str::to_owned));
    copy.edit_allow_multiple_responses(survey.allow_multiple_responses());
    copy.edit_response_cap(survey.response_cap());
    copy.edit_metadata(survey.metadata().clone());
    copy.edit_campaign(survey.campaign_id());
    copy.edit_graph(survey.graph().cloned());
    Ok(copy)
}

fn apply(survey: &mut Survey<Draft>, patch: SurveyPatch) -> Result<(), SurveyError> {
    if let Some(title) = patch.title {
        survey.edit_title(title)?;
    }
    if let Some(description) = patch.description {
        survey.edit_description(description);
    }
    if let Some(visibility) = patch.visibility {
        survey.edit_visibility(visibility);
    }
    if let Some(allowed) = patch.allow_multiple_responses {
        survey.edit_allow_multiple_responses(allowed);
    }
    if let Some(cap) = patch.response_cap {
        survey.edit_response_cap(cap);
    }
    if let Some(metadata) = patch.metadata {
        survey.edit_metadata(metadata);
    }
    if let Some(graph) = patch.graph {
        survey.edit_graph(graph);
    }
    Ok(())
}

impl<S, C, E> SurveyService<S, C, E>
where
    S: SurveyRepository,
    C: CampaignRepository,
    E: EventEmitter,
{
    pub fn new(surveys: S, campaigns: C, emitter: E) -> Self {
        Self {
            surveys,
            campaigns,
            emitter,
        }
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<SurveyRecord, CoreError> {
        self.surveys
            .find(organization_id, id)
            .await?
            .ok_or_else(|| SurveyError::SurveyNotFound.into())
    }

    async fn save_draft(&mut self, survey: Survey<Draft>) -> Result<Survey<Draft>, CoreError> {
        let saved = self.surveys.update(&survey.into()).await?;
        Ok(draft_of(saved)?)
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        title: String,
        visibility: Visibility,
    ) -> Result<Survey<Draft>, CoreError> {
        let survey = Survey::new(id, organization_id, title, visibility)?;
        let saved = self.surveys.insert(&survey.into()).await?;
        Ok(draft_of(saved)?)
    }

    pub async fn update_draft(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        patch: SurveyPatch,
    ) -> Result<Survey<Draft>, CoreError> {
        let mut survey = draft_of(self.find(organization_id, id).await?)?;
        apply(&mut survey, patch)?;
        self.save_draft(survey).await
    }

    pub async fn duplicate(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        new_id: SurveyId,
    ) -> Result<Survey<Draft>, CoreError> {
        let copy = match &self.find(organization_id, id).await? {
            SurveyRecord::Draft(survey) => copy_of(survey, new_id),
            SurveyRecord::Opened(survey) => copy_of(survey, new_id),
            SurveyRecord::Closed(survey) => copy_of(survey, new_id),
        }?;
        let saved = self.surveys.insert(&copy.into()).await?;
        Ok(draft_of(saved)?)
    }

    pub async fn open(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        now: DateTime<Utc>,
        public_slug: Option<PublicSlug>,
    ) -> Result<Survey<Opened>, CoreError> {
        let survey = draft_of(self.find(organization_id, id).await?)?;
        let campaign =
            campaign_of(&mut self.campaigns, organization_id, survey.campaign_id()).await?;
        let opened = survey.open(now, public_slug)?;
        let saved = opened_of(self.surveys.update(&opened.into()).await?)?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyOpened::new(&saved, campaign.as_ref())?,
        )?;
        Ok(saved)
    }

    pub async fn close(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        now: DateTime<Utc>,
    ) -> Result<Survey<Closed>, CoreError> {
        let survey = opened_of(self.find(organization_id, id).await?)?;
        let closed = survey.close(now)?;
        let saved = self.surveys.update(&closed.into()).await?;
        Ok(closed_of(saved)?)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<(), CoreError> {
        self.find(organization_id, id).await?;
        if self.surveys.has_responses(organization_id, id).await? {
            return Err(SurveyError::SurveyHasResponses.into());
        }
        self.surveys.delete(organization_id, id).await
    }

    pub async fn attach_campaign(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        campaign_id: Option<CampaignId>,
    ) -> Result<Survey<Draft>, CoreError> {
        let mut survey = draft_of(self.find(organization_id, id).await?)?;
        campaign_of(&mut self.campaigns, organization_id, campaign_id).await?;
        survey.edit_campaign(campaign_id);
        self.save_draft(survey).await
    }
}
