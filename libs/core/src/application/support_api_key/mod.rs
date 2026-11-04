use authz::{Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use mestier_macros::transactional;

use crate::{
    CreateApiKeyCommand, CreatedApiKey, ListApiKeysCommand, OrganizationId, PresentedApiKey,
    RevokeApiKeyCommand, SupportApiKey, SupportApiKeyId, SupportApiKeyRepository, api_key_matches,
    application::{MestierAuthorizer, MestierUseCase, policy},
    domain::{member::ports::MemberRepository, role::ports::RoleRepository},
    generate_api_key, validate_api_key_name,
};

#[cfg(test)]
pub(crate) mod tests;

async fn authorize<M, R>(
    authorizer: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authorizer,
        &actor,
        "organization.update",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(support_api_key, member, role, authz)]
    pub async fn create_support_api_key(
        &self,
        command: CreateApiKeyCommand,
    ) -> Result<CreatedApiKey, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut support_api_key_repository = support_api_key_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let name = validate_api_key_name(&command.name)?;
        let generated = generate_api_key()?;
        let key = SupportApiKey {
            id: SupportApiKeyId(generate_uuid_v7()),
            organization_id: command.organization_id,
            name,
            prefix: generated.prefix,
            secret_hash: generated.secret_hash,
            created_at: Utc::now(),
            last_used_at: None,
            revoked_at: None,
        };
        let key = support_api_key_repository.create(&key).await?;

        Ok(CreatedApiKey {
            key,
            plaintext: generated.plaintext,
        })
    }

    #[transactional(support_api_key, member, role, authz)]
    pub async fn list_support_api_keys(
        &self,
        command: ListApiKeysCommand,
    ) -> Result<Vec<SupportApiKey>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut support_api_key_repository = support_api_key_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        support_api_key_repository
            .list(command.organization_id)
            .await
    }

    #[transactional(support_api_key, member, role, authz)]
    pub async fn revoke_support_api_key(
        &self,
        command: RevokeApiKeyCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        let mut support_api_key_repository = support_api_key_repository;

        authorize(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        if support_api_key_repository
            .revoke(command.organization_id, command.key_id)
            .await?
        {
            Ok(())
        } else {
            Err(CoreError::NotFound)
        }
    }

    #[transactional(support_api_key)]
    pub async fn authenticate_support_api_key(
        &self,
        plaintext: &str,
    ) -> Result<Option<OrganizationId>, CoreError> {
        let mut support_api_key_repository = support_api_key_repository;

        let Some(presented) = PresentedApiKey::parse(plaintext) else {
            return Ok(None);
        };
        let Some(key) = support_api_key_repository
            .find_by_prefix(&presented.prefix)
            .await?
        else {
            return Ok(None);
        };
        if !api_key_matches(&key.secret_hash, &presented) || key.revoked_at.is_some() {
            return Ok(None);
        }

        support_api_key_repository.touch_last_used(key.id).await?;

        Ok(Some(key.organization_id))
    }
}
