//! Wire-format conformance tests against canonical examples from the
//! OpenID AuthZen 1.0 specification.
//!
//! These tests pin down the JSON shape of every type so refactors cannot
//! silently drift away from spec compliance. When a test fails, either
//! the type changed in a non-compliant way or the spec changed and the
//! fixture needs updating — *not* the type's serde attributes.

use authz::{
    AccessEvaluationRequest, AccessEvaluationsRequest, AccessEvaluationsResponse, Action,
    BoxedEvaluation, Context, Decision, Resource, Subject, SubjectKind,
};
use serde_json::json;

/// Round-trip helper: serialize, parse to `Value`, compare to expected.
/// Avoids brittle string comparison and ignores whitespace/key ordering.
fn assert_serialize_eq<T: serde::Serialize>(value: &T, expected: serde_json::Value) {
    let actual = serde_json::to_value(value).expect("serialize");
    assert_eq!(actual, expected);
}

fn assert_round_trip<T>(value: T, expected: serde_json::Value)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    assert_serialize_eq(&value, expected.clone());
    let parsed: T = serde_json::from_value(expected).expect("deserialize");
    assert_eq!(parsed, value);
}

// ---------------------------------------------------------------------------
// Subject (AuthZen 1.0 §5.1.1)
// ---------------------------------------------------------------------------

#[test]
fn subject_minimal_round_trip() {
    let subject = Subject::new(SubjectKind::User, "alice@acmecorp.com");
    assert_round_trip(
        subject,
        json!({ "type": "user", "id": "alice@acmecorp.com" }),
    );
}

#[test]
fn subject_with_properties_round_trip() {
    let subject = Subject::new(SubjectKind::User, "alice")
        .with_property("department", "sales")
        .with_property("level", 4);
    assert_round_trip(
        subject,
        json!({
            "type": "user",
            "id": "alice",
            "properties": { "department": "sales", "level": 4 }
        }),
    );
}

#[test]
fn subject_accepts_unknown_type() {
    let raw = json!({ "type": "service-account", "id": "robot-1" });
    let s: Subject = serde_json::from_value(raw).unwrap();
    assert_eq!(s.r#type, "service-account");
    assert!(matches!(s.kind(), SubjectKind::Other(t) if t == "service-account"));
}

#[test]
fn subject_system_helper() {
    let s = Subject::system();
    assert!(s.is_system());
    assert_eq!(s.r#type, "system");
}

// ---------------------------------------------------------------------------
// Action (AuthZen 1.0 §5.1.2)
// ---------------------------------------------------------------------------

#[test]
fn action_minimal_round_trip() {
    assert_round_trip(Action::new("can_read"), json!({ "name": "can_read" }));
}

#[test]
fn action_with_properties_round_trip() {
    let action = Action::new("document.print").with_property("copies", 2);
    assert_round_trip(
        action,
        json!({ "name": "document.print", "properties": { "copies": 2 } }),
    );
}

// ---------------------------------------------------------------------------
// Resource (AuthZen 1.0 §5.1.3)
// ---------------------------------------------------------------------------

#[test]
fn resource_minimal_round_trip() {
    assert_round_trip(
        Resource::new("account", "123"),
        json!({ "type": "account", "id": "123" }),
    );
}

#[test]
fn resource_with_properties_round_trip() {
    let resource = Resource::new("organization", "01HZ").with_property("slug", "acme");
    assert_round_trip(
        resource,
        json!({
            "type": "organization",
            "id": "01HZ",
            "properties": { "slug": "acme" }
        }),
    );
}

// ---------------------------------------------------------------------------
// Context (AuthZen 1.0 §5.1.4)
// ---------------------------------------------------------------------------

#[test]
fn context_is_transparent_object() {
    let ctx = Context::new()
        .with("time", "1985-10-26T01:22-07:00")
        .with("ip", "203.0.113.10");
    assert_round_trip(
        ctx,
        json!({ "time": "1985-10-26T01:22-07:00", "ip": "203.0.113.10" }),
    );
}

// ---------------------------------------------------------------------------
// Single Access Evaluation Request / Response (AuthZen 1.0 §5.2)
// ---------------------------------------------------------------------------

#[test]
fn access_evaluation_request_canonical_example() {
    // Verbatim from the AuthZen 1.0 spec, §5.2.1 example.
    let req = AccessEvaluationRequest::new(
        Subject::new(SubjectKind::User, "alice@acmecorp.com"),
        Action::new("can_read"),
        Resource::new("account", "123"),
    )
    .with_context(Context::new().with("time", "1985-10-26T01:22-07:00"));

    assert_round_trip(
        req,
        json!({
            "subject":  { "type": "user", "id": "alice@acmecorp.com" },
            "action":   { "name": "can_read" },
            "resource": { "type": "account", "id": "123" },
            "context":  { "time": "1985-10-26T01:22-07:00" }
        }),
    );
}

#[test]
fn access_evaluation_request_omits_absent_context() {
    let req = AccessEvaluationRequest::new(
        Subject::new(SubjectKind::User, "alice"),
        Action::new("can_read"),
        Resource::new("account", "123"),
    );
    assert_serialize_eq(
        &req,
        json!({
            "subject":  { "type": "user", "id": "alice" },
            "action":   { "name": "can_read" },
            "resource": { "type": "account", "id": "123" }
        }),
    );
}

#[test]
fn decision_allow_minimal() {
    assert_round_trip(Decision::allow(), json!({ "decision": true }));
}

#[test]
fn decision_deny_with_reason_context() {
    // Verbatim from the AuthZen 1.0 spec, §5.2.2 deny example.
    let decision = Decision::deny().with_context(
        Context::new()
            .with(
                "reason_admin",
                json!({ "en": "Insufficient privileges. Subject does not have admin role." }),
            )
            .with("reason_user", json!({ "en": "Please contact your admin." })),
    );

    assert_round_trip(
        decision,
        json!({
            "decision": false,
            "context": {
                "reason_admin": { "en": "Insufficient privileges. Subject does not have admin role." },
                "reason_user":  { "en": "Please contact your admin." }
            }
        }),
    );
}

// ---------------------------------------------------------------------------
// Boxed Access Evaluations Request / Response (AuthZen 1.0 §5.3)
// ---------------------------------------------------------------------------

#[test]
fn access_evaluations_request_with_top_level_defaults() {
    // Top-level subject + action act as defaults for every evaluation.
    let req = AccessEvaluationsRequest {
        subject: Some(Subject::new(SubjectKind::User, "alice@acmecorp.com")),
        action: Some(Action::new("can_read")),
        evaluations: vec![
            BoxedEvaluation {
                resource: Some(Resource::new("account", "123")),
                ..Default::default()
            },
            BoxedEvaluation {
                resource: Some(Resource::new("account", "456")),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_round_trip(
        req,
        json!({
            "subject":     { "type": "user", "id": "alice@acmecorp.com" },
            "action":      { "name": "can_read" },
            "evaluations": [
                { "resource": { "type": "account", "id": "123" } },
                { "resource": { "type": "account", "id": "456" } }
            ]
        }),
    );
}

#[test]
fn access_evaluations_response_round_trip() {
    let resp = AccessEvaluationsResponse {
        evaluations: vec![Decision::allow(), Decision::deny()],
    };
    assert_round_trip(
        resp,
        json!({
            "evaluations": [ { "decision": true }, { "decision": false } ]
        }),
    );
}

#[test]
fn boxed_evaluation_overrides_only_present_fields() {
    let raw = json!({
        "resource": { "type": "account", "id": "789" }
    });
    let entry: BoxedEvaluation = serde_json::from_value(raw).unwrap();
    assert!(entry.subject.is_none());
    assert!(entry.action.is_none());
    assert!(entry.context.is_none());
    assert_eq!(entry.resource.as_ref().unwrap().id, "789");
}

// ---------------------------------------------------------------------------
// Tolerance: empty `properties` map and `properties: null` are both accepted
// on the wire even though we never emit them.
// ---------------------------------------------------------------------------

#[test]
fn subject_accepts_empty_properties_object() {
    let raw = json!({ "type": "user", "id": "alice", "properties": {} });
    let s: Subject = serde_json::from_value(raw).unwrap();
    assert!(s.properties.is_empty());
}
