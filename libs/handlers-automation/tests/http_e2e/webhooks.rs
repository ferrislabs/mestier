use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    client, harness,
    triggers::{graph_of, save, unique_path},
};

async fn new_workflow(app: &harness::App) -> String {
    let created: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Webhooks" }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("json");
    created["data"]["id"].as_str().expect("an id").to_owned()
}

fn webhook_kind(
    path: &str,
    credential_id: Option<Uuid>,
    contract: Value,
    on_invalid: &str,
) -> Value {
    json!({
        "Webhook": {
            "method": "POST",
            "path": path,
            "credential_id": credential_id,
            "contract": contract,
            "on_invalid": on_invalid
        }
    })
}

async fn published(app: &harness::App, kind: Value) -> String {
    let workflow_id = new_workflow(app).await;
    let response = save(app, &workflow_id, graph_of(vec![("t1", kind)])).await;
    assert_eq!(response.status(), 201, "{}", response.text().await.unwrap());
    workflow_id
}

async fn credential(app: &harness::App, kind: &str, data: Value) -> Uuid {
    let created: Value = client()
        .post(app.credentials_url())
        .bearer_auth(&app.token)
        .json(&json!({ "kind": kind, "name": "Webhook", "origin": "supplied", "data": data }))
        .send()
        .await
        .expect("the api answers the credential create call")
        .json()
        .await
        .expect("json");
    created["data"]["id"]
        .as_str()
        .unwrap_or_else(|| panic!("a credential id in {created}"))
        .parse()
        .expect("a uuid")
}

async fn run_count(app: &harness::App, workflow_id: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM automation.run WHERE workflow_id = $1")
        .bind(workflow_id.parse::<Uuid>().expect("a uuid"))
        .fetch_one(&app.pool)
        .await
        .expect("count the runs")
}

async fn post_json(app: &harness::App, path: &str, body: &Value) -> reqwest::Response {
    client()
        .post(app.webhook_url(path))
        .json(body)
        .send()
        .await
        .expect("the api answers the webhook call")
}

type Headers<'a> = Vec<(&'a str, &'a str)>;

#[derive(Debug, PartialEq)]
struct Seen {
    status: u16,
    content_type: Option<String>,
    body: String,
}

async fn seen(response: reqwest::Response) -> Seen {
    Seen {
        status: response.status().as_u16(),
        content_type: response
            .headers()
            .get("content-type")
            .map(|value| value.to_str().unwrap().to_owned()),
        body: response.text().await.expect("a body"),
    }
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_valid_post_starts_a_run_whose_payload_is_the_body() {
    let app = harness::start().await;
    let path = unique_path("paiement/recu");
    let contract =
        json!([{ "name": "montant", "kind": "Number", "required": true, "example": "2340" }]);
    let workflow_id = published(&app, webhook_kind(&path, None, contract, "Reject")).await;
    let body = json!({ "montant": 2340, "extra": { "kept": true } });

    let response = post_json(&app, &path, &body).await;

    assert_eq!(response.status(), 202);
    let accepted: Value = response.json().await.expect("json");
    assert_eq!(accepted.as_object().unwrap().len(), 1, "{accepted}");
    let run_id = accepted["run_id"].as_str().expect("a run id").to_owned();
    let run: Value = client()
        .get(app.run_url(&run_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the run read")
        .json()
        .await
        .expect("json");
    assert_eq!(run["data"]["trigger_payload"], body);
    assert_eq!(run["data"]["trigger_id"], "t1");
    assert_eq!(run["data"]["workflow_id"], workflow_id);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_body_breaking_the_contract_is_refused_with_its_keys_or_accepted_as_configured() {
    let app = harness::start().await;
    let contract = json!([
        { "name": "montant", "kind": "Number", "required": true },
        { "name": "evenement", "kind": "Text", "required": true },
        { "name": "client", "kind": "Object", "required": false }
    ]);
    let rejecting_path = unique_path("reject");
    let accepting_path = unique_path("accept");
    let rejecting = published(
        &app,
        webhook_kind(&rejecting_path, None, contract.clone(), "Reject"),
    )
    .await;
    let accepting = published(
        &app,
        webhook_kind(&accepting_path, None, contract, "Accept"),
    )
    .await;
    let broken = json!({ "montant": "beaucoup", "client": [] });

    let refused = post_json(&app, &rejecting_path, &broken).await;
    assert_eq!(refused.status(), 400);
    let body: Value = refused.json().await.expect("json");
    assert_eq!(body["code"], "E_WEBHOOK_BODY_INVALID");
    assert_eq!(body["keys"], json!(["montant", "evenement", "client"]));
    assert_eq!(run_count(&app, &rejecting).await, 0);

    let accepted = post_json(&app, &accepting_path, &broken).await;
    assert_eq!(accepted.status(), 202);
    assert_eq!(run_count(&app, &accepting).await, 1);

    let missing = post_json(&app, &rejecting_path, &json!({ "evenement": "x" })).await;
    let body: Value = missing.json().await.expect("json");
    assert_eq!(body["keys"], json!(["montant"]));

    let not_an_object = post_json(&app, &rejecting_path, &json!([1, 2])).await;
    assert_eq!(not_an_object.status(), 400);
    let body: Value = not_an_object.json().await.expect("json");
    assert_eq!(body["keys"], json!(["montant", "evenement"]));

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_body_that_is_not_json_or_not_a_json_type_is_refused_before_it_reaches_the_contract() {
    let app = harness::start().await;
    let path = unique_path("types");
    let workflow_id = published(&app, webhook_kind(&path, None, json!([]), "Accept")).await;
    let send = |content_type: Option<&'static str>, body: &'static str| {
        let mut request = client().post(app.webhook_url(&path)).body(body);
        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }
        async move {
            request
                .send()
                .await
                .expect("the api answers")
                .status()
                .as_u16()
        }
    };

    assert_eq!(send(Some("text/plain"), "{}").await, 415);
    assert_eq!(
        send(Some("application/x-www-form-urlencoded"), "a=b").await,
        415
    );
    assert_eq!(send(None, "{\"a\":1}").await, 415);
    assert_eq!(send(Some("application/json"), "{not json").await, 400);
    assert_eq!(send(Some("application/json"), "").await, 400);
    assert_eq!(run_count(&app, &workflow_id).await, 0);

    assert_eq!(
        send(Some("application/json; charset=utf-8"), "{\"a\":1}").await,
        202
    );
    assert_eq!(
        send(Some("application/vnd.api+json"), "{\"a\":1}").await,
        202
    );
    assert_eq!(send(None, "").await, 202);
    assert_eq!(run_count(&app, &workflow_id).await, 3);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_body_over_one_mebibyte_is_refused_and_one_at_the_limit_is_taken() {
    let app = harness::start().await;
    let path = unique_path("size");
    let workflow_id = published(&app, webhook_kind(&path, None, json!([]), "Accept")).await;
    let limit = 1024 * 1024;
    let sized = |bytes: usize| {
        let filler = "x".repeat(bytes - "{\"a\":\"\"}".len());
        format!("{{\"a\":\"{filler}\"}}")
    };

    let at_the_limit = client()
        .post(app.webhook_url(&path))
        .header("content-type", "application/json")
        .body(sized(limit))
        .send()
        .await
        .expect("the api answers");
    let over = client()
        .post(app.webhook_url(&path))
        .header("content-type", "application/json")
        .body(sized(limit + 1))
        .send()
        .await
        .expect("the api answers");

    assert_eq!(at_the_limit.status(), 202);
    assert_eq!(over.status(), 413);
    let body: Value = over.json().await.expect("json");
    assert_eq!(body["code"], "E_WEBHOOK_BODY_TOO_LARGE");
    assert_eq!(run_count(&app, &workflow_id).await, 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn each_credential_kind_is_checked_and_a_refusal_starts_nothing() {
    let app = harness::start().await;
    let bearer = credential(&app, "bearer_token", json!({ "token": "s3cret-token" })).await;
    let basic = credential(
        &app,
        "http_basic",
        json!({ "username": "alice", "password": "p4ss" }),
    )
    .await;
    let header = credential(
        &app,
        "http_header",
        json!({ "header_name": "X-Signature", "header_value": "sig-123" }),
    )
    .await;
    let cases: Vec<(Uuid, Headers, Vec<Headers>)> = vec![
        (
            bearer,
            vec![("authorization", "Bearer s3cret-token")],
            vec![
                vec![],
                vec![("authorization", "Bearer s3cret-token-2")],
                vec![("authorization", "Basic s3cret-token")],
                vec![("x-signature", "sig-123")],
            ],
        ),
        (
            basic,
            vec![("authorization", "Basic YWxpY2U6cDRzcw==")],
            vec![
                vec![],
                vec![("authorization", "Basic YWxpY2U6d3Jvbmc=")],
                vec![("authorization", "Bearer YWxpY2U6cDRzcw==")],
            ],
        ),
        (
            header,
            vec![("x-signature", "sig-123")],
            vec![
                vec![],
                vec![("x-signature", "sig-124")],
                vec![("authorization", "Bearer sig-123")],
            ],
        ),
    ];

    for (credential_id, good, bad) in cases {
        let path = unique_path("secured");
        let workflow_id = published(
            &app,
            webhook_kind(&path, Some(credential_id), json!([]), "Reject"),
        )
        .await;
        let call = |headers: &[(&str, &str)]| {
            let mut request = client().post(app.webhook_url(&path)).json(&json!({}));
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
            async move { request.send().await.expect("the api answers") }
        };

        for headers in &bad {
            let refused = call(headers).await;
            assert_eq!(refused.status(), 401, "{headers:?}");
            let text = refused.text().await.unwrap();
            for secret in ["s3cret-token", "p4ss", "sig-123"] {
                assert!(!text.contains(secret), "{text}");
            }
        }
        assert_eq!(run_count(&app, &workflow_id).await, 0);
        assert_eq!(call(&good).await.status(), 202);
        assert_eq!(run_count(&app, &workflow_id).await, 1);
    }

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_credential_is_checked_before_the_body_is_read() {
    let app = harness::start().await;
    let token = credential(&app, "bearer_token", json!({ "token": "s3cret-token" })).await;
    let path = unique_path("order");
    let contract = json!([{ "name": "montant", "kind": "Number", "required": true }]);
    let workflow_id = published(&app, webhook_kind(&path, Some(token), contract, "Reject")).await;
    let huge = format!("{{\"a\":\"{}\"}}", "x".repeat(1024 * 1024 + 10));

    let too_large = client()
        .post(app.webhook_url(&path))
        .header("content-type", "application/json")
        .body(huge)
        .send()
        .await
        .expect("the api answers");
    let wrong_type = client()
        .post(app.webhook_url(&path))
        .header("content-type", "text/plain")
        .body("hello")
        .send()
        .await
        .expect("the api answers");
    let not_json = client()
        .post(app.webhook_url(&path))
        .header("content-type", "application/json")
        .body("{broken")
        .send()
        .await
        .expect("the api answers");
    let broken_contract = client()
        .post(app.webhook_url(&path))
        .json(&json!({ "montant": "x" }))
        .send()
        .await
        .expect("the api answers");

    for response in [too_large, wrong_type, not_json, broken_contract] {
        assert_eq!(response.status(), 401);
    }
    assert_eq!(run_count(&app, &workflow_id).await, 0);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_unknown_path_a_wrong_method_a_disabled_and_an_archived_workflow_answer_alike() {
    let app = harness::start().await;
    let path = unique_path("hidden");
    let token = credential(&app, "bearer_token", json!({ "token": "s3cret-token" })).await;
    let workflow_id = published(&app, webhook_kind(&path, Some(token), json!([]), "Reject")).await;
    let authorized =
        |request: reqwest::RequestBuilder| request.header("authorization", "Bearer s3cret-token");
    let reference = seen(
        authorized(client().post(app.webhook_url(&unique_path("nothing"))))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(reference.status, 404);

    for method in ["GET", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"] {
        let response =
            authorized(client().request(method.parse().unwrap(), app.webhook_url(&path)))
                .send()
                .await
                .unwrap();
        let mut observed = seen(response).await;
        if method == "HEAD" {
            observed.body = reference.body.clone();
        }
        assert_eq!(observed, reference, "{method}");
    }
    for odd in ["Upper-Case", "a//b", "webhooks/x", "a.b", ""] {
        let response = authorized(client().post(format!("{}/{odd}", app.webhook_base())))
            .send()
            .await
            .unwrap();
        assert_eq!(seen(response).await, reference, "`{odd}`");
    }

    let patch = |body: Value| {
        let url = app.workflow_url(&workflow_id);
        let token = app.token.clone();
        async move {
            let response = client()
                .patch(url)
                .bearer_auth(token)
                .json(&body)
                .send()
                .await
                .expect("the api answers the update");
            assert!(response.status().is_success(), "{}", response.status());
        }
    };
    let call = || async {
        seen(
            authorized(client().post(app.webhook_url(&path)))
                .json(&json!({}))
                .send()
                .await
                .unwrap(),
        )
        .await
    };

    assert_eq!(call().await.status, 202);
    patch(json!({ "enabled": false })).await;
    assert_eq!(call().await, reference);
    patch(json!({ "enabled": true })).await;
    assert_eq!(call().await.status, 202);
    patch(json!({ "archived": true })).await;
    assert_eq!(call().await, reference);
    assert_eq!(run_count(&app, &workflow_id).await, 2);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn two_workflows_cannot_answer_the_same_path_and_the_path_moves_when_it_is_freed() {
    let app = harness::start().await;
    let path = unique_path("shared");
    let first = published(&app, webhook_kind(&path, None, json!([]), "Reject")).await;
    let second = new_workflow(&app).await;

    let refused = save(
        &app,
        &second,
        graph_of(vec![("t1", webhook_kind(&path, None, json!([]), "Reject"))]),
    )
    .await;
    assert_eq!(refused.status(), 409);
    assert_eq!(post_json(&app, &path, &json!({})).await.status(), 202);
    assert_eq!(run_count(&app, &first).await, 1);
    assert_eq!(run_count(&app, &second).await, 0);

    assert_eq!(
        save(&app, &first, graph_of(vec![("t1", json!("Manual"))]))
            .await
            .status(),
        201
    );
    assert_eq!(post_json(&app, &path, &json!({})).await.status(), 404);
    assert_eq!(
        save(
            &app,
            &second,
            graph_of(vec![("t1", webhook_kind(&path, None, json!([]), "Reject"))]),
        )
        .await
        .status(),
        201
    );
    assert_eq!(post_json(&app, &path, &json!({})).await.status(), 202);
    assert_eq!(run_count(&app, &second).await, 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn each_method_answers_only_on_the_route_that_declares_it() {
    let app = harness::start().await;
    let path = unique_path("verbs");
    let workflow_id = published(
        &app,
        json!({ "Webhook": { "method": "GET", "path": path } }),
    )
    .await;

    let get = client().get(app.webhook_url(&path)).send().await.unwrap();
    let post = client()
        .post(app.webhook_url(&path))
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(get.status(), 202);
    assert_eq!(post.status(), 404);
    assert_eq!(run_count(&app, &workflow_id).await, 1);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_rate_limit_answers_before_the_route_is_even_looked_up() {
    let setup = harness::start().await;
    let path = unique_path("limited");
    published(&setup, webhook_kind(&path, None, json!([]), "Reject")).await;
    let limited = harness::start_with_rate_limit(3).await;
    let from = format!("203.0.113.{}", Uuid::now_v7().as_u128() % 250 + 1);
    let call = |path: String| {
        let request = client()
            .post(limited.webhook_url(&path))
            .header("x-forwarded-for", from.clone())
            .json(&json!({}));
        async move { request.send().await.expect("the api answers") }
    };

    let mut statuses = Vec::new();
    for _ in 0..3 {
        statuses.push(call(path.clone()).await.status().as_u16());
    }
    let known = call(path.clone()).await;
    let unknown = call(unique_path("unknown")).await;

    assert_eq!(statuses, [202, 202, 202]);
    assert_eq!(known.status(), 429);
    assert!(known.headers().get("retry-after").is_some());
    assert_eq!(unknown.status(), 429);

    limited.cleanup().await;
    setup.cleanup().await;
}

async fn raw_exchange(app: &harness::App, request: Vec<u8>) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let address = app.base_url.trim_start_matches("http://");
    let mut stream = tokio::net::TcpStream::connect(address)
        .await
        .expect("connect to the api");
    let _ = stream.write_all(&request).await;
    let mut answer = Vec::new();
    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read_buf(&mut answer),
    )
    .await;
    String::from_utf8_lossy(&answer).into_owned()
}

fn head(path: &str, extra: &str) -> String {
    format!(
        "POST /webhooks/{path} HTTP/1.1\r\nHost: webhook.test\r\nContent-Type: application/json\r\nConnection: close\r\n{extra}\r\n"
    )
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_body_streamed_past_the_limit_without_a_length_is_cut_off() {
    let app = harness::start().await;
    let path = unique_path("chunked");
    let workflow_id = published(&app, webhook_kind(&path, None, json!([]), "Accept")).await;
    let limit = 1024 * 1024;
    let mut request = head(&path, "Transfer-Encoding: chunked\r\n").into_bytes();
    for size in [limit / 2, limit / 2 + 10] {
        request.extend_from_slice(format!("{size:x}\r\n").as_bytes());
        request.extend(std::iter::repeat_n(b'a', size));
        request.extend_from_slice(b"\r\n");
    }
    request.extend_from_slice(b"0\r\n\r\n");

    let answer = raw_exchange(&app, request).await;

    assert!(answer.starts_with("HTTP/1.1 413"), "{answer}");
    assert_eq!(run_count(&app, &workflow_id).await, 0);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_declared_length_past_the_limit_is_refused_before_any_of_the_body_arrives() {
    let app = harness::start().await;
    let path = unique_path("declared");
    published(&app, webhook_kind(&path, None, json!([]), "Accept")).await;

    let answer = raw_exchange(
        &app,
        head(&path, "Content-Length: 5000000\r\n").into_bytes(),
    )
    .await;

    assert!(answer.starts_with("HTTP/1.1 413"), "{answer}");

    app.cleanup().await;
}
