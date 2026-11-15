use chrono::{DateTime, Utc};
use common::CoreError;
use events::EventEmitter;

use super::{SurveyPatch, campaigns_of};
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
    copy.edit_campaigns(survey.campaign_ids().to_vec());
    copy.edit_graph(survey.graph().cloned());
    Ok(copy)
}

fn apply_settings<St: Stage>(
    survey: &mut Survey<St>,
    patch: &mut SurveyPatch,
) -> Result<(), SurveyError> {
    if let Some(title) = patch.title.take() {
        survey.edit_title(title)?;
    }
    if let Some(description) = patch.description.take() {
        survey.edit_description(description);
    }
    if let Some(allowed) = patch.allow_multiple_responses {
        survey.edit_allow_multiple_responses(allowed);
    }
    if let Some(cap) = patch.response_cap {
        survey.edit_response_cap(cap);
    }
    if let Some(metadata) = patch.metadata.take() {
        survey.edit_metadata(metadata);
    }
    Ok(())
}

fn frozen_visibility<St: Stage>(
    survey: &Survey<St>,
    visibility: Option<Visibility>,
) -> Result<(), SurveyError> {
    match visibility {
        Some(visibility) if visibility != survey.visibility() => {
            Err(SurveyError::SurveyVisibilityFrozen)
        }
        _ => Ok(()),
    }
}

fn change_graph<St: Stage>(
    survey: &mut Survey<St>,
    graph: St::Input,
    frozen: bool,
) -> Result<(), SurveyError> {
    if frozen {
        survey.revise_graph(graph)?;
    } else {
        survey.edit_graph(graph);
    }
    Ok(())
}

fn with_campaigns(record: SurveyRecord, campaign_ids: Vec<CampaignId>) -> SurveyRecord {
    match record {
        SurveyRecord::Draft(mut survey) => {
            survey.edit_campaigns(campaign_ids);
            SurveyRecord::Draft(survey)
        }
        SurveyRecord::Opened(mut survey) => {
            survey.edit_campaigns(campaign_ids);
            SurveyRecord::Opened(survey)
        }
        SurveyRecord::Closed(mut survey) => {
            survey.edit_campaigns(campaign_ids);
            SurveyRecord::Closed(survey)
        }
    }
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

    async fn find_locked(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<SurveyRecord, CoreError> {
        self.surveys
            .find_for_update(organization_id, id)
            .await?
            .ok_or_else(|| SurveyError::SurveyNotFound.into())
    }

    async fn graph_frozen(
        &mut self,
        record: &SurveyRecord,
        patch: &SurveyPatch,
    ) -> Result<bool, CoreError> {
        if matches!(record, SurveyRecord::Draft(_)) {
            return Ok(false);
        }
        match (record.graph_version(), &patch.graph) {
            (Some(version), Some(_)) => {
                self.surveys
                    .has_graph_version(record.organization_id(), record.id(), version)
                    .await
            }
            _ => Ok(false),
        }
    }

    async fn store_campaigns(
        &mut self,
        record: SurveyRecord,
        campaign_ids: Vec<CampaignId>,
    ) -> Result<SurveyRecord, CoreError> {
        let organization_id = record.organization_id();
        campaigns_of(&mut self.campaigns, organization_id, &campaign_ids).await?;
        let edited = with_campaigns(record, campaign_ids);
        self.surveys
            .save_campaigns(organization_id, edited.id(), edited.campaign_ids())
            .await?;
        Ok(edited)
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

    pub async fn update(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        mut patch: SurveyPatch,
    ) -> Result<SurveyRecord, CoreError> {
        let record = self.find_locked(organization_id, id).await?;
        let frozen = self.graph_frozen(&record, &patch).await?;
        let edited = match record {
            SurveyRecord::Draft(mut survey) => {
                apply_settings(&mut survey, &mut patch)?;
                if let Some(visibility) = patch.visibility {
                    survey.edit_visibility(visibility);
                }
                if let Some(graph) = patch.graph {
                    change_graph(&mut survey, graph, frozen)?;
                }
                SurveyRecord::Draft(survey)
            }
            SurveyRecord::Opened(mut survey) => {
                frozen_visibility(&survey, patch.visibility)?;
                apply_settings(&mut survey, &mut patch)?;
                if let Some(graph) = patch.graph {
                    let graph = graph.ok_or(SurveyError::SurveyWithoutGraph)?;
                    change_graph(&mut survey, graph, frozen)?;
                }
                SurveyRecord::Opened(survey)
            }
            SurveyRecord::Closed(mut survey) => {
                frozen_visibility(&survey, patch.visibility)?;
                apply_settings(&mut survey, &mut patch)?;
                if let Some(graph) = patch.graph {
                    let graph = graph.ok_or(SurveyError::SurveyWithoutGraph)?;
                    change_graph(&mut survey, graph, frozen)?;
                }
                SurveyRecord::Closed(survey)
            }
        };
        self.surveys.update(&edited).await
    }

    pub async fn set_campaigns(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        campaign_ids: Vec<CampaignId>,
    ) -> Result<SurveyRecord, CoreError> {
        let record = self.find_locked(organization_id, id).await?;
        self.store_campaigns(record, campaign_ids).await
    }

    pub async fn add_campaign(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        campaign_id: CampaignId,
    ) -> Result<SurveyRecord, CoreError> {
        let record = self.find_locked(organization_id, id).await?;
        let mut campaign_ids = record.campaign_ids().to_vec();
        if !campaign_ids.contains(&campaign_id) {
            campaign_ids.push(campaign_id);
        }
        self.store_campaigns(record, campaign_ids).await
    }

    pub async fn duplicate(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        new_id: SurveyId,
    ) -> Result<Survey<Draft>, CoreError> {
        let copy = match &self.find_locked(organization_id, id).await? {
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
        let survey = draft_of(self.find_locked(organization_id, id).await?)?;
        let campaigns =
            campaigns_of(&mut self.campaigns, organization_id, survey.campaign_ids()).await?;
        let opened = survey.open(now, public_slug)?;
        let saved = opened_of(self.surveys.update(&opened.into()).await?)?;
        self.emitter.emit(
            saved.organization_id(),
            &SurveyOpened::new(&saved, &campaigns)?,
        )?;
        Ok(saved)
    }

    pub async fn close(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
        now: DateTime<Utc>,
    ) -> Result<Survey<Closed>, CoreError> {
        let survey = opened_of(self.find_locked(organization_id, id).await?)?;
        let closed = survey.close(now)?;
        let saved = self.surveys.update(&closed.into()).await?;
        Ok(closed_of(saved)?)
    }

    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> Result<(), CoreError> {
        self.find_locked(organization_id, id).await?;
        if self.surveys.has_responses(organization_id, id).await? {
            return Err(SurveyError::SurveyHasResponses.into());
        }
        self.surveys.delete(organization_id, id).await
    }
}
