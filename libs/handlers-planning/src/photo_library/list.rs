use auth::Identity;
use axum::{
    Extension,
    extract::{RawQuery, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    require_org_membership, resolve_actor,
};
use mestier_core::{ListLibraryPhotosCommand, ProjectId};
use url::form_urlencoded;

use crate::photo_library::{PhotosPath, response::LibraryPhotoResponse, unprocessable};

#[derive(Debug, Default)]
pub(crate) struct ListQuery {
    page: Option<u64>,
    per_page: Option<u64>,
    query: Option<String>,
    tags: Vec<String>,
    project_id: Option<ProjectId>,
    to_sort: bool,
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
                "project_id" => {
                    parsed.project_id =
                        Some(value.parse().map(ProjectId).map_err(|_| {
                            ApiError::Validation("project_id: not a uuid".to_owned())
                        })?);
                }
                "to_sort" => {
                    parsed.to_sort = value.parse().map_err(|_| {
                        ApiError::Validation("to_sort: expected true or false".to_owned())
                    })?;
                }
                _ => {}
            }
        }
        Ok(parsed)
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
    path = "/api/v1/organizations/{organization_id}/photo-library/photos",
    operation_id = "listLibraryPhotos",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("page" = Option<u64>, Query, description = "Page number, from 1"),
        ("per_page" = Option<u64>, Query, description = "Items per page, at most 100"),
        ("q" = Option<String>, Query, description = "Case-insensitive text searched in captions and tags"),
        ("tag" = Option<Vec<String>>, Query, description = "Tag filter, repeatable: a photo must carry every tag given"),
        ("project_id" = Option<mestier_core::ProjectId>, Query, description = "Only photos from this project"),
        ("to_sort" = Option<bool>, Query, description = "Only photos with neither caption nor tag"),
    ),
    responses(
        (status = 200, description = "The photos, newest first", body = inline(DataEnvelope<Vec<LibraryPhotoResponse>>)),
        (status = 400, description = "A query parameter is malformed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "An empty or too long tag filter"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PhotosPath { organization_id }: PhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    RawQuery(raw): RawQuery,
) -> Result<Response<LibraryPhotoResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let query = ListQuery::parse(raw.as_deref())?;

    let pagination = query.pagination();
    let per_page = pagination.per_page();
    let page = pagination.page();
    let (photos, total) = state
        .usecase
        .list_library_photos(ListLibraryPhotosCommand {
            actor,
            organization_id,
            query: query.query,
            tags: query.tags,
            project_id: query.project_id,
            to_sort: query.to_sort,
            limit: per_page,
            offset: pagination.offset(),
        })
        .await?
        .map_err(unprocessable)?;

    let items: Vec<LibraryPhotoResponse> = photos.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_tags_and_every_filter_are_read() {
        let query = ListQuery::parse(Some(
            "q=massif%20rose&tag=rosier&tag=terrasse&to_sort=true&page=2&per_page=10",
        ))
        .unwrap();

        assert_eq!(query.query.as_deref(), Some("massif rose"));
        assert_eq!(query.tags, vec!["rosier", "terrasse"]);
        assert!(query.to_sort);
        assert_eq!(query.pagination().page(), 2);
        assert_eq!(query.pagination().per_page(), 10);
    }

    #[test]
    fn an_absent_query_means_no_filter() {
        let query = ListQuery::parse(None).unwrap();

        assert!(query.tags.is_empty());
        assert!(!query.to_sort);
        assert_eq!(query.pagination().page(), 1);
    }

    #[test]
    fn a_malformed_value_is_refused() {
        assert!(ListQuery::parse(Some("to_sort=maybe")).is_err());
        assert!(ListQuery::parse(Some("project_id=nope")).is_err());
        assert!(ListQuery::parse(Some("page=-1")).is_err());
    }
}
