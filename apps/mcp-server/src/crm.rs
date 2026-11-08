//! Customers, their contacts and deals.
//!
//! The API replaces a whole resource on update, so every `update_*` tool
//! reads the current state, overlays the fields it was given and sends the
//! result. A field left out keeps its value; there is no way to clear one.

use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::{
    session::Paging,
    tools::{ListArgs, MestierTools, finish, json_result, paging},
};

const CUSTOMER_KEYS: [&str; 14] = [
    "status",
    "name",
    "registration_number",
    "phone",
    "email",
    "kind",
    "website",
    "vat_number",
    "owner_member_id",
    "lead_source",
    "notes",
    "payment_terms_days",
    "billing_contact_id",
    "billing_address_id",
];
const CONTACT_KEYS: [&str; 7] = [
    "first_name",
    "last_name",
    "role",
    "phone",
    "mobile",
    "email",
    "is_primary",
];
const DEAL_KEYS: [&str; 7] = [
    "customer_id",
    "title",
    "amount_cents",
    "expected_close_date",
    "owner_member_id",
    "source",
    "notes",
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerStatus {
    Prospect,
    Client,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerKind {
    Individual,
    Professional,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub enum LeadSource {
    #[serde(rename = "bouche_a_oreille")]
    WordOfMouth,
    #[serde(rename = "site_web")]
    Website,
    #[serde(rename = "recommandation")]
    Referral,
    #[serde(rename = "salon")]
    TradeShow,
    #[serde(rename = "architecte")]
    Architect,
    #[serde(rename = "autre")]
    Other,
}

/// Stages a person moves a deal to. `LOST` needs a reason.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DealStage {
    New,
    Visit,
    QuoteSent,
    Negotiation,
    Won,
    Lost,
}

fn value<T: serde::Serialize>(item: &T) -> Value {
    serde_json::to_value(item).unwrap_or(Value::Null)
}

fn put<T: serde::Serialize>(body: &mut Map<String, Value>, key: &str, item: &Option<T>) {
    if let Some(item) = item {
        body.insert(key.to_owned(), value(item));
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CustomerArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the customer.")]
    pub customer_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateCustomerArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Name of the customer: a company, or a person's full name.")]
    pub name: String,
    #[schemars(description = "PROSPECT until they buy, then CLIENT.")]
    pub status: CustomerStatus,
    #[schemars(description = "INDIVIDUAL (particulier) or PROFESSIONAL. Defaults to INDIVIDUAL.")]
    pub kind: Option<CustomerKind>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[schemars(description = "SIRET of a company.")]
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub website: Option<String>,
    pub lead_source: Option<LeadSource>,
    pub notes: Option<String>,
    #[schemars(description = "Payment terms in days.")]
    pub payment_terms_days: Option<i64>,
}

impl CreateCustomerArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("name".to_owned(), json!(self.name));
        body.insert("status".to_owned(), value(&self.status));
        put(&mut body, "kind", &self.kind);
        put(&mut body, "email", &self.email);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "registration_number", &self.registration_number);
        put(&mut body, "vat_number", &self.vat_number);
        put(&mut body, "website", &self.website);
        put(&mut body, "lead_source", &self.lead_source);
        put(&mut body, "notes", &self.notes);
        put(&mut body, "payment_terms_days", &self.payment_terms_days);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateCustomerArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the customer.")]
    pub customer_id: String,
    pub name: Option<String>,
    #[schemars(description = "PROSPECT or CLIENT. A customer cannot be archived from here.")]
    pub status: Option<CustomerStatus>,
    pub kind: Option<CustomerKind>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub website: Option<String>,
    pub lead_source: Option<LeadSource>,
    pub notes: Option<String>,
    pub payment_terms_days: Option<i64>,
}

impl UpdateCustomerArgs {
    fn patch(&self) -> Map<String, Value> {
        let mut body = Map::new();
        put(&mut body, "name", &self.name);
        put(&mut body, "status", &self.status);
        put(&mut body, "kind", &self.kind);
        put(&mut body, "email", &self.email);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "registration_number", &self.registration_number);
        put(&mut body, "vat_number", &self.vat_number);
        put(&mut body, "website", &self.website);
        put(&mut body, "lead_source", &self.lead_source);
        put(&mut body, "notes", &self.notes);
        put(&mut body, "payment_terms_days", &self.payment_terms_days);
        body
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateContactArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the customer the contact belongs to.")]
    pub customer_id: String,
    pub first_name: String,
    pub last_name: String,
    #[schemars(description = "Job title or role, for example \"Gérant\".")]
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    #[schemars(
        description = "Whether this is the main contact of the customer. Defaults to false."
    )]
    pub is_primary: Option<bool>,
}

impl CreateContactArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("first_name".to_owned(), json!(self.first_name));
        body.insert("last_name".to_owned(), json!(self.last_name));
        body.insert(
            "is_primary".to_owned(),
            json!(self.is_primary.unwrap_or(false)),
        );
        put(&mut body, "role", &self.role);
        put(&mut body, "email", &self.email);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "mobile", &self.mobile);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateContactArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the contact, from get_customer.")]
    pub contact_id: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
    pub is_primary: Option<bool>,
}

impl UpdateContactArgs {
    fn patch(&self) -> Map<String, Value> {
        let mut body = Map::new();
        put(&mut body, "first_name", &self.first_name);
        put(&mut body, "last_name", &self.last_name);
        put(&mut body, "role", &self.role);
        put(&mut body, "email", &self.email);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "mobile", &self.mobile);
        put(&mut body, "is_primary", &self.is_primary);
        body
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListDealsArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "Page number, starting at 1.")]
    pub page: Option<u64>,
    #[schemars(description = "Items per page, 1 to 100.")]
    pub per_page: Option<u64>,
    #[schemars(description = "Only the deals of this customer (UUID).")]
    pub customer_id: Option<String>,
    #[schemars(description = "Only the deals at this stage.")]
    pub stage: Option<DealStage>,
    #[schemars(description = "Text to look for in the title.")]
    pub query: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct DealArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the deal.")]
    pub deal_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateDealArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the customer the deal is for.")]
    pub customer_id: String,
    pub title: String,
    #[schemars(description = "Expected amount in cents, excluding tax. Defaults to 0.")]
    pub amount_cents: Option<i64>,
    #[schemars(description = "Expected close date, YYYY-MM-DD.")]
    pub expected_close_date: Option<String>,
    pub source: Option<LeadSource>,
    pub notes: Option<String>,
}

impl CreateDealArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("customer_id".to_owned(), json!(self.customer_id));
        body.insert("title".to_owned(), json!(self.title));
        put(&mut body, "amount_cents", &self.amount_cents);
        put(&mut body, "expected_close_date", &self.expected_close_date);
        put(&mut body, "source", &self.source);
        put(&mut body, "notes", &self.notes);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateDealArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the deal.")]
    pub deal_id: String,
    pub title: Option<String>,
    #[schemars(description = "Expected amount in cents, excluding tax.")]
    pub amount_cents: Option<i64>,
    #[schemars(description = "Expected close date, YYYY-MM-DD.")]
    pub expected_close_date: Option<String>,
    pub source: Option<LeadSource>,
    pub notes: Option<String>,
}

impl UpdateDealArgs {
    fn patch(&self) -> Map<String, Value> {
        let mut body = Map::new();
        put(&mut body, "title", &self.title);
        put(&mut body, "amount_cents", &self.amount_cents);
        put(&mut body, "expected_close_date", &self.expected_close_date);
        put(&mut body, "source", &self.source);
        put(&mut body, "notes", &self.notes);
        body
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct MoveDealArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the deal.")]
    pub deal_id: String,
    pub stage: DealStage,
    #[schemars(description = "Why the deal was lost. Required when the stage is LOST.")]
    pub lost_reason: Option<String>,
}

impl MoveDealArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("stage".to_owned(), value(&self.stage));
        put(&mut body, "lost_reason", &self.lost_reason);
        Value::Object(body)
    }
}

fn deal_query(args: &ListDealsArgs) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if let Some(customer) = &args.customer_id {
        query.push(("customer_id", customer.clone()));
    }
    if let Some(Value::String(stage)) = args.stage.map(|stage| value(&stage)) {
        query.push(("stage", stage));
    }
    if let Some(text) = &args.query {
        query.push(("q", text.clone()));
    }
    query
}

#[tool_router(router = crm_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "List the customers (clients et prospects) of an organization (paginated), with city, primary contact, revenue over 12 months and overdue amount. Each has an id usable with get_customer."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_customers"))]
    async fn list_customers(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_customers") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "customers", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one customer by id, with its contacts.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_customer"))]
    async fn get_customer(
        &self,
        Parameters(args): Parameters<CustomerArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_customer") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let org = args.organization.as_deref();
                let mut customer = s.get_global(org, "customers", &args.customer_id).await?;
                let contacts = s
                    .get_global_children(org, "customers", &args.customer_id, "contacts")
                    .await?;
                customer["contacts"] = contacts;
                Ok(json_result(&customer))
            }
            .await,
        )
    }

    #[tool(description = "Create a customer (client or prospect). Returns the customer.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_customer"))]
    async fn create_customer(
        &self,
        Parameters(args): Parameters<CreateCustomerArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_customer") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_in_org(args.organization.as_deref(), "customers", &args.body())
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a customer. Only the given fields change. A field cannot be cleared. Returns the customer."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_customer"))]
    async fn update_customer(
        &self,
        Parameters(args): Parameters<UpdateCustomerArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_customer") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_owned(
                        args.organization.as_deref(),
                        "customers",
                        &args.customer_id,
                        None,
                        &CUSTOMER_KEYS,
                        args.patch(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(description = "Add a contact person to a customer. Returns the contact.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_customer_contact"))]
    async fn create_customer_contact(
        &self,
        Parameters(args): Parameters<CreateContactArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_customer_contact") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .post_owned(
                        args.organization.as_deref(),
                        "customers",
                        &args.customer_id,
                        "contacts",
                        &args.body(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a contact of a customer. Only the given fields change. A field cannot be cleared. Returns the contact."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_customer_contact"))]
    async fn update_customer_contact(
        &self,
        Parameters(args): Parameters<UpdateContactArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_customer_contact") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_owned(
                        args.organization.as_deref(),
                        "customer-contacts",
                        &args.contact_id,
                        Some(("customers", "customer_id")),
                        &CONTACT_KEYS,
                        args.patch(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the deals (opportunités) of an organization (paginated), optionally for one customer, at one stage, or matching a text. Each has an id usable with get_deal."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_deals"))]
    async fn list_deals(
        &self,
        Parameters(args): Parameters<ListDealsArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_deals") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let page = Paging {
                    page: args.page,
                    per_page: args.per_page,
                };
                let body = s
                    .list_filtered(
                        args.organization.as_deref(),
                        "deals",
                        page,
                        deal_query(&args),
                    )
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Get one deal by id: customer, stage, amount and quote count.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_deal"))]
    async fn get_deal(
        &self,
        Parameters(args): Parameters<DealArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_deal") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_global(args.organization.as_deref(), "deals", &args.deal_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a deal (opportunité) for a customer, at the NEW stage. Returns the deal."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_deal"))]
    async fn create_deal(
        &self,
        Parameters(args): Parameters<CreateDealArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_deal") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_in_org(args.organization.as_deref(), "deals", &args.body())
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update the title, amount, close date, source or notes of a deal. Only the given fields change. Use move_deal_stage to change its stage. Returns the deal."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_deal"))]
    async fn update_deal(
        &self,
        Parameters(args): Parameters<UpdateDealArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_deal") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_owned(
                        args.organization.as_deref(),
                        "deals",
                        &args.deal_id,
                        None,
                        &DEAL_KEYS,
                        args.patch(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Move a deal to another stage of the pipeline: NEW, VISIT, QUOTE_SENT, NEGOTIATION, WON or LOST. LOST needs a lost_reason. Returns the deal."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "move_deal_stage"))]
    async fn move_deal_stage(
        &self,
        Parameters(args): Parameters<MoveDealArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "move_deal_stage") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .post_owned(
                        args.organization.as_deref(),
                        "deals",
                        &args.deal_id,
                        "stage",
                        &args.body(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }
}
