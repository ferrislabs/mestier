use std::collections::HashSet;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::error::CollectionError;

pub const MAX_COLLECTION_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_COLLECTION_TITLE_LENGTH: usize = 200;
pub const MAX_COLLECTION_VERSION_LENGTH: usize = 100;
pub const MAX_COLLECTION_URL_LENGTH: usize = 2048;
pub const MAX_COLLECTION_FILE_NAME_LENGTH: usize = 255;
pub const MAX_OPERATION_ID_LENGTH: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
}

impl ParameterLocation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParameterLocation,
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub schema: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestBody {
    pub schema: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub method: HttpMethod,
    pub path: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub request_body: Option<RequestBody>,
    #[serde(default)]
    pub response: Option<Value>,
    #[serde(default)]
    pub response_status: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecurityKind {
    Http {
        scheme: String,
    },
    ApiKey {
        location: ApiKeyLocation,
        parameter_name: String,
    },
    Oauth2,
    OpenIdConnect,
    Other {
        declared_type: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityScheme {
    pub name: String,
    pub kind: SecurityKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollectionSource {
    Url { url: String },
    File { file_name: String },
}

impl CollectionSource {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Url { .. } => "url",
            Self::File { .. } => "file",
        }
    }

    pub fn value(&self) -> &str {
        match self {
            Self::Url { url } => url,
            Self::File { file_name } => file_name,
        }
    }

    pub fn from_parts(kind: &str, value: String) -> Option<Self> {
        match kind {
            "url" => Some(Self::Url { url: value }),
            "file" => Some(Self::File { file_name: value }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionDraftInput {
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub source: CollectionSource,
    pub security: Vec<SecurityScheme>,
    pub operations: Vec<Operation>,
    pub schemas: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionDraft {
    pub(crate) title: String,
    pub(crate) version: String,
    pub(crate) description: String,
    pub(crate) base_url: String,
    pub(crate) source: CollectionSource,
    pub(crate) security: Vec<SecurityScheme>,
    pub(crate) operations: Vec<Operation>,
    pub(crate) schemas: Map<String, Value>,
}

fn invalid(reason: impl Into<String>) -> CollectionError {
    CollectionError::InvalidModel(reason.into())
}

fn json_size(value: &impl Serialize) -> usize {
    serde_json::to_vec(value).map_or(0, |bytes| bytes.len())
}

fn model_size(input: &CollectionDraftInput) -> usize {
    input.description.len()
        + json_size(&input.security)
        + json_size(&input.operations)
        + json_size(&input.schemas)
}

fn check_text(
    label: &str,
    value: &str,
    limit: usize,
    allow_empty: bool,
) -> Result<(), CollectionError> {
    if !allow_empty && value.is_empty() {
        return Err(invalid(format!("the {label} cannot be empty")));
    }
    if value.chars().count() > limit {
        return Err(invalid(format!(
            "the {label} holds at most {limit} characters"
        )));
    }
    Ok(())
}

fn check_source(source: &CollectionSource) -> Result<(), CollectionError> {
    match source {
        CollectionSource::Url { url } => {
            check_text("source url", url.trim(), MAX_COLLECTION_URL_LENGTH, false)
        }
        CollectionSource::File { file_name } => check_text(
            "source file name",
            file_name.trim(),
            MAX_COLLECTION_FILE_NAME_LENGTH,
            false,
        ),
    }
}

fn check_operation(operation: &Operation) -> Result<(), CollectionError> {
    let id = operation.id.trim();
    if id.is_empty() {
        return Err(invalid("an operation id cannot be empty"));
    }
    if id.chars().count() > MAX_OPERATION_ID_LENGTH {
        return Err(invalid(format!(
            "operation id `{id}` holds more than {MAX_OPERATION_ID_LENGTH} characters"
        )));
    }
    if !operation.path.starts_with('/') {
        return Err(invalid(format!(
            "operation `{id}` has a path `{}` that does not start with `/`",
            operation.path
        )));
    }
    let mut seen: HashSet<(ParameterLocation, &str)> = HashSet::new();
    for parameter in &operation.parameters {
        if parameter.name.trim().is_empty() {
            return Err(invalid(format!(
                "operation `{id}` has a parameter with no name"
            )));
        }
        if !seen.insert((parameter.location, parameter.name.as_str())) {
            return Err(invalid(format!(
                "operation `{id}` declares the {} parameter `{}` twice",
                parameter.location.as_str(),
                parameter.name
            )));
        }
    }
    Ok(())
}

impl CollectionDraft {
    pub fn new(input: CollectionDraftInput) -> Result<Self, CollectionError> {
        let size = model_size(&input);
        if size > MAX_COLLECTION_BYTES {
            return Err(CollectionError::Oversized {
                size,
                limit: MAX_COLLECTION_BYTES,
            });
        }

        let title = input.title.trim().to_owned();
        check_text("title", &title, MAX_COLLECTION_TITLE_LENGTH, false)?;
        let version = input.version.trim().to_owned();
        check_text("version", &version, MAX_COLLECTION_VERSION_LENGTH, false)?;
        check_text("base url", &input.base_url, MAX_COLLECTION_URL_LENGTH, true)?;
        check_source(&input.source)?;

        if input.operations.is_empty() {
            return Err(invalid("a collection needs at least one operation"));
        }
        let mut operation_ids: HashSet<&str> = HashSet::new();
        for operation in &input.operations {
            check_operation(operation)?;
            if !operation_ids.insert(operation.id.as_str()) {
                return Err(invalid(format!(
                    "operation `{}` is declared more than once",
                    operation.id
                )));
            }
        }

        let mut scheme_names: HashSet<&str> = HashSet::new();
        for scheme in &input.security {
            if scheme.name.trim().is_empty() {
                return Err(invalid("a security scheme needs a name"));
            }
            if !scheme_names.insert(scheme.name.as_str()) {
                return Err(invalid(format!(
                    "security scheme `{}` is declared more than once",
                    scheme.name
                )));
            }
        }

        Ok(Self {
            title,
            version,
            description: input.description,
            base_url: input.base_url,
            source: input.source,
            security: input.security,
            operations: input.operations,
            schemas: input.schemas,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub slug: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub source: CollectionSource,
    pub imported_at: DateTime<Utc>,
    pub security: Vec<SecurityScheme>,
    pub operations: Vec<Operation>,
    pub schemas: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionVersion {
    pub version: String,
    pub imported_at: DateTime<Utc>,
    pub source: CollectionSource,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionDetail {
    pub collection: Collection,
    pub history: Vec<CollectionVersion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionSummary {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub slug: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub source: CollectionSource,
    pub imported_at: DateTime<Utc>,
    pub security: Vec<SecurityScheme>,
    pub operation_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportCollectionCommand {
    pub org_id: OrganizationId,
    pub draft: CollectionDraft,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplaceCollectionCommand {
    pub org_id: OrganizationId,
    pub id: Uuid,
    pub draft: CollectionDraft,
}

impl Collection {
    pub fn find_operation(&self, operation_id: &str) -> Option<&Operation> {
        self.operations
            .iter()
            .find(|operation| operation.id == operation_id)
    }

    pub fn from_draft(
        draft: CollectionDraft,
        id: Uuid,
        org_id: OrganizationId,
        slug: String,
        imported_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            org_id,
            slug,
            title: draft.title,
            version: draft.version,
            description: draft.description,
            base_url: draft.base_url,
            source: draft.source,
            imported_at,
            security: draft.security,
            operations: draft.operations,
            schemas: draft.schemas,
        }
    }

    pub fn replaced_by(&self, draft: CollectionDraft, imported_at: DateTime<Utc>) -> Self {
        Self::from_draft(draft, self.id, self.org_id, self.slug.clone(), imported_at)
    }

    pub fn version_entry(&self) -> CollectionVersion {
        CollectionVersion {
            version: self.version.clone(),
            imported_at: self.imported_at,
            source: self.source.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn operation(id: &str) -> Operation {
        Operation {
            id: id.to_owned(),
            method: HttpMethod::Get,
            path: format!("/{id}"),
            summary: String::new(),
            tag: String::new(),
            parameters: Vec::new(),
            request_body: None,
            response: None,
            response_status: None,
        }
    }

    fn input() -> CollectionDraftInput {
        CollectionDraftInput {
            title: "Annuaire".to_owned(),
            version: "1.0.0".to_owned(),
            description: String::new(),
            base_url: "https://api.example.com".to_owned(),
            source: CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned(),
            },
            security: Vec::new(),
            operations: vec![operation("list")],
            schemas: Map::new(),
        }
    }

    fn invalid_reason(input: CollectionDraftInput) -> String {
        match CollectionDraft::new(input) {
            Err(CollectionError::InvalidModel(reason)) => reason,
            other => panic!("expected an invalid model, got {other:?}"),
        }
    }

    #[test]
    fn a_complete_model_is_accepted_and_its_text_trimmed() {
        let mut candidate = input();
        candidate.title = "  Annuaire  ".to_owned();
        candidate.version = " 1.0.0 ".to_owned();

        let draft = CollectionDraft::new(candidate).unwrap();

        assert_eq!(draft.title, "Annuaire");
        assert_eq!(draft.version, "1.0.0");
        assert_eq!(draft.operations.len(), 1);
    }

    #[test]
    fn an_empty_or_blank_title_is_refused() {
        for title in ["", "   "] {
            let mut candidate = input();
            candidate.title = title.to_owned();
            assert!(invalid_reason(candidate).contains("title"));
        }
    }

    #[test]
    fn a_title_over_the_limit_is_refused() {
        let mut candidate = input();
        candidate.title = "t".repeat(MAX_COLLECTION_TITLE_LENGTH + 1);

        assert!(invalid_reason(candidate).contains("title"));
    }

    #[test]
    fn a_blank_version_is_refused() {
        let mut candidate = input();
        candidate.version = "  ".to_owned();

        assert!(invalid_reason(candidate).contains("version"));
    }

    #[test]
    fn a_base_url_over_the_limit_is_refused_but_an_empty_one_is_kept() {
        let mut candidate = input();
        candidate.base_url = "h".repeat(MAX_COLLECTION_URL_LENGTH + 1);
        assert!(invalid_reason(candidate).contains("base url"));

        let mut candidate = input();
        candidate.base_url = String::new();
        assert_eq!(CollectionDraft::new(candidate).unwrap().base_url, "");
    }

    #[test]
    fn a_source_without_a_value_is_refused() {
        let mut candidate = input();
        candidate.source = CollectionSource::File {
            file_name: " ".to_owned(),
        };

        assert!(invalid_reason(candidate).contains("source"));
    }

    #[test]
    fn a_model_without_operation_is_refused() {
        let mut candidate = input();
        candidate.operations = Vec::new();

        assert!(invalid_reason(candidate).contains("operation"));
    }

    #[test]
    fn two_operations_with_the_same_id_are_refused_and_the_id_is_named() {
        let mut candidate = input();
        candidate.operations = vec![operation("list"), operation("list")];

        let reason = invalid_reason(candidate);

        assert!(reason.contains("list"), "{reason}");
    }

    #[test]
    fn an_operation_id_that_is_blank_is_refused() {
        let mut candidate = input();
        candidate.operations = vec![operation(" ")];

        assert!(invalid_reason(candidate).contains("operation id"));
    }

    #[test]
    fn an_operation_path_must_start_with_a_slash() {
        let mut candidate = input();
        candidate.operations[0].path = "list".to_owned();

        let reason = invalid_reason(candidate);

        assert!(
            reason.contains("list") && reason.contains("path"),
            "{reason}"
        );
    }

    #[test]
    fn a_parameter_without_a_name_or_declared_twice_is_refused() {
        let parameter = |name: &str| Parameter {
            name: name.to_owned(),
            location: ParameterLocation::Query,
            required: false,
            description: String::new(),
            schema: json!({ "type": "string" }),
        };

        let mut blank = input();
        blank.operations[0].parameters = vec![parameter("")];
        assert!(invalid_reason(blank).contains("parameter"));

        let mut twice = input();
        twice.operations[0].parameters = vec![parameter("limit"), parameter("limit")];
        assert!(invalid_reason(twice).contains("limit"));
    }

    #[test]
    fn the_same_name_in_two_locations_is_not_a_duplicate() {
        let mut candidate = input();
        candidate.operations[0].parameters = vec![
            Parameter {
                name: "id".to_owned(),
                location: ParameterLocation::Path,
                required: true,
                description: String::new(),
                schema: Value::Null,
            },
            Parameter {
                name: "id".to_owned(),
                location: ParameterLocation::Query,
                required: false,
                description: String::new(),
                schema: Value::Null,
            },
        ];

        assert!(CollectionDraft::new(candidate).is_ok());
    }

    #[test]
    fn a_security_scheme_must_be_named_once() {
        let scheme = |name: &str| SecurityScheme {
            name: name.to_owned(),
            kind: SecurityKind::Oauth2,
        };

        let mut blank = input();
        blank.security = vec![scheme(" ")];
        assert!(invalid_reason(blank).contains("security"));

        let mut twice = input();
        twice.security = vec![scheme("oauth"), scheme("oauth")];
        assert!(invalid_reason(twice).contains("oauth"));
    }

    fn padded(bytes: usize) -> Map<String, Value> {
        let mut schemas = Map::new();
        schemas.insert(
            "Big".to_owned(),
            json!({ "description": "x".repeat(bytes) }),
        );
        schemas
    }

    #[test]
    fn a_model_over_two_mebibytes_is_oversized_and_says_by_how_much() {
        let mut candidate = input();
        candidate.schemas = padded(MAX_COLLECTION_BYTES);

        let error = CollectionDraft::new(candidate).unwrap_err();

        let CollectionError::Oversized { size, limit } = error else {
            panic!("expected Oversized, got {error:?}");
        };
        assert_eq!(limit, MAX_COLLECTION_BYTES);
        assert!(size > MAX_COLLECTION_BYTES);
    }

    #[test]
    fn a_model_just_under_two_mebibytes_is_accepted() {
        let mut candidate = input();
        candidate.schemas = padded(MAX_COLLECTION_BYTES - 1024);

        assert!(CollectionDraft::new(candidate).is_ok());
    }

    #[test]
    fn a_long_description_counts_toward_the_size() {
        let mut candidate = input();
        candidate.description = "d".repeat(MAX_COLLECTION_BYTES + 1);

        assert!(matches!(
            CollectionDraft::new(candidate),
            Err(CollectionError::Oversized { .. })
        ));
    }

    #[test]
    fn the_size_is_checked_before_the_structure() {
        let mut candidate = input();
        candidate.operations = Vec::new();
        candidate.schemas = padded(MAX_COLLECTION_BYTES);

        assert!(matches!(
            CollectionDraft::new(candidate),
            Err(CollectionError::Oversized { .. })
        ));
    }

    #[test]
    fn an_operation_serializes_in_snake_case_with_an_uppercase_method() {
        let mut op = operation("create");
        op.method = HttpMethod::Post;
        op.request_body = Some(RequestBody {
            schema: json!({ "type": "object" }),
        });
        op.response_status = Some("201".to_owned());

        let value = serde_json::to_value(&op).unwrap();

        assert_eq!(value["method"], "POST");
        assert_eq!(
            value["request_body"],
            json!({ "schema": { "type": "object" } })
        );
        assert_eq!(value["response_status"], "201");
        let back: Operation = serde_json::from_value(value).unwrap();
        assert_eq!(back, op);
    }

    #[test]
    fn a_security_scheme_serializes_with_a_typed_kind() {
        let bearer = SecurityScheme {
            name: "bearerAuth".to_owned(),
            kind: SecurityKind::Http {
                scheme: "bearer".to_owned(),
            },
        };
        let key = SecurityScheme {
            name: "apiKey".to_owned(),
            kind: SecurityKind::ApiKey {
                location: ApiKeyLocation::Header,
                parameter_name: "X-API-Key".to_owned(),
            },
        };

        assert_eq!(
            serde_json::to_value(&bearer).unwrap(),
            json!({ "name": "bearerAuth", "kind": { "type": "http", "scheme": "bearer" } })
        );
        assert_eq!(
            serde_json::to_value(&key).unwrap(),
            json!({
                "name": "apiKey",
                "kind": { "type": "api_key", "location": "header", "parameter_name": "X-API-Key" }
            })
        );
        assert_eq!(
            serde_json::to_value(SecurityKind::Oauth2).unwrap(),
            json!({ "type": "oauth2" })
        );
    }

    #[test]
    fn a_source_serializes_like_the_front_end_union() {
        assert_eq!(
            serde_json::to_value(CollectionSource::File {
                file_name: "spec.yaml".to_owned()
            })
            .unwrap(),
            json!({ "kind": "file", "file_name": "spec.yaml" })
        );
        assert_eq!(
            CollectionSource::from_parts("url", "https://x".to_owned()),
            Some(CollectionSource::Url {
                url: "https://x".to_owned()
            })
        );
        assert_eq!(CollectionSource::from_parts("ftp", "x".to_owned()), None);
    }

    #[test]
    fn replacing_keeps_the_identity_and_the_slug() {
        let draft = CollectionDraft::new(input()).unwrap();
        let id = Uuid::from_u128(7);
        let org = OrganizationId(Uuid::from_u128(1));
        let first_import = Utc::now();
        let collection =
            Collection::from_draft(draft, id, org, "annuaire".to_owned(), first_import);

        let mut next_input = input();
        next_input.title = "Annuaire v2".to_owned();
        next_input.version = "2.0.0".to_owned();
        let later = first_import + chrono::Duration::seconds(5);
        let replaced = collection.replaced_by(CollectionDraft::new(next_input).unwrap(), later);

        assert_eq!(replaced.id, id);
        assert_eq!(replaced.org_id, org);
        assert_eq!(replaced.slug, "annuaire");
        assert_eq!(replaced.title, "Annuaire v2");
        assert_eq!(replaced.imported_at, later);
        let entry = collection.version_entry();
        assert_eq!(entry.version, "1.0.0");
        assert_eq!(entry.imported_at, first_import);
    }

    #[test]
    fn an_operation_is_found_by_its_id() {
        let draft = CollectionDraft::new(input()).unwrap();
        let collection = Collection::from_draft(
            draft,
            Uuid::from_u128(1),
            OrganizationId(Uuid::from_u128(2)),
            "a".to_owned(),
            Utc::now(),
        );

        assert!(collection.find_operation("list").is_some());
        assert!(collection.find_operation("missing").is_none());
    }
}
