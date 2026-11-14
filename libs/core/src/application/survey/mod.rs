use authz::{Authorizer, Resource, Subject};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use common::CoreError;

use crate::application::policy;
use crate::domain::feature_flag::ports::FeatureFlagRepository;
use crate::domain::member::ports::MemberRepository;
use crate::domain::role::ports::RoleRepository;
use crate::domain::survey::ports::SurveyRepository;
use crate::domain::survey::{PublicSlug, ResumeToken, SurveyError, SurveyRecord};
use crate::{FeatureFlag, MemberId, OrganizationId};

mod clock;
pub mod commands;
mod management;
mod response;

#[cfg(test)]
mod tests;

const PUBLIC_SLUG_BYTES: usize = 16;
const RESUME_TOKEN_BYTES: usize = 32;

fn random_url_safe(bytes: usize) -> Result<String, CoreError> {
    let mut buffer = vec![0u8; bytes];
    SystemRandom::new()
        .fill(&mut buffer)
        .map_err(|_| CoreError::Internal("the system random source failed".to_owned()))?;
    Ok(URL_SAFE_NO_PAD.encode(buffer))
}

fn new_public_slug() -> Result<PublicSlug, CoreError> {
    Ok(PublicSlug::new(random_url_safe(PUBLIC_SLUG_BYTES)?)?)
}

fn new_resume_token() -> Result<ResumeToken, CoreError> {
    Ok(ResumeToken::new(random_url_safe(RESUME_TOKEN_BYTES)?)?)
}

async fn require_surveys<F: FeatureFlagRepository>(
    flags: &mut F,
    organization_id: OrganizationId,
) -> Result<(), CoreError> {
    if flags
        .is_enabled(organization_id, FeatureFlag::Surveys)
        .await?
    {
        Ok(())
    } else {
        Err(CoreError::NotFound)
    }
}

fn organization(organization_id: OrganizationId) -> Resource {
    Resource::new("organization", organization_id.0.to_string())
}

async fn require_survey_reader<M, R, A>(
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
    authz: &A,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
    A: Authorizer,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    if policy::holds(
        authz,
        &actor,
        "survey.manage",
        organization(organization_id),
    )
    .await?
    {
        return Ok(());
    }
    policy::require(authz, &actor, "survey.view", organization(organization_id)).await
}

async fn require_member<M, R>(
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<MemberId, CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::subject_member_id(&actor).ok_or(CoreError::Forbidden {
        reason: Some("a members survey is answered by a member of its organization".to_owned()),
    })
}

async fn public_survey<S, F>(
    surveys: &mut S,
    flags: &mut F,
    slug: &PublicSlug,
) -> Result<SurveyRecord, CoreError>
where
    S: SurveyRepository,
    F: FeatureFlagRepository,
{
    let record = surveys
        .find_by_public_slug(slug)
        .await?
        .ok_or(SurveyError::SurveyNotFound)?;
    if flags
        .is_enabled(record.organization_id(), FeatureFlag::Surveys)
        .await?
    {
        Ok(record)
    } else {
        Err(SurveyError::SurveyNotFound.into())
    }
}
