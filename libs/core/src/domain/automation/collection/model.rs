use std::collections::HashSet;

use chrono::{DateTime, Utc};
use common::OrganizationId;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use super::address::{cleaned_source_url, normalized_base_url};
use super::error::CollectionError;
use super::operation::path_placeholders;

pub const MAX_COLLECTION_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_COLLECTION_TITLE_LENGTH: usize = 200;
pub const MAX_COLLECTION_VERSION_LENGTH: usize = 100;
pub const MAX_COLLECTION_URL_LENGTH: usize = 2048;
pub const MAX_COLLECTION_FILE_NAME_LENGTH: usize = 255;
pub const MAX_OPERATION_ID_LENGTH: usize = 200;
pub const MAX_COLLECTION_OPERATIONS: usize = 1000;
pub const MAX_OPERATION_PARAMETERS: usize = 100;
pub const MAX_OPERATION_PATH_LENGTH: usize = 2048;
pub const MAX_PATH_PLACEHOLDERS: usize = 32;
pub const MAX_COLLECTION_DESCRIPTION_LENGTH: usize = 10_000;
pub const MAX_COLLECTIONS_PER_ORGANIZATION: usize = 200;
pub const MAX_HISTORY_ENTRIES: usize = 50;
pub const TRANSPORT_HEADERS: [&str; 5] = [
    "host",
    "content-length",
    "transfer-encoding",
    "connection",
    "proxy-authorization",
];

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

fn check_source(source: CollectionSource) -> Result<CollectionSource, CollectionError> {
    match source {
        CollectionSource::Url { url } => {
            check_text("source url", url.trim(), MAX_COLLECTION_URL_LENGTH, false)?;
            Ok(CollectionSource::Url {
                url: cleaned_source_url(&url).map_err(invalid)?,
            })
        }
        CollectionSource::File { file_name } => {
            check_text(
                "source file name",
                file_name.trim(),
                MAX_COLLECTION_FILE_NAME_LENGTH,
                false,
            )?;
            Ok(CollectionSource::File { file_name })
        }
    }
}

fn check_transport_header(name: &str, owner: &str) -> Result<(), CollectionError> {
    if TRANSPORT_HEADERS.contains(&name.trim().to_ascii_lowercase().as_str()) {
        return Err(invalid(format!(
            "{owner} cannot use the header `{name}`, which belongs to the transport"
        )));
    }
    Ok(())
}

fn check_operation(operation: &Operation) -> Result<(), CollectionError> {
    let id = operation.id.as_str();
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
    if operation.path.chars().count() > MAX_OPERATION_PATH_LENGTH {
        return Err(invalid(format!(
            "operation `{id}` has a path longer than {MAX_OPERATION_PATH_LENGTH} characters"
        )));
    }
    if path_placeholders(&operation.path).len() > MAX_PATH_PLACEHOLDERS {
        return Err(invalid(format!(
            "operation `{id}` has a path with more than {MAX_PATH_PLACEHOLDERS} placeholders"
        )));
    }
    if operation.parameters.len() > MAX_OPERATION_PARAMETERS {
        return Err(invalid(format!(
            "operation `{id}` declares more than {MAX_OPERATION_PARAMETERS} parameters"
        )));
    }
    let mut seen: HashSet<(ParameterLocation, &str)> = HashSet::new();
    for parameter in &operation.parameters {
        if parameter.name.trim().is_empty() {
            return Err(invalid(format!(
                "operation `{id}` has a parameter with no name"
            )));
        }
        if parameter.location == ParameterLocation::Header {
            check_transport_header(&parameter.name, &format!("operation `{id}`"))?;
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

fn contains_nul(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains('\0'),
        Value::Array(items) => items.iter().any(contains_nul),
        Value::Object(map) => map
            .iter()
            .any(|(key, value)| key.contains('\0') || contains_nul(value)),
        _ => false,
    }
}

fn check_no_nul(input: &CollectionDraftInput) -> Result<(), CollectionError> {
    let plain = [
        &input.title,
        &input.version,
        &input.description,
        &input.base_url,
        match &input.source {
            CollectionSource::Url { url } => url,
            CollectionSource::File { file_name } => file_name,
        },
    ];
    let hidden = plain.iter().any(|text| text.contains('\0'))
        || input
            .schemas
            .iter()
            .any(|(key, value)| key.contains('\0') || contains_nul(value))
        || serde_json::to_value(&input.operations).is_ok_and(|value| contains_nul(&value))
        || serde_json::to_value(&input.security).is_ok_and(|value| contains_nul(&value));
    if hidden {
        return Err(invalid(
            "the model cannot contain a NUL character (\\u0000)",
        ));
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
        check_no_nul(&input)?;

        let title = input.title.trim().to_owned();
        check_text("title", &title, MAX_COLLECTION_TITLE_LENGTH, false)?;
        let version = input.version.trim().to_owned();
        check_text("version", &version, MAX_COLLECTION_VERSION_LENGTH, false)?;
        check_text(
            "description",
            &input.description,
            MAX_COLLECTION_DESCRIPTION_LENGTH,
            true,
        )?;
        check_text("base url", &input.base_url, MAX_COLLECTION_URL_LENGTH, true)?;
        let base_url = normalized_base_url(&input.base_url).map_err(invalid)?;
        let source = check_source(input.source)?;

        if input.operations.is_empty() {
            return Err(invalid("a collection needs at least one operation"));
        }
        if input.operations.len() > MAX_COLLECTION_OPERATIONS {
            return Err(invalid(format!(
                "a collection holds at most {MAX_COLLECTION_OPERATIONS} operations"
            )));
        }
        let mut operations = input.operations;
        for operation in &mut operations {
            operation.id = operation.id.trim().to_owned();
        }
        let mut operation_ids: HashSet<&str> = HashSet::new();
        for operation in &operations {
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
            if let SecurityKind::ApiKey { parameter_name, .. } = &scheme.kind {
                check_transport_header(
                    parameter_name,
                    &format!("security scheme `{}`", scheme.name),
                )?;
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
            base_url,
            source,
            security: input.security,
            operations,
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

#[derive(Debug, Clone, PartialEq)]
pub struct CollectionVersion {
    pub version: String,
    pub imported_at: DateTime<Utc>,
    pub source: CollectionSource,
    pub base_url: String,
    pub security: Vec<SecurityScheme>,
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
            base_url: self.base_url.clone(),
            security: self.security.clone(),
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

    fn invalid_with(mutate: impl FnOnce(&mut CollectionDraftInput)) -> String {
        let mut candidate = input();
        mutate(&mut candidate);
        invalid_reason(candidate)
    }

    #[test]
    fn a_base_url_is_trimmed_and_must_be_a_plain_http_or_https_address() {
        let mut candidate = input();
        candidate.base_url = "  https://api.example.com/v1/  ".to_owned();
        assert_eq!(
            CollectionDraft::new(candidate).unwrap().base_url,
            "https://api.example.com/v1/"
        );

        for refused in [
            "ftp://files.example.com",
            "/v1",
            "not a url",
            "https://{region}.example.com",
            "https://user:secret@api.example.com",
            "https://user@api.example.com",
            "https://api.example.com/v1?key=1",
            "https://api.example.com/v1?",
            "https://api.example.com/v1#section",
            "https://",
        ] {
            let reason = invalid_with(|candidate| candidate.base_url = refused.to_owned());

            assert!(reason.contains("base url"), "{refused}: {reason}");
        }
    }

    #[test]
    fn a_blank_base_url_is_kept_empty() {
        let mut candidate = input();
        candidate.base_url = "   ".to_owned();

        assert_eq!(CollectionDraft::new(candidate).unwrap().base_url, "");
    }

    #[test]
    fn the_source_url_loses_its_query_fragment_and_credentials_before_storage() {
        let mut candidate = input();
        candidate.source = CollectionSource::Url {
            url: "https://bot:secret@api.example.com/openapi.json?token=abc&v=2#top".to_owned(),
        };

        let draft = CollectionDraft::new(candidate).unwrap();

        assert_eq!(
            draft.source,
            CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned()
            }
        );
    }

    #[test]
    fn a_source_url_that_is_not_an_http_address_is_refused() {
        for refused in [
            "javascript:alert(1)",
            "not a url",
            "ftp://x.example.com/spec",
        ] {
            let reason = invalid_with(|candidate| {
                candidate.source = CollectionSource::Url {
                    url: refused.to_owned(),
                };
            });

            assert!(reason.contains("source"), "{refused}: {reason}");
        }
    }

    #[test]
    fn operation_ids_are_stored_trimmed_and_compared_trimmed() {
        let mut candidate = input();
        candidate.operations = vec![operation("  list  ")];
        let draft = CollectionDraft::new(candidate).unwrap();
        assert_eq!(draft.operations[0].id, "list");

        let reason = invalid_with(|candidate| {
            candidate.operations = vec![operation("list"), operation(" list ")];
        });
        assert!(reason.contains("list"), "{reason}");
    }

    fn hide_nul(candidate: &mut CollectionDraftInput, place: &str) {
        let nul = "bad\u{0}value";
        match place {
            "title" => candidate.title = nul.to_owned(),
            "description" => candidate.description = nul.to_owned(),
            "version" => candidate.version = nul.to_owned(),
            "schema value" => {
                candidate
                    .schemas
                    .insert("A".to_owned(), json!({ "x": [{ "description": nul }] }));
            }
            "schema key" => {
                candidate.schemas.insert(nul.to_owned(), json!({}));
            }
            "summary" => candidate.operations[0].summary = nul.to_owned(),
            "body schema" => {
                candidate.operations[0].request_body = Some(RequestBody {
                    schema: json!({ "enum": [nul] }),
                });
            }
            other => panic!("unknown place {other}"),
        }
    }

    #[test]
    fn a_nul_character_is_refused_wherever_it_hides() {
        for place in [
            "title",
            "description",
            "version",
            "schema value",
            "schema key",
            "summary",
            "body schema",
        ] {
            let reason = invalid_with(|candidate| hide_nul(candidate, place));

            assert!(reason.contains("NUL"), "{place}: {reason}");
        }
    }

    #[test]
    fn a_backslash_followed_by_u0000_text_is_not_a_nul() {
        let mut candidate = input();
        candidate.description = "the escape \\u0000 written out".to_owned();

        assert!(CollectionDraft::new(candidate).is_ok());
    }

    #[test]
    fn a_collection_holds_at_most_a_thousand_operations() {
        let many = |count: usize| (0..count).map(|n| operation(&format!("op{n}"))).collect();

        let mut at_the_limit = input();
        at_the_limit.operations = many(1000);
        assert!(CollectionDraft::new(at_the_limit).is_ok());

        let reason = invalid_with(|candidate| candidate.operations = many(1001));
        assert!(reason.contains("1000"), "{reason}");
    }

    fn parameter_named(name: String) -> Parameter {
        Parameter {
            name,
            location: ParameterLocation::Query,
            required: false,
            description: String::new(),
            schema: Value::Null,
        }
    }

    #[test]
    fn an_operation_holds_at_most_a_hundred_parameters() {
        let mut at_the_limit = input();
        at_the_limit.operations[0].parameters =
            (0..100).map(|n| parameter_named(format!("p{n}"))).collect();
        assert!(CollectionDraft::new(at_the_limit).is_ok());

        let reason = invalid_with(|candidate| {
            candidate.operations[0].parameters =
                (0..101).map(|n| parameter_named(format!("p{n}"))).collect();
        });
        assert!(
            reason.contains("100") && reason.contains("list"),
            "{reason}"
        );
    }

    #[test]
    fn a_path_holds_at_most_2048_characters_and_32_placeholders() {
        let mut long_enough = input();
        long_enough.operations[0].path = format!("/{}", "a".repeat(2047));
        assert!(CollectionDraft::new(long_enough).is_ok());

        let reason = invalid_with(|candidate| {
            candidate.operations[0].path = format!("/{}", "a".repeat(2048));
        });
        assert!(reason.contains("2048"), "{reason}");

        let placeholders =
            |count: usize| (0..count).map(|n| format!("/{{p{n}}}")).collect::<String>();
        let mut at_the_limit = input();
        at_the_limit.operations[0].path = placeholders(32);
        assert!(CollectionDraft::new(at_the_limit).is_ok());

        let reason = invalid_with(|candidate| {
            candidate.operations[0].path = placeholders(33);
        });
        assert!(reason.contains("32"), "{reason}");
    }

    #[test]
    fn a_description_holds_at_most_ten_thousand_characters() {
        let mut at_the_limit = input();
        at_the_limit.description = "é".repeat(10_000);
        assert!(CollectionDraft::new(at_the_limit).is_ok());

        let reason = invalid_with(|candidate| candidate.description = "é".repeat(10_001));
        assert!(
            reason.contains("description") && reason.contains("10000"),
            "{reason}"
        );
    }

    #[test]
    fn the_headers_the_transport_owns_cannot_be_declared() {
        for name in [
            "Host",
            "content-length",
            "Transfer-Encoding",
            "CONNECTION",
            "Proxy-Authorization",
        ] {
            let reason = invalid_with(|candidate| {
                candidate.operations[0].parameters = vec![Parameter {
                    name: name.to_owned(),
                    location: ParameterLocation::Header,
                    required: false,
                    description: String::new(),
                    schema: Value::Null,
                }];
            });
            assert!(reason.contains(name), "{name}: {reason}");

            let reason = invalid_with(|candidate| {
                candidate.security = vec![SecurityScheme {
                    name: "key".to_owned(),
                    kind: SecurityKind::ApiKey {
                        location: ApiKeyLocation::Header,
                        parameter_name: name.to_owned(),
                    },
                }];
            });
            assert!(reason.contains(name), "{name}: {reason}");
        }

        let mut fine = input();
        fine.operations[0].parameters = vec![Parameter {
            name: "X-Trace".to_owned(),
            location: ParameterLocation::Header,
            required: false,
            description: String::new(),
            schema: Value::Null,
        }];
        fine.security = vec![SecurityScheme {
            name: "key".to_owned(),
            kind: SecurityKind::ApiKey {
                location: ApiKeyLocation::Header,
                parameter_name: "X-API-Key".to_owned(),
            },
        }];
        assert!(CollectionDraft::new(fine).is_ok());
    }

    #[test]
    fn a_query_parameter_may_share_a_name_with_a_forbidden_header() {
        let mut candidate = input();
        candidate.operations[0].parameters = vec![parameter_named("host".to_owned())];

        assert!(CollectionDraft::new(candidate).is_ok());
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
