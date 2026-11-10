use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    api::Fault,
    photos::{merge_tags, normalize_tag},
    session::{Session, uuid},
    tools::{MestierTools, finish, json_result},
};

pub const DEFAULT_LIMIT: u64 = 10;
pub const MAX_LIMIT: u64 = 50;
pub const DEFAULT_TEXT_CHARS: usize = 20_000;
pub const MAX_TEXT_CHARS: usize = 50_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Project,
    Customer,
    Quote,
}

impl ResourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Customer => "customer",
            Self::Quote => "quote",
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchKnowledgeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(
        description = "Words to look for, in French, in titles, tags, notes and the text of files. Best matches first. Omit to list the newest articles."
    )]
    pub query: Option<String>,
    #[schemars(description = "Only articles carrying all of these tags.")]
    pub tags: Option<Vec<String>>,
    #[schemars(
        description = "With resource_id: only the articles linked to this project, customer or quote."
    )]
    pub resource_type: Option<ResourceKind>,
    #[schemars(description = "UUID of the project, customer or quote. Goes with resource_type.")]
    pub resource_id: Option<String>,
    #[schemars(description = "Articles per page. Default 10, at most 50.")]
    pub limit: Option<u64>,
    #[schemars(description = "Page number, starting at 1.")]
    pub page: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct GetKnowledgeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the article (id from search_knowledge).")]
    pub article_id: String,
    #[schemars(
        description = "Where to start in the text, in characters. Default 0. Use it to read on when text_truncated is true."
    )]
    pub offset: Option<usize>,
    #[schemars(description = "Characters of text to return. Default 20000, at most 50000.")]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateNoteArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Title, 1 to 200 characters.")]
    pub title: String,
    #[schemars(description = "Text of the note, up to 100000 characters.")]
    pub body: String,
    #[schemars(description = "Tags, at most 20. Lowercased and trimmed.")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateKnowledgeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the article.")]
    pub article_id: String,
    #[schemars(description = "New title. Omit to keep it.")]
    pub title: Option<String>,
    #[schemars(description = "New text. Notes only: the text of a file cannot be edited.")]
    pub body: Option<String>,
    #[schemars(description = "Tags to add. At most 20 tags per article.")]
    pub add_tags: Option<Vec<String>>,
    #[schemars(description = "Tags to remove.")]
    pub remove_tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct LinkKnowledgeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the article.")]
    pub article_id: String,
    #[schemars(description = "Kind of resource to attach the article to.")]
    pub resource_type: ResourceKind,
    #[schemars(description = "UUID of the project, customer or quote.")]
    pub resource_id: String,
}

pub fn search_limit(requested: Option<u64>) -> u64 {
    requested.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

/// A window of `text`, counted in characters, and whether text remains after it.
pub fn text_window(text: &str, offset: usize, max_chars: Option<usize>) -> (String, bool) {
    let size = max_chars
        .unwrap_or(DEFAULT_TEXT_CHARS)
        .clamp(1, MAX_TEXT_CHARS);
    let mut chars = text.chars().skip(offset);
    let window: String = chars.by_ref().take(size).collect();
    (window, chars.next().is_some())
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

fn slim(article: &Value) -> Value {
    json!({
        "id": article["id"],
        "kind": article["kind"],
        "title": article["title"],
        "tags": article["tags"],
        "body": article["body"],
        "file": article["file"],
        "updated_at": article["updated_at"],
    })
}

impl Session<'_> {
    pub async fn search_knowledge(&self, args: &SearchKnowledgeArgs) -> Result<Value, Fault> {
        if args.resource_type.is_some() != args.resource_id.is_some() {
            return Err(Fault::new("`resource_type` et `resource_id` vont ensemble"));
        }
        let resource_id = args.resource_id.as_deref().map(uuid).transpose()?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        let limit = search_limit(args.limit);
        let page = args.page.unwrap_or(1).max(1);
        let mut query = vec![("page", page.to_string()), ("per_page", limit.to_string())];
        if let Some(text) = args.query.as_deref().filter(|text| !text.trim().is_empty()) {
            query.push(("q", text.trim().to_owned()));
        }
        for tag in args.tags.as_deref().unwrap_or_default() {
            query.push(("tag", normalize_tag(tag)));
        }
        if let (Some(kind), Some(id)) = (args.resource_type, resource_id) {
            query.push(("resource_type", kind.as_str().to_owned()));
            query.push(("resource_id", id));
        }
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/knowledge/articles"),
                &query,
            )
            .await?;
        let articles: Vec<Value> = body["data"].as_array().cloned().unwrap_or_default();
        Ok(json!({
            "articles": articles.iter().map(slim).collect::<Vec<_>>(),
            "page": body["pagination"]["current_page"].as_u64().unwrap_or(page),
            "total": body["pagination"]["total"],
        }))
    }

    pub async fn get_knowledge(&self, args: &GetKnowledgeArgs) -> Result<Value, Fault> {
        let id = uuid(&args.article_id)?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/knowledge/articles/{id}"),
                &[],
            )
            .await?;
        let article = &body["data"];
        let mut out = slim(article);
        out["links"] = article["links"].clone();
        if let Some(text) = article["extracted_text"].as_str() {
            let offset = args.offset.unwrap_or(0);
            let (window, truncated) = text_window(text, offset, args.max_chars);
            out["text"] = json!(window);
            out["text_offset"] = json!(offset);
            out["text_length"] = json!(text.chars().count());
            out["text_truncated"] = json!(truncated);
        }
        Ok(out)
    }

    pub async fn create_note(&self, args: &CreateNoteArgs) -> Result<Value, Fault> {
        let org = self.organization_id(args.organization.as_deref()).await?;
        let tags =
            merge_tags(&[], args.tags.as_deref().unwrap_or_default(), &[]).map_err(Fault::new)?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/knowledge/notes"),
                &json!({"title": args.title, "body": args.body, "tags": tags}),
            )
            .await?;
        Ok(slim(&body["data"]))
    }

    pub async fn update_knowledge(&self, args: &UpdateKnowledgeArgs) -> Result<Value, Fault> {
        let id = uuid(&args.article_id)?;
        let add = args.add_tags.as_deref().unwrap_or_default();
        let remove = args.remove_tags.as_deref().unwrap_or_default();
        if args.title.is_none() && args.body.is_none() && add.is_empty() && remove.is_empty() {
            return Err(Fault::new("aucune modification demandée"));
        }
        let org = self.organization_id(args.organization.as_deref()).await?;
        let path = format!("organizations/{org}/knowledge/articles/{id}");
        let mut patch = json!({});
        if let Some(title) = &args.title {
            patch["title"] = json!(title);
        }
        if let Some(text) = &args.body {
            patch["body"] = json!(text);
        }
        if !add.is_empty() || !remove.is_empty() {
            let current = self.api.get(self.token, &path, &[]).await?;
            let tags =
                merge_tags(&strings(&current["data"]["tags"]), add, remove).map_err(Fault::new)?;
            patch["tags"] = json!(tags);
        }
        let body = self.api.patch(self.token, &path, &patch).await?;
        Ok(slim(&body["data"]))
    }

    pub async fn link_knowledge(&self, args: &LinkKnowledgeArgs) -> Result<Value, Fault> {
        let id = uuid(&args.article_id)?;
        let resource_id = uuid(&args.resource_id)?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/knowledge/articles/{id}/links"),
                &json!({"resource_type": args.resource_type.as_str(), "resource_id": resource_id}),
            )
            .await?;
        Ok(body["data"].clone())
    }
}

#[tool_router(router = knowledge_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "Search the organization's knowledge base: documents (quotes, price lists, terms, datasheets) and notes. query is searched in French in titles, tags, notes and the text read from files, best matches first; tags (all must match); resource_type + resource_id restrict to the articles linked to a project, customer or quote. Returns {articles:[{id, kind, title, tags, body, file, updated_at}], page, total}; a file's text is read with get_knowledge_article. limit defaults to 10, at most 50."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "search_knowledge"))]
    async fn search_knowledge(
        &self,
        Parameters(args): Parameters<SearchKnowledgeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "search_knowledge") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.search_knowledge(&args).await?)) }.await)
    }

    #[tool(
        description = "Read one article of the knowledge base. For a file, text is the text read from it (absent for a scan or an image), returned in a window of max_chars characters from offset: when text_truncated is true, call again with a larger offset. Also returns links, the projects, customers and quotes the article is attached to."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_knowledge_article"))]
    async fn get_knowledge_article(
        &self,
        Parameters(args): Parameters<GetKnowledgeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_knowledge_article") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.get_knowledge(&args).await?)) }.await)
    }

    #[tool(
        description = "Create a note in the knowledge base: a title, a text and tags. Files are added by the user in the web app, never through this tool. Returns the note. Attach it to a project, customer or quote with link_knowledge_article."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_knowledge_note"))]
    async fn create_knowledge_note(
        &self,
        Parameters(args): Parameters<CreateNoteArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_knowledge_note") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.create_note(&args).await?)) }.await)
    }

    #[tool(
        description = "Change the title, the text (notes only) or the tags of a knowledge base article: add_tags and remove_tags change the tags without resending the others. Articles cannot be deleted. Returns the article."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_knowledge_article"))]
    async fn update_knowledge_article(
        &self,
        Parameters(args): Parameters<UpdateKnowledgeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_knowledge_article") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.update_knowledge(&args).await?)) }.await)
    }

    #[tool(
        description = "Attach a knowledge base article to a project, a customer or a quote. Attaching twice keeps one link. Links cannot be removed through this tool. Returns the link."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "link_knowledge_article"))]
    async fn link_knowledge_article(
        &self,
        Parameters(args): Parameters<LinkKnowledgeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "link_knowledge_article") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.link_knowledge(&args).await?)) }.await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_defaults_and_is_capped() {
        assert_eq!(search_limit(None), 10);
        assert_eq!(search_limit(Some(500)), 50);
        assert_eq!(search_limit(Some(0)), 1);
    }

    #[test]
    fn the_text_window_counts_characters_and_says_when_more_remains() {
        let (first, more) = text_window("éàùçabcdef", 0, Some(4));
        assert_eq!((first.as_str(), more), ("éàùç", true));
        let (rest, more) = text_window("éàùçabcdef", 4, Some(50));
        assert_eq!((rest.as_str(), more), ("abcdef", false));
        let (nothing, more) = text_window("abc", 10, None);
        assert_eq!((nothing.as_str(), more), ("", false));
    }

    #[test]
    fn the_text_window_is_capped() {
        let long = "x".repeat(MAX_TEXT_CHARS + 10);
        let (window, more) = text_window(&long, 0, Some(1_000_000));
        assert_eq!(window.chars().count(), MAX_TEXT_CHARS);
        assert!(more);
    }
}
