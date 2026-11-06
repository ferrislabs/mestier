use chrono::{DateTime, Utc};
use mestier_core::{
    ApiKeyLocation, Collection, CollectionDetail, CollectionDraft, CollectionDraftInput,
    CollectionError, CollectionSource, CollectionSummary, CollectionVersion, HttpMethod, Operation,
    OrganizationId, Parameter, ParameterLocation, RequestBody, SecurityKind, SecurityScheme,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethodDto {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl From<HttpMethod> for HttpMethodDto {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => Self::Get,
            HttpMethod::Post => Self::Post,
            HttpMethod::Put => Self::Put,
            HttpMethod::Patch => Self::Patch,
            HttpMethod::Delete => Self::Delete,
        }
    }
}

impl From<HttpMethodDto> for HttpMethod {
    fn from(value: HttpMethodDto) -> Self {
        match value {
            HttpMethodDto::Get => Self::Get,
            HttpMethodDto::Post => Self::Post,
            HttpMethodDto::Put => Self::Put,
            HttpMethodDto::Patch => Self::Patch,
            HttpMethodDto::Delete => Self::Delete,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParameterLocationDto {
    Path,
    Query,
    Header,
}

impl From<ParameterLocation> for ParameterLocationDto {
    fn from(value: ParameterLocation) -> Self {
        match value {
            ParameterLocation::Path => Self::Path,
            ParameterLocation::Query => Self::Query,
            ParameterLocation::Header => Self::Header,
        }
    }
}

impl From<ParameterLocationDto> for ParameterLocation {
    fn from(value: ParameterLocationDto) -> Self {
        match value {
            ParameterLocationDto::Path => Self::Path,
            ParameterLocationDto::Query => Self::Query,
            ParameterLocationDto::Header => Self::Header,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ParameterDto {
    pub name: String,
    pub location: ParameterLocationDto,
    pub required: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub schema: Value,
}

impl From<Parameter> for ParameterDto {
    fn from(value: Parameter) -> Self {
        Self {
            name: value.name,
            location: value.location.into(),
            required: value.required,
            description: value.description,
            schema: value.schema,
        }
    }
}

impl From<ParameterDto> for Parameter {
    fn from(value: ParameterDto) -> Self {
        Self {
            name: value.name,
            location: value.location.into(),
            required: value.required,
            description: value.description,
            schema: value.schema,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct RequestBodyDto {
    pub schema: Value,
}

impl From<RequestBody> for RequestBodyDto {
    fn from(value: RequestBody) -> Self {
        Self {
            schema: value.schema,
        }
    }
}

impl From<RequestBodyDto> for RequestBody {
    fn from(value: RequestBodyDto) -> Self {
        Self {
            schema: value.schema,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct OperationDto {
    pub id: String,
    pub method: HttpMethodDto,
    pub path: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub parameters: Vec<ParameterDto>,
    #[serde(default)]
    pub request_body: Option<RequestBodyDto>,
    #[serde(default)]
    pub response: Option<Value>,
    #[serde(default)]
    pub response_status: Option<String>,
}

impl From<Operation> for OperationDto {
    fn from(value: Operation) -> Self {
        Self {
            id: value.id,
            method: value.method.into(),
            path: value.path,
            summary: value.summary,
            tag: value.tag,
            parameters: value.parameters.into_iter().map(Into::into).collect(),
            request_body: value.request_body.map(Into::into),
            response: value.response,
            response_status: value.response_status,
        }
    }
}

impl From<OperationDto> for Operation {
    fn from(value: OperationDto) -> Self {
        Self {
            id: value.id,
            method: value.method.into(),
            path: value.path,
            summary: value.summary,
            tag: value.tag,
            parameters: value.parameters.into_iter().map(Into::into).collect(),
            request_body: value.request_body.map(Into::into),
            response: value.response,
            response_status: value.response_status,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyLocationDto {
    Header,
    Query,
    Cookie,
}

impl From<ApiKeyLocation> for ApiKeyLocationDto {
    fn from(value: ApiKeyLocation) -> Self {
        match value {
            ApiKeyLocation::Header => Self::Header,
            ApiKeyLocation::Query => Self::Query,
            ApiKeyLocation::Cookie => Self::Cookie,
        }
    }
}

impl From<ApiKeyLocationDto> for ApiKeyLocation {
    fn from(value: ApiKeyLocationDto) -> Self {
        match value {
            ApiKeyLocationDto::Header => Self::Header,
            ApiKeyLocationDto::Query => Self::Query,
            ApiKeyLocationDto::Cookie => Self::Cookie,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecurityKindDto {
    Http {
        scheme: String,
    },
    ApiKey {
        location: ApiKeyLocationDto,
        parameter_name: String,
    },
    Oauth2,
    OpenIdConnect,
    Other {
        declared_type: String,
    },
}

impl From<SecurityKind> for SecurityKindDto {
    fn from(value: SecurityKind) -> Self {
        match value {
            SecurityKind::Http { scheme } => Self::Http { scheme },
            SecurityKind::ApiKey {
                location,
                parameter_name,
            } => Self::ApiKey {
                location: location.into(),
                parameter_name,
            },
            SecurityKind::Oauth2 => Self::Oauth2,
            SecurityKind::OpenIdConnect => Self::OpenIdConnect,
            SecurityKind::Other { declared_type } => Self::Other { declared_type },
        }
    }
}

impl From<SecurityKindDto> for SecurityKind {
    fn from(value: SecurityKindDto) -> Self {
        match value {
            SecurityKindDto::Http { scheme } => Self::Http { scheme },
            SecurityKindDto::ApiKey {
                location,
                parameter_name,
            } => Self::ApiKey {
                location: location.into(),
                parameter_name,
            },
            SecurityKindDto::Oauth2 => Self::Oauth2,
            SecurityKindDto::OpenIdConnect => Self::OpenIdConnect,
            SecurityKindDto::Other { declared_type } => Self::Other { declared_type },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SecuritySchemeDto {
    pub name: String,
    pub kind: SecurityKindDto,
}

impl From<SecurityScheme> for SecuritySchemeDto {
    fn from(value: SecurityScheme) -> Self {
        Self {
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

impl From<SecuritySchemeDto> for SecurityScheme {
    fn from(value: SecuritySchemeDto) -> Self {
        Self {
            name: value.name,
            kind: value.kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CollectionSourceDto {
    Url { url: String },
    File { file_name: String },
}

impl From<CollectionSource> for CollectionSourceDto {
    fn from(value: CollectionSource) -> Self {
        match value {
            CollectionSource::Url { url } => Self::Url { url },
            CollectionSource::File { file_name } => Self::File { file_name },
        }
    }
}

impl From<CollectionSourceDto> for CollectionSource {
    fn from(value: CollectionSourceDto) -> Self {
        match value {
            CollectionSourceDto::Url { url } => Self::Url { url },
            CollectionSourceDto::File { file_name } => Self::File { file_name },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, ToSchema)]
pub struct CollectionRequest {
    pub title: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    pub base_url: String,
    pub source: CollectionSourceDto,
    #[serde(default)]
    pub security: Vec<SecuritySchemeDto>,
    pub operations: Vec<OperationDto>,
    #[serde(default)]
    pub schemas: Map<String, Value>,
}

impl CollectionRequest {
    pub fn into_draft(self) -> Result<CollectionDraft, CollectionError> {
        CollectionDraft::new(CollectionDraftInput {
            title: self.title,
            version: self.version,
            description: self.description,
            base_url: self.base_url,
            source: self.source.into(),
            security: self.security.into_iter().map(Into::into).collect(),
            operations: self.operations.into_iter().map(Into::into).collect(),
            schemas: self.schemas,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CollectionVersionResponse {
    pub version: String,
    pub imported_at: DateTime<Utc>,
    pub source: CollectionSourceDto,
}

impl From<CollectionVersion> for CollectionVersionResponse {
    fn from(value: CollectionVersion) -> Self {
        Self {
            version: value.version,
            imported_at: value.imported_at,
            source: value.source.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CollectionSummaryResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub slug: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub source: CollectionSourceDto,
    pub imported_at: DateTime<Utc>,
    pub security: Vec<SecuritySchemeDto>,
    pub operation_count: u32,
}

impl From<CollectionSummary> for CollectionSummaryResponse {
    fn from(value: CollectionSummary) -> Self {
        Self {
            id: value.id,
            organization_id: value.org_id,
            slug: value.slug,
            title: value.title,
            version: value.version,
            description: value.description,
            base_url: value.base_url,
            source: value.source.into(),
            imported_at: value.imported_at,
            security: value.security.into_iter().map(Into::into).collect(),
            operation_count: u32::try_from(value.operation_count).unwrap_or(u32::MAX),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct CollectionResponse {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub slug: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub base_url: String,
    pub source: CollectionSourceDto,
    pub imported_at: DateTime<Utc>,
    pub security: Vec<SecuritySchemeDto>,
    pub operations: Vec<OperationDto>,
    pub schemas: Map<String, Value>,
    pub history: Vec<CollectionVersionResponse>,
}

impl CollectionResponse {
    pub fn new(collection: Collection, history: Vec<CollectionVersion>) -> Self {
        Self {
            id: collection.id,
            organization_id: collection.org_id,
            slug: collection.slug,
            title: collection.title,
            version: collection.version,
            description: collection.description,
            base_url: collection.base_url,
            source: collection.source.into(),
            imported_at: collection.imported_at,
            security: collection.security.into_iter().map(Into::into).collect(),
            operations: collection.operations.into_iter().map(Into::into).collect(),
            schemas: collection.schemas,
            history: history.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<CollectionDetail> for CollectionResponse {
    fn from(value: CollectionDetail) -> Self {
        Self::new(value.collection, value.history)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use mestier_core::{Collection, CollectionDetail, CollectionVersion, OrganizationId};
    use serde_json::{Value, json};
    use uuid::Uuid;

    use super::*;

    fn request_json() -> Value {
        json!({
            "title": "  API Adresse  ",
            "version": "2.1.0",
            "description": "Search addresses",
            "base_url": "https://api.example.com/v2",
            "source": { "kind": "url", "url": "https://api.example.com/openapi.json" },
            "security": [
                { "name": "bearerAuth", "kind": { "type": "http", "scheme": "bearer" } },
                {
                    "name": "keyAuth",
                    "kind": { "type": "api_key", "location": "header", "parameter_name": "X-API-Key" }
                },
                { "name": "oauth", "kind": { "type": "oauth2" } }
            ],
            "operations": [
                {
                    "id": "getAddress",
                    "method": "GET",
                    "path": "/addresses/{id}",
                    "summary": "Read one",
                    "tag": "Addresses",
                    "parameters": [
                        {
                            "name": "id",
                            "location": "path",
                            "required": true,
                            "description": "The id",
                            "schema": { "type": "string" }
                        }
                    ],
                    "request_body": null,
                    "response": { "$ref": "#/components/schemas/Address" },
                    "response_status": "200"
                },
                {
                    "id": "createAddress",
                    "method": "POST",
                    "path": "/addresses",
                    "request_body": { "schema": { "type": "object" } }
                }
            ],
            "schemas": { "Address": { "type": "object" } }
        })
    }

    fn collection() -> Collection {
        let request: CollectionRequest = serde_json::from_value(request_json()).unwrap();
        let draft = request.into_draft().unwrap();
        Collection::from_draft(
            draft,
            Uuid::from_u128(5),
            OrganizationId(Uuid::from_u128(1)),
            "api-adresse".to_owned(),
            Utc.with_ymd_and_hms(2026, 11, 6, 9, 30, 0).unwrap(),
        )
    }

    #[test]
    fn a_request_becomes_a_validated_draft_with_every_part_carried() {
        let request: CollectionRequest = serde_json::from_value(request_json()).unwrap();

        let draft = request.into_draft().unwrap();
        let collection = Collection::from_draft(
            draft,
            Uuid::from_u128(5),
            OrganizationId(Uuid::from_u128(1)),
            "api-adresse".to_owned(),
            Utc::now(),
        );

        assert_eq!(collection.title, "API Adresse");
        assert_eq!(collection.operations.len(), 2);
        assert_eq!(collection.security.len(), 3);
        assert_eq!(collection.operations[0].parameters[0].name, "id");
        assert!(collection.operations[1].request_body.is_some());
        assert!(collection.schemas.contains_key("Address"));
    }

    #[test]
    fn an_operation_omitting_its_optional_parts_still_reads() {
        let request: CollectionRequest = serde_json::from_value(request_json()).unwrap();

        let second = &request.operations[1];

        assert_eq!(second.summary, "");
        assert!(second.parameters.is_empty());
        assert!(second.response.is_none());
        assert!(second.response_status.is_none());
    }

    #[test]
    fn an_unknown_method_or_security_type_is_refused_by_the_wire_format() {
        let mut bad_method = request_json();
        bad_method["operations"][0]["method"] = json!("TRACE");
        assert!(serde_json::from_value::<CollectionRequest>(bad_method).is_err());

        let mut bad_location = request_json();
        bad_location["operations"][0]["parameters"][0]["location"] = json!("cookie");
        assert!(serde_json::from_value::<CollectionRequest>(bad_location).is_err());

        let mut bad_source = request_json();
        bad_source["source"] = json!({ "kind": "ftp", "url": "ftp://x" });
        assert!(serde_json::from_value::<CollectionRequest>(bad_source).is_err());
    }

    #[test]
    fn an_invalid_model_is_refused_with_the_reason() {
        let mut candidate = request_json();
        candidate["operations"] = json!([]);
        let request: CollectionRequest = serde_json::from_value(candidate).unwrap();

        let error = request.into_draft().unwrap_err();

        assert!(error.to_string().contains("operation"), "{error}");
    }

    #[test]
    fn the_full_response_carries_the_model_and_the_history_newest_first() {
        let collection = collection();
        let history = vec![
            CollectionVersion {
                version: "2.0.0".to_owned(),
                imported_at: Utc.with_ymd_and_hms(2026, 11, 5, 9, 0, 0).unwrap(),
                source: mestier_core::CollectionSource::File {
                    file_name: "v2.yaml".to_owned(),
                },
            },
            CollectionVersion {
                version: "1.0.0".to_owned(),
                imported_at: Utc.with_ymd_and_hms(2026, 11, 4, 9, 0, 0).unwrap(),
                source: mestier_core::CollectionSource::Url {
                    url: "https://api.example.com/openapi.json".to_owned(),
                },
            },
        ];

        let response = CollectionResponse::from(CollectionDetail {
            collection,
            history,
        });
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["id"], "00000000-0000-0000-0000-000000000005");
        assert_eq!(
            json["organization_id"],
            "00000000-0000-0000-0000-000000000001"
        );
        assert_eq!(json["slug"], "api-adresse");
        assert_eq!(json["base_url"], "https://api.example.com/v2");
        assert_eq!(json["imported_at"], "2026-11-06T09:30:00Z");
        assert_eq!(json["source"]["kind"], "url");
        assert_eq!(json["operations"][0]["method"], "GET");
        assert_eq!(json["operations"][0]["parameters"][0]["location"], "path");
        assert_eq!(
            json["operations"][1]["request_body"]["schema"]["type"],
            "object"
        );
        assert_eq!(json["security"][1]["kind"]["parameter_name"], "X-API-Key");
        assert_eq!(json["schemas"]["Address"]["type"], "object");
        assert_eq!(json["history"][0]["version"], "2.0.0");
        assert_eq!(json["history"][0]["source"]["file_name"], "v2.yaml");
        assert_eq!(json["history"][1]["version"], "1.0.0");
    }

    #[test]
    fn a_model_read_back_and_sent_again_is_the_same_request() {
        let collection = collection();
        let response = CollectionResponse::from(CollectionDetail {
            collection: collection.clone(),
            history: Vec::new(),
        });
        let mut sent = serde_json::to_value(&response).unwrap();
        let object = sent.as_object_mut().unwrap();
        for key in ["id", "organization_id", "slug", "imported_at", "history"] {
            object.remove(key);
        }

        let request: CollectionRequest = serde_json::from_value(sent).unwrap();
        let draft = request.into_draft().unwrap();
        let again = Collection::from_draft(
            draft,
            collection.id,
            collection.org_id,
            collection.slug.clone(),
            collection.imported_at,
        );

        assert_eq!(again, collection);
    }

    #[test]
    fn the_summary_is_light_and_counts_the_operations() {
        let collection = collection();
        let summary = mestier_core::CollectionSummary {
            id: collection.id,
            org_id: collection.org_id,
            slug: collection.slug.clone(),
            title: collection.title.clone(),
            version: collection.version.clone(),
            description: collection.description.clone(),
            base_url: collection.base_url.clone(),
            source: collection.source.clone(),
            imported_at: collection.imported_at,
            security: collection.security.clone(),
            operation_count: collection.operations.len(),
        };

        let json = serde_json::to_value(CollectionSummaryResponse::from(summary)).unwrap();

        assert_eq!(json["operation_count"], 2);
        assert_eq!(json["slug"], "api-adresse");
        assert_eq!(json["security"].as_array().unwrap().len(), 3);
        let object = json.as_object().unwrap();
        assert!(!object.contains_key("operations"));
        assert!(!object.contains_key("schemas"));
        assert!(!object.contains_key("history"));
    }
}
