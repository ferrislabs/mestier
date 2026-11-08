//! The catalogue, suppliers, and the purchase, expense and reporting reads.
//!
//! Neither a product nor a supplier is archived from here: the product
//! update keeps whatever archive state the product has, and no tool
//! archives a supplier.

use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::tools::{ListArgs, MestierTools, finish, json_result, paging};

const PRODUCT_KEYS: [&str; 14] = [
    "name",
    "sku",
    "unit",
    "unit_price_cents",
    "default_vat_rate_bp",
    "description",
    "photo_keys",
    "supplier_id",
    "manufacturer",
    "technical_specs",
    "datasheet_keys",
    "category",
    "billing_frequency",
    "cost_cents",
];
const SUPPLIER_KEYS: [&str; 12] = [
    "name",
    "registration_number",
    "vat_number",
    "address",
    "phone",
    "email",
    "iban",
    "payment_terms_days",
    "default_category_id",
    "notes",
    "contacts",
    "addresses",
];

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
pub struct CreateProductArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    pub name: String,
    #[schemars(
        description = "Unit code, for example \"u\", \"m2\", \"h\", or a custom unit of the organization."
    )]
    pub unit: String,
    #[schemars(description = "Selling price per unit in cents, excluding tax.")]
    pub unit_price_cents: i32,
    pub sku: Option<String>,
    #[schemars(description = "Default VAT rate in basis points, 2000 for 20 %.")]
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub technical_specs: Option<String>,
    pub category: Option<String>,
    #[schemars(description = "Purchase cost per unit in cents.")]
    pub cost_cents: Option<i32>,
    #[schemars(description = "UUID of the supplier, see list_suppliers.")]
    pub supplier_id: Option<String>,
}

impl CreateProductArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("name".to_owned(), json!(self.name));
        body.insert("unit".to_owned(), json!(self.unit));
        body.insert("unit_price_cents".to_owned(), json!(self.unit_price_cents));
        body.insert("photo_keys".to_owned(), json!([]));
        put(&mut body, "sku", &self.sku);
        put(&mut body, "default_vat_rate_bp", &self.default_vat_rate_bp);
        put(&mut body, "description", &self.description);
        put(&mut body, "manufacturer", &self.manufacturer);
        put(&mut body, "technical_specs", &self.technical_specs);
        put(&mut body, "category", &self.category);
        put(&mut body, "cost_cents", &self.cost_cents);
        put(&mut body, "supplier_id", &self.supplier_id);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateProductArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the product.")]
    pub product_id: String,
    pub name: Option<String>,
    pub unit: Option<String>,
    #[schemars(description = "Selling price per unit in cents, excluding tax.")]
    pub unit_price_cents: Option<i32>,
    pub sku: Option<String>,
    #[schemars(description = "Default VAT rate in basis points.")]
    pub default_vat_rate_bp: Option<i32>,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub technical_specs: Option<String>,
    pub category: Option<String>,
    #[schemars(description = "Purchase cost per unit in cents.")]
    pub cost_cents: Option<i32>,
    #[schemars(description = "UUID of the supplier.")]
    pub supplier_id: Option<String>,
}

impl UpdateProductArgs {
    fn patch(&self, current: &Value) -> Map<String, Value> {
        let mut body = Map::new();
        put(&mut body, "name", &self.name);
        put(&mut body, "unit", &self.unit);
        put(&mut body, "unit_price_cents", &self.unit_price_cents);
        put(&mut body, "sku", &self.sku);
        put(&mut body, "default_vat_rate_bp", &self.default_vat_rate_bp);
        put(&mut body, "description", &self.description);
        put(&mut body, "manufacturer", &self.manufacturer);
        put(&mut body, "technical_specs", &self.technical_specs);
        put(&mut body, "category", &self.category);
        put(&mut body, "cost_cents", &self.cost_cents);
        put(&mut body, "supplier_id", &self.supplier_id);
        body.insert(
            "archived".to_owned(),
            json!(current["archived_at"].is_string()),
        );
        body
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateSupplierArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    pub name: String,
    #[schemars(description = "SIRET.")]
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub iban: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub notes: Option<String>,
}

impl CreateSupplierArgs {
    fn body(&self) -> Value {
        let mut body = Map::new();
        body.insert("name".to_owned(), json!(self.name));
        put(&mut body, "registration_number", &self.registration_number);
        put(&mut body, "vat_number", &self.vat_number);
        put(&mut body, "address", &self.address);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "email", &self.email);
        put(&mut body, "iban", &self.iban);
        put(&mut body, "payment_terms_days", &self.payment_terms_days);
        put(&mut body, "notes", &self.notes);
        Value::Object(body)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateSupplierArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the supplier, see list_suppliers.")]
    pub supplier_id: String,
    pub name: Option<String>,
    pub registration_number: Option<String>,
    pub vat_number: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub iban: Option<String>,
    pub payment_terms_days: Option<i32>,
    pub notes: Option<String>,
}

impl UpdateSupplierArgs {
    fn patch(&self) -> Map<String, Value> {
        let mut body = Map::new();
        put(&mut body, "name", &self.name);
        put(&mut body, "registration_number", &self.registration_number);
        put(&mut body, "vat_number", &self.vat_number);
        put(&mut body, "address", &self.address);
        put(&mut body, "phone", &self.phone);
        put(&mut body, "email", &self.email);
        put(&mut body, "iban", &self.iban);
        put(&mut body, "payment_terms_days", &self.payment_terms_days);
        put(&mut body, "notes", &self.notes);
        body
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct OrganizationArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct PeriodArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "First day of the period, YYYY-MM-DD.")]
    pub from: String,
    #[schemars(description = "Last day of the period, YYYY-MM-DD.")]
    pub to: String,
}

#[tool_router(router = reference_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "Create a product in the catalogue. Returns the product; its id is usable with add_doe_supply_from_product."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_product"))]
    async fn create_product(
        &self,
        Parameters(args): Parameters<CreateProductArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_product") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_in_org(args.organization.as_deref(), "products", &args.body())
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a catalogue product. Only the given fields change; a field cannot be cleared, and an archived product stays archived. Returns the product."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_product"))]
    async fn update_product(
        &self,
        Parameters(args): Parameters<UpdateProductArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_product") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_owned_with(
                        args.organization.as_deref(),
                        "products",
                        &args.product_id,
                        None,
                        &PRODUCT_KEYS,
                        |current| Ok(args.patch(current)),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the suppliers (fournisseurs) of an organization (paginated). Each has an id usable with update_supplier."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_suppliers"))]
    async fn list_suppliers(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_suppliers") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "suppliers", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "Create a supplier (fournisseur). Returns the supplier.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_supplier"))]
    async fn create_supplier(
        &self,
        Parameters(args): Parameters<CreateSupplierArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_supplier") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_in_org(args.organization.as_deref(), "suppliers", &args.body())
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a supplier. Only the given fields change and its contacts and addresses are kept; a field cannot be cleared. Returns the supplier."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_supplier"))]
    async fn update_supplier(
        &self,
        Parameters(args): Parameters<UpdateSupplierArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_supplier") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .replace_in_org(
                        args.organization.as_deref(),
                        "suppliers",
                        &args.supplier_id,
                        &SUPPLIER_KEYS,
                        args.patch(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "List the supplier invoices (factures fournisseur) of an organization (paginated)."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_supplier_invoices"))]
    async fn list_supplier_invoices(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_supplier_invoices") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(
                        args.organization.as_deref(),
                        "supplier-invoices",
                        paging(&args),
                    )
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(description = "List the expenses (dépenses) of an organization (paginated).")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_expenses"))]
    async fn list_expenses(
        &self,
        Parameters(args): Parameters<ListArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_expenses") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let body = s
                    .list(args.organization.as_deref(), "expenses", paging(&args))
                    .await?;
                Ok(json_result(&body))
            }
            .await,
        )
    }

    #[tool(
        description = "What each customer still owes across every issued invoice (impayés), for an organization."
    )]
    #[tracing::instrument(
        name = "mcp.tool",
        skip_all,
        fields(tool = "list_outstanding_invoices")
    )]
    async fn list_outstanding_invoices(
        &self,
        Parameters(args): Parameters<OrganizationArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_outstanding_invoices") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_in_org(args.organization.as_deref(), "invoices/outstanding", &[])
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Profitability (rentabilité) of an organization over a period. Money fields are left out when the user may not see costs."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_profitability"))]
    async fn get_profitability(
        &self,
        Parameters(args): Parameters<PeriodArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_profitability") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_in_org(
                        args.organization.as_deref(),
                        "reporting/profitability",
                        &[("from", args.from.clone()), ("to", args.to.clone())],
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(description = "Hours worked by the team of an organization over a period.")]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_worked_hours"))]
    async fn get_worked_hours(
        &self,
        Parameters(args): Parameters<PeriodArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_worked_hours") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_in_org(
                        args.organization.as_deref(),
                        "reporting/worked-hours",
                        &[("from", args.from.clone()), ("to", args.to.clone())],
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }
}
