use base64::{Engine, engine::general_purpose::STANDARD};
use http::request::Parts;
use mcp_tools::CLIENT_HEADER;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Extension, wrapper::Parameters},
    model::{
        CallToolResult, ContentBlock, MetaObject, ResourceContents, ServerCapabilities,
        ServerConfig,
    },
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    api::{ApiClient, Fault},
    edit::{MakerLabel, NewSection, PerPage, SectionKind, SectionPatch, SectionRef, SupplyPatch},
    files::{FileFetcher, FileParam, Source, Target},
    maintenance::ContractDraft,
    session::{FileRequest, HeaderPatch, ManualSupply, Paging, PdfOutcome, Session, SupplyRequest},
};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Page number, starting at 1.")]
    pub page: Option<u64>,
    #[schemars(description = "Items per page, 1 to 100.")]
    pub per_page: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ProjectArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the project.")]
    pub project_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DoeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ProductArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the product.")]
    pub product_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct QuoteArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the quote.")]
    pub quote_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct InvoiceArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the invoice.")]
    pub invoice_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct SearchProductsArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(
        description = "Text to look for, case-insensitive, in the product name, SKU or manufacturer."
    )]
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateDoeArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(
        description = "UUID of the project that gets the DOE. A project has at most one DOE."
    )]
    pub project_id: String,
    #[schemars(
        description = "UUID of the DOE model to start from. Optional when the organization has exactly one active model; see list_doe_models."
    )]
    pub model_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct AddSupplyArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Id of the target supplies section of the DOE (see get_doe). Takes precedence over section_title."
    )]
    pub section_id: Option<String>,
    #[schemars(
        description = "Title of the target supplies section, case-insensitive. Optional when the DOE has a single supplies section."
    )]
    pub section_title: Option<String>,
    #[schemars(
        description = "UUID of the catalogue product whose technical sheet is copied into the DOE."
    )]
    pub product_id: String,
    #[schemars(description = "Line of the DPGF (price breakdown) this supply answers to.")]
    pub dpgf_ref: Option<String>,
    #[schemars(description = "Where the supply is used, for example the area of the site.")]
    pub destination: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct AddManualSupplyArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Id of the target supplies section of the DOE (see get_doe). Takes precedence over section_title."
    )]
    pub section_id: Option<String>,
    #[schemars(
        description = "Title of the target supplies section, case-insensitive. Optional when the DOE has a single supplies section."
    )]
    pub section_title: Option<String>,
    #[schemars(
        description = "What the supply is, for example \"Banc Athène\" or \"Pouzzolane 10/20\". Required."
    )]
    pub designation: String,
    #[schemars(
        description = "Heading printed in the DOE when it differs from the designation, for example \"Plantation d'arbres\" for \"Arbres\"."
    )]
    pub title: Option<String>,
    #[schemars(description = "Line of the DPGF (price breakdown) this supply answers to.")]
    pub dpgf_ref: Option<String>,
    #[schemars(description = "Where the supply is used, for example the area of the site.")]
    pub destination: Option<String>,
    #[schemars(description = "Name of the supplier the supply was bought from.")]
    pub supplier_name: Option<String>,
    #[schemars(
        description = "Supplier address, several lines allowed. Ignored without supplier_name."
    )]
    pub supplier_address: Option<String>,
    #[schemars(description = "Supplier phone. Ignored without supplier_name.")]
    pub supplier_phone: Option<String>,
    #[schemars(description = "Supplier email. Ignored without supplier_name.")]
    pub supplier_email: Option<String>,
    #[schemars(description = "Manufacturer, or the nursery for plants.")]
    pub maker: Option<String>,
    #[schemars(
        description = "Technical characteristics, one per line (dimensions, colour, reference, sizes of plants...)."
    )]
    pub specs: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateHeaderArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "New title. Cannot be blank.")]
    pub title: Option<String>,
    #[schemars(description = "New place. An empty string clears it.")]
    pub place: Option<String>,
    #[schemars(description = "New lot. An empty string clears it.")]
    pub lot: Option<String>,
    #[schemars(description = "New issue date, ISO format YYYY-MM-DD.")]
    pub issued_on: Option<String>,
    #[schemars(description = "New author. An empty string clears it.")]
    pub author: Option<String>,
    #[schemars(description = "New revision label, 1 to 8 characters.")]
    pub revision: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateSupplyArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Id of the supply to change (see get_doe).")]
    pub supply_id: String,
    #[schemars(description = "Heading printed in the DOE. An empty string clears it.")]
    pub title: Option<String>,
    #[schemars(description = "What the supply is. Cannot be blank.")]
    pub designation: Option<String>,
    #[schemars(description = "Line of the DPGF. An empty string clears it.")]
    pub dpgf_ref: Option<String>,
    #[schemars(description = "Where the supply is used. An empty string clears it.")]
    pub destination: Option<String>,
    #[schemars(description = "Supplier name. An empty string removes the whole supplier.")]
    pub supplier_name: Option<String>,
    #[schemars(
        description = "Supplier address. An empty string clears it. Needs an existing supplier or supplier_name."
    )]
    pub supplier_address: Option<String>,
    #[schemars(
        description = "Supplier phone. An empty string clears it. Needs an existing supplier or supplier_name."
    )]
    pub supplier_phone: Option<String>,
    #[schemars(
        description = "Supplier email. An empty string clears it. Needs an existing supplier or supplier_name."
    )]
    pub supplier_email: Option<String>,
    #[schemars(
        description = "Manufacturer, or the nursery for plants. An empty string clears it."
    )]
    pub maker: Option<String>,
    #[schemars(
        description = "Technical characteristics, one per line. An empty string clears them."
    )]
    pub specs: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DeleteSupplyArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Id of the supply to remove (see get_doe).")]
    pub supply_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct AddSectionArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Title of the new section. Cannot be blank.")]
    pub title: String,
    #[schemars(description = "Kind of section: supplies, photos, plans or blank_pages.")]
    pub kind: SectionKind,
    #[schemars(
        description = "Supplies sections only: manufacturer or nursery. Default manufacturer."
    )]
    pub maker_label: Option<MakerLabel>,
    #[schemars(
        description = "Photos sections only: photos per page, one, two or four. Default two."
    )]
    pub per_page: Option<PerPage>,
    #[schemars(description = "Blank pages sections only: number of pages, 1 to 20. Default 1.")]
    pub count: Option<u8>,
    #[schemars(description = "1-based position among the sections. Default: at the end.")]
    pub position: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateSectionArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Id of the section (see get_doe). Takes precedence over section_title."
    )]
    pub section_id: Option<String>,
    #[schemars(
        description = "Title of the section, case-insensitive, when section_id is not given."
    )]
    pub section_title: Option<String>,
    #[schemars(description = "New title. Cannot be blank.")]
    pub title: Option<String>,
    #[schemars(description = "Whether the section is printed in the DOE.")]
    pub enabled: Option<bool>,
    #[schemars(description = "Supplies sections only: manufacturer or nursery.")]
    pub maker_label: Option<MakerLabel>,
    #[schemars(description = "Photos sections only: photos per page, one, two or four.")]
    pub per_page: Option<PerPage>,
    #[schemars(description = "Blank pages sections only: number of pages, 1 to 20.")]
    pub count: Option<u8>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DeleteSectionArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Id of the section to delete (see get_doe).")]
    pub section_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ReorderSectionsArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Ids of all the sections of the DOE, each exactly once, in the new order."
    )]
    pub section_ids: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct AddFileArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(
        description = "Where the file goes: photo (a photos section), plan (a plans section), supply_photo or supply_datasheet (a supply), cover_photo (the DOE cover, replacing any previous one; no caption)."
    )]
    pub target: Target,
    #[schemars(
        description = "photo and plan only: id of the target section (see get_doe). Takes precedence over section_title."
    )]
    pub section_id: Option<String>,
    #[schemars(
        description = "photo and plan only: title of the target section, case-insensitive. Optional when the DOE has a single section of that kind."
    )]
    pub section_title: Option<String>,
    #[schemars(
        description = "supply_photo and supply_datasheet only: id of the supply (see get_doe)."
    )]
    pub supply_id: Option<String>,
    #[schemars(description = "Caption of the photo or plan. Datasheets have no caption.")]
    pub caption: Option<String>,
    #[schemars(
        description = "The file attached by the user in the conversation. Use this when the client offers file attachments."
    )]
    pub file: Option<FileParam>,
    #[schemars(
        description = "Base64 content of the file (5 MB decoded at most), for clients without file attachments. Give exactly one of file and content_base64."
    )]
    pub content_base64: Option<String>,
    #[schemars(description = "Name of the file, informational.")]
    pub file_name: Option<String>,
    #[schemars(
        description = "Declared MIME type, informational: the type is detected from the content."
    )]
    pub mime_type: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateFileArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Storage key of the file (see get_doe).")]
    pub key: String,
    #[schemars(
        description = "New caption. An empty string clears it. Not available for datasheets."
    )]
    pub caption: Option<String>,
    #[schemars(
        description = "Moves a photo or plan to this other section of the same kind (id, see get_doe)."
    )]
    pub section_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DeleteFileArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DOE.")]
    pub doe_id: String,
    #[schemars(description = "Storage key of the file to remove from the DOE (see get_doe).")]
    pub key: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct MaintenanceDueArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(
        description = "ISO week to prepare, written like 2026-W43. The recap of a week also holds what was due before it."
    )]
    pub week: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct MaintenanceContractArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the maintenance contract (contract_id of a recap line).")]
    pub contract_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateMaintenanceDraftArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[serde(flatten)]
    pub draft: ContractDraft,
}

#[derive(Clone)]
pub struct MestierTools {
    api: ApiClient,
    files: FileFetcher,
    tool_router: ToolRouter<Self>,
}

impl MestierTools {
    pub fn new(api: ApiClient, files: FileFetcher) -> Self {
        Self {
            api,
            files,
            tool_router: Self::tool_router(),
        }
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect()
    }

    fn session<'a>(&'a self, parts: &'a Parts, tool: &str) -> Result<Session<'a>, CallToolResult> {
        let Some(token) = bearer(parts) else {
            return Err(failure(&Fault::new("jeton d'accès absent")));
        };
        let client = parts
            .headers
            .get(CLIENT_HEADER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("-");
        tracing::info!(tool, client, "tool call");
        Ok(Session::new(&self.api, token))
    }
}

fn finish(outcome: Result<CallToolResult, Fault>) -> CallToolResult {
    outcome.unwrap_or_else(|fault| failure(&fault))
}

pub fn bearer(parts: &Parts) -> Option<&str> {
    let header = parts
        .headers
        .get(http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    let (scheme, token) = header.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty()).then_some(token.trim())
}

fn failure(fault: &Fault) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(fault.to_string())])
}

fn json_result(value: &Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

fn file_param_meta() -> MetaObject {
    let mut meta = MetaObject::new();
    meta.0
        .insert("openai/fileParams".to_owned(), json!(["file"]));
    meta
}

fn paging(args: &ListArgs) -> Paging {
    Paging {
        page: args.page,
        per_page: args.per_page,
    }
}

#[tool_router(router = tool_router)]
impl MestierTools {
    #[tool(
        description = "List the organizations the user belongs to and that allow AI access. Returns id, name and slug; use the slug as the `organization` argument of the other tools."
    )]
    async fn list_organizations(&self, Extension(parts): Extension<Parts>) -> CallToolResult {
        let s = match self.session(&parts, "list_organizations") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { s.list_organizations().await.map(|v| json_result(&v)) }.await)
    }

    #[tool(
        description = "List the projects of an organization (paginated, active projects only). Returns data and meta; each project has an id usable with get_project and create_doe."
    )]
    async fn list_projects(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_projects") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "projects", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one project of an organization by id.")]
    async fn get_project(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_project") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_scoped(args.organization.as_deref(), "projects", &args.project_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the DOEs (dossiers des ouvrages exécutés) of an organization, most recently edited first. Returns summaries with ids usable with get_doe."
    )]
    async fn list_does(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_does") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "does", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(
        description = "Get the full content of a DOE: header, parties and sections with their supplies. Section ids returned here are used by add_doe_supply_from_product."
    )]
    async fn get_doe(
        &self,
        Parameters(args): Parameters<DoeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_doe") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_scoped(args.organization.as_deref(), "does", &args.doe_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the active DOE models of an organization. A model is the starting structure of a new DOE; its id is the `model_id` of create_doe."
    )]
    async fn list_doe_models(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_doe_models") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "doe-models", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(
        description = "Search the product catalogue of an organization by name, SKU or manufacturer (case-insensitive substring). Returns at most 50 products with their id, price and technical sheet."
    )]
    async fn search_products(
        &self,
        Parameters(args): Parameters<SearchProductsArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "search_products") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let found = s
                    .search_products(args.organization.as_deref(), &args.query)
                    .await?;
                Ok(json_result(&found))
            }
            .await,
        )
    }

    #[tool(
        description = "Get one catalogue product by id, with its technical sheet (specs, photos, datasheets)."
    )]
    async fn get_product(
        &self,
        Parameters(args): Parameters<ProductArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_product") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_global(args.organization.as_deref(), "products", &args.product_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(description = "List the quotes (devis) of an organization (paginated).")]
    async fn list_quotes(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_quotes") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "quotes", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one quote (devis) by id, with its lines.")]
    async fn get_quote(
        &self,
        Parameters(args): Parameters<QuoteArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_quote") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_global(args.organization.as_deref(), "quotes", &args.quote_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(description = "List the invoices (factures) of an organization (paginated).")]
    async fn list_invoices(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_invoices") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "invoices", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one invoice (facture) by id, with its lines.")]
    async fn get_invoice(
        &self,
        Parameters(args): Parameters<InvoiceArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_invoice") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_global(args.organization.as_deref(), "invoices", &args.invoice_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create the DOE of a project from a DOE model. A project has at most one DOE. Returns the new DOE."
    )]
    async fn create_doe(
        &self,
        Parameters(args): Parameters<CreateDoeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_doe") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_doe(
                        args.organization.as_deref(),
                        &args.project_id,
                        args.model_id.as_deref(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Add a supply to a DOE by copying a catalogue product's technical sheet into a supplies section. The copy is a snapshot: later product edits do not change the DOE. Returns the updated DOE."
    )]
    async fn add_doe_supply_from_product(
        &self,
        Parameters(args): Parameters<AddSupplyArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "add_doe_supply_from_product") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .add_supply(
                        args.organization.as_deref(),
                        SupplyRequest {
                            doe_id: &args.doe_id,
                            section_id: args.section_id.as_deref(),
                            section_title: args.section_title.as_deref(),
                            product_id: &args.product_id,
                            dpgf_ref: args.dpgf_ref.as_deref(),
                            destination: args.destination.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Add a supply to a DOE from details given by the user, without a catalogue product: designation, DPGF line, destination, supplier, manufacturer and technical characteristics. Use add_doe_supply_from_product instead when the supply exists in the catalogue. Returns the updated DOE."
    )]
    async fn add_doe_supply(
        &self,
        Parameters(args): Parameters<AddManualSupplyArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "add_doe_supply") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .add_manual_supply(
                        args.organization.as_deref(),
                        ManualSupply {
                            doe_id: &args.doe_id,
                            section_id: args.section_id.as_deref(),
                            section_title: args.section_title.as_deref(),
                            designation: &args.designation,
                            title: args.title.as_deref(),
                            dpgf_ref: args.dpgf_ref.as_deref(),
                            destination: args.destination.as_deref(),
                            supplier_name: args.supplier_name.as_deref(),
                            supplier_address: args.supplier_address.as_deref(),
                            supplier_phone: args.supplier_phone.as_deref(),
                            supplier_email: args.supplier_email.as_deref(),
                            maker: args.maker.as_deref(),
                            specs: args.specs.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update the header of a DOE (title, place, lot, issue date, author, revision). Only the given fields change; sections and parties are kept. Returns the updated DOE."
    )]
    async fn update_doe_header(
        &self,
        Parameters(args): Parameters<UpdateHeaderArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_doe_header") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_header(
                        args.organization.as_deref(),
                        &args.doe_id,
                        HeaderPatch {
                            title: args.title.as_deref(),
                            place: args.place.as_deref(),
                            lot: args.lot.as_deref(),
                            issued_on: args.issued_on.as_deref(),
                            author: args.author.as_deref(),
                            revision: args.revision.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a supply of a DOE. Only the given fields change; an empty string clears an optional field, and an empty supplier_name removes the supplier. The designation cannot be blank. Returns the updated DOE."
    )]
    async fn update_doe_supply(
        &self,
        Parameters(args): Parameters<UpdateSupplyArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_doe_supply") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_supply(
                        args.organization.as_deref(),
                        &args.doe_id,
                        &args.supply_id,
                        SupplyPatch {
                            title: args.title.as_deref(),
                            designation: args.designation.as_deref(),
                            dpgf_ref: args.dpgf_ref.as_deref(),
                            destination: args.destination.as_deref(),
                            supplier_name: args.supplier_name.as_deref(),
                            supplier_address: args.supplier_address.as_deref(),
                            supplier_phone: args.supplier_phone.as_deref(),
                            supplier_email: args.supplier_email.as_deref(),
                            maker: args.maker.as_deref(),
                            specs: args.specs.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Remove a supply from a DOE (the DOE itself is never deleted). Its photos and datasheets leave the DOE with it. Returns the updated DOE."
    )]
    async fn delete_doe_supply(
        &self,
        Parameters(args): Parameters<DeleteSupplyArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "delete_doe_supply") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .delete_supply(args.organization.as_deref(), &args.doe_id, &args.supply_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Add a section to a DOE: kind supplies, photos, plans or blank_pages, at a 1-based position (default: at the end). Settings maker_label, per_page and count only apply to their own kind. Returns the updated DOE with the new section id."
    )]
    async fn add_doe_section(
        &self,
        Parameters(args): Parameters<AddSectionArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "add_doe_section") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .add_section(
                        args.organization.as_deref(),
                        &args.doe_id,
                        NewSection {
                            title: &args.title,
                            kind: args.kind,
                            maker_label: args.maker_label,
                            per_page: args.per_page,
                            count: args.count,
                            position: args.position,
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a section of a DOE, designated by section_id or section_title: rename it, enable or disable it, or change its settings (maker_label for supplies, per_page for photos, count for blank_pages). A setting that does not match the section kind is refused. Returns the updated DOE."
    )]
    async fn update_doe_section(
        &self,
        Parameters(args): Parameters<UpdateSectionArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_doe_section") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_section(
                        args.organization.as_deref(),
                        &args.doe_id,
                        SectionRef {
                            id: args.section_id.as_deref(),
                            title: args.section_title.as_deref(),
                        },
                        SectionPatch {
                            title: args.title.as_deref(),
                            enabled: args.enabled,
                            maker_label: args.maker_label,
                            per_page: args.per_page,
                            count: args.count,
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Delete an empty section of a DOE by section_id. A section holding supplies, photos or plans is refused: remove its content first. Blank pages sections count as empty. Returns the updated DOE."
    )]
    async fn delete_doe_section(
        &self,
        Parameters(args): Parameters<DeleteSectionArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "delete_doe_section") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .delete_section(args.organization.as_deref(), &args.doe_id, &args.section_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Reorder the sections of a DOE. section_ids must list every section id of the DOE exactly once, in the new order (see get_doe). Returns the updated DOE."
    )]
    async fn reorder_doe_sections(
        &self,
        Parameters(args): Parameters<ReorderSectionsArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "reorder_doe_sections") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .reorder_sections(
                        args.organization.as_deref(),
                        &args.doe_id,
                        &args.section_ids,
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        meta = file_param_meta(),
        description = "Add a file to a DOE: target photo (photos section), plan (plans section), supply_photo or supply_datasheet (a supply, by supply_id), cover_photo (the cover picture, printed when the organization's style shows it). Attach the file through `file` when the client supports attachments, else give content_base64; exactly one of the two. The type is detected from the content: JPEG, PNG and WebP, plus PDF for plans and datasheets. Returns the updated DOE."
    )]
    async fn add_doe_file(
        &self,
        Parameters(args): Parameters<AddFileArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "add_doe_file") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let source = Source::pick(args.file.as_ref(), args.content_base64.as_deref())?;
                let data = s
                    .add_file(
                        args.organization.as_deref(),
                        &args.doe_id,
                        args.target,
                        FileRequest {
                            section_id: args.section_id.as_deref(),
                            section_title: args.section_title.as_deref(),
                            supply_id: args.supply_id.as_deref(),
                            caption: args.caption.as_deref(),
                        },
                        &self.files,
                        source,
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a file of a DOE by its storage key: change the caption (photos, plans, supply photos; an empty string clears it) and/or move a photo or plan to another section of the same kind with section_id. Datasheets have no caption. Returns the updated DOE."
    )]
    async fn update_doe_file(
        &self,
        Parameters(args): Parameters<UpdateFileArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_doe_file") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_file(
                        args.organization.as_deref(),
                        &args.doe_id,
                        &args.key,
                        args.caption.as_deref(),
                        args.section_id.as_deref(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Remove a file from a DOE by its storage key, wherever it is (photos, plans, supply photos, supply datasheets, cover photo). The stored object itself is kept. Returns the updated DOE."
    )]
    async fn delete_doe_file(
        &self,
        Parameters(args): Parameters<DeleteFileArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "delete_doe_file") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .delete_file(args.organization.as_deref(), &args.doe_id, &args.key)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the maintenance services (entretien) to schedule for an ISO week, for example 2026-W43: the ones due in that week, a window still open in it, and the ones overdue from before. Grouped by town, overdue first. Services already carried by an open task, finished or on-demand services, draft contracts and archived contracts are not listed. Each line has its contract_id, usable with get_maintenance_contract."
    )]
    async fn list_maintenance_due(
        &self,
        Parameters(args): Parameters<MaintenanceDueArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_maintenance_due") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let recap = s
                    .maintenance_due(args.organization.as_deref(), &args.week)
                    .await?;
                Ok(json_result(&recap))
            }
            .await,
        )
    }

    #[tool(
        description = "Get one maintenance contract (contrat d'entretien) by id: customer, site, season, status (DRAFT or ACTIVE) and its lines with their cadence."
    )]
    async fn get_maintenance_contract(
        &self,
        Parameters(args): Parameters<MaintenanceContractArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_maintenance_contract") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_scoped(
                        args.organization.as_deref(),
                        "maintenance/contracts",
                        &args.contract_id,
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a maintenance contract (contrat d'entretien) as a DRAFT from a paper or Excel sheet: one contract for one customer site and one season, with a line per service and its cadence (rolling every N days, windows of weeks, or on demand). A draft is never due and stays out of the weekly recap until a person reviews and activates it in Mestier; this tool cannot activate a contract. The customer and the site must already exist. Returns the draft."
    )]
    async fn create_maintenance_contract_draft(
        &self,
        Parameters(args): Parameters<CreateMaintenanceDraftArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_maintenance_contract_draft") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_maintenance_draft(args.organization.as_deref(), &args.draft)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Export a DOE as a PDF, returned as an embedded application/pdf resource. Refused above 10 MB."
    )]
    async fn export_doe_pdf(
        &self,
        Parameters(args): Parameters<DoeArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "export_doe_pdf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                match s
                    .export_pdf(args.organization.as_deref(), &args.doe_id)
                    .await?
                {
                    PdfOutcome::Pdf { bytes, uri } => {
                        let resource = ResourceContents::blob(STANDARD.encode(bytes), uri)
                            .with_mime_type("application/pdf");
                        Ok(CallToolResult::success(vec![ContentBlock::resource(
                            resource,
                        )]))
                    }
                    PdfOutcome::TooLarge => Err(Fault::new(
                        "PDF trop volumineux pour être transmis (plus de 10 Mo)",
                    )),
                }
            }
            .await,
        )
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for MestierTools {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(
                "Mestier ERP tools. Call list_organizations first when unsure which organization to use.",
            )
    }
}
