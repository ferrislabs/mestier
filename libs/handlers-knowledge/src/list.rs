use auth::Identity;
use axum::{
    Extension,
    extract::{RawQuery, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    require_org_membership, resolve_actor,
};
use mestier_core::{
    KnowledgeFolderId, KnowledgeFolderScope, KnowledgeResource, KnowledgeResourceType,
    ListKnowledgeCommand,
};
use url::form_urlencoded;
use uuid::Uuid;

use crate::{ArticlesPath, response::KnowledgeArticleResponse, unprocessable};

#[derive(Debug, Default)]
pub(crate) struct ListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    query: Option<String>,
    tags: Vec<String>,
    resource_type: Option<KnowledgeResourceType>,
    resource_id: Option<Uuid>,
    folder: Option<FolderParam>,
    recursive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FolderParam {
    Root,
    Folder(KnowledgeFolderId),
}

impl ListQuery {
    pub(crate) fn parse(raw: Option<&str>) -> Result<Self, ApiError> {
        let mut parsed = Self::default();
        for (key, value) in form_urlencoded::parse(raw.unwrap_or_default().as_bytes()) {
            match key.as_ref() {
                "page" => parsed.page = Some(number("page", &value)?),
                "per_page" => parsed.per_page = Some(number("per_page", &value)?),
                "q" => parsed.query = Some(value.into_owned()),
                "tag" => parsed.tags.push(value.into_owned()),
                "resource_type" => {
                    parsed.resource_type = Some(value.parse().map_err(|()| {
                        ApiError::Validation(
                            "resource_type: expected project, customer or quote".to_owned(),
                        )
                    })?);
                }
                "resource_id" => {
                    parsed.resource_id =
                        Some(value.parse().map_err(|_| {
                            ApiError::Validation("resource_id: not a uuid".to_owned())
                        })?);
                }
                "folder" => {
                    parsed.folder = Some(if value == "root" {
                        FolderParam::Root
                    } else {
                        FolderParam::Folder(value.parse().map_err(|_| {
                            ApiError::Validation("folder: expected root or a uuid".to_owned())
                        })?)
                    });
                }
                "recursive" => {
                    parsed.recursive = value.parse().map_err(|_| {
                        ApiError::Validation("recursive: expected true or false".to_owned())
                    })?;
                }
                _ => {}
            }
        }
        if parsed.resource_type.is_some() != parsed.resource_id.is_some() {
            return Err(ApiError::Validation(
                "resource_type and resource_id go together".to_owned(),
            ));
        }
        Ok(parsed)
    }

    fn resource(&self) -> Option<KnowledgeResource> {
        Some(KnowledgeResource {
            resource_type: self.resource_type?,
            id: self.resource_id?,
        })
    }

    /// The root with its whole tree is the whole drive: no folder filter.
    fn folder(&self) -> Option<KnowledgeFolderScope> {
        match (self.folder?, self.recursive) {
            (FolderParam::Root, true) => None,
            (FolderParam::Root, false) => Some(KnowledgeFolderScope::Root),
            (FolderParam::Folder(id), false) => Some(KnowledgeFolderScope::Directly(id)),
            (FolderParam::Folder(id), true) => Some(KnowledgeFolderScope::Within(id)),
        }
    }

    fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            per_page: self.per_page,
        }
    }
}

fn number(name: &str, value: &str) -> Result<u64, ApiError> {
    value
        .parse()
        .map_err(|_| ApiError::Validation(format!("{name}: expected a positive integer")))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/articles",
    operation_id = "listKnowledgeArticles",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("page" = Option<u64>, Query, description = "Page number, from 1"),
        ("per_page" = Option<u64>, Query, description = "Items per page, at most 100"),
        ("q" = Option<String>, Query, description = "Full-text search in French over title, tags, note and the text of files; best matches first"),
        ("tag" = Option<Vec<String>>, Query, description = "Tag filter, repeatable: an article must carry every tag given"),
        ("resource_type" = Option<String>, Query, description = "project, customer or quote. Goes with resource_id"),
        ("resource_id" = Option<uuid::Uuid>, Query, description = "Only the articles linked to this resource"),
        ("folder" = Option<String>, Query, description = "root, or a folder id: only the articles placed there. Absent searches the whole drive"),
        ("recursive" = Option<bool>, Query, description = "With folder, also the articles of every folder below it"),
    ),
    responses(
        (status = 200, description = "The articles, without the text read from files", body = inline(DataEnvelope<Vec<KnowledgeArticleResponse>>)),
        (status = 400, description = "A query parameter is malformed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Folder not found, or held by another organization"),
        (status = 422, description = "An empty or too long tag filter"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ArticlesPath { organization_id }: ArticlesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    RawQuery(raw): RawQuery,
) -> Result<Response<KnowledgeArticleResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let query = ListQuery::parse(raw.as_deref())?;

    let pagination = query.pagination();
    let per_page = pagination.per_page();
    let page = pagination.page();
    let resource = query.resource();
    let folder = query.folder();
    let (articles, total) = state
        .usecase
        .list_knowledge_articles(ListKnowledgeCommand {
            actor,
            organization_id,
            query: query.query,
            tags: query.tags,
            resource,
            folder,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?
        .map_err(unprocessable)?;

    let items: Vec<KnowledgeArticleResponse> = articles.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);
    Ok(Response::Paginated(Page::new(items, meta)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_filter_is_read() {
        let id = Uuid::nil();
        let query = ListQuery::parse(Some(&format!(
            "q=arrosage%20auto&tag=devis&tag=2026&resource_type=project&resource_id={id}&page=2"
        )))
        .unwrap();
        assert_eq!(query.query.as_deref(), Some("arrosage auto"));
        assert_eq!(query.tags, vec!["devis", "2026"]);
        assert_eq!(query.resource().unwrap().id, id);
        assert_eq!(query.pagination().page(), 2);
    }

    #[test]
    fn the_folder_filter_reads_root_a_folder_and_its_tree() {
        let id = Uuid::nil();
        assert_eq!(
            ListQuery::parse(Some("folder=root")).unwrap().folder(),
            Some(KnowledgeFolderScope::Root)
        );
        assert_eq!(
            ListQuery::parse(Some("folder=root&recursive=true"))
                .unwrap()
                .folder(),
            None
        );
        assert_eq!(
            ListQuery::parse(Some(&format!("folder={id}")))
                .unwrap()
                .folder(),
            Some(KnowledgeFolderScope::Directly(KnowledgeFolderId(id)))
        );
        assert_eq!(
            ListQuery::parse(Some(&format!("folder={id}&recursive=true")))
                .unwrap()
                .folder(),
            Some(KnowledgeFolderScope::Within(KnowledgeFolderId(id)))
        );
        assert!(ListQuery::parse(Some("folder=nowhere")).is_err());
    }

    #[test]
    fn a_resource_filter_needs_both_halves() {
        assert!(ListQuery::parse(Some("resource_type=quote")).is_err());
        assert!(ListQuery::parse(Some(&format!("resource_id={}", Uuid::nil()))).is_err());
    }

    #[test]
    fn a_malformed_value_is_refused() {
        assert!(ListQuery::parse(Some("resource_type=invoice&resource_id=x")).is_err());
        assert!(ListQuery::parse(Some("page=-1")).is_err());
    }
}
