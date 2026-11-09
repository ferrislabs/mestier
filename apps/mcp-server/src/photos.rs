use base64::{Engine, engine::general_purpose::STANDARD};
use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    api::Fault,
    session::{Session, uuid},
    tools::{MestierTools, finish, json_result},
};

pub const DEFAULT_LIMIT: u64 = 8;
pub const MAX_LIMIT_WITH_THUMBNAILS: u64 = 12;
pub const MAX_LIMIT: u64 = 50;
pub const MAX_BATCH: usize = 50;
pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_CHARS: usize = 40;
const MAX_THUMBNAIL_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchPhotosArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Text to look for, case-insensitive, in captions and tags.")]
    pub query: Option<String>,
    #[schemars(description = "Only photos carrying all of these tags.")]
    pub tags: Option<Vec<String>>,
    #[schemars(description = "UUID of the project the photos come from.")]
    pub project_id: Option<String>,
    #[schemars(description = "True to list only the photos still to sort: no caption and no tag.")]
    pub to_sort: Option<bool>,
    #[schemars(
        description = "Photos per page. Default 8, at most 12 with thumbnails and 50 without."
    )]
    pub limit: Option<u64>,
    #[schemars(description = "Page number, starting at 1.")]
    pub page: Option<u64>,
    #[schemars(
        description = "Return one JPEG thumbnail per photo after the text, in the same order. Default true."
    )]
    pub include_thumbnails: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct TagItem {
    #[schemars(description = "UUID of the library photo (id from search_photo_library).")]
    pub photo_id: String,
    #[schemars(description = "New caption. An empty string clears it. Omit to keep it.")]
    pub caption: Option<String>,
    #[schemars(description = "Tags to add. Lowercased and trimmed; at most 20 tags per photo.")]
    pub add_tags: Option<Vec<String>>,
    #[schemars(description = "Tags to remove.")]
    pub remove_tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct TagPhotosArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Photos to caption or tag, at most 50.")]
    pub items: Vec<TagItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum PlaceTarget {
    Section,
    Supply,
    Cover,
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct Placement {
    #[schemars(description = "UUID of the library photo (id from search_photo_library).")]
    pub photo_id: String,
    #[schemars(
        description = "Where the photo goes: section (a photos section), supply (a supply's photos) or cover (the cover photo)."
    )]
    pub target: PlaceTarget,
    #[schemars(
        description = "Id of the photos section (target=section). Optional when the DOE has a single photos section; created when it has none."
    )]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_id: Option<String>,
    #[schemars(description = "Title of the photos section, case-insensitive (target=section).")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_title: Option<String>,
    #[schemars(description = "Id of the supply (target=supply).")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supply_id: Option<String>,
    #[schemars(description = "Caption in the DOE. Defaults to the library caption.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct PlacePhotosArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Photos to place, at most 50. Each is copied into the DOE; later changes to the library do not alter the DOE."
    )]
    pub placements: Vec<Placement>,
}

pub fn normalize_tag(tag: &str) -> String {
    tag.trim().to_lowercase()
}

pub fn merge_tags(
    current: &[String],
    add: &[String],
    remove: &[String],
) -> Result<Vec<String>, String> {
    let removed: Vec<String> = remove.iter().map(|tag| normalize_tag(tag)).collect();
    let mut tags: Vec<String> = Vec::new();
    for tag in current.iter().chain(add) {
        let tag = normalize_tag(tag);
        if tag.is_empty() || removed.contains(&tag) || tags.contains(&tag) {
            continue;
        }
        if tag.chars().count() > MAX_TAG_CHARS {
            return Err(format!(
                "étiquette trop longue (plus de {MAX_TAG_CHARS} caractères) : « {tag} »"
            ));
        }
        tags.push(tag);
    }
    if tags.len() > MAX_TAGS {
        return Err(format!("plus de {MAX_TAGS} étiquettes sur une photo"));
    }
    Ok(tags)
}

pub fn search_limit(requested: Option<u64>, thumbnails: bool) -> u64 {
    let cap = if thumbnails {
        MAX_LIMIT_WITH_THUMBNAILS
    } else {
        MAX_LIMIT
    };
    requested.unwrap_or(DEFAULT_LIMIT).clamp(1, cap)
}

fn batch<T>(items: &[T], what: &str) -> Result<(), Fault> {
    if items.is_empty() {
        return Err(Fault::new(format!("aucun élément dans `{what}`")));
    }
    if items.len() > MAX_BATCH {
        return Err(Fault::new(format!(
            "trop d'éléments dans `{what}` : {MAX_BATCH} au plus"
        )));
    }
    Ok(())
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn slim(photo: &Value) -> Value {
    json!({
        "id": photo["id"],
        "caption": photo["caption"],
        "tags": photo["tags"],
        "project_id": photo["project_id"],
        "taken_at": photo["taken_at"],
        "width": photo["width"],
        "height": photo["height"],
    })
}

pub struct SearchFilter<'a> {
    pub query: Option<&'a str>,
    pub tags: &'a [String],
    pub project_id: Option<&'a str>,
    pub to_sort: bool,
    pub limit: Option<u64>,
    pub page: Option<u64>,
    pub include_thumbnails: bool,
}

pub struct PhotoSearch {
    pub listing: Value,
    pub thumbnails: Vec<Vec<u8>>,
}

impl Session<'_> {
    pub async fn search_photos(
        &self,
        organization: Option<&str>,
        filter: SearchFilter<'_>,
    ) -> Result<PhotoSearch, Fault> {
        let project_id = filter.project_id.map(uuid).transpose()?;
        let org = self.organization_id(organization).await?;
        let limit = search_limit(filter.limit, filter.include_thumbnails);
        let page = filter.page.unwrap_or(1).max(1);
        let mut query = vec![("page", page.to_string()), ("per_page", limit.to_string())];
        if let Some(text) = filter.query.filter(|text| !text.trim().is_empty()) {
            query.push(("q", text.trim().to_owned()));
        }
        for tag in filter.tags {
            query.push(("tag", normalize_tag(tag)));
        }
        if let Some(project_id) = project_id {
            query.push(("project_id", project_id));
        }
        if filter.to_sort {
            query.push(("to_sort", "true".to_owned()));
        }
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/photo-library/photos"),
                &query,
            )
            .await?;
        let photos: Vec<Value> = body["data"].as_array().cloned().unwrap_or_default();
        let listing = json!({
            "photos": photos.iter().map(slim).collect::<Vec<_>>(),
            "page": body["pagination"]["current_page"].as_u64().unwrap_or(page),
            "total": body["pagination"]["total"],
        });
        let mut thumbnails = Vec::new();
        if filter.include_thumbnails {
            for photo in &photos {
                let id = photo["id"]
                    .as_str()
                    .ok_or_else(|| Fault::new("photo sans identifiant"))?;
                let bytes = self
                    .api
                    .get_bytes(
                        self.token,
                        &format!("organizations/{org}/photo-library/photos/{id}/thumbnail"),
                        MAX_THUMBNAIL_BYTES,
                    )
                    .await?
                    .ok_or_else(|| Fault::new("miniature trop volumineuse"))?;
                thumbnails.push(bytes);
            }
        }
        Ok(PhotoSearch {
            listing,
            thumbnails,
        })
    }

    pub async fn tag_photos(
        &self,
        organization: Option<&str>,
        items: &[TagItem],
    ) -> Result<Value, Fault> {
        batch(items, "items")?;
        let org = self.organization_id(organization).await?;
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let outcome = self.tag_one(&org, item).await;
            results.push(match outcome {
                Ok(()) => json!({"photo_id": item.photo_id, "status": "updated"}),
                Err(fault) => {
                    json!({"photo_id": item.photo_id, "status": "failed", "error": fault.message})
                }
            });
        }
        Ok(json!({"results": results}))
    }

    async fn tag_one(&self, org: &str, item: &TagItem) -> Result<(), Fault> {
        let id = uuid(&item.photo_id)?;
        let add = item.add_tags.as_deref().unwrap_or_default();
        let remove = item.remove_tags.as_deref().unwrap_or_default();
        if item.caption.is_none() && add.is_empty() && remove.is_empty() {
            return Err(Fault::new("aucune modification demandée"));
        }
        let path = format!("organizations/{org}/photo-library/photos/{id}");
        let current = self.api.get(self.token, &path, &[]).await?;
        let tags =
            merge_tags(&strings(&current["data"]["tags"]), add, remove).map_err(Fault::new)?;
        let mut body = json!({"tags": tags});
        if let Some(caption) = &item.caption {
            body["caption"] = json!(caption);
        }
        self.api.patch(self.token, &path, &body).await?;
        Ok(())
    }

    pub async fn place_photos(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        placements: &[Placement],
    ) -> Result<Value, Fault> {
        batch(placements, "placements")?;
        let doe_id = uuid(doe_id)?;
        for placement in placements {
            uuid(&placement.photo_id)?;
        }
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/does/{doe_id}/photos/place"),
                &json!({"placements": placements}),
            )
            .await?;
        let outcome = body.get("data").unwrap_or(&body);
        Ok(json!({"doe": outcome["doe"], "results": outcome["results"]}))
    }
}

fn search_result(search: PhotoSearch) -> CallToolResult {
    let mut content = vec![ContentBlock::text(search.listing.to_string())];
    content.extend(
        search
            .thumbnails
            .iter()
            .map(|bytes| ContentBlock::image(STANDARD.encode(bytes), "image/jpeg")),
    );
    CallToolResult::success(content)
}

#[tool_router(router = photo_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "Search the organization's photo library (photothèque). Filters: query (caption and tags), tags (all must match), project_id, to_sort (no caption and no tag). Returns a JSON text {photos, page, total} then one JPEG thumbnail per photo in the same order. limit defaults to 8, at most 12 with thumbnails and 50 without (include_thumbnails=false)."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "search_photo_library"))]
    async fn search_photo_library(
        &self,
        Parameters(args): Parameters<SearchPhotosArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "search_photo_library") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let tags = args.tags.unwrap_or_default();
                let search = s
                    .search_photos(
                        args.organization.as_deref(),
                        SearchFilter {
                            query: args.query.as_deref(),
                            tags: &tags,
                            project_id: args.project_id.as_deref(),
                            to_sort: args.to_sort.unwrap_or(false),
                            limit: args.limit,
                            page: args.page,
                            include_thumbnails: args.include_thumbnails.unwrap_or(true),
                        },
                    )
                    .await?;
                Ok(search_result(search))
            }
            .await,
        )
    }

    #[tool(
        description = "Caption and tag library photos, at most 50 per call: each item sets a caption and/or adds and removes tags (at most 20 per photo). One failing item does not stop the others. Returns {results:[{photo_id, status, error?}]}. Photos cannot be deleted."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "tag_library_photos"))]
    async fn tag_library_photos(
        &self,
        Parameters(args): Parameters<TagPhotosArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "tag_library_photos") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .tag_photos(args.organization.as_deref(), &args.items)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Place library photos in a DOE, at most 50 per call. target is section (a photos section, created when the DOE has none), supply (a supply's photos) or cover (the cover photo, replacing the previous one). Each photo is copied into the DOE. One failing placement does not stop the others. Returns {doe, results:[{photo_id, status, key?, error?}]}."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "place_doe_files"))]
    async fn place_doe_files(
        &self,
        Parameters(args): Parameters<PlacePhotosArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "place_doe_files") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .place_photos(args.organization.as_deref(), &args.doe_id, &args.placements)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tags(items: &[&str]) -> Vec<String> {
        items.iter().map(|tag| (*tag).to_owned()).collect()
    }

    #[test]
    fn tags_are_trimmed_lowercased_and_deduplicated() {
        let merged = merge_tags(
            &tags(&["Rosier"]),
            &tags(&[" ROSIER ", "Terrasse", ""]),
            &[],
        );
        assert_eq!(merged, Ok(tags(&["rosier", "terrasse"])));
    }

    #[test]
    fn removal_wins_over_current_tags_whatever_the_case() {
        let merged = merge_tags(&tags(&["a", "b"]), &[], &tags(&[" A "]));
        assert_eq!(merged, Ok(tags(&["b"])));
    }

    #[test]
    fn a_twenty_first_tag_is_refused() {
        let many: Vec<String> = (0..21).map(|n| format!("t{n}")).collect();
        assert!(merge_tags(&[], &many, &[]).is_err());
        assert_eq!(merge_tags(&[], &many[..20], &[]).map(|t| t.len()), Ok(20));
    }

    #[test]
    fn a_tag_over_forty_characters_is_refused() {
        assert!(merge_tags(&[], &["x".repeat(41)], &[]).is_err());
    }

    #[test]
    fn the_limit_is_capped_by_the_thumbnails() {
        assert_eq!(search_limit(None, true), 8);
        assert_eq!(search_limit(Some(40), true), 12);
        assert_eq!(search_limit(Some(40), false), 40);
        assert_eq!(search_limit(Some(500), false), 50);
        assert_eq!(search_limit(Some(0), true), 1);
    }
}
