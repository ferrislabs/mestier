use base64::{Engine, engine::general_purpose::STANDARD};
use http::request::Parts;
use mcp_tools::CLIENT_HEADER;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ResourceContents, ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    api::{ApiClient, Fault},
    session::{HeaderPatch, ManualSupply, Paging, PdfOutcome, Session, SupplyRequest},
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

#[derive(Clone)]
pub struct MestierTools {
    api: ApiClient,
    tool_router: ToolRouter<Self>,
}

impl MestierTools {
    pub fn new(api: ApiClient) -> Self {
        Self {
            api,
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
