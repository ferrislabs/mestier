//! The DAF tools: the sheets of a project, their dossier header, a library
//! photo on a sheet and the PDF exports. Everything goes through the HTTP
//! API under the organization path, which is what scopes it. There is no
//! delete tool, and attachments are not editable here.

use base64::{Engine, engine::general_purpose::STANDARD};
use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, ResourceContents},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::{
    api::{Fault, MAX_PDF_BYTES},
    session::{PdfOutcome, Session, uuid},
    tools::{MestierTools, finish, json_result},
};

const STATUSES: [&str; 4] = ["pending", "accepted", "rejected_with_comments", "rejected"];

const UPDATE_KEYS: [&str; 16] = [
    "number",
    "designation",
    "price_ref",
    "supplier",
    "installer",
    "specs",
    "observations",
    "photo_key",
    "attachments",
    "project_owner",
    "project_manager",
    "status",
    "sent_on",
    "issued_on",
    "returned_on",
    "review_note",
];

#[derive(Debug, Default)]
pub struct DafPatch<'a> {
    pub number: Option<u16>,
    pub designation: Option<&'a str>,
    pub price_ref: Option<&'a str>,
    pub installer: Option<&'a str>,
    pub specs: Option<&'a str>,
    pub observations: Option<&'a str>,
    pub project_owner: Option<&'a str>,
    pub project_manager: Option<&'a str>,
    pub status: Option<&'a str>,
    pub sent_on: Option<&'a str>,
    pub issued_on: Option<&'a str>,
    pub returned_on: Option<&'a str>,
    pub review_note: Option<&'a str>,
    pub supplier_name: Option<&'a str>,
    pub supplier_address: Option<&'a str>,
    pub supplier_phone: Option<&'a str>,
    pub supplier_email: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct DossierPatch<'a> {
    pub title: Option<&'a str>,
    pub issuer: Option<&'a str>,
    pub project_owner: Option<&'a str>,
    pub project_manager: Option<&'a str>,
}

fn optional_text(value: &str) -> Value {
    if value.is_empty() {
        Value::Null
    } else {
        json!(value)
    }
}

fn required_text(field: &str, value: &str) -> Result<Value, Fault> {
    if value.trim().is_empty() {
        Err(Fault::new(format!("`{field}` ne peut pas être vide")))
    } else {
        Ok(json!(value))
    }
}

fn date_is_valid(raw: &str) -> bool {
    let mut parts = raw.split('-');
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let digits =
        |part: &str, len: usize| part.len() == len && part.bytes().all(|b| b.is_ascii_digit());
    if !(digits(year, 4) && digits(month, 2) && digits(day, 2)) {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        year.parse::<u32>(),
        month.parse::<u32>(),
        day.parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=last).contains(&day)
}

fn date(field: &str, value: &str, clearable: bool) -> Result<Value, Fault> {
    if value.is_empty() && clearable {
        return Ok(Value::Null);
    }
    if date_is_valid(value) {
        Ok(json!(value))
    } else {
        Err(Fault::new(format!(
            "`{field}` doit être une date AAAA-MM-JJ{}, reçu « {value} »",
            if clearable {
                " (ou une chaîne vide pour l'effacer)"
            } else {
                ""
            }
        )))
    }
}

fn supplier(current: &Value, patch: &DafPatch<'_>) -> Result<Option<Value>, Fault> {
    if patch.supplier_name.is_none()
        && patch.supplier_address.is_none()
        && patch.supplier_phone.is_none()
        && patch.supplier_email.is_none()
    {
        return Ok(None);
    }
    let mut next = match current {
        Value::Object(object) => object.clone(),
        _ => Map::new(),
    };
    if let Some(name) = patch.supplier_name {
        next.insert("name".into(), required_text("supplier_name", name)?);
    }
    if !next.get("name").is_some_and(Value::is_string) {
        return Err(Fault::new(
            "la DAF n'a pas encore de fournisseur : `supplier_name` est obligatoire pour en ajouter un",
        ));
    }
    for (key, value) in [
        ("address", patch.supplier_address),
        ("phone", patch.supplier_phone),
        ("email", patch.supplier_email),
    ] {
        if let Some(value) = value {
            next.insert(key.into(), optional_text(value));
        }
    }
    for key in ["address", "phone", "email"] {
        next.entry(key).or_insert(Value::Null);
    }
    Ok(Some(Value::Object(next)))
}

/// The PUT body of a DAF: the whole current sheet, with only the given
/// fields changed, and the `updated_at` that was read as the expected one.
pub fn update_body(daf: &Value, patch: &DafPatch<'_>) -> Result<Value, Fault> {
    let mut body = Map::new();
    for key in UPDATE_KEYS {
        body.insert(key.to_owned(), daf[key].clone());
    }
    body.insert("expected_updated_at".to_owned(), daf["updated_at"].clone());

    if let Some(number) = patch.number {
        if !(1..=999).contains(&number) {
            return Err(Fault::new("`number` doit être compris entre 1 et 999"));
        }
        body.insert("number".into(), json!(number));
    }
    if let Some(value) = patch.designation {
        body.insert("designation".into(), required_text("designation", value)?);
    }
    for (key, value) in [
        ("price_ref", patch.price_ref),
        ("installer", patch.installer),
        ("specs", patch.specs),
        ("observations", patch.observations),
        ("project_owner", patch.project_owner),
        ("project_manager", patch.project_manager),
        ("review_note", patch.review_note),
    ] {
        if let Some(value) = value {
            body.insert(key.into(), optional_text(value));
        }
    }
    if let Some(status) = patch.status {
        if !STATUSES.contains(&status) {
            return Err(Fault::new(format!(
                "statut inconnu « {status} » : {}",
                STATUSES.join(", ")
            )));
        }
        body.insert("status".into(), json!(status));
    }
    for (key, value, clearable) in [
        ("sent_on", patch.sent_on, true),
        ("returned_on", patch.returned_on, true),
        ("issued_on", patch.issued_on, false),
    ] {
        if let Some(value) = value {
            body.insert(key.into(), date(key, value, clearable)?);
        }
    }
    if let Some(next) = supplier(&daf["supplier"], patch)? {
        body.insert("supplier".into(), next);
    }
    Ok(Value::Object(body))
}

/// The PUT body of a dossier header, from the dossier that was read.
pub fn dossier_body(dossier: &Value, patch: &DossierPatch<'_>) -> Result<Value, Fault> {
    let mut body = Map::new();
    for key in ["title", "issuer", "project_owner", "project_manager"] {
        body.insert(key.to_owned(), dossier[key].clone());
    }
    for (key, value) in [("title", patch.title), ("issuer", patch.issuer)] {
        if let Some(value) = value {
            body.insert(key.into(), required_text(key, value)?);
        }
    }
    for (key, value) in [
        ("project_owner", patch.project_owner),
        ("project_manager", patch.project_manager),
    ] {
        if let Some(value) = value {
            body.insert(key.into(), optional_text(value));
        }
    }
    Ok(Value::Object(body))
}

impl Session<'_> {
    pub async fn list_dafs(
        &self,
        organization: Option<&str>,
        project_id: &str,
    ) -> Result<Value, Fault> {
        let project_id = uuid(project_id)?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/projects/{project_id}/dafs"),
                &[],
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn get_daf(&self, organization: Option<&str>, daf_id: &str) -> Result<Value, Fault> {
        let daf_id = uuid(daf_id)?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/dafs/{daf_id}"),
                &[],
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn create_daf(
        &self,
        organization: Option<&str>,
        project_id: &str,
        designation: Option<&str>,
        product_id: Option<&str>,
    ) -> Result<Value, Fault> {
        let project_id = uuid(project_id)?;
        let product_id = product_id.map(uuid).transpose()?;
        let designation = designation.map(str::trim).filter(|text| !text.is_empty());
        if designation.is_none() && product_id.is_none() {
            return Err(Fault::new(
                "`designation` ou `product_id` est obligatoire pour créer une DAF",
            ));
        }
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/projects/{project_id}/dafs"),
                &json!({"designation": designation, "product_id": product_id}),
            )
            .await?;
        Ok(body["data"].clone())
    }

    async fn update_daf_once(
        &self,
        org: &str,
        daf_id: &str,
        patch: &DafPatch<'_>,
    ) -> Result<Value, Fault> {
        let path = format!("organizations/{org}/dafs/{daf_id}");
        let current = self.api.get(self.token, &path, &[]).await?;
        let body = update_body(&current["data"], patch)?;
        let sent = self.api.put(self.token, &path, &body).await?;
        Ok(sent["data"].clone())
    }

    pub async fn update_daf(
        &self,
        organization: Option<&str>,
        daf_id: &str,
        patch: DafPatch<'_>,
    ) -> Result<Value, Fault> {
        let daf_id = uuid(daf_id)?;
        let org = self.organization_id(organization).await?;
        match self.update_daf_once(&org, &daf_id, &patch).await {
            Err(fault) if fault.is_stale_read() => {
                self.update_daf_once(&org, &daf_id, &patch).await
            }
            outcome => outcome,
        }
    }

    pub async fn update_daf_dossier(
        &self,
        organization: Option<&str>,
        project_id: &str,
        patch: DossierPatch<'_>,
    ) -> Result<Value, Fault> {
        let project_id = uuid(project_id)?;
        let org = self.organization_id(organization).await?;
        let current = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/projects/{project_id}/dafs"),
                &[],
            )
            .await?;
        let body = dossier_body(&current["data"]["dossier"], &patch)?;
        let sent = self
            .api
            .put(
                self.token,
                &format!("organizations/{org}/projects/{project_id}/daf-dossier"),
                &body,
            )
            .await?;
        Ok(sent["data"].clone())
    }

    pub async fn place_daf_photo(
        &self,
        organization: Option<&str>,
        daf_id: &str,
        photo_id: &str,
    ) -> Result<Value, Fault> {
        let daf_id = uuid(daf_id)?;
        let photo_id = uuid(photo_id)?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/dafs/{daf_id}/photo/place"),
                &json!({"photo_id": photo_id}),
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn export_daf_pdf(
        &self,
        organization: Option<&str>,
        daf_id: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<PdfOutcome, Fault> {
        let (path, name) = match (daf_id, project_id) {
            (Some(id), None) => {
                let id = uuid(id)?;
                (format!("dafs/{id}/pdf"), format!("dafs/{id}.pdf"))
            }
            (None, Some(id)) => {
                let id = uuid(id)?;
                (
                    format!("projects/{id}/dafs/pdf"),
                    format!("projects/{id}/dafs.pdf"),
                )
            }
            _ => {
                return Err(Fault::new(
                    "donnez exactement un des arguments `daf_id` ou `project_id`",
                ));
            }
        };
        let org = self.organization_id(organization).await?;
        let bytes = self
            .api
            .get_bytes(
                self.token,
                &format!("organizations/{org}/{path}"),
                MAX_PDF_BYTES,
            )
            .await?;
        Ok(match bytes {
            Some(bytes) => PdfOutcome::Pdf {
                bytes,
                uri: format!("mestier://organizations/{org}/{name}"),
            },
            None => PdfOutcome::TooLarge,
        })
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListDafsArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the project (see list_projects).")]
    pub project_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct GetDafArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DAF (see list_dafs).")]
    pub daf_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateDafArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the project that gets the DAF.")]
    pub project_id: String,
    #[schemars(
        description = "Designation of the supplied item, at most 200 characters. Required unless product_id is given, in which case the product name is the default."
    )]
    pub designation: Option<String>,
    #[schemars(
        description = "UUID of a catalogue product (see search_products). Its supplier, specs, first photo and datasheets are copied into the DAF as a snapshot."
    )]
    pub product_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateDafArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DAF (see list_dafs).")]
    pub daf_id: String,
    #[schemars(
        description = "Number of the DAF, 1 to 999, unique in the project. A number already taken by another DAF is refused."
    )]
    pub number: Option<u16>,
    #[schemars(description = "Designation, at most 200 characters. Cannot be blank.")]
    pub designation: Option<String>,
    #[schemars(description = "Price reference (N° PRIX). An empty string clears it.")]
    pub price_ref: Option<String>,
    #[schemars(description = "Installer (poseur). An empty string clears it.")]
    pub installer: Option<String>,
    #[schemars(description = "Characteristics of the item. An empty string clears it.")]
    pub specs: Option<String>,
    #[schemars(
        description = "Free OBSERVATIONS block of the sheet (destination, reference, warranties). An empty string clears it."
    )]
    pub observations: Option<String>,
    #[schemars(
        description = "Project owner (maitrise d'ouvrage) printed on this DAF only. An empty string clears it, which prints the dossier's."
    )]
    pub project_owner: Option<String>,
    #[schemars(
        description = "Project manager (maitrise d'oeuvre) printed on this DAF only. An empty string clears it, which prints the dossier's."
    )]
    pub project_manager: Option<String>,
    #[schemars(
        description = "Answer of the project manager: pending, accepted, rejected_with_comments or rejected."
    )]
    pub status: Option<String>,
    #[schemars(description = "Date the DAF was sent, YYYY-MM-DD. An empty string clears it.")]
    pub sent_on: Option<String>,
    #[schemars(description = "Issue date, YYYY-MM-DD. Cannot be cleared.")]
    pub issued_on: Option<String>,
    #[schemars(description = "Date the answer came back, YYYY-MM-DD. An empty string clears it.")]
    pub returned_on: Option<String>,
    #[schemars(
        description = "Remark of the project manager, printed in the Observation column of the summary. An empty string clears it."
    )]
    pub review_note: Option<String>,
    #[schemars(
        description = "Supplier name. Cannot be blank. Required to add a supplier the DAF does not have yet."
    )]
    pub supplier_name: Option<String>,
    #[schemars(description = "Supplier address. An empty string clears it.")]
    pub supplier_address: Option<String>,
    #[schemars(description = "Supplier phone. An empty string clears it.")]
    pub supplier_phone: Option<String>,
    #[schemars(description = "Supplier e-mail. An empty string clears it.")]
    pub supplier_email: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateDafDossierArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the project whose DAF dossier is changed.")]
    pub project_id: String,
    #[schemars(description = "Title of the dossier. Cannot be blank.")]
    pub title: Option<String>,
    #[schemars(description = "Issuer (emetteur). Cannot be blank.")]
    pub issuer: Option<String>,
    #[schemars(
        description = "Project owner (maitrise d'ouvrage) printed on every DAF that has none of its own. An empty string clears it."
    )]
    pub project_owner: Option<String>,
    #[schemars(
        description = "Project manager (maitrise d'oeuvre) printed on every DAF that has none of its own. An empty string clears it."
    )]
    pub project_manager: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct PlaceDafPhotoArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the DAF (see list_dafs).")]
    pub daf_id: String,
    #[schemars(
        description = "UUID of a photo of the organization's photo library (see search_photo_library)."
    )]
    pub photo_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ExportDafPdfArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of one DAF. Give this or project_id, not both.")]
    pub daf_id: Option<String>,
    #[schemars(
        description = "UUID of a project, to export its whole DAF dossier. Give this or daf_id, not both."
    )]
    pub project_id: Option<String>,
}

#[tool_router(router = daf_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "List the DAF (demandes d'agrement de fourniture) of a project: returns {dossier, dafs}. The dossier is the common header (title, issuer, project owner, project manager). dafs is the summary by number (id, number, designation, status, dates, review note); ids are usable with get_daf."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_dafs"))]
    async fn list_dafs(
        &self,
        Parameters(args): Parameters<ListDafsArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_dafs") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .list_dafs(args.organization.as_deref(), &args.project_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Get the full content of a DAF: designation, supplier, specs, observations, photo, attachments, parties, status and follow-up dates."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_daf"))]
    async fn get_daf(
        &self,
        Parameters(args): Parameters<GetDafArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_daf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .get_daf(args.organization.as_deref(), &args.daf_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Create a DAF in a project, with the next free number. Give a designation, or a catalogue product_id to copy its supplier, specs, first photo and datasheets (a snapshot: later product edits do not change the DAF). A project has at most 200 DAF. Returns the new DAF."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_daf"))]
    async fn create_daf(
        &self,
        Parameters(args): Parameters<CreateDafArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_daf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .create_daf(
                        args.organization.as_deref(),
                        &args.project_id,
                        args.designation.as_deref(),
                        args.product_id.as_deref(),
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update a DAF. Only the given fields change; an empty string clears an optional text field (price_ref, installer, specs, observations, project_owner, project_manager, review_note, supplier_address, supplier_phone, supplier_email) and the optional dates sent_on and returned_on. Dates are YYYY-MM-DD. status is pending, accepted, rejected_with_comments or rejected. supplier_* change the supplier snapshot of the DAF (supplier_name is required to add one). Attachments and the photo are not editable here (use place_daf_photo for the photo), and a DAF cannot be deleted through MCP. A concurrent edit is refused. Returns the updated DAF."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_daf"))]
    async fn update_daf(
        &self,
        Parameters(args): Parameters<UpdateDafArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_daf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_daf(
                        args.organization.as_deref(),
                        &args.daf_id,
                        DafPatch {
                            number: args.number,
                            designation: args.designation.as_deref(),
                            price_ref: args.price_ref.as_deref(),
                            installer: args.installer.as_deref(),
                            specs: args.specs.as_deref(),
                            observations: args.observations.as_deref(),
                            project_owner: args.project_owner.as_deref(),
                            project_manager: args.project_manager.as_deref(),
                            status: args.status.as_deref(),
                            sent_on: args.sent_on.as_deref(),
                            issued_on: args.issued_on.as_deref(),
                            returned_on: args.returned_on.as_deref(),
                            review_note: args.review_note.as_deref(),
                            supplier_name: args.supplier_name.as_deref(),
                            supplier_address: args.supplier_address.as_deref(),
                            supplier_phone: args.supplier_phone.as_deref(),
                            supplier_email: args.supplier_email.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Update the DAF dossier of a project, the header shared by its DAF (title, issuer, project owner, project manager). Only the given fields change; an empty string clears project_owner or project_manager. Returns the updated dossier."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_daf_dossier"))]
    async fn update_daf_dossier(
        &self,
        Parameters(args): Parameters<UpdateDafDossierArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_daf_dossier") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .update_daf_dossier(
                        args.organization.as_deref(),
                        &args.project_id,
                        DossierPatch {
                            title: args.title.as_deref(),
                            issuer: args.issuer.as_deref(),
                            project_owner: args.project_owner.as_deref(),
                            project_manager: args.project_manager.as_deref(),
                        },
                    )
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Place a photo of the photo library (see search_photo_library) as the image of a DAF, replacing any previous one. The photo is copied: deleting it from the library later keeps the DAF's image. Returns the updated DAF."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "place_daf_photo"))]
    async fn place_daf_photo(
        &self,
        Parameters(args): Parameters<PlaceDafPhotoArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "place_daf_photo") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                let data = s
                    .place_daf_photo(args.organization.as_deref(), &args.daf_id, &args.photo_id)
                    .await?;
                Ok(json_result(&data))
            }
            .await,
        )
    }

    #[tool(
        description = "Export DAF as a PDF, returned as an embedded application/pdf resource: one DAF (daf_id) or the whole dossier of a project with its summary (project_id). Give exactly one of the two. Refused above 10 MB."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "export_daf_pdf"))]
    async fn export_daf_pdf(
        &self,
        Parameters(args): Parameters<ExportDafPdfArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "export_daf_pdf") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(
            async {
                match s
                    .export_daf_pdf(
                        args.organization.as_deref(),
                        args.daf_id.as_deref(),
                        args.project_id.as_deref(),
                    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn daf() -> Value {
        json!({
            "id": "00000000-0000-4000-8000-000000000001",
            "project_id": "00000000-0000-4000-8000-000000000002",
            "number": 3,
            "designation": "Ganivelle",
            "price_ref": "P12",
            "source_product_id": null,
            "supplier": {"name": "Acme", "address": "1 rue", "phone": null, "email": null},
            "installer": "Poseur",
            "specs": "Chataignier",
            "observations": null,
            "photo_key": "doe/a.png",
            "attachments": [{"key": "doe/f.pdf", "name": "fiche.pdf"}],
            "project_owner": "MOA",
            "project_manager": null,
            "status": "pending",
            "sent_on": "2026-11-01",
            "issued_on": "2026-10-30",
            "returned_on": null,
            "review_note": "a revoir",
            "created_at": "2026-10-30T08:00:00Z",
            "updated_at": "2026-11-02T09:00:00Z"
        })
    }

    #[test]
    fn an_empty_patch_resends_the_sheet_with_the_read_updated_at() {
        let body = update_body(&daf(), &DafPatch::default()).unwrap();
        let mut expected = daf();
        let object = expected.as_object_mut().unwrap();
        for key in ["id", "project_id", "source_product_id", "created_at"] {
            object.remove(key);
        }
        let updated_at = object.remove("updated_at").unwrap();
        object.insert("expected_updated_at".into(), updated_at);
        assert_eq!(body, expected);
    }

    #[test]
    fn only_the_given_fields_change() {
        let patch = DafPatch {
            designation: Some("Ganivelle 2 m"),
            status: Some("accepted"),
            returned_on: Some("2026-11-10"),
            ..DafPatch::default()
        };
        let body = update_body(&daf(), &patch).unwrap();
        let mut expected = update_body(&daf(), &DafPatch::default()).unwrap();
        expected["designation"] = json!("Ganivelle 2 m");
        expected["status"] = json!("accepted");
        expected["returned_on"] = json!("2026-11-10");
        assert_eq!(body, expected);
    }

    #[test]
    fn an_empty_string_clears_optional_texts_and_dates() {
        let patch = DafPatch {
            price_ref: Some(""),
            project_owner: Some(""),
            review_note: Some(""),
            sent_on: Some(""),
            ..DafPatch::default()
        };
        let body = update_body(&daf(), &patch).unwrap();
        assert_eq!(body["price_ref"], Value::Null);
        assert_eq!(body["project_owner"], Value::Null);
        assert_eq!(body["review_note"], Value::Null);
        assert_eq!(body["sent_on"], Value::Null);
        assert_eq!(body["installer"], "Poseur");
    }

    #[test]
    fn an_invalid_status_is_refused() {
        let patch = DafPatch {
            status: Some("approved"),
            ..DafPatch::default()
        };
        assert!(update_body(&daf(), &patch).is_err());
    }

    #[test]
    fn an_invalid_date_is_refused() {
        for bad in [
            "02/11/2026",
            "2026-13-01",
            "2026-02-30",
            "2026-1-1",
            "demain",
        ] {
            let patch = DafPatch {
                sent_on: Some(bad),
                ..DafPatch::default()
            };
            assert!(update_body(&daf(), &patch).is_err(), "{bad}");
        }
        let leap = DafPatch {
            returned_on: Some("2028-02-29"),
            ..DafPatch::default()
        };
        assert!(update_body(&daf(), &leap).is_ok());
    }

    #[test]
    fn the_issue_date_and_the_designation_cannot_be_cleared() {
        let no_date = DafPatch {
            issued_on: Some(""),
            ..DafPatch::default()
        };
        assert!(update_body(&daf(), &no_date).is_err());
        let no_name = DafPatch {
            designation: Some("  "),
            ..DafPatch::default()
        };
        assert!(update_body(&daf(), &no_name).is_err());
    }

    #[test]
    fn a_number_outside_one_to_999_is_refused() {
        for bad in [0, 1000] {
            let patch = DafPatch {
                number: Some(bad),
                ..DafPatch::default()
            };
            assert!(update_body(&daf(), &patch).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_supplier_snapshot_is_patched_field_by_field() {
        let patch = DafPatch {
            supplier_address: Some(""),
            supplier_phone: Some("0102030405"),
            ..DafPatch::default()
        };
        let body = update_body(&daf(), &patch).unwrap();
        assert_eq!(
            body["supplier"],
            json!({"name": "Acme", "address": null, "phone": "0102030405", "email": null})
        );
    }

    #[test]
    fn a_supplier_needs_a_name_to_be_added() {
        let mut bare = daf();
        bare["supplier"] = Value::Null;
        let phone_only = DafPatch {
            supplier_phone: Some("0102030405"),
            ..DafPatch::default()
        };
        assert!(update_body(&bare, &phone_only).is_err());
        let named = DafPatch {
            supplier_name: Some("Nouveau"),
            ..DafPatch::default()
        };
        let body = update_body(&bare, &named).unwrap();
        assert_eq!(
            body["supplier"],
            json!({"name": "Nouveau", "address": null, "phone": null, "email": null})
        );
        let blank = DafPatch {
            supplier_name: Some(""),
            ..DafPatch::default()
        };
        assert!(update_body(&daf(), &blank).is_err());
    }

    #[test]
    fn the_dossier_patch_changes_only_the_given_fields() {
        let dossier = json!({
            "project_id": "x", "title": "DAF", "issuer": "Treyve",
            "project_owner": "MOA", "project_manager": "MOE", "updated_at": null
        });
        let patch = DossierPatch {
            title: Some("Recap"),
            project_manager: Some(""),
            ..DossierPatch::default()
        };
        assert_eq!(
            dossier_body(&dossier, &patch).unwrap(),
            json!({"title": "Recap", "issuer": "Treyve", "project_owner": "MOA", "project_manager": null})
        );
        let blank = DossierPatch {
            issuer: Some(" "),
            ..DossierPatch::default()
        };
        assert!(dossier_body(&dossier, &blank).is_err());
    }
}
