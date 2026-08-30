//! End-to-end tests for #308's role management API.
//!
//! Real socket, real database, real auth. The fixture seeds one organization
//! with its seeded `owner` role (`MANAGE_ROLES | MANAGE_MEMBERS`), a member
//! with no role at all, and a second ordinary member — the assign-role
//! target.

mod harness;
mod issuer;

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn roles_are_listed_created_updated_and_deleted() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman",
            "permissions": ["VIEW_PLANNING", "MANAGE_PLANNING"],
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(created["data"]["name"], "foreman", "{created}");
    assert_eq!(created["data"]["is_seeded"], false, "{created}");
    let role_id = created["data"]["id"]
        .as_str()
        .expect("the created role has an id")
        .to_owned();

    let listed: serde_json::Value = client
        .get(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    let names: Vec<&str> = listed["data"]
        .as_array()
        .expect("the roles are an array")
        .iter()
        .map(|r| r["name"].as_str().expect("a role has a name"))
        .collect();
    assert!(
        names.contains(&"owner") && names.contains(&"foreman"),
        "expected both the seeded owner and the new role: {listed}"
    );

    let updated: serde_json::Value = client
        .patch(app.url(&format!("/roles/{role_id}")))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman-renamed",
            "permissions": ["VIEW_PLANNING"],
        }))
        .send()
        .await
        .expect("the api answers the update call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(updated["data"]["name"], "foreman-renamed", "{updated}");
    assert_eq!(
        updated["data"]["permissions"],
        serde_json::json!(["VIEW_PLANNING"]),
        "a custom role's name and permissions are both freely editable: {updated}"
    );

    let deleted = client
        .delete(app.url(&format!("/roles/{role_id}")))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert_eq!(deleted.status(), 204, "an unassigned custom role deletes");

    let listed_after: serde_json::Value = client
        .get(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    let ids_after: Vec<&str> = listed_after["data"]
        .as_array()
        .expect("the roles are an array")
        .iter()
        .map(|r| r["id"].as_str().expect("a role has an id"))
        .collect();
    assert!(
        !ids_after.contains(&role_id.as_str()),
        "the deleted role must not reappear: {listed_after}"
    );

    app.cleanup().await;
}

/// `role.manage` gates every role write — a member with no role at all is
/// refused, while the fixture's own owner, whose seeded role carries the
/// bit, still succeeds.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn role_manage_gates_role_writes() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "name": "foreman",
        "permissions": ["VIEW_PLANNING"],
    });

    let refused = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.restricted_token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the create call");
    assert_eq!(
        refused.status(),
        403,
        "a member with no role must be refused on a role write"
    );

    let created = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the create call");
    assert_eq!(
        created.status(),
        201,
        "the fixture's own owner, whose role carries role.manage, must still succeed"
    );

    app.cleanup().await;
}

/// #308's binding invariant: renaming a seeded role away from its name would
/// let the delete-protection guard miss it entirely afterward, so the name
/// itself is fixed — but its permissions stay editable, which is the whole
/// point of the feature.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_seeded_roles_name_is_fixed_but_its_permissions_stay_editable() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .patch(app.url(&format!("/roles/{}", app.owner_role_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "not-owner",
            "permissions": ["MANAGE_ROLES", "MANAGE_MEMBERS"],
        }))
        .send()
        .await
        .expect("the api answers the update call");
    assert_eq!(
        refused.status(),
        409,
        "renaming the seeded owner role must be refused"
    );

    let allowed: serde_json::Value = client
        .patch(app.url(&format!("/roles/{}", app.owner_role_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "owner",
            "permissions": ["MANAGE_ROLES", "MANAGE_MEMBERS", "VIEW_REPORTS"],
        }))
        .send()
        .await
        .expect("the api answers the update call")
        .json()
        .await
        .expect("the answer is json");
    // Canonical order (`Permissions::NAMED`'s declared order), not the order
    // posted — the response echoes what the bitfield holds, not the request.
    let mut permissions: Vec<&str> = allowed["data"]["permissions"]
        .as_array()
        .expect("permissions is an array")
        .iter()
        .map(|v| v.as_str().expect("a permission name"))
        .collect();
    permissions.sort_unstable();
    let mut expected = vec!["MANAGE_ROLES", "MANAGE_MEMBERS", "VIEW_REPORTS"];
    expected.sort_unstable();
    assert_eq!(
        permissions, expected,
        "the same name is not a rename, and the permission change must go through: {allowed}"
    );

    app.cleanup().await;
}

/// An organization that deletes `owner` has locked itself out — the seeded
/// role can never be deleted, whatever it is currently named.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_seeded_role_cannot_be_deleted() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .delete(app.url(&format!("/roles/{}", app.owner_role_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert_eq!(
        refused.status(),
        409,
        "deleting the seeded owner role must be refused"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn assigning_a_role_to_a_member_is_reflected_in_their_role_ids() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman",
            "permissions": ["VIEW_PLANNING"],
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    let role_id = created["data"]["id"]
        .as_str()
        .expect("the created role has an id")
        .to_owned();

    let before: serde_json::Value = client
        .get(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        before["data"]["role_ids"],
        serde_json::json!([]),
        "the fixture's other member starts with no role: {before}"
    );

    let assigned = client
        .post(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({ "role_id": role_id }))
        .send()
        .await
        .expect("the api answers the assign call");
    assert_eq!(assigned.status(), 204, "assigning a role succeeds");

    let after: serde_json::Value = client
        .get(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        after["data"]["role_ids"],
        serde_json::json!([role_id]),
        "the newly assigned role must show up: {after}"
    );

    app.cleanup().await;
}

/// `member_roles.role_id` cascades on delete — refusing outright, rather
/// than silently unassigning, is what keeps a delete from quietly stripping
/// a member's permissions (#308).
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn deleting_a_role_still_assigned_to_a_member_is_refused() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman",
            "permissions": ["VIEW_PLANNING"],
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    let role_id = created["data"]["id"]
        .as_str()
        .expect("the created role has an id")
        .to_owned();

    client
        .post(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({ "role_id": role_id }))
        .send()
        .await
        .expect("the api answers the assign call");

    let refused = client
        .delete(app.url(&format!("/roles/{role_id}")))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the delete call");
    assert_eq!(
        refused.status(),
        409,
        "a role still held by a member must be refused, not silently unassigned"
    );

    app.cleanup().await;
}

/// A client posting a bit name that does not exist is a stale build or a
/// typo, not something to silently drop.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn creating_a_role_with_an_unknown_permission_name_is_rejected() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman",
            "permissions": ["NOT_A_REAL_BIT"],
        }))
        .send()
        .await
        .expect("the api answers the create call");
    assert_eq!(refused.status(), 400, "an unknown bit name must be a 400");

    app.cleanup().await;
}

/// The documented gap this PR closes: assigning a role has always had a
/// counterpart to undo it, and unassigning a role the member never held is
/// not an error either — symmetric with `assign_role`'s own `ON CONFLICT DO
/// NOTHING`.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn unassigning_a_role_removes_it_from_the_members_role_ids() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(app.url(&format!("/organizations/{}/roles", app.organization_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "name": "foreman",
            "permissions": ["VIEW_PLANNING"],
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the answer is json");
    let role_id = created["data"]["id"]
        .as_str()
        .expect("the created role has an id")
        .to_owned();

    let assigned = client
        .post(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({ "role_id": role_id }))
        .send()
        .await
        .expect("the api answers the assign call");
    assert_eq!(assigned.status(), 204, "assigning a role succeeds");

    let unassigned = client
        .delete(app.url(&format!("/members/{}/roles/{role_id}", app.other_member_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the unassign call");
    assert_eq!(unassigned.status(), 204, "unassigning a held role succeeds");

    let after: serde_json::Value = client
        .get(app.url(&format!("/members/{}/roles", app.other_member_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        after["data"]["role_ids"],
        serde_json::json!([]),
        "the unassigned role must no longer show up: {after}"
    );

    // Idempotent: unassigning a role the member no longer holds is still a
    // success, symmetric with `assign_role`'s `ON CONFLICT DO NOTHING`.
    let unassigned_again = client
        .delete(app.url(&format!("/members/{}/roles/{role_id}", app.other_member_id)))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the unassign call");
    assert_eq!(
        unassigned_again.status(),
        204,
        "unassigning an already-unheld role is not an error"
    );

    app.cleanup().await;
}
