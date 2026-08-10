//! Shared plumbing for the three `odoo.*` connectors (#202): each is a
//! typed envelope over one Odoo external-API call — named fields, an
//! imposed `odoo_api` credential (`AuthRequirement::Exactly("odoo_api")`),
//! and translated errors. That envelope is the only difference from calling
//! the same endpoint through `http.request` by hand.
//!
//! Odoo's external API (`/jsonrpc`) needs two calls per action: `authenticate`
//! resolves the credential's username and API key to a numeric `uid`, and
//! `execute_kw` performs the actual model call under that `uid`. Both go
//! through this module so every action translates a rejected credential —
//! `common.authenticate` answering `false` — into the same message naming
//! which credential was wrong, rather than each connector reimplementing
//! the handshake and disagreeing about how.

mod create_invoice;
mod create_partner;
mod update_partner;

pub use create_invoice::OdooCreateInvoiceConnector;
pub use create_partner::OdooCreatePartnerConnector;
pub use update_partner::OdooUpdatePartnerConnector;

use common::OrganizationId;
use reqwest::{Client, Url};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::infrastructure::automation::webhook::address_policy::PrivateNetworkAccess;

use super::http_client::{
    DEFAULT_TIMEOUT, build_guarded_client, describe_reqwest_error, refuse_literal_private_address,
};

/// The `odoo_api` credential's fields, exactly as `validate_credential_data`
/// checked them against the scheme in `domain::automation::connector::scheme`
/// (`base_url`, `database`, `username`, `api_key`).
#[derive(Debug)]
struct OdooCredential {
    base_url: String,
    database: String,
    username: String,
    api_key: String,
    /// The credential's own name — carried through so a rejection can say
    /// *which* credential was wrong, not just that one was.
    name: String,
}

impl OdooCredential {
    fn from_fields(name: &str, fields: &Map<String, Value>) -> Result<Self, String> {
        let field = |field_name: &'static str| -> Result<String, String> {
            fields
                .get(field_name)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("the Odoo credential is missing field `{field_name}`"))
        };

        Ok(Self {
            base_url: field("base_url")?,
            database: field("database")?,
            username: field("username")?,
            api_key: field("api_key")?,
            name: name.to_owned(),
        })
    }
}

#[derive(Debug)]
enum OdooError {
    /// `common.authenticate` answered `false`: the username or API key is
    /// wrong. Named so an artisan juggling several Odoo connections knows
    /// which one to fix — a bare `HTTP 401` never would.
    RejectedCredential { name: String },
    /// The endpoint answered, but not with 2xx.
    Http { status: u16, body: String },
    /// A JSON-RPC `error` object came back on an otherwise-2xx response.
    Rpc { message: String },
    /// Never reached the endpoint at all — network, DNS, the guard, or a
    /// timeout.
    Network(String),
}

impl OdooError {
    fn describe(&self) -> String {
        match self {
            OdooError::RejectedCredential { name } => {
                format!("the Odoo credential `{name}` was rejected: invalid username or API key")
            }
            OdooError::Http { status, body } => {
                format!("Odoo answered {status} with body: {body}")
            }
            OdooError::Rpc { message } => format!("Odoo reported an error: {message}"),
            OdooError::Network(message) => message.clone(),
        }
    }
}

/// Shared by every `odoo.*` connector: the guarded client, and the
/// authenticate-then-execute call that turns a resolved model action into a
/// `Result` its caller renders as a `ConnectorOutcome`.
pub(super) struct OdooClient {
    usecase: MestierUseCase,
    client: Client,
    /// `None` only via the test seam [`Self::with_client`] — see
    /// `http_request::HttpRequestConnector` for why this mirrors that
    /// design exactly.
    access: Option<PrivateNetworkAccess>,
}

impl OdooClient {
    pub fn new(usecase: MestierUseCase, access: PrivateNetworkAccess) -> Self {
        let client = build_guarded_client(access).expect("building the guarded HTTP client failed");
        Self {
            usecase,
            client,
            access: Some(access),
        }
    }

    #[cfg(test)]
    fn with_client(usecase: MestierUseCase, client: Client) -> Self {
        Self {
            usecase,
            client,
            access: None,
        }
    }

    /// Opens the credential, authenticates, and calls `execute_kw` — the
    /// full round trip every `odoo.*` action needs. `args` is `execute_kw`'s
    /// own argument list (e.g. `[{"name": "Ada"}]` for a `create`).
    pub(super) async fn call(
        &self,
        org_id: OrganizationId,
        credential_id: Uuid,
        model: &str,
        method: &str,
        args: Value,
    ) -> Result<Value, String> {
        let (metadata, plaintext) = self
            .usecase
            .open_credential(org_id, credential_id)
            .await
            .map_err(|error| format!("the Odoo credential is unusable: {error}"))?;

        let fields = match serde_json::from_slice(&plaintext) {
            Ok(Value::Object(map)) => map,
            _ => return Err("the Odoo credential's data is not usable".to_owned()),
        };
        let credential = OdooCredential::from_fields(&metadata.name, &fields)?;

        if let Some(access) = self.access {
            let url = Url::parse(&credential.base_url)
                .map_err(|error| format!("the Odoo credential's base_url is invalid: {error}"))?;
            refuse_literal_private_address(&url, access)?;
        }

        let uid = authenticate(&self.client, &credential)
            .await
            .map_err(|error| error.describe())?;
        execute_kw(&self.client, &credential, uid, model, method, args)
            .await
            .map_err(|error| error.describe())
    }
}

async fn json_rpc_call(
    client: &Client,
    base_url: &str,
    service: &str,
    method: &str,
    args: Value,
) -> Result<Value, OdooError> {
    let url = format!("{}/jsonrpc", base_url.trim_end_matches('/'));
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "call",
        "params": { "service": service, "method": method, "args": args },
        "id": Uuid::new_v4().to_string(),
    });

    let response = client
        .post(&url)
        .timeout(DEFAULT_TIMEOUT)
        .json(&payload)
        .send()
        .await
        .map_err(|error| OdooError::Network(describe_reqwest_error(&error)))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(OdooError::Http {
            status: status.as_u16(),
            body,
        });
    }

    let body: Value = response.json().await.map_err(|error| {
        OdooError::Network(format!("Odoo's response was not valid JSON: {error}"))
    })?;

    if let Some(error) = body.get("error") {
        return Err(OdooError::Rpc {
            message: error.to_string(),
        });
    }

    Ok(body.get("result").cloned().unwrap_or(Value::Null))
}

async fn authenticate(client: &Client, credential: &OdooCredential) -> Result<i64, OdooError> {
    let result = json_rpc_call(
        client,
        &credential.base_url,
        "common",
        "authenticate",
        json!([
            credential.database,
            credential.username,
            credential.api_key,
            {}
        ]),
    )
    .await?;

    match result.as_i64() {
        Some(uid) if uid > 0 => Ok(uid),
        _ => Err(OdooError::RejectedCredential {
            name: credential.name.clone(),
        }),
    }
}

async fn execute_kw(
    client: &Client,
    credential: &OdooCredential,
    uid: i64,
    model: &str,
    method: &str,
    args: Value,
) -> Result<Value, OdooError> {
    json_rpc_call(
        client,
        &credential.base_url,
        "object",
        "execute_kw",
        json!([
            credential.database,
            uid,
            credential.api_key,
            model,
            method,
            args
        ]),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(pairs: &[(&str, &str)]) -> Map<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect()
    }

    #[test]
    fn a_complete_set_of_fields_is_accepted() {
        let credential = OdooCredential::from_fields(
            "Production Odoo",
            &fields(&[
                ("base_url", "https://odoo.example.com"),
                ("database", "prod"),
                ("username", "bot"),
                ("api_key", "secret"),
            ]),
        )
        .expect("all four fields are present");

        assert_eq!(credential.base_url, "https://odoo.example.com");
        assert_eq!(credential.name, "Production Odoo");
    }

    #[test]
    fn a_missing_field_is_refused_and_named() {
        let error = OdooCredential::from_fields(
            "Broken",
            &fields(&[
                ("base_url", "https://odoo.example.com"),
                ("database", "prod"),
                ("username", "bot"),
            ]),
        )
        .unwrap_err();

        assert!(error.contains("api_key"), "{error}");
    }

    #[test]
    fn a_rejected_credential_error_names_it() {
        let error = OdooError::RejectedCredential {
            name: "Production Odoo".to_owned(),
        };

        assert!(
            error.describe().contains("Production Odoo"),
            "{}",
            error.describe()
        );
        assert!(error.describe().contains("invalid"), "{}", error.describe());
    }

    #[test]
    fn an_http_error_carries_the_status_and_body() {
        let error = OdooError::Http {
            status: 500,
            body: "boom".to_owned(),
        };

        let described = error.describe();
        assert!(described.contains("500"), "{described}");
        assert!(described.contains("boom"), "{described}");
    }
}
