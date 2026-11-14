use common::CoreError;

use super::record::{ResponseRecord, SurveyRecord};
use super::{Campaign, CampaignId, ResponseId, ResumeToken, SurveyId};
use crate::{MemberId, OrganizationId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait SurveyRepository: Send {
    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SurveyId,
    ) -> impl Future<Output = Result<Option<SurveyRecord>, CoreError>> + Send;

    fn insert(
        &mut self,
        survey: &SurveyRecord,
    ) -> impl Future<Output = Result<SurveyRecord, CoreError>> + Send;

    fn update(
        &mut self,
        survey: &SurveyRecord,
    ) -> impl Future<Output = Result<SurveyRecord, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<SurveyRecord>, u64), CoreError>> + Send;

    fn list_by_campaign(
        &mut self,
        organization_id: OrganizationId,
        campaign_id: CampaignId,
    ) -> impl Future<Output = Result<Vec<SurveyRecord>, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CampaignRepository: Send {
    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: CampaignId,
    ) -> impl Future<Output = Result<Option<Campaign>, CoreError>> + Send;

    fn insert(
        &mut self,
        campaign: &Campaign,
    ) -> impl Future<Output = Result<Campaign, CoreError>> + Send;

    fn update(
        &mut self,
        campaign: &Campaign,
    ) -> impl Future<Output = Result<Campaign, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Campaign>, u64), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ResponseRepository: Send {
    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: ResponseId,
    ) -> impl Future<Output = Result<Option<ResponseRecord>, CoreError>> + Send;

    fn insert(
        &mut self,
        response: &ResponseRecord,
    ) -> impl Future<Output = Result<ResponseRecord, CoreError>> + Send;

    fn update(
        &mut self,
        response: &ResponseRecord,
    ) -> impl Future<Output = Result<ResponseRecord, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<ResponseRecord>, u64), CoreError>> + Send;

    fn list_for_survey(
        &mut self,
        organization_id: OrganizationId,
        survey_id: SurveyId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<ResponseRecord>, u64), CoreError>> + Send;

    fn find_by_resume_token(
        &mut self,
        survey_id: SurveyId,
        token: &ResumeToken,
    ) -> impl Future<Output = Result<Option<ResponseRecord>, CoreError>> + Send;

    fn count_completed(
        &mut self,
        survey_id: SurveyId,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn find_completed_by_member(
        &mut self,
        survey_id: SurveyId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Option<ResponseRecord>, CoreError>> + Send;

    fn find_completed_by_identifier(
        &mut self,
        survey_id: SurveyId,
        value: &str,
    ) -> impl Future<Output = Result<Option<ResponseRecord>, CoreError>> + Send;
}
