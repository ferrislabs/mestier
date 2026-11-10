use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    CreateKnowledgeNoteCommand, CustomerId, DeleteKnowledgeCommand, GetKnowledgeCommand,
    KnowledgeArticle, KnowledgeArticleWithLinks, KnowledgeLink, KnowledgeOutcome,
    KnowledgeResource, KnowledgeResourceType, LinkKnowledgeCommand, ListKnowledgeCommand,
    OrganizationId, ProjectId, QuoteId, UpdateKnowledgeCommand, UploadKnowledgeFileCommand,
    application::{MestierAuthorizer, MestierFileStorageService, MestierUseCase, policy},
    domain::{
        customer::ports::CustomerRepository, knowledge::service::KnowledgeService,
        member::ports::MemberRepository, project::ports::ProjectRepository,
        quote::ports::QuoteRepository, role::ports::RoleRepository,
    },
};

async fn require<M, R>(
    authz: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    action: &str,
    members: &mut M,
    roles: &mut R,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        action,
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

/// A resource that is missing or belongs to another organization is the same
/// answer: not found.
async fn require_resource<P, C, Q>(
    organization_id: OrganizationId,
    resource: KnowledgeResource,
    projects: &mut P,
    customers: &mut C,
    quotes: &mut Q,
) -> Result<(), CoreError>
where
    P: ProjectRepository,
    C: CustomerRepository,
    Q: QuoteRepository,
{
    let owner = match resource.resource_type {
        KnowledgeResourceType::Project => projects
            .find_by_id(ProjectId(resource.id))
            .await?
            .map(|project| project.organization_id),
        KnowledgeResourceType::Customer => customers
            .find_by_id(CustomerId(resource.id))
            .await?
            .map(|customer| customer.organization_id),
        KnowledgeResourceType::Quote => quotes
            .find_by_id(QuoteId(resource.id))
            .await?
            .map(|quote| quote.organization_id),
    };
    if owner == Some(organization_id) {
        Ok(())
    } else {
        Err(CoreError::NotFound)
    }
}

impl MestierUseCase {
    #[transactional(knowledge_article, member, role, authz)]
    pub async fn create_knowledge_note(
        &self,
        command: CreateKnowledgeNoteCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .create_note(command)
            .await
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn upload_knowledge_file(
        &self,
        command: UploadKnowledgeFileCommand,
        storage: &MestierFileStorageService,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .upload_file(command, storage)
            .await
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn get_knowledge_article(
        &self,
        command: GetKnowledgeCommand,
    ) -> Result<KnowledgeArticleWithLinks, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        let mut service = KnowledgeService::new(knowledge_article_repository);
        let article = service.get(command.organization_id, command.id).await?;
        let links = service.links(command.organization_id, command.id).await?;
        Ok(KnowledgeArticleWithLinks { article, links })
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn list_knowledge_articles(
        &self,
        command: ListKnowledgeCommand,
    ) -> Result<KnowledgeOutcome<(Vec<KnowledgeArticle>, u64)>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.view",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .list(
                command.organization_id,
                command.query,
                command.tags,
                command.resource,
                command.limit,
                command.offset,
            )
            .await
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn update_knowledge_article(
        &self,
        command: UpdateKnowledgeCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .update(command)
            .await
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn delete_knowledge_article(
        &self,
        command: DeleteKnowledgeCommand,
    ) -> Result<KnowledgeArticle, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .delete(command.organization_id, command.id)
            .await
    }

    #[transactional(knowledge_article, project, customer, quote, member, role, authz)]
    pub async fn link_knowledge_article(
        &self,
        command: LinkKnowledgeCommand,
    ) -> Result<KnowledgeLink, CoreError> {
        let mut project_repository = project_repository;
        let mut customer_repository = customer_repository;
        let mut quote_repository = quote_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;
        require_resource(
            command.organization_id,
            command.resource,
            &mut project_repository,
            &mut customer_repository,
            &mut quote_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .link(command.organization_id, command.id, command.resource)
            .await
    }

    #[transactional(knowledge_article, member, role, authz)]
    pub async fn unlink_knowledge_article(
        &self,
        command: LinkKnowledgeCommand,
    ) -> Result<(), CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require(
            authz,
            command.actor.clone(),
            command.organization_id,
            "knowledge.manage",
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        KnowledgeService::new(knowledge_article_repository)
            .unlink(command.organization_id, command.id, command.resource)
            .await
    }
}
