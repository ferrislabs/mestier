use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::api::{ApiClient, Fault, MAX_PDF_BYTES};

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
                    Fault(format!(
                        "organisation inconnue « {slug} ». Organisations disponibles : {}",
                        slugs()
                    ))
                })?,
            None => match organizations.as_slice() {
                [] => return Err(Fault::new("aucune organisation accessible")),
                [only] => only,
                _ => {
                    return Err(Fault(format!(
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
                Err(Fault(format!(
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

    pub async fn add_supply(
        &self,
        organization: Option<&str>,
        request: SupplyRequest<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(request.doe_id)?;
        let product_id = uuid(request.product_id)?;
        let org = self.organization_id(organization).await?;
        let mut doe = self.fetch_doe(&org, &doe_id).await?;
        let index = pick_section(&doe, request.section_id, request.section_title)?;
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
        doe["sections"][index]["content"]["supplies"]
            .as_array_mut()
            .ok_or_else(|| Fault::new("section de fournitures illisible"))?
            .push(supply);
        self.store_doe(&org, &doe_id, &doe).await
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
        let mut doe = self.fetch_doe(&org, &doe_id).await?;
        let index = pick_section(&doe, request.section_id, request.section_title)?;
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
        doe["sections"][index]["content"]["supplies"]
            .as_array_mut()
            .ok_or_else(|| Fault::new("section de fournitures illisible"))?
            .push(supply);
        self.store_doe(&org, &doe_id, &doe).await
    }

    pub async fn update_header(
        &self,
        organization: Option<&str>,
        doe_id: &str,
        patch: HeaderPatch<'_>,
    ) -> Result<Value, Fault> {
        let doe_id = uuid(doe_id)?;
        let org = self.organization_id(organization).await?;
        let mut doe = self.fetch_doe(&org, &doe_id).await?;
        let fields = [
            ("title", patch.title, false),
            ("place", patch.place, true),
            ("lot", patch.lot, true),
            ("issued_on", patch.issued_on, false),
            ("author", patch.author, true),
            ("revision", patch.revision, false),
        ];
        for (name, value, clearable) in fields {
            if let Some(value) = value {
                doe[name] = if clearable && value.is_empty() {
                    Value::Null
                } else {
                    json!(value)
                };
            }
        }
        self.store_doe(&org, &doe_id, &doe).await
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

fn array(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

fn uuid(raw: &str) -> Result<String, Fault> {
    Uuid::parse_str(raw)
        .map(|id| id.to_string())
        .map_err(|_| Fault(format!("identifiant invalide : « {raw} »")))
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
        "parties",
        "sections",
    ] {
        body.insert(key.to_owned(), doe[key].clone());
    }
    Value::Object(body)
}

fn pick_section(
    doe: &Value,
    section_id: Option<&str>,
    section_title: Option<&str>,
) -> Result<usize, Fault> {
    let sections = doe["sections"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    let supplies_sections: Vec<usize> = sections
        .iter()
        .enumerate()
        .filter(|(_, section)| section["content"]["kind"] == "supplies")
        .map(|(index, _)| index)
        .collect();
    let describe = || {
        supplies_sections
            .iter()
            .map(|&index| {
                format!(
                    "{} ({})",
                    sections[index]["title"].as_str().unwrap_or("?"),
                    sections[index]["id"].as_str().unwrap_or("?")
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let found = match (section_id, section_title) {
        (Some(id), _) => sections
            .iter()
            .position(|section| section["id"].as_str() == Some(id)),
        (None, Some(title)) => sections.iter().position(|section| {
            section["title"]
                .as_str()
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(title))
        }),
        (None, None) => {
            return match supplies_sections.as_slice() {
                [only] => Ok(*only),
                _ => Err(Fault(format!(
                    "précisez `section_id` ou `section_title` parmi les sections de fournitures : {}",
                    describe()
                ))),
            };
        }
    };
    let index = found.ok_or_else(|| {
        Fault(format!(
            "section introuvable. Sections de fournitures : {}",
            describe()
        ))
    })?;
    if !supplies_sections.contains(&index) {
        return Err(Fault(format!(
            "cette section n'est pas une section de fournitures. Sections de fournitures : {}",
            describe()
        )));
    }
    Ok(index)
}
