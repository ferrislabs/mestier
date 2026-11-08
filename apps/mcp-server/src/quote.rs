//! Quotes (devis).
//!
//! A quote update replaces the lines too, so an update that names no line
//! carries the current ones over. A quote past DRAFT is locked: the API
//! refuses the change and the refusal is shown as is.

use base64::{Engine, engine::general_purpose::STANDARD};
use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ResourceContents},
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::{
    api::Fault,
    session::PdfOutcome,
    tools::{MestierTools, finish, json_result},
};

const QUOTE_KEYS: [&str; 11] = [
    "title",
    "customer_id",
    "customer_context_id",
    "deal_id",
    "status",
    "valid_until",
    "deposit_percent",
    "owner_member_id",
    "notes",
    "billing_contact_id",
    "billing_address_id",
];
const LINE_KEYS: [&str; 11] = [
    "service_rate_id",
    "product_id",
    "label",
    "quantity",
    "unit",
    "unit_price_cents",
    "vat_rate_bp",
    "discount_percent",
    "billing_frequency",
    "notes",
    "photo_keys",
];

/// Statuses a person sets from outside the quote. `ACCEPTED` commits the
/// customer and `SUPERSEDED` is set by a revision: neither is offered.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettableStatus {
    Sent,
    Declined,
    Cancelled,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct LineArg {
    #[schemars(description = "What is sold, as printed on the quote.")]
    pub label: String,
    #[schemars(description = "Quantity as a decimal string, for example \"2.5\".")]
    pub quantity: String,
    #[schemars(
        description = "Unit code, for example \"h\", \"m2\", \"u\", or a custom unit of the organization."
    )]
    pub unit: String,
    #[schemars(description = "Unit price in cents, excluding tax.")]
    pub unit_price_cents: i32,
    #[schemars(
        description = "VAT rate in basis points, 2000 for 20 %. Defaults to the organization's rate."
    )]
    pub vat_rate_bp: Option<i32>,
    #[schemars(description = "Discount in percent as a decimal string, for example \"10\".")]
    pub discount_percent: Option<String>,
    #[schemars(description = "UUID of the catalogue product the line comes from.")]
    pub product_id: Option<String>,
    #[schemars(description = "UUID of the service rate the line comes from.")]
    pub service_rate_id: Option<String>,
    pub notes: Option<String>,
}

impl LineArg {
    fn body(&self) -> Value {
        let mut line = Map::new();
        line.insert("label".to_owned(), json!(self.label));
        line.insert("quantity".to_owned(), json!(self.quantity));
        line.insert("unit".to_owned(), json!(self.unit));
        line.insert("unit_price_cents".to_owned(), json!(self.unit_price_cents));
        line.insert("notes".to_owned(), json!(self.notes));
        line.insert("photo_keys".to_owned(), json!([]));
        line.insert("service_rate_id".to_owned(), json!(self.service_rate_id));
        if let Some(rate) = self.vat_rate_bp {
            line.insert("vat_rate_bp".to_owned(), json!(rate));
        }
        if let Some(discount) = &self.discount_percent {
            line.insert("discount_percent".to_owned(), json!(discount));
        }
        if let Some(product) = &self.product_id {
            line.insert("product_id".to_owned(), json!(product));
        }
        Value::Object(line)
    }
}

fn put<T: Serialize>(body: &mut Map<String, Value>, key: &str, item: &Option<T>) {
    if let Some(item) = item {
        body.insert(
            key.to_owned(),
            serde_json::to_value(item).unwrap_or(Value::Null),
        );
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateQuoteArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    pub title: String,
    #[schemars(description = "UUID of the customer, see list_customers.")]
    pub customer_id: String,
    #[schemars(
        description = "UUID of the customer's site or address the work is for, from get_customer."
    )]
    pub customer_context_id: String,
    #[schemars(description = "UUID of the deal the quote belongs to, see list_deals.")]
    pub deal_id: Option<String>,
    #[schemars(description = "Last day the quote is valid, YYYY-MM-DD.")]
    pub valid_until: Option<String>,
    #[schemars(description = "Deposit in percent as a decimal string, for example \"30\".")]
    pub deposit_percent: Option<String>,
    pub notes: Option<String>,
    #[schemars(description = "The quote lines. At least one.")]
    pub lines: Vec<LineArg>,
}

impl CreateQuoteArgs {
    fn body(&self) -> Result<Value, Fault> {
        if self.lines.is_empty() {
            return Err(Fault::new("un devis a besoin d'au moins une ligne"));
        }
        let mut body = Map::new();
        body.insert("title".to_owned(), json!(self.title));
        body.insert("customer_id".to_owned(), json!(self.customer_id));
        body.insert(
            "customer_context_id".to_owned(),
            json!(self.customer_context_id),
        );
        put(&mut body, "deal_id", &self.deal_id);
        put(&mut body, "valid_until", &self.valid_until);
        put(&mut body, "deposit_percent", &self.deposit_percent);
        put(&mut body, "notes", &self.notes);
        body.insert(
            "lines".to_owned(),
            Value::Array(self.lines.iter().map(LineArg::body).collect()),
        );
        Ok(Value::Object(body))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateQuoteArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the quote.")]
    pub quote_id: String,
    pub title: Option<String>,
    #[schemars(description = "UUID of the customer's site or address.")]
    pub customer_context_id: Option<String>,
    #[schemars(description = "UUID of the deal to link the quote to.")]
    pub deal_id: Option<String>,
    #[schemars(description = "Last day the quote is valid, YYYY-MM-DD.")]
    pub valid_until: Option<String>,
    #[schemars(description = "Deposit in percent as a decimal string.")]
    pub deposit_percent: Option<String>,
    pub notes: Option<String>,
    #[schemars(
        description = "The complete list of lines, replacing the current ones. Leave out to keep the lines as they are."
    )]
    pub lines: Option<Vec<LineArg>>,
}

impl UpdateQuoteArgs {
    fn overlay(&self, current: &Value) -> Result<Map<String, Value>, Fault> {
        let mut body = Map::new();
        put(&mut body, "title", &self.title);
        put(&mut body, "customer_context_id", &self.customer_context_id);
        put(&mut body, "deal_id", &self.deal_id);
        put(&mut body, "valid_until", &self.valid_until);
        put(&mut body, "deposit_percent", &self.deposit_percent);
        put(&mut body, "notes", &self.notes);
        let lines = match &self.lines {
            Some(lines) if lines.is_empty() => {
                return Err(Fault::new("un devis a besoin d'au moins une ligne"));
            }
            Some(lines) => lines.iter().map(LineArg::body).collect(),
            None => carried_lines(current),
        };
        body.insert("lines".to_owned(), Value::Array(lines));
        Ok(body)
    }
}

fn carried_lines(current: &Value) -> Vec<Value> {
    current["lines"]
        .as_array()
        .map(|lines| {
            lines
                .iter()
                .map(|line| {
                    let mut kept = Map::new();
                    for key in LINE_KEYS {
                        kept.insert(key.to_owned(), line[key].clone());
                    }
                    Value::Object(kept)
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct QuoteStatusArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the quote.")]
    pub quote_id: String,
    #[schemars(
        description = "SENT once the quote went to the customer, DECLINED if the customer refused, CANCELLED if it is withdrawn. Accepting a quote is done by a person in Mestier."
    )]
    pub status: SettableStatus,
    #[schemars(description = "Why the customer declined. Only used with DECLINED.")]
    pub decline_reason: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct QuoteIdArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the quote.")]
    pub quote_id: String,
}

#[tool_router(router = quote_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "Create a quote (devis) as a DRAFT for a customer, with its lines. Returns the quote with its reference and totals."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_quote"))]
    async fn create_quote(
        &self,
        Parameters(args): Parameters<CreateQuoteArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_quote") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = args.body()?;
                let data = s
                    .create_in_org(args.organization.as_deref(), "quotes", &body)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a DRAFT quote. Only the given fields change; the lines are kept unless `lines` is given, and then it replaces them all. A sent quote is locked and the update is refused. Returns the quote."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_quote"))]
    async fn update_quote(
        &self,
        Parameters(args): Parameters<UpdateQuoteArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_quote") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_owned_with(
                        args.organization.as_deref(),
                        "quotes",
                        &args.quote_id,
                        None,
                        &QUOTE_KEYS,
                        |current| args.overlay(current),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Set the status of a quote to SENT, DECLINED or CANCELLED. It cannot accept a quote: that stays a person's decision in Mestier. Returns the quote."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "set_quote_status"))]
    async fn set_quote_status(
        &self,
        Parameters(args): Parameters<QuoteStatusArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "set_quote_status") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let mut patch = Map::new();
                patch.insert("status".to_owned(), json!(args.status));
                put(&mut patch, "decline_reason", &args.decline_reason);
                let data = s
                    .patch_owned(
                        args.organization.as_deref(),
                        "quotes",
                        &args.quote_id,
                        "status",
                        &Value::Object(patch),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a new DRAFT revision of a sent or declined quote, to change it after the customer's feedback. The previous quote is marked superseded. Returns the revision."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_quote_revision"))]
    async fn create_quote_revision(
        &self,
        Parameters(args): Parameters<QuoteIdArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_quote_revision") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .post_owned(
                        args.organization.as_deref(),
                        "quotes",
                        &args.quote_id,
                        "revision",
                        &json!({}),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Export a quote as a PDF, returned as an embedded application/pdf resource. Refused above 10 MB, and when the organization's legal identity is incomplete."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "export_quote_pdf"))]
    async fn export_quote_pdf(
        &self,
        Parameters(args): Parameters<QuoteIdArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "export_quote_pdf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                match s
                    .export_owned_pdf(args.organization.as_deref(), "quotes", &args.quote_id)
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
