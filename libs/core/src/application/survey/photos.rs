use chrono::{DateTime, Duration, Utc};
use common::CoreError;
use mestier_macros::transactional;

use super::commands::UploadMemberResponsePhotoCommand;
use super::{public_survey, require_member, require_surveys};
use crate::OrganizationId;
use crate::application::MestierUseCase;
use crate::domain::file_storage::{ports::FileStorage, service::FileStorageService};
use crate::domain::photo_library::normalize::NormalizedImage;
use crate::domain::survey::ports::ResponseRepository;
use crate::domain::survey::service::{ResponseLocator, ResponseService};
use crate::domain::survey::{PublicSlug, ResponseId, ResumeToken};

pub const ABANDONED_RESPONSE_DAYS: i64 = 30;
const ABANDONED_RESPONSES_PER_TICK: u64 = 100;

impl MestierUseCase {
    #[transactional(survey, survey_campaign, survey_response, feature_flag, emitter)]
    pub async fn upload_public_response_photo<F: FileStorage>(
        &self,
        slug: &PublicSlug,
        resume_token: ResumeToken,
        image: NormalizedImage,
        storage: &FileStorageService<F>,
    ) -> Result<String, CoreError> {
        let mut survey_repository = survey_repository;
        let mut feature_flag_repository = feature_flag_repository;
        let survey =
            public_survey(&mut survey_repository, &mut feature_flag_repository, slug).await?;

        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            survey_campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Public {
            survey_id: survey.id(),
            token: resume_token,
        };
        service
            .upload_photo(
                survey.organization_id(),
                &locator,
                image.bytes,
                Utc::now(),
                storage,
            )
            .await
    }

    #[transactional(
        survey,
        survey_campaign,
        survey_response,
        feature_flag,
        member,
        role,
        emitter
    )]
    pub async fn upload_member_response_photo<F: FileStorage>(
        &self,
        command: UploadMemberResponsePhotoCommand,
        storage: &FileStorageService<F>,
    ) -> Result<String, CoreError> {
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
            survey_campaign_repository,
            emitter,
        );
        let locator = ResponseLocator::Member {
            id: command.id,
            member_id,
        };
        service
            .upload_photo(
                command.organization_id,
                &locator,
                command.image.bytes,
                Utc::now(),
                storage,
            )
            .await
    }

    pub async fn purge_abandoned_survey_photos(
        &self,
        now: DateTime<Utc>,
    ) -> Result<usize, CoreError> {
        match &self.file_storage {
            Some(storage) => self.purge_abandoned_survey_photos_in(now, storage).await,
            None => Ok(0),
        }
    }

    pub(crate) async fn purge_abandoned_survey_photos_in<F: FileStorage>(
        &self,
        now: DateTime<Utc>,
        storage: &FileStorageService<F>,
    ) -> Result<usize, CoreError> {
        let before = now - Duration::days(ABANDONED_RESPONSE_DAYS);
        let mut purged = 0;
        for (organization_id, id) in self.abandoned_survey_responses(before).await? {
            match self
                .purge_abandoned_survey_response(organization_id, id, before, storage)
                .await
            {
                Ok(true) => purged += 1,
                Ok(false) => {}
                Err(error) => tracing::error!(
                    %error,
                    organization_id = %organization_id.0,
                    response_id = %id,
                    "purging an abandoned survey response failed"
                ),
            }
        }
        Ok(purged)
    }

    #[transactional(survey_response)]
    async fn abandoned_survey_responses(
        &self,
        before: DateTime<Utc>,
    ) -> Result<Vec<(OrganizationId, ResponseId)>, CoreError> {
        let mut survey_response_repository = survey_response_repository;
        let abandoned = survey_response_repository
            .list_abandoned(before, ABANDONED_RESPONSES_PER_TICK)
            .await?;
        Ok(abandoned
            .iter()
            .map(|response| (response.organization_id(), response.id()))
            .collect())
    }

    #[transactional(survey, survey_campaign, survey_response, emitter)]
    async fn purge_abandoned_survey_response<F: FileStorage>(
        &self,
        organization_id: OrganizationId,
        id: ResponseId,
        before: DateTime<Utc>,
        storage: &FileStorageService<F>,
    ) -> Result<bool, CoreError> {
        let mut service = ResponseService::new(
            survey_response_repository,
            survey_repository,
            survey_campaign_repository,
            emitter,
        );
        service
            .purge_abandoned(organization_id, id, before, storage)
            .await
    }
}
