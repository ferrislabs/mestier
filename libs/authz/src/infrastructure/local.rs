//! In-process Policy Decision Point.
//!
//! Implements [`Authorizer`] without leaving the process. Three layers
//! of decision, evaluated in order:
//!
//! 1. **System bypass** — subjects of type `system` (see [`Subject::system`])
//!    are always allowed. Used for seeds, cron jobs, CLI tools and
//!    internal webhooks where there is no authenticated principal.
//!
//! 2. **Super-admin bypass** — a subject whose `iam.roles` property
//!    contains the configured super-admin role (default `mestier:admin`)
//!    is allowed regardless of the action.
//!
//! 3. **RBAC check** — the action name is looked up in an injectable
//!    action map (`name → required permission bits`). The subject's
//!    aggregated permission bitfield is read from
//!    `subject.properties["mestier.permissions"]` (i64). The action is
//!    allowed iff every required bit is set.
//!
//! The bit values themselves are owned by the application layer (see
//! `core::Permissions`); this engine only knows that an action requires
//! some bits and that the subject carries some bits. That keeps the
//! `authz` crate free of any business-domain dependency.
//!
//! [`Authorizer`]: crate::Authorizer
//! [`Subject::system`]: crate::Subject::system

use std::collections::HashMap;

use serde_json::json;

use crate::domain::{
    errors::AuthzError,
    ports::Authorizer,
    types::{
        action::Action,
        context::Context,
        decision::Decision,
        evaluation::{
            AccessEvaluationRequest, AccessEvaluationsRequest, AccessEvaluationsResponse,
            BoxedEvaluation,
        },
        resource::Resource,
        subject::Subject,
    },
};

/// Property key carrying the subject's aggregated permission bitfield.
pub const SUBJECT_PERMISSIONS_KEY: &str = "mestier.permissions";

/// Property key carrying the subject's IAM (Ferriskey) realm roles as a
/// JSON array of strings.
pub const SUBJECT_IAM_ROLES_KEY: &str = "iam.roles";

/// Default IAM role that bypasses every authorization check.
pub const DEFAULT_SUPER_ADMIN_ROLE: &str = "mestier:admin";

/// In-process Policy Decision Point. Construct via
/// [`LocalPolicyEngine::builder`] and inject as a generic parameter into
/// services. `Clone` is cheap (a `HashMap` of `String→i64` and a single
/// `String`) and the engine is immutable once built, so callers either
/// clone it or take it by reference depending on what fits best.
#[derive(Clone)]
pub struct LocalPolicyEngine {
    actions: HashMap<String, i64>,
    super_admin_role: String,
}

impl LocalPolicyEngine {
    pub fn builder() -> LocalPolicyEngineBuilder {
        LocalPolicyEngineBuilder::default()
    }

    /// Every action name this engine was built with (#309): the map is
    /// hand-maintained, and an entry no use case ever passes to
    /// `policy::require` is a rule that does not run — a caller walks this
    /// to assert each one is actually reachable, rather than duplicating
    /// the list by hand where it would drift the moment `default_authorizer`
    /// gains or loses an entry.
    pub fn action_names(&self) -> impl Iterator<Item = &str> {
        self.actions.keys().map(String::as_str)
    }

    fn required_bits(&self, action: &Action) -> Option<i64> {
        self.actions.get(&action.name).copied()
    }

    fn has_super_admin(&self, subject: &Subject) -> bool {
        subject
            .properties
            .get(SUBJECT_IAM_ROLES_KEY)
            .and_then(|v| v.as_array())
            .map(|roles| {
                roles
                    .iter()
                    .filter_map(|v| v.as_str())
                    .any(|r| r == self.super_admin_role)
            })
            .unwrap_or(false)
    }

    fn subject_permission_bits(subject: &Subject) -> i64 {
        subject
            .properties
            .get(SUBJECT_PERMISSIONS_KEY)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }

    fn decide(&self, subject: &Subject, action: &Action, _resource: &Resource) -> Decision {
        if subject.is_system() {
            return Decision::allow();
        }
        if self.has_super_admin(subject) {
            return Decision::allow();
        }

        let Some(required) = self.required_bits(action) else {
            return Decision::deny().with_context(
                Context::new()
                    .with("reason", "unknown action")
                    .with("action", action.name.clone()),
            );
        };

        let granted = Self::subject_permission_bits(subject);
        if (granted & required) == required {
            Decision::allow()
        } else {
            Decision::deny().with_context(
                Context::new()
                    .with("reason", "insufficient permissions")
                    .with("action", action.name.clone())
                    .with("required_bits", json!(required))
                    .with("granted_bits", json!(granted)),
            )
        }
    }

    /// Resolve a [`BoxedEvaluation`] entry against the top-level defaults
    /// of an enclosing boxed request, returning the materialized values.
    fn resolve_entry<'a>(
        defaults: &'a AccessEvaluationsRequest,
        entry: &'a BoxedEvaluation,
    ) -> Result<(&'a Subject, &'a Action, &'a Resource), AuthzError> {
        let subject = entry
            .subject
            .as_ref()
            .or(defaults.subject.as_ref())
            .ok_or_else(|| AuthzError::InvalidRequest("missing subject".into()))?;
        let action = entry
            .action
            .as_ref()
            .or(defaults.action.as_ref())
            .ok_or_else(|| AuthzError::InvalidRequest("missing action".into()))?;
        let resource = entry
            .resource
            .as_ref()
            .or(defaults.resource.as_ref())
            .ok_or_else(|| AuthzError::InvalidRequest("missing resource".into()))?;
        Ok((subject, action, resource))
    }
}

impl Authorizer for LocalPolicyEngine {
    async fn evaluate(&self, request: &AccessEvaluationRequest) -> Result<Decision, AuthzError> {
        Ok(self.decide(&request.subject, &request.action, &request.resource))
    }

    async fn evaluations(
        &self,
        request: &AccessEvaluationsRequest,
    ) -> Result<AccessEvaluationsResponse, AuthzError> {
        let mut out = Vec::with_capacity(request.evaluations.len());
        for entry in &request.evaluations {
            let (subject, action, resource) = Self::resolve_entry(request, entry)?;
            out.push(self.decide(subject, action, resource));
        }
        Ok(AccessEvaluationsResponse { evaluations: out })
    }
}

/// Builder for [`LocalPolicyEngine`]. Keeps the engine itself immutable
/// once constructed so it can be wrapped in an `Arc` and shared across
/// tasks without locking.
#[derive(Default)]
pub struct LocalPolicyEngineBuilder {
    actions: HashMap<String, i64>,
    super_admin_role: Option<String>,
}

impl LocalPolicyEngineBuilder {
    /// Register a single action with the permission bits required to
    /// allow it. Re-registering an action replaces the previous bits.
    pub fn action(mut self, name: impl Into<String>, required_bits: i64) -> Self {
        self.actions.insert(name.into(), required_bits);
        self
    }

    /// Bulk registration helper for callers that already hold a list.
    pub fn actions<I, S>(mut self, actions: I) -> Self
    where
        I: IntoIterator<Item = (S, i64)>,
        S: Into<String>,
    {
        for (name, bits) in actions {
            self.actions.insert(name.into(), bits);
        }
        self
    }

    /// Override the IAM role granting super-admin bypass. Defaults to
    /// [`DEFAULT_SUPER_ADMIN_ROLE`].
    pub fn super_admin_role(mut self, role: impl Into<String>) -> Self {
        self.super_admin_role = Some(role.into());
        self
    }

    pub fn build(self) -> LocalPolicyEngine {
        LocalPolicyEngine {
            actions: self.actions,
            super_admin_role: self
                .super_admin_role
                .unwrap_or_else(|| DEFAULT_SUPER_ADMIN_ROLE.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::subject::SubjectKind;

    const MANAGE_ORG: i64 = 1 << 0;
    const MANAGE_MEMBERS: i64 = 1 << 1;

    fn engine() -> LocalPolicyEngine {
        LocalPolicyEngine::builder()
            .action("organization.update", MANAGE_ORG)
            .action("member.remove", MANAGE_MEMBERS)
            .build()
    }

    fn req(subject: Subject, action: &str, resource: Resource) -> AccessEvaluationRequest {
        AccessEvaluationRequest::new(subject, Action::new(action), resource)
    }

    #[tokio::test]
    async fn system_subject_is_always_allowed() {
        let pdp = engine();
        let r = req(
            Subject::system(),
            "anything.unmapped",
            Resource::new("organization", "1"),
        );
        assert!(pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn super_admin_role_bypasses_check() {
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice")
            .with_property(SUBJECT_IAM_ROLES_KEY, json!(["mestier:admin", "viewer"]));
        let r = req(
            subject,
            "anything.unmapped",
            Resource::new("organization", "1"),
        );
        assert!(pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn allows_when_required_bits_present() {
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice")
            .with_property(SUBJECT_PERMISSIONS_KEY, MANAGE_ORG | MANAGE_MEMBERS);
        let r = req(
            subject,
            "organization.update",
            Resource::new("organization", "1"),
        );
        assert!(pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn denies_when_bits_missing() {
        let pdp = engine();
        let subject =
            Subject::new(SubjectKind::User, "alice").with_property(SUBJECT_PERMISSIONS_KEY, 0_i64);
        let r = req(
            subject,
            "organization.update",
            Resource::new("organization", "1"),
        );
        let d = pdp.evaluate(&r).await.unwrap();
        assert!(!d.is_allowed());
        let ctx = d.context.expect("denial carries context");
        assert_eq!(ctx.0.get("reason").unwrap(), "insufficient permissions");
        assert_eq!(ctx.0.get("required_bits").unwrap(), MANAGE_ORG);
        assert_eq!(ctx.0.get("granted_bits").unwrap(), 0);
    }

    #[tokio::test]
    async fn denies_unknown_action_with_reason() {
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice").with_property(
            SUBJECT_PERMISSIONS_KEY,
            i64::MAX, // even with every bit, an unknown action must deny
        );
        let r = req(subject, "nope.unmapped", Resource::new("organization", "1"));
        let d = pdp.evaluate(&r).await.unwrap();
        assert!(!d.is_allowed());
        let ctx = d.context.unwrap();
        assert_eq!(ctx.0.get("reason").unwrap(), "unknown action");
    }

    #[tokio::test]
    async fn missing_permissions_property_treats_as_zero() {
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice"); // no properties
        let r = req(
            subject,
            "organization.update",
            Resource::new("organization", "1"),
        );
        assert!(!pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn malformed_iam_roles_is_treated_as_no_roles() {
        // `iam.roles` is the wrong shape (string instead of array): we
        // do not panic, we simply do not recognize a super-admin.
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice")
            .with_property(SUBJECT_IAM_ROLES_KEY, "mestier:admin")
            .with_property(SUBJECT_PERMISSIONS_KEY, 0_i64);
        let r = req(
            subject,
            "organization.update",
            Resource::new("organization", "1"),
        );
        assert!(!pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn custom_super_admin_role_is_honored() {
        let pdp = LocalPolicyEngine::builder()
            .action("organization.update", MANAGE_ORG)
            .super_admin_role("mestier:root")
            .build();
        let subject = Subject::new(SubjectKind::User, "alice")
            .with_property(SUBJECT_IAM_ROLES_KEY, json!(["mestier:admin"])); // not "mestier:root"
        let r = req(
            subject,
            "organization.update",
            Resource::new("organization", "1"),
        );
        assert!(!pdp.evaluate(&r).await.unwrap().is_allowed());
    }

    #[tokio::test]
    async fn boxed_evaluations_use_top_level_defaults() {
        let pdp = engine();
        let subject = Subject::new(SubjectKind::User, "alice")
            .with_property(SUBJECT_PERMISSIONS_KEY, MANAGE_ORG);

        let req = AccessEvaluationsRequest {
            subject: Some(subject),
            action: Some(Action::new("organization.update")),
            evaluations: vec![
                BoxedEvaluation {
                    resource: Some(Resource::new("organization", "1")),
                    ..Default::default()
                },
                BoxedEvaluation {
                    // overrides the action: this one needs MANAGE_MEMBERS,
                    // which alice does NOT have → deny.
                    action: Some(Action::new("member.remove")),
                    resource: Some(Resource::new("member", "42")),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let resp = pdp.evaluations(&req).await.unwrap();
        assert_eq!(resp.evaluations.len(), 2);
        assert!(resp.evaluations[0].is_allowed());
        assert!(!resp.evaluations[1].is_allowed());
    }

    #[tokio::test]
    async fn boxed_evaluations_reject_missing_subject() {
        let pdp = engine();
        // No top-level subject and no per-entry subject → InvalidRequest.
        let req = AccessEvaluationsRequest {
            action: Some(Action::new("organization.update")),
            evaluations: vec![BoxedEvaluation {
                resource: Some(Resource::new("organization", "1")),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = pdp.evaluations(&req).await.unwrap_err();
        assert!(matches!(err, AuthzError::InvalidRequest(_)));
    }
}
