use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::{
    api::{ApiClient, Fault, MAX_PDF_BYTES},
    edit::{self, FilePlacement, NewSection, SectionKind, SectionPatch, SectionRef, SupplyPatch},
    files::{FileFetcher, Source, Target},
    maintenance::ContractDraft,
};

const PRODUCT_PAGE_SIZE: u64 = 100;
const PRODUCT_PAGE_CAP: u64 = 50;
const MAX_SEARCH_RESULTS: usize = 50;

#[derive(Debug, Clone, Copy, Default)]
pub struct Paging {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

impl Paging {
    fn query(self) -> Vec<(&'static str, String)> {
        let mut query = Vec::new();
        if let Some(page) = self.page {
            query.push(("page", page.to_string()));
        }
        if let Some(per_page) = self.per_page {
            query.push(("per_page", per_page.to_string()));
        }
        query
    }
}

#[derive(Debug, Default)]
pub struct HeaderPatch<'a> {
    pub title: Option<&'a str>,
    pub place: Option<&'a str>,
    pub lot: Option<&'a str>,
    pub issued_on: Option<&'a str>,
    pub author: Option<&'a str>,
    pub revision: Option<&'a str>,
}

#[derive(Debug, Default)]
pub struct SupplyRequest<'a> {
    pub doe_id: &'a str,
    pub section_id: Option<&'a str>,
    pub section_title: Option<&'a str>,
    pub product_id: &'a str,
    pub dpgf_ref: Option<&'a str>,
    pub destination: Option<&'a str>,
}

/// A supply typed in by the user rather than copied from the catalogue.
pub struct ManualSupply<'a> {
    pub doe_id: &'a str,
    pub section_id: Option<&'a str>,
    pub section_title: Option<&'a str>,
    pub designation: &'a str,
    pub title: Option<&'a str>,
    pub dpgf_ref: Option<&'a str>,
    pub destination: Option<&'a str>,
    pub supplier_name: Option<&'a str>,
    pub supplier_address: Option<&'a str>,
    pub supplier_phone: Option<&'a str>,
    pub supplier_email: Option<&'a str>,
    pub maker: Option<&'a str>,
    pub specs: Option<&'a str>,
}

pub enum PdfOutcome {
    Pdf { bytes: Vec<u8>, uri: String },
    TooLarge,
}

/// One tool call: the API client and the delegated token of the request that
/// carries it. Dropped when the call ends.
pub struct Session<'a> {
    api: &'a ApiClient,
    token: &'a str,
}

impl<'a> Session<'a> {
    pub fn new(api: &'a ApiClient, token: &'a str) -> Self {
        Self { api, token }
    }

    pub async fn organizations(&self) -> Result<Vec<Value>, Fault> {
        let body = self
            .api
            .get(self.token, "users/@me/organizations", &[])
            .await?;
        Ok(array(&body["data"]))
    }

    pub async fn list_organizations(&self) -> Result<Value, Fault> {
        let organizations = self.organizations().await?;
        let slim: Vec<Value> = organizations
            .iter()
            .map(|org| json!({"id": org["id"], "name": org["name"], "slug": org["slug"]}))
            .collect();
        Ok(Value::Array(slim))
    }

    pub async fn organization_id(&self, slug: Option<&str>) -> Result<String, Fault> {
        let organizations = self.organizations().await?;
        let slugs = || {
            organizations
                .iter()
                .filter_map(|org| org["slug"].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let chosen = match slug {
            Some(slug) => organizations
                .iter()
                .find(|org| org["slug"].as_str() == Some(slug))
                .ok_or_else(|| {
                    Fault::new(format!(
                        "organisation inconnue « {slug} ». Organisations disponibles : {}",
                        slugs()
                    ))
                })?,
            None => match organizations.as_slice() {
                [] => return Err(Fault::new("aucune organisation accessible")),
                [only] => only,
                _ => {
                    return Err(Fault::new(format!(
                        "plusieurs organisations : précisez l'argument `organization` parmi : {}",
                        slugs()
                    )));
                }
            },
        };
        chosen["id"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| Fault::new("organisation sans identifiant"))
    }

    pub async fn list(
        &self,
        organization: Option<&str>,
        collection: &str,
        paging: Paging,
    ) -> Result<Value, Fault> {
        let org = self.organization_id(organization).await?;
        self.api
            .get(
                self.token,
                &format!("organizations/{org}/{collection}"),
                &paging.query(),
            )
            .await
    }

    pub async fn get_scoped(
        &self,
        organization: Option<&str>,
        collection: &str,
        id: &str,
    ) -> Result<Value, Fault> {
        let id = uuid(id)?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/{collection}/{id}"),
                &[],
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn get_global(
        &self,
        organization: Option<&str>,
        collection: &str,
        id: &str,
    ) -> Result<Value, Fault> {
        let id = uuid(id)?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .get(self.token, &format!("{collection}/{id}"), &[])
            .await?;
        let data = body["data"].clone();
        match data["organization_id"].as_str() {
            Some(owner) if owner != org => {
                Err(Fault::new("ressource introuvable dans cette organisation"))
            }
            _ => Ok(data),
        }
    }

    pub async fn search_products(
        &self,
        organization: Option<&str>,
        query: &str,
    ) -> Result<Value, Fault> {
        let org = self.organization_id(organization).await?;
        let needle = query.to_lowercase();
        let mut found = Vec::new();
        for page in 1..=PRODUCT_PAGE_CAP {
            let body = self
                .api
                .get(
                    self.token,
                    &format!("organizations/{org}/products"),
                    &[
                        ("page", page.to_string()),
                        ("per_page", PRODUCT_PAGE_SIZE.to_string()),
                    ],
                )
                .await?;
            for product in array(&body["data"]) {
                let matches = ["name", "sku", "manufacturer"].iter().any(|field| {
                    product[*field]
                        .as_str()
                        .is_some_and(|text| text.to_lowercase().contains(&needle))
                });
                if matches {
                    found.push(product);
                }
            }
            if found.len() >= MAX_SEARCH_RESULTS || body["meta"]["next_page"].is_null() {
                break;
            }
        }
        found.truncate(MAX_SEARCH_RESULTS);
        Ok(Value::Array(found))
    }

    pub async fn maintenance_due(
        &self,
        organization: Option<&str>,
        week: &str,
    ) -> Result<Value, Fault> {
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/maintenance/recap"),
                &[("week", week.to_owned())],
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn create_maintenance_draft(
        &self,
        organization: Option<&str>,
        draft: &ContractDraft,
    ) -> Result<Value, Fault> {
        let request = draft.request_body()?;
        let org = self.organization_id(organization).await?;
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/maintenance/contracts"),
                &request,
            )
            .await?;
        Ok(body["data"].clone())
    }

    pub async fn create_doe(
        &self,
        organization: Option<&str>,
        project_id: &str,
        model_id: Option<&str>,
    ) -> Result<Value, Fault> {
        let project_id = uuid(project_id)?;
        let org = self.organization_id(organization).await?;
        let model_id = match model_id {
            Some(id) => uuid(id)?,
            None => self.only_model(&org).await?,
        };
        let body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/projects/{project_id}/doe"),
                &json!({"model_id": model_id}),
            )
            .await?;
        Ok(body["data"].clone())
    }

    async fn only_model(&self, org: &str) -> Result<String, Fault> {
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/doe-models"),
                &[("per_page", "100".to_owned())],
            )
            .await?;
        let models: Vec<Value> = array(&body["data"])
            .into_iter()
            .filter(|model| model["archived_at"].is_null())
            .collect();
        match models.as_slice() {
            [only] => only["id"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| Fault::new("modèle sans identifiant")),
            [] => Err(Fault::new("aucun modèle de DOE disponible")),
            _ => {
                let listed = models
                    .iter()
                    .map(|model| {
                        format!(
                            "{} ({})",
                            model["id"].as_str().unwrap_or("?"),
                            model["name"].as_str().unwrap_or("?")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                Err(Fault::new(format!(
                    "plusieurs modèles de DOE : précisez `model_id` parmi : {listed}"
                )))
            }
        }
    }

    async fn fetch_doe(&self, org: &str, doe_id: &str) -> Result<Value, Fault> {
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/does/{doe_id}"),
                &[],
            )
            .await?;
        Ok(body["data"].clone())
    }

    async fn store_doe(&self, org: &str, doe_id: &str, doe: &Value) -> Result<Value, Fault> {
        let body = self
            .api
            .put(
                self.token,
                &format!("organizations/{org}/does/{doe_id}"),
                &update_body(doe),
            )
            .await?;
        Ok(body["data"].clone())
    }

    async fn write_once<F>(&self, org: &str, doe_id: &str, modify: &F) -> Result<Value, Fault>
    where
        F: Fn(&mut Value) -> Result<(), Fault>,
    {
        let mut doe = self.fetch_doe(org, doe_id).await?;
        modify(&mut doe)?;
        self.store_doe(org, doe_id, &doe).await
    }

    async fn write_doe<F>(&self, org: &str, doe_id: &str, modify: F) -> Result<Value, Fault>
    where
        F: Fn(&mut Value) -> Result<(), Fault>,
    {
        match self.write_once(org, doe_id, &modify).await {
            Err(fault) if fault.is_stale_read() => self.write_once(org, doe_id, &modify).await,
            outcome => outcome,
        }
    }

    pub async fn add_supply(
        &self,
        organization: Option<&str>,
        request: SupplyRequest<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(request.doe_id)?;
        let product_id = uuid(request.product_id)?;
        let org = self.organization_id(organization).await?;
        let draft = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/does/{doe_id}/supplies/draft"),
                &json!({"product_id": product_id}),
            )
            .await?;
        let mut supply = draft["data"].clone();
        if let Some(object) = supply.as_object_mut() {
            if let Some(reference) = request.dpgf_ref {
                object.insert("dpgf_ref".into(), json!(reference));
            }
            if let Some(destination) = request.destination {
                object.insert("destination".into(), json!(destination));
            }
        }
        let section = SectionRef {
            id: request.section_id,
            title: request.section_title,
        };
        self.write_doe(&org, &doe_id, |doe| {
            push_supply(doe, section, supply.clone())
        })
        .await
    }

    pub async fn add_manual_supply(
        &self,
        organization: Option<&str>,
        request: ManualSupply<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(request.doe_id)?;
        if request.designation.trim().is_empty() {
            return Err(Fault::new(
                "la désignation de la fourniture est obligatoire",
            ));
        }
        let org = self.organization_id(organization).await?;
        let supplier = request.supplier_name.map(|name| {
            json!({
                "name": name,
                "address": request.supplier_address,
                "phone": request.supplier_phone,
                "email": request.supplier_email,
            })
        });
        let supply = json!({
            "id": Uuid::now_v7().to_string(),
            "source_product_id": null,
            "title": request.title,
            "designation": request.designation,
            "dpgf_ref": request.dpgf_ref,
            "destination": request.destination,
            "supplier": supplier,
            "maker": request.maker,
            "specs": request.specs,
            "photos": [],
            "datasheet_keys": [],
        });
        let section = SectionRef {
            id: request.section_id,
            title: request.section_title,
        };
        self.write_doe(&org, &doe_id, |doe| {
            push_supply(doe, section, supply.clone())
        })
        .await
    }

    pub async fn update_header(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        patch: HeaderPatch<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(doe_id)?;
        let org = self.organization_id(organization).await?;
        let fields = [
            ("title", patch.title, false),
            ("place", patch.place, true),
            ("lot", patch.lot, true),
            ("issued_on", patch.issued_on, false),
            ("author", patch.author, true),
            ("revision", patch.revision, false),
        ];
        self.write_doe(&org, &doe_id, |doe| {
            for (name, value, clearable) in fields {
                if let Some(value) = value {
                    doe[name] = if clearable && value.is_empty() {
                        Value::Null
                    } else {
                        json!(value)
                    };
                }
            }
            Ok(())
        })
        .await
    }

    pub async fn export_pdf(
        &self,
        organization: Option<&str>,
        doe_id: &str,
    ) -> Result<PdfOutcome, Fault> {
        let doe_id = uuid(doe_id)?;
        let org = self.organization_id(organization).await?;
        let bytes = self
            .api
            .get_bytes(
                self.token,
                &format!("organizations/{org}/does/{doe_id}/pdf"),
                MAX_PDF_BYTES,
            )
            .await?;
        Ok(match bytes {
            Some(bytes) => PdfOutcome::Pdf {
                bytes,
                uri: format!("mestier://organizations/{org}/does/{doe_id}.pdf"),
            },
            None => PdfOutcome::TooLarge,
        })
    }
}

#[derive(Debug, Default)]
pub struct FileRequest<'a> {
    pub section_id: Option<&'a str>,
    pub section_title: Option<&'a str>,
    pub supply_id: Option<&'a str>,
    pub caption: Option<&'a str>,
}

impl Session<'_> {
    async fn edit_doe<F>(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        modify: F,
    ) -> Result<Value, Fault>
    where
        F: Fn(&mut Value) -> Result<(), Fault>,
    {
        let doe_id = uuid(doe_id)?;
        let org = self.organization_id(organization).await?;
        self.write_doe(&org, &doe_id, modify).await
    }

    pub async fn update_supply(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        supply_id: &str,
        patch: SupplyPatch<'_>,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::update_supply(doe, supply_id, &patch)
        })
        .await
    }

    pub async fn delete_supply(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        supply_id: &str,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::delete_supply(doe, supply_id)
        })
        .await
    }

    pub async fn add_section(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        section: NewSection<'_>,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::add_section(doe, &section).map(|_| ())
        })
        .await
    }

    pub async fn update_section(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        section: SectionRef<'_>,
        patch: SectionPatch<'_>,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::update_section(doe, section, &patch)
        })
        .await
    }

    pub async fn delete_section(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        section_id: &str,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::delete_section(doe, section_id)
        })
        .await
    }

    pub async fn reorder_sections(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        section_ids: &[String],
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::reorder_sections(doe, section_ids)
        })
        .await
    }

    pub async fn update_file(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        key: &str,
        caption: Option<&str>,
        section_id: Option<&str>,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| {
            edit::update_file(doe, key, caption, section_id)
        })
        .await
    }

    pub async fn delete_file(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        key: &str,
    ) -> Result<Value, Fault> {
        self.edit_doe(organization, doe_id, |doe| edit::delete_file(doe, key))
            .await
    }

    pub async fn add_file(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        target: Target,
        request: FileRequest<'_>,
        fetcher: &FileFetcher,
        source: Source<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(doe_id)?;
        let org = self.organization_id(organization).await?;
        let placement = FilePlacement {
            target,
            section: SectionRef {
                id: request.section_id,
                title: request.section_title,
            },
            supply_id: request.supply_id,
            caption: request.caption,
        };
        let mut probe = self.fetch_doe(&org, &doe_id).await?;
        edit::add_file(&mut probe, &placement, "pending")?;
        let upload = fetcher.acquire(source, target).await?;
        let key = self
            .api
            .upload(self.token, "doe", upload.kind.mime(), upload.bytes)
            .await?;
        self.write_doe(&org, &doe_id, |doe| edit::add_file(doe, &placement, &key))
            .await
    }
}

fn array(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

fn uuid(raw: &str) -> Result<String, Fault> {
    Uuid::parse_str(raw)
        .map(|id| id.to_string())
        .map_err(|_| Fault::new(format!("identifiant invalide : « {raw} »")))
}

fn update_body(doe: &Value) -> Value {
    let mut body = Map::new();
    for key in [
        "title",
        "place",
        "lot",
        "issued_on",
        "author",
        "revision",
        "cover_photo_key",
        "parties",
        "sections",
        "updated_at",
    ] {
        body.insert(key.to_owned(), doe[key].clone());
    }
    if let Some(updated_at) = body.remove("updated_at") {
        body.insert("expected_updated_at".to_owned(), updated_at);
    }
    Value::Object(body)
}

fn push_supply(doe: &mut Value, section: SectionRef<'_>, supply: Value) -> Result<(), Fault> {
    let index = edit::pick_section(doe, section, SectionKind::Supplies)?;
    doe["sections"][index]["content"]["supplies"]
        .as_array_mut()
        .ok_or_else(|| Fault::new("section de fournitures illisible"))?
        .push(supply);
    Ok(())
}
