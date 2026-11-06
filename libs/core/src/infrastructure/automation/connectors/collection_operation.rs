use reqwest::{Client, Method, Url};
use serde_json::Value;

use crate::application::MestierUseCase;
use crate::domain::automation::collection::{
    Collection, HttpMethod, Identification, Offered, build_request, select_identification,
};
use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::run::{
    Connector, ConnectorInput, ConnectorOutcome, DryRunInput, StoredOperation,
};
use crate::domain::automation::workflow::OperationRef;
use crate::infrastructure::automation::webhook::address_policy::PrivateNetworkAccess;

use super::http_client::{
    DEFAULT_TIMEOUT, HttpAuth, PreparedRequest, build_guarded_client, client_for_auth,
    describe_reqwest_error, open_http_auth, outcome_for, read_response,
    refuse_literal_private_address, send,
};

pub struct CollectionOperationConnector {
    usecase: MestierUseCase,
    client: Client,
    access: Option<PrivateNetworkAccess>,
}

fn failed(error: impl Into<String>) -> ConnectorOutcome {
    ConnectorOutcome::Failed {
        error: error.into(),
    }
}

fn method_of(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::GET,
        HttpMethod::Post => Method::POST,
        HttpMethod::Put => Method::PUT,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Delete => Method::DELETE,
    }
}

fn offered_by(auth: &HttpAuth) -> Offered {
    match auth {
        HttpAuth::Bearer { .. } | HttpAuth::Jwt { .. } => Offered::Bearer,
        HttpAuth::Basic { .. } => Offered::Basic,
        HttpAuth::Header { .. } => Offered::Header,
        HttpAuth::Query { .. } => Offered::Query,
        HttpAuth::Digest { .. }
        | HttpAuth::Custom(_)
        | HttpAuth::OAuth1(_)
        | HttpAuth::ClientCertificate(_) => Offered::Other,
    }
}

impl CollectionOperationConnector {
    pub fn new(usecase: MestierUseCase, access: PrivateNetworkAccess) -> Self {
        let client = build_guarded_client(access).expect("building the guarded HTTP client failed");
        Self {
            usecase,
            client,
            access: Some(access),
        }
    }

    #[cfg(test)]
    pub(super) fn with_client(usecase: MestierUseCase, client: Client) -> Self {
        Self {
            usecase,
            client,
            access: None,
        }
    }

    async fn perform(
        &self,
        collection: &Collection,
        reference: &OperationRef,
        input: &ConnectorInput<'_>,
    ) -> ConnectorOutcome {
        let title = &collection.title;
        let Some(operation) = collection.find_operation(&reference.operation_id) else {
            return failed(format!(
                "operation `{}` no longer exists in collection `{title}`",
                reference.operation_id
            ));
        };
        let built = match build_request(collection, operation, input.config) {
            Ok(built) => built,
            Err(error) => {
                return failed(format!(
                    "collection `{title}` operation `{}`: {error}",
                    operation.id
                ));
            }
        };
        let mut url = match Url::parse(&built.url) {
            Ok(url) => url,
            Err(error) => {
                return failed(format!(
                    "collection `{title}` has a base url that is not a valid URL: {error}"
                ));
            }
        };
        if !matches!(url.scheme(), "http" | "https") {
            return failed(format!(
                "collection `{title}` has a base url that does not use http or https"
            ));
        }
        if let Some(access) = self.access
            && let Err(error) = refuse_literal_private_address(&url, access)
        {
            return failed(error);
        }

        if let Err(refusal) = select_identification(&collection.security, None) {
            return failed(refusal.to_string());
        }
        let auth = match input.credential_id {
            Some(credential_id) => {
                match open_http_auth(&self.usecase, input.org_id, credential_id).await {
                    Ok(auth) => Some(auth),
                    Err(error) => return failed(error),
                }
            }
            None => None,
        };
        let identification =
            match select_identification(&collection.security, auth.as_ref().map(offered_by)) {
                Ok(identification) => identification,
                Err(refusal) => return failed(refusal.to_string()),
            };

        if !built.query.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (name, value) in &built.query {
                pairs.append_pair(name, value);
            }
        }

        let mut request = PreparedRequest::new(
            method_of(built.method),
            url,
            built.headers.clone(),
            built.body.clone(),
        );
        match (identification, &auth) {
            (Identification::Credential, Some(auth)) => {
                if let Err(error) = auth.prepare(&mut request) {
                    return failed(error);
                }
            }
            (Identification::HeaderNamed(name), Some(HttpAuth::Header { value, .. })) => {
                request.set_header(&name, value.clone());
            }
            (Identification::QueryNamed(name), Some(HttpAuth::Query { value, .. })) => {
                request.set_query(&name, value);
            }
            _ => {}
        }

        let client = match client_for_auth(&self.client, self.access, auth.as_ref()) {
            Ok(client) => client,
            Err(error) => return failed(error),
        };
        let response = match send(&client, &request, DEFAULT_TIMEOUT, auth.as_ref()).await {
            Ok(response) => response,
            Err(error) => return failed(describe_reqwest_error(&error)),
        };
        match read_response(response).await {
            Ok(captured) => outcome_for(captured),
            Err(error) => failed(error),
        }
    }
}

impl Connector for CollectionOperationConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let Some(reference) = input.operation else {
            return ConnectorOutcome::Failed {
                error: "the node carries no operation reference".to_owned(),
            };
        };
        let collection = match self
            .usecase
            .find_collection(input.org_id, reference.collection_id)
            .await
        {
            Ok(Some(collection)) => collection,
            Ok(None) => {
                return ConnectorOutcome::Failed {
                    error: format!("collection `{}` no longer exists", reference.collection_id),
                };
            }
            Err(error) => {
                return ConnectorOutcome::Failed {
                    error: format!(
                        "collection `{}` could not be read: {error}",
                        reference.collection_id
                    ),
                };
            }
        };

        self.perform(&collection, reference, &input).await
    }

    fn dry_run(descriptor: &ConnectorDescriptor, input: &DryRunInput<'_>) -> ConnectorOutcome {
        let Some(StoredOperation {
            collection,
            operation,
        }) = input.stored
        else {
            return failed("the node's operation no longer exists in its collection");
        };
        if let Err(error) = build_request(collection, operation, input.config) {
            return failed(format!(
                "collection `{}` operation `{}`: {error}",
                collection.title, operation.id
            ));
        }

        let mut output = descriptor.sample_output.clone();
        if let Value::Object(fields) = &mut output {
            if let Some(status) = operation
                .response_status
                .as_deref()
                .and_then(|status| status.parse::<u16>().ok())
                .filter(|status| (200..300).contains(status))
            {
                fields.insert("status".to_owned(), Value::from(status));
            }
            if let Some(body) = &operation.response {
                fields.insert("body".to_owned(), body.clone());
            }
        }
        ConnectorOutcome::Produced(output)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use common::OrganizationId;
    use serde_json::{Map, Value, json};
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::domain::automation::collection::{
        ApiKeyLocation, CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod,
        ImportCollectionCommand, Operation, Parameter, ParameterLocation, ReplaceCollectionCommand,
        RequestBody, SecurityKind, SecurityScheme,
    };
    use crate::domain::automation::run::{DryRunInput, StoredOperation};
    use crate::infrastructure::automation::connectors::test_support::{StubResponse, TestServer};
    use crate::infrastructure::realtime::EventHub;

    const COLLECTION_ID: u128 = 77;

    fn lazy_usecase() -> MestierUseCase {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn test_client() -> Client {
        Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("a plain client builds without a custom resolver")
    }

    fn parameter(name: &str, location: ParameterLocation, required: bool) -> Parameter {
        Parameter {
            name: name.to_owned(),
            location,
            required,
            description: String::new(),
            schema: json!({ "type": "string" }),
        }
    }

    fn get_address(path: &str) -> Operation {
        Operation {
            id: "getAddress".to_owned(),
            method: HttpMethod::Get,
            path: path.to_owned(),
            summary: String::new(),
            tag: String::new(),
            parameters: vec![
                parameter("id", ParameterLocation::Path, true),
                parameter("limit", ParameterLocation::Query, false),
                parameter("X-Trace", ParameterLocation::Header, false),
            ],
            request_body: None,
            response: None,
            response_status: None,
        }
    }

    fn create_address() -> Operation {
        Operation {
            id: "createAddress".to_owned(),
            method: HttpMethod::Post,
            path: "/addresses".to_owned(),
            summary: String::new(),
            tag: String::new(),
            parameters: Vec::new(),
            request_body: Some(RequestBody {
                schema: json!({ "type": "object" }),
            }),
            response: None,
            response_status: None,
        }
    }

    fn draft(base_url: &str, security: Vec<SecurityScheme>, path: &str) -> CollectionDraft {
        CollectionDraft::new(CollectionDraftInput {
            title: "Addresses".to_owned(),
            version: "1.0.0".to_owned(),
            description: String::new(),
            base_url: base_url.to_owned(),
            source: CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned(),
            },
            security,
            operations: vec![get_address(path), create_address()],
            schemas: Map::new(),
        })
        .unwrap()
    }

    fn collection(base_url: &str, security: Vec<SecurityScheme>) -> Collection {
        Collection::from_draft(
            draft(base_url, security, "/addresses/{id}"),
            Uuid::from_u128(COLLECTION_ID),
            OrganizationId(Uuid::from_u128(1)),
            "addresses".to_owned(),
            Utc::now(),
        )
    }

    fn reference(operation_id: &str) -> OperationRef {
        OperationRef {
            collection_id: Uuid::from_u128(COLLECTION_ID),
            operation_id: operation_id.to_owned(),
        }
    }

    fn config(value: Value) -> Map<String, Value> {
        value.as_object().cloned().expect("a config is an object")
    }

    fn input<'a>(
        config: &'a Map<String, Value>,
        reference: &'a OperationRef,
        credential_id: Option<Uuid>,
    ) -> ConnectorInput<'a> {
        ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config,
            credential_id,
            operation: Some(reference),
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        let ConnectorOutcome::Failed { error } = outcome else {
            panic!("expected Failed, got {outcome:?}");
        };
        error
    }

    fn oauth2() -> SecurityScheme {
        SecurityScheme {
            name: "petstore_auth".to_owned(),
            kind: SecurityKind::Oauth2,
        }
    }

    fn descriptor() -> crate::domain::automation::connector::ConnectorDescriptor {
        crate::domain::automation::connector::connector_catalogue()
            .get("collection.operation", 1)
            .expect("collection.operation is described")
            .clone()
    }

    fn dry_run_of(
        collection: &Collection,
        operation_id: &str,
        config: &Map<String, Value>,
    ) -> ConnectorOutcome {
        CollectionOperationConnector::dry_run(
            &descriptor(),
            &DryRunInput {
                config,
                stored: collection
                    .find_operation(operation_id)
                    .map(|operation| StoredOperation {
                        collection,
                        operation,
                    }),
            },
        )
    }

    fn collection_answering(response: Option<Value>, status: Option<&str>) -> Collection {
        let mut collection = collection("https://api.example.com", Vec::new());
        let operation = collection
            .operations
            .iter_mut()
            .find(|operation| operation.id == "getAddress")
            .unwrap();
        operation.response = response;
        operation.response_status = status.map(str::to_owned);
        collection
    }

    #[test]
    fn a_dry_run_answers_with_the_response_example_stored_on_the_operation() {
        let collection =
            collection_answering(Some(json!({ "id": "a-1", "city": "Lyon" })), Some("201"));
        let cfg = config(json!({ "path": { "id": "1" } }));

        let outcome = dry_run_of(&collection, "getAddress", &cfg);

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(json!({
                "status": 201,
                "headers": {},
                "body": { "id": "a-1", "city": "Lyon" }
            }))
        );
    }

    #[test]
    fn a_dry_run_of_an_operation_with_no_stored_response_answers_a_bare_ok() {
        let collection = collection_answering(None, None);
        let cfg = config(json!({ "path": { "id": "1" } }));

        let outcome = dry_run_of(&collection, "getAddress", &cfg);

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(json!({ "status": 200, "headers": {}, "body": {} }))
        );
    }

    #[test]
    fn a_status_that_is_not_a_success_falls_back_to_200() {
        for declared in [
            "default", "2XX", "", "404", "500", "301", "100", "199", "300",
        ] {
            let collection = collection_answering(Some(json!({ "ok": true })), Some(declared));
            let cfg = config(json!({ "path": { "id": "1" } }));

            let outcome = dry_run_of(&collection, "getAddress", &cfg);

            assert_eq!(
                outcome,
                ConnectorOutcome::Produced(json!({
                    "status": 200,
                    "headers": {},
                    "body": { "ok": true }
                })),
                "{declared}"
            );
        }
    }

    #[test]
    fn a_success_status_is_kept_as_declared() {
        for declared in [200u16, 201, 202, 204, 299] {
            let collection =
                collection_answering(Some(json!({ "ok": true })), Some(&declared.to_string()));
            let cfg = config(json!({ "path": { "id": "1" } }));

            let outcome = dry_run_of(&collection, "getAddress", &cfg);

            assert_eq!(
                outcome,
                ConnectorOutcome::Produced(json!({
                    "status": declared,
                    "headers": {},
                    "body": { "ok": true }
                })),
                "{declared}"
            );
        }
    }

    #[test]
    fn a_dry_run_whose_resolved_config_no_longer_fits_the_operation_fails_naming_it() {
        let collection = collection_answering(None, None);
        let cfg = config(json!({ "query": { "limit": 5 } }));

        let error = failure(dry_run_of(&collection, "getAddress", &cfg));

        assert!(error.contains("path.id"), "{error}");
        assert!(
            error.contains("Addresses") && error.contains("getAddress"),
            "{error}"
        );
    }

    #[test]
    fn a_dry_run_without_a_resolvable_operation_fails() {
        let collection = collection_answering(None, None);
        let cfg = config(json!({}));

        let error = failure(dry_run_of(&collection, "deleteAddress", &cfg));

        assert!(error.contains("operation"), "{error}");
    }

    #[tokio::test]
    async fn the_node_sends_the_substituted_path_the_query_the_headers_and_the_body() {
        let server =
            TestServer::respond_always(StubResponse::json(201, &json!({ "id": 42 }))).await;
        let collection = Collection {
            operations: vec![
                Operation {
                    parameters: vec![
                        parameter("id", ParameterLocation::Path, true),
                        parameter("limit", ParameterLocation::Query, false),
                        parameter("X-Trace", ParameterLocation::Header, false),
                    ],
                    request_body: Some(RequestBody {
                        schema: json!({ "type": "object" }),
                    }),
                    method: HttpMethod::Put,
                    ..get_address("/addresses/{id}")
                },
                create_address(),
            ],
            ..collection(&format!("{}/v1/", server.url), Vec::new())
        };
        let cfg = config(json!({
            "path": { "id": "a b" },
            "query": { "limit": 5 },
            "headers": { "X-Trace": "t-1" },
            "body": { "street": "1 rue de la Paix" }
        }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let ConnectorOutcome::Produced(value) = outcome else {
            panic!("expected Produced, got {outcome:?}");
        };
        assert_eq!(value["status"], 201);
        assert_eq!(value["body"], json!({ "id": 42 }));
        assert_eq!(value["headers"]["content-type"], json!("application/json"));
        crate::infrastructure::automation::connectors::test_support::assert_same_shape(
            &value,
            "collection.operation",
        );
        let request = server.last_request();
        assert_eq!(request.method, "PUT");
        assert_eq!(request.path, "/v1/addresses/a%20b?limit=5");
        assert_eq!(
            request.headers.get("x-trace").map(String::as_str),
            Some("t-1")
        );
        assert_eq!(request.body_json(), json!({ "street": "1 rue de la Paix" }));
        assert_eq!(
            request.headers.get("content-type").map(String::as_str),
            Some("application/json")
        );
        assert!(!request.headers.contains_key("authorization"));
    }

    #[tokio::test]
    async fn a_non_2xx_answer_fails_with_the_body_kept_like_http_request() {
        let server = TestServer::respond_always(StubResponse::json(
            404,
            &json!({ "error": "no such address" }),
        ))
        .await;
        let collection = collection(&server.url, Vec::new());
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(
            error.contains("404") && error.contains("no such address"),
            "{error}"
        );
    }

    #[tokio::test]
    async fn an_operation_that_left_the_collection_fails_naming_both() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, Vec::new());
        let cfg = config(json!({}));
        let node = reference("deleteAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(
            error.contains("deleteAddress") && error.contains("Addresses"),
            "{error}"
        );
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn a_missing_required_parameter_fails_before_any_call_and_names_it() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, Vec::new());
        let cfg = config(json!({ "query": { "limit": 5 } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(error.contains("path.id"), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn a_parameter_the_operation_no_longer_declares_fails_before_any_call() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, Vec::new());
        let cfg = config(json!({ "path": { "id": "1" }, "query": { "sort": "asc" } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(error.contains("query.sort"), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn the_target_comes_from_the_stored_collection_never_from_the_node() {
        let honest = TestServer::respond_always(StubResponse::text(200, "stored")).await;
        let decoy = TestServer::respond_always(StubResponse::text(200, "decoy")).await;
        let collection = collection(&honest.url, Vec::new());
        let cfg = config(json!({
            "path": { "id": "1" },
            "url": format!("{}/stolen", decoy.url),
            "method": "DELETE",
            "base_url": decoy.url
        }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        assert_eq!(honest.last_request().method, "GET");
        assert_eq!(honest.last_request().path, "/addresses/1");
        assert!(decoy.requests().is_empty());
    }

    #[tokio::test]
    async fn headers_repeated_under_another_case_fail_before_any_call() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, Vec::new());
        let cfg = config(json!({
            "path": { "id": "1" },
            "headers": { "X-Trace": "a", "x-trace": "b" }
        }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(error.contains("headers.x-trace"), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn dot_segments_and_empty_path_values_fail_before_any_call() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, Vec::new());
        let node = reference("getAddress");

        for value in ["..", ".", ""] {
            let cfg = config(json!({ "path": { "id": value } }));

            let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
                .perform(&collection, &node, &input(&cfg, &node, None))
                .await;

            let error = failure(outcome);
            assert!(error.contains("path.id"), "{value:?}: {error}");
        }
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn a_base_url_on_a_private_address_is_refused_at_run_time() {
        let collection = collection("http://10.255.255.1:9", Vec::new());
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome =
            CollectionOperationConnector::new(lazy_usecase(), PrivateNetworkAccess::Denied)
                .perform(&collection, &node, &input(&cfg, &node, None))
                .await;

        let error = failure(outcome);
        assert!(error.contains("private"), "{error}");
    }

    #[tokio::test]
    async fn a_literal_ipv6_base_url_is_refused_at_run_time() {
        for (base_url, reason) in [
            ("http://[::1]:9", "loopback"),
            ("http://[::ffff:a9fe:a9fe]", "metadata"),
            ("http://[fd00:ec2::254]", "metadata"),
        ] {
            let collection = collection(base_url, Vec::new());
            let cfg = config(json!({ "path": { "id": "1" } }));
            let node = reference("getAddress");

            let outcome =
                CollectionOperationConnector::new(lazy_usecase(), PrivateNetworkAccess::Allowed)
                    .perform(&collection, &node, &input(&cfg, &node, None))
                    .await;

            let error = failure(outcome);
            assert!(error.contains(reason), "{base_url}: {error}");
        }
    }

    #[tokio::test]
    async fn the_cloud_metadata_endpoint_is_refused_even_when_private_access_is_allowed() {
        let collection = collection("http://169.254.169.254", Vec::new());
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome =
            CollectionOperationConnector::new(lazy_usecase(), PrivateNetworkAccess::Allowed)
                .perform(&collection, &node, &input(&cfg, &node, None))
                .await;

        let error = failure(outcome);
        assert!(error.contains("metadata"), "{error}");
    }

    #[tokio::test]
    async fn a_name_that_resolves_to_loopback_is_refused_by_the_guarded_resolver() {
        let collection = collection("http://localhost:9", Vec::new());
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome =
            CollectionOperationConnector::new(lazy_usecase(), PrivateNetworkAccess::Allowed)
                .perform(&collection, &node, &input(&cfg, &node, None))
                .await;

        let error = failure(outcome);
        assert!(error.contains("loopback"), "{error}");
    }

    #[tokio::test]
    async fn a_base_url_that_is_not_an_http_address_fails_and_names_the_collection() {
        for base_url in ["", "/v1", "ftp://files.example.com", "not a url"] {
            let collection = Collection {
                base_url: base_url.to_owned(),
                ..collection("https://api.example.com", Vec::new())
            };
            let cfg = config(json!({ "path": { "id": "1" } }));
            let node = reference("getAddress");

            let outcome =
                CollectionOperationConnector::new(lazy_usecase(), PrivateNetworkAccess::Denied)
                    .perform(&collection, &node, &input(&cfg, &node, None))
                    .await;

            let error = failure(outcome);
            assert!(error.contains("Addresses"), "{base_url}: {error}");
        }
    }

    #[tokio::test]
    async fn an_oauth2_collection_is_refused_with_the_stable_code_and_no_call_is_made() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(&server.url, vec![oauth2()]);
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(
                &collection,
                &node,
                &input(&cfg, &node, Some(Uuid::from_u128(5))),
            )
            .await;

        let error = failure(outcome);
        assert!(error.starts_with("unsupported_security_scheme"), "{error}");
        assert!(error.contains("petstore_auth"), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn an_api_key_in_a_cookie_is_refused_with_the_stable_code() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(
            &server.url,
            vec![SecurityScheme {
                name: "key".to_owned(),
                kind: SecurityKind::ApiKey {
                    location: ApiKeyLocation::Cookie,
                    parameter_name: "session".to_owned(),
                },
            }],
        );
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        let error = failure(outcome);
        assert!(error.starts_with("unsupported_security_scheme"), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn an_api_key_in_the_query_without_a_credential_is_an_anonymous_call() {
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let collection = collection(
            &server.url,
            vec![SecurityScheme {
                name: "key".to_owned(),
                kind: SecurityKind::ApiKey {
                    location: ApiKeyLocation::Query,
                    parameter_name: "api_key".to_owned(),
                },
            }],
        );
        let cfg = config(json!({ "path": { "id": "1" } }));
        let node = reference("getAddress");

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .perform(&collection, &node, &input(&cfg, &node, None))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        assert!(!server.last_request().path.contains("api_key"));
    }

    #[tokio::test]
    async fn a_node_with_no_operation_reference_fails_without_touching_the_database() {
        let cfg = config(json!({}));
        let bare = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &cfg,
            credential_id: None,
            operation: None,
        };

        let outcome = CollectionOperationConnector::with_client(lazy_usecase(), test_client())
            .execute(bare)
            .await;

        assert!(failure(outcome).contains("operation reference"));
    }

    async fn make_pool() -> PgPool {
        crate::application::test_support::automation_pool().await
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn usecase_with_cipher(pool: PgPool) -> MestierUseCase {
        use crate::infrastructure::automation::webhook::secret::SecretCipher;
        use std::sync::Arc;

        let mut usecase = MestierUseCase::new(pool, default_authorizer(), EventHub::new());
        usecase.cipher = Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap()));
        usecase
    }

    async fn credential(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        kind: &str,
        data: Value,
    ) -> Uuid {
        use crate::domain::automation::credential::{CreateCredentialCommand, CredentialOrigin};

        usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id,
                kind: kind.to_owned(),
                name: format!("{kind} credential"),
                origin: CredentialOrigin::Supplied,
                data: Some(data),
            })
            .await
            .unwrap()
            .0
            .id
    }

    async fn run_stored(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        collection_id: Uuid,
        operation_id: &str,
        cfg: &Map<String, Value>,
        credential_id: Option<Uuid>,
    ) -> ConnectorOutcome {
        let node = OperationRef {
            collection_id,
            operation_id: operation_id.to_owned(),
        };
        CollectionOperationConnector::with_client(usecase.clone(), test_client())
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: cfg,
                credential_id,
                operation: Some(&node),
            })
            .await
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_the_collection_changes_the_next_run_with_no_change_to_the_node() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-update").await;
        let usecase = usecase_with_cipher(pool);
        let server =
            TestServer::respond_always(StubResponse::json(200, &json!({ "ok": true }))).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(&server.url, Vec::new(), "/addresses/{id}"),
            })
            .await
            .unwrap();
        let cfg = config(json!({ "path": { "id": "7" } }));

        let before = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, None).await;
        usecase
            .replace_collection(ReplaceCollectionCommand {
                org_id,
                id: stored.id,
                draft: draft(&server.url, Vec::new(), "/v2/places/{id}"),
            })
            .await
            .unwrap();
        let after = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, None).await;

        assert!(
            matches!(before, ConnectorOutcome::Produced(_)),
            "{before:?}"
        );
        assert!(matches!(after, ConnectorOutcome::Produced(_)), "{after:?}");
        let paths: Vec<String> = server.requests().into_iter().map(|r| r.path).collect();
        assert_eq!(paths, ["/addresses/7", "/v2/places/7"]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_deleted_collection_fails_the_run_and_names_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-gone").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(&server.url, Vec::new(), "/addresses/{id}"),
            })
            .await
            .unwrap();
        usecase.delete_collection(org_id, stored.id).await.unwrap();
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, None).await;

        let error = failure(outcome);
        assert!(error.contains(&stored.id.to_string()), "{error}");
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_collection_of_another_organization_is_not_reachable_from_a_run() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-own").await;
        let stranger = seed_organization(&pool, "collection-stranger").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let foreign = usecase
            .import_collection(ImportCollectionCommand {
                org_id: stranger,
                draft: draft(&server.url, Vec::new(), "/addresses/{id}"),
            })
            .await
            .unwrap();
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome = run_stored(&usecase, org_id, foreign.id, "getAddress", &cfg, None).await;

        assert!(failure(outcome).contains("no longer exists"));
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_bearer_scheme_and_a_bearer_credential_send_an_authorization_header() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-bearer").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "bearerAuth".to_owned(),
                        kind: SecurityKind::Http {
                            scheme: "bearer".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let token = credential(
            &usecase,
            org_id,
            "bearer_token",
            json!({ "token": "abc123" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome =
            run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(token)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        assert_eq!(
            server
                .last_request()
                .headers
                .get("authorization")
                .map(String::as_str),
            Some("Bearer abc123")
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_basic_scheme_and_an_http_basic_credential_send_a_basic_header() {
        use base64::{Engine, engine::general_purpose::STANDARD};

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-basic").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "basicAuth".to_owned(),
                        kind: SecurityKind::Http {
                            scheme: "basic".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let login = credential(
            &usecase,
            org_id,
            "http_basic",
            json!({ "username": "bot", "password": "secret" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome =
            run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(login)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let expected = format!("Basic {}", STANDARD.encode("bot:secret"));
        assert_eq!(
            server.last_request().headers.get("authorization"),
            Some(&expected)
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_api_key_in_a_header_is_sent_under_the_name_the_collection_declares() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-apikey").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "keyAuth".to_owned(),
                        kind: SecurityKind::ApiKey {
                            location: ApiKeyLocation::Header,
                            parameter_name: "X-API-Key".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let key = credential(
            &usecase,
            org_id,
            "http_header",
            json!({ "header_name": "X-Something-Else", "header_value": "shh" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(key)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(
            request.headers.get("x-api-key").map(String::as_str),
            Some("shh")
        );
        assert!(!request.headers.contains_key("x-something-else"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_api_key_in_the_query_is_sent_under_the_name_the_collection_declares() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-query-key").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "keyAuth".to_owned(),
                        kind: SecurityKind::ApiKey {
                            location: ApiKeyLocation::Query,
                            parameter_name: "key".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let key = credential(
            &usecase,
            org_id,
            "api_key_query",
            json!({ "param_name": "something_else", "param_value": "shh" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(key)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(request.path, "/addresses/7?key=shh");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_query_credential_against_a_header_scheme_is_a_mismatch_and_sends_nothing() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-query-mismatch").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "keyAuth".to_owned(),
                        kind: SecurityKind::ApiKey {
                            location: ApiKeyLocation::Header,
                            parameter_name: "X-Key".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let key = credential(
            &usecase,
            org_id,
            "api_key_query",
            json!({ "param_name": "key", "param_value": "shh" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome = run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(key)).await;

        assert!(
            failure(outcome).contains("the credential offers"),
            "mismatch"
        );
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_identification_replaces_a_header_of_the_same_name_coming_from_the_config() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-override").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let with_headers = |security: Vec<SecurityScheme>| {
            let mut draft = draft(&server.url, security, "/addresses/{id}");
            draft.operations[0].parameters.push(parameter(
                "Authorization",
                ParameterLocation::Header,
                false,
            ));
            draft.operations[0].parameters.push(parameter(
                "X-API-Key",
                ParameterLocation::Header,
                false,
            ));
            draft
        };
        let bearer = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: with_headers(vec![SecurityScheme {
                    name: "bearerAuth".to_owned(),
                    kind: SecurityKind::Http {
                        scheme: "bearer".to_owned(),
                    },
                }]),
            })
            .await
            .unwrap();
        let keyed = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: with_headers(vec![SecurityScheme {
                    name: "keyAuth".to_owned(),
                    kind: SecurityKind::ApiKey {
                        location: ApiKeyLocation::Header,
                        parameter_name: "X-API-Key".to_owned(),
                    },
                }]),
            })
            .await
            .unwrap();
        let token = credential(
            &usecase,
            org_id,
            "bearer_token",
            json!({ "token": "abc123" }),
        )
        .await;
        let key = credential(
            &usecase,
            org_id,
            "http_header",
            json!({ "header_name": "X-Whatever", "header_value": "shh" }),
        )
        .await;
        let cfg = config(json!({
            "path": { "id": "7" },
            "headers": { "authorization": "Bearer evil", "x-api-key": "evil" }
        }));

        let outcome =
            run_stored(&usecase, org_id, bearer.id, "getAddress", &cfg, Some(token)).await;
        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(request.header_values("authorization"), ["Bearer abc123"]);

        let outcome = run_stored(&usecase, org_id, keyed.id, "getAddress", &cfg, Some(key)).await;
        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        let request = server.last_request();
        assert_eq!(request.header_values("x-api-key"), ["shh"]);
        assert_eq!(request.header_values("authorization"), ["Bearer evil"]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_the_declared_schemes_cannot_use_is_a_named_mismatch() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-mismatch").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(
                    &server.url,
                    vec![SecurityScheme {
                        name: "bearerAuth".to_owned(),
                        kind: SecurityKind::Http {
                            scheme: "bearer".to_owned(),
                        },
                    }],
                    "/addresses/{id}",
                ),
            })
            .await
            .unwrap();
        let login = credential(
            &usecase,
            org_id,
            "http_basic",
            json!({ "username": "bot", "password": "secret" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome =
            run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(login)).await;

        let error = failure(outcome);
        assert!(
            error.contains("bearerAuth") && error.contains("basic"),
            "{error}"
        );
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_collection_declaring_no_scheme_applies_the_credential_as_it_is() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-noscheme").await;
        let usecase = usecase_with_cipher(pool);
        let server = TestServer::respond_always(StubResponse::text(200, "ok")).await;
        let stored = usecase
            .import_collection(ImportCollectionCommand {
                org_id,
                draft: draft(&server.url, Vec::new(), "/addresses/{id}"),
            })
            .await
            .unwrap();
        let token = credential(
            &usecase,
            org_id,
            "bearer_token",
            json!({ "token": "abc123" }),
        )
        .await;
        let cfg = config(json!({ "path": { "id": "7" } }));

        let outcome =
            run_stored(&usecase, org_id, stored.id, "getAddress", &cfg, Some(token)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Produced(_)),
            "{outcome:?}"
        );
        assert_eq!(
            server
                .last_request()
                .headers
                .get("authorization")
                .map(String::as_str),
            Some("Bearer abc123")
        );
    }
}
