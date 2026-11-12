use std::{collections::HashMap, convert::Infallible, str::FromStr};

use axum::{
    extract::{Path, Request, State},
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::Route,
};
use common::OrganizationId;
use mestier_core::{FeatureFlag, MestierUseCase};
use tower::{Layer, Service};

use crate::{ApiError, AppState};

pub const FEATURE_DISABLED_MESSAGE: &str =
    "Cette fonctionnalité n'est pas activée pour cette organisation.";

pub async fn require_feature_enabled(
    usecase: &MestierUseCase,
    organization_id: OrganizationId,
    flag: FeatureFlag,
) -> Result<(), ApiError> {
    if usecase.is_feature_enabled(organization_id, flag).await? {
        Ok(())
    } else {
        Err(ApiError::ForbiddenWithReason(
            FEATURE_DISABLED_MESSAGE.to_owned(),
        ))
    }
}

pub async fn require_feature(
    state: &AppState,
    organization_id: OrganizationId,
    flag: FeatureFlag,
) -> Result<(), ApiError> {
    require_feature_enabled(&state.usecase, organization_id, flag).await
}

#[derive(Clone)]
struct FeatureGate {
    usecase: MestierUseCase,
    flag: FeatureFlag,
}

async fn feature_gate(
    State(gate): State<FeatureGate>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let organization_id = params
        .get("organization_id")
        .and_then(|raw| OrganizationId::from_str(raw).ok())
        .ok_or_else(|| ApiError::BadRequest("missing organization_id".to_owned()))?;

    require_feature_enabled(&gate.usecase, organization_id, gate.flag).await?;

    Ok(next.run(request).await)
}

pub fn require_feature_layer(
    usecase: &MestierUseCase,
    flag: FeatureFlag,
) -> impl Layer<
    Route,
    Service = impl Service<
        Request,
        Response = Response,
        Error = Infallible,
        Future = impl Send + 'static,
    > + Clone
              + Send
              + Sync
              + 'static,
> + Clone
+ Send
+ Sync
+ 'static {
    from_fn_with_state(
        FeatureGate {
            usecase: usecase.clone(),
            flag,
        },
        feature_gate,
    )
}

#[cfg(test)]
mod tests {
    use axum::{Router, body::Body, routing::get};
    use http::{Request, StatusCode};
    use mestier_core::{EventHub, PlatformAdmin, application::default_authorizer};
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    use super::*;

    const PATH: &str = "/api/v1/organizations/{organization_id}/probe";

    fn router(usecase: &MestierUseCase) -> Router {
        Router::new()
            .route(PATH, get(|| async { "ok" }))
            .layer(require_feature_layer(usecase, FeatureFlag::DriveAi))
    }

    async fn status(router: &Router, organization: &str) -> StatusCode {
        router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/organizations/{organization}/probe"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn a_path_without_a_valid_organization_id_is_a_bad_request() {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
        let usecase = MestierUseCase::new(pool, default_authorizer(), EventHub::new());

        assert_eq!(
            status(&router(&usecase), "not-a-uuid").await,
            StatusCode::BAD_REQUEST
        );
    }

    async fn seed_user(pool: &PgPool) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Staff', $4)",
        )
        .bind(id)
        .bind(format!("staff-{id}@example.com"))
        .bind(format!("staff-{id}"))
        .bind(format!("sub-{id}"))
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_org(pool: &PgPool, owner: Uuid) -> Uuid {
        let id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Org', $2, $3)",
        )
        .bind(id)
        .bind(format!("org-{id}"))
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
        id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_gated_route_is_closed_until_the_flag_is_enabled_and_only_for_that_organization() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&url).await.unwrap();
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        let staff = seed_user(&pool).await;
        let (a, b) = (seed_org(&pool, staff).await, seed_org(&pool, staff).await);
        let router = router(&usecase);
        let admin = PlatformAdmin::from_realm_roles(&["mestier:admin".to_owned()]).unwrap();

        let before_a = status(&router, &a.to_string()).await;
        let before_b = status(&router, &b.to_string()).await;
        usecase
            .set_organization_feature(
                admin,
                mestier_core::UserId(staff),
                OrganizationId(a),
                FeatureFlag::DriveAi,
                true,
            )
            .await
            .unwrap();
        let after_a = status(&router, &a.to_string()).await;
        let after_b = status(&router, &b.to_string()).await;

        for org in [a, b] {
            sqlx::query("DELETE FROM automation.event WHERE org_id = $1")
                .bind(org)
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM organizations WHERE id = $1")
                .bind(org)
                .execute(&pool)
                .await
                .unwrap();
        }
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(staff)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(before_a, StatusCode::FORBIDDEN);
        assert_eq!(before_b, StatusCode::FORBIDDEN);
        assert_eq!(after_a, StatusCode::OK);
        assert_eq!(after_b, StatusCode::FORBIDDEN);
    }
}
