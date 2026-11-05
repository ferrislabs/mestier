use serde_json::{Value, json};

use crate::{client, harness};

async fn post_folder(app: &harness::App, token: &str, body: Value) -> reqwest::Response {
    client()
        .post(app.folders_url())
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the folder create call")
}

async fn created_folder(app: &harness::App, body: Value) -> Value {
    let response = post_folder(app, &app.token, body).await;
    assert_eq!(response.status(), 201);
    let envelope: Value = response.json().await.expect("the answer is json");
    envelope["data"].clone()
}

fn id_of(folder: &Value) -> &str {
    folder["id"].as_str().expect("a folder carries an id")
}

async fn code_of(response: reqwest::Response) -> (u16, Value) {
    let status = response.status().as_u16();
    (status, response.json().await.expect("the error is json"))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn there_is_no_default_folder_and_a_created_one_has_the_contracted_shape() {
    let app = harness::start().await;

    let listed: Value = client()
        .get(app.folders_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the folder list call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(listed["data"], json!([]));

    let folder = created_folder(&app, json!({ "name": "  Facturation " })).await;

    let mut keys: Vec<&str> = folder
        .as_object()
        .expect("a folder is an object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["created_at", "id", "name", "parent_id"]);
    assert_eq!(folder["name"], "Facturation");
    assert_eq!(folder["parent_id"], Value::Null);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn the_list_is_flat_and_carries_each_parent() {
    let app = harness::start().await;
    let root = created_folder(&app, json!({ "name": "Root" })).await;
    let child = created_folder(&app, json!({ "name": "Child", "parent_id": root["id"] })).await;

    let listed: Value = client()
        .get(app.folders_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the folder list call")
        .json()
        .await
        .expect("the answer is json");

    let folders = listed["data"].as_array().expect("a list");
    assert_eq!(folders.len(), 2);
    let listed_child = folders
        .iter()
        .find(|folder| folder["id"] == child["id"])
        .expect("the child is listed");
    assert_eq!(listed_child["parent_id"], root["id"]);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn invalid_names_and_duplicate_siblings_are_refused() {
    let app = harness::start().await;
    let root = created_folder(&app, json!({ "name": "Facturation" })).await;

    for name in ["", "   ", &"x".repeat(81)] {
        let (status, body) =
            code_of(post_folder(&app, &app.token, json!({ "name": name })).await).await;
        assert_eq!(
            (status, &body["code"]),
            (400, &json!("E_VALIDATION")),
            "{name:?}"
        );
    }

    let (status, body) =
        code_of(post_folder(&app, &app.token, json!({ "name": "FACTURATION" })).await).await;
    assert_eq!((status, &body["code"]), (409, &json!("E_CONFLICT")));

    let below = post_folder(
        &app,
        &app.token,
        json!({ "name": "FACTURATION", "parent_id": root["id"] }),
    )
    .await;
    assert_eq!(below.status(), 201, "the same name is fine one level down");

    let (status, _) = code_of(
        post_folder(
            &app,
            &app.token,
            json!({ "name": "Orphan", "parent_id": uuid::Uuid::now_v7() }),
        )
        .await,
    )
    .await;
    assert_eq!(status, 409, "a parent that does not exist");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_folder_is_renamed_moved_and_refused_a_loop() {
    let app = harness::start().await;
    let root = created_folder(&app, json!({ "name": "Root" })).await;
    let child = created_folder(&app, json!({ "name": "Child", "parent_id": root["id"] })).await;

    let renamed: Value = client()
        .patch(app.folder_url(id_of(&child)))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Renamed" }))
        .send()
        .await
        .expect("the api answers the rename")
        .json()
        .await
        .expect("json");
    assert_eq!(renamed["data"]["name"], "Renamed");
    assert_eq!(
        renamed["data"]["parent_id"], root["id"],
        "an absent parent_id changes nothing"
    );

    let to_root: Value = client()
        .patch(app.folder_url(id_of(&child)))
        .bearer_auth(&app.token)
        .json(&json!({ "parent_id": null }))
        .send()
        .await
        .expect("the api answers the move")
        .json()
        .await
        .expect("json");
    assert_eq!(to_root["data"]["parent_id"], Value::Null);

    let under_child = client()
        .patch(app.folder_url(id_of(&root)))
        .bearer_auth(&app.token)
        .json(&json!({ "parent_id": child["id"] }))
        .send()
        .await
        .expect("the api answers the move");
    assert_eq!(
        under_child.status(),
        200,
        "child left the root, so this is no loop"
    );

    let loop_back = client()
        .patch(app.folder_url(id_of(&child)))
        .bearer_auth(&app.token)
        .json(&json!({ "parent_id": root["id"] }))
        .send()
        .await
        .expect("the api answers the move");
    assert_eq!(loop_back.status(), 409);

    let into_itself = client()
        .patch(app.folder_url(id_of(&root)))
        .bearer_auth(&app.token)
        .json(&json!({ "parent_id": root["id"] }))
        .send()
        .await
        .expect("the api answers the move");
    assert_eq!(into_itself.status(), 409);

    let missing = client()
        .patch(app.folder_url(&uuid::Uuid::now_v7().to_string()))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Nope" }))
        .send()
        .await
        .expect("the api answers the rename");
    assert_eq!(missing.status(), 404);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_non_empty_folder_answers_its_own_code_until_it_is_emptied() {
    let app = harness::start().await;
    let parent = created_folder(&app, json!({ "name": "Parent" })).await;
    let child = created_folder(&app, json!({ "name": "Child", "parent_id": parent["id"] })).await;
    let workflow: Value = client()
        .post(app.workflows_url())
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Filed", "folder_id": child["id"] }))
        .send()
        .await
        .expect("the api answers the workflow create call")
        .json()
        .await
        .expect("json");
    let workflow_id = workflow["data"]["id"].as_str().expect("an id");

    let with_child = client()
        .delete(app.folder_url(id_of(&parent)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete");
    let (status, body) = code_of(with_child).await;
    assert_eq!((status, &body["code"]), (409, &json!("E_FOLDER_NOT_EMPTY")));
    assert_eq!(body["status"], 409);

    let with_workflow = client()
        .delete(app.folder_url(id_of(&child)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete");
    let (status, body) = code_of(with_workflow).await;
    assert_eq!((status, &body["code"]), (409, &json!("E_FOLDER_NOT_EMPTY")));

    let archived = client()
        .patch(app.workflow_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({ "archived": true }))
        .send()
        .await
        .expect("the api answers the archive");
    assert_eq!(archived.status(), 200);
    let still_held = client()
        .delete(app.folder_url(id_of(&child)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete");
    assert_eq!(
        still_held.status(),
        409,
        "an archived workflow still fills its folder"
    );

    let moved_out = client()
        .patch(app.workflow_url(workflow_id))
        .bearer_auth(&app.token)
        .json(&json!({ "folder_id": null }))
        .send()
        .await
        .expect("the api answers the move");
    assert_eq!(moved_out.status(), 200);

    for folder in [&child, &parent] {
        let deleted = client()
            .delete(app.folder_url(id_of(folder)))
            .bearer_auth(&app.token)
            .send()
            .await
            .expect("the api answers the delete");
        assert_eq!(deleted.status(), 204);
    }

    let gone = client()
        .delete(app.folder_url(id_of(&parent)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete");
    assert_eq!(gone.status(), 404);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn folders_follow_the_automation_permission_split_and_stay_inside_their_organization() {
    let app = harness::start().await;
    let folder = created_folder(&app, json!({ "name": "Mine" })).await;

    let view_list = client()
        .get(app.folders_url())
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers the view-only list");
    assert_eq!(view_list.status(), 200);

    let view_create = post_folder(&app, &app.view_only_token, json!({ "name": "Nope" })).await;
    assert_eq!(view_create.status(), 403);
    let view_update = client()
        .patch(app.folder_url(id_of(&folder)))
        .bearer_auth(&app.view_only_token)
        .json(&json!({ "name": "Nope" }))
        .send()
        .await
        .expect("the api answers");
    assert_eq!(view_update.status(), 403);
    let view_delete = client()
        .delete(app.folder_url(id_of(&folder)))
        .bearer_auth(&app.view_only_token)
        .send()
        .await
        .expect("the api answers");
    assert_eq!(view_delete.status(), 403);

    let no_role = client()
        .get(app.folders_url())
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .expect("the api answers the no-role list");
    assert_eq!(no_role.status(), 403);

    let outsider = client()
        .get(app.folders_url())
        .bearer_auth(&app.outsider_token)
        .send()
        .await
        .expect("the api answers the outsider list");
    assert_eq!(
        outsider.status(),
        403,
        "holding the bits elsewhere is not membership here"
    );

    app.cleanup().await;
}
