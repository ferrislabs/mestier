use authz::Subject;

use crate::OrganizationId;
use crate::domain::photo_library::normalize::NormalizedImage;
use crate::domain::survey::service::{CampaignPatch, SurveyPatch};
use crate::domain::survey::{
    Answers, CampaignId, Completed, Graph, GraphVersion, Metadata, Response, ResponseId, SurveyId,
    SurveyRecord, Visibility, Window,
};

#[derive(Debug, Clone)]
pub struct CreateSurveyCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub title: String,
    pub visibility: Visibility,
    pub campaign_ids: Vec<CampaignId>,
}

#[derive(Debug, Clone)]
pub struct UpdateSurveyCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurveyId,
    pub patch: SurveyPatch,
    pub campaign_ids: Option<Vec<CampaignId>>,
}

#[derive(Debug, Clone)]
pub struct SetSurveyCampaignsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurveyId,
    pub campaign_ids: Vec<CampaignId>,
}

#[derive(Debug, Clone)]
pub struct SurveyGraphVersionCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurveyId,
    pub version: u32,
}

#[derive(Debug, Clone)]
pub struct SurveyCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurveyId,
}

#[derive(Debug, Clone)]
pub struct ListSurveysCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub campaign_id: Option<CampaignId>,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct CreateCampaignCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub window: Window,
    pub metadata: Metadata,
}

#[derive(Debug, Clone)]
pub struct UpdateCampaignCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: CampaignId,
    pub patch: CampaignPatch,
}

#[derive(Debug, Clone)]
pub struct CampaignCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: CampaignId,
}

#[derive(Debug, Clone)]
pub struct ListCampaignsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct ListSurveyResponsesCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletedSurveyResponses {
    pub survey: SurveyRecord,
    pub responses: Vec<Response<Completed>>,
    pub total: u64,
    pub versions: Vec<GraphVersion>,
}

impl CompletedSurveyResponses {
    pub fn graph_of(&self, response: &Response<Completed>) -> Option<&Graph> {
        let version = response.graph_version();
        self.versions
            .iter()
            .find(|stored| stored.version() == version)
            .map(GraphVersion::graph)
            .or_else(|| {
                (self.survey.graph_version() == Some(version))
                    .then(|| self.survey.graph())
                    .flatten()
            })
    }
}

#[derive(Debug, Clone)]
pub struct OpenSurveyInCampaignCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurveyId,
    pub campaign_id: CampaignId,
}

#[derive(Debug, Clone)]
pub struct SurveyResponseCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: ResponseId,
}

#[derive(Debug, Clone)]
pub struct StartMemberResponseCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub survey_id: SurveyId,
}

#[derive(Debug, Clone)]
pub struct SaveMemberResponseCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: ResponseId,
    pub answers: Answers,
}

#[derive(Debug, Clone)]
pub struct UploadMemberResponsePhotoCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: ResponseId,
    pub image: NormalizedImage,
}
