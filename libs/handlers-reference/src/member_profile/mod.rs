use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use http::StatusCode;
use mestier_core::{
    ArchiveMemberCommand, GetMemberProfileCommand, ListMemberProfilesCommand, MemberId,
    MemberProfile, MemberProfileError, MemberProfileInput, OrganizationId, PutMemberProfileCommand,
    ReportsPolicy, RestoreMemberCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list_profiles)
        .typed_get(get_profile)
        .typed_put(put_profile)
        .typed_post(archive)
        .typed_post(restore)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/member-profiles")]
pub struct MemberProfilesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/profile")]
pub struct MemberProfilePath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/archive")]
pub struct ArchiveMemberPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/members/{member_id}/restore")]
pub struct RestoreMemberPath {
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct PostalAddressBody {
    #[serde(default)]
    pub street: Option<String>,
    #[serde(default)]
    pub complement: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
}

fn present<'de, D>(deserializer: D) -> Result<Option<Option<MemberId>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<MemberId>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct ArchiveMemberRequest {
    /// When the key is present, every active direct report of the archived
    /// member moves to this member (or to nobody when null), atomically.
    #[schema(value_type = Option<MemberId>)]
    #[serde(default, deserialize_with = "present")]
    pub reassign_reports_to: Option<Option<MemberId>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PutMemberProfileRequest {
    #[serde(default)]
    pub work_email: Option<String>,
    #[serde(default)]
    pub personal_email: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
    #[serde(default)]
    pub landline: Option<String>,
    #[serde(default)]
    pub address: PostalAddressBody,
    #[serde(default)]
    pub job_title: Option<String>,
    #[serde(default)]
    pub manager_member_id: Option<MemberId>,
    #[schema(value_type = Option<String>, format = Date)]
    #[serde(default)]
    pub work_permit_end_date: Option<NaiveDate>,
}

impl From<PutMemberProfileRequest> for MemberProfileInput {
    fn from(value: PutMemberProfileRequest) -> Self {
        Self {
            work_email: value.work_email,
            personal_email: value.personal_email,
            mobile: value.mobile,
            landline: value.landline,
            street: value.address.street,
            complement: value.address.complement,
            postal_code: value.address.postal_code,
            city: value.address.city,
            country: value.address.country,
            job_title: value.job_title,
            manager_member_id: value.manager_member_id,
            work_permit_end_date: value.work_permit_end_date,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberProfileAddressResponse {
    pub street: String,
    pub complement: String,
    pub postal_code: Option<String>,
    pub city: String,
    pub country: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberProfileResponse {
    pub member_id: MemberId,
    pub organization_id: OrganizationId,
    pub work_email: Option<String>,
    pub personal_email: Option<String>,
    pub mobile: Option<String>,
    pub landline: Option<String>,
    pub address: MemberProfileAddressResponse,
    pub job_title: Option<String>,
    pub manager_member_id: Option<MemberId>,
    #[schema(value_type = Option<String>, format = Date)]
    pub work_permit_end_date: Option<NaiveDate>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl From<MemberProfile> for MemberProfileResponse {
    fn from(value: MemberProfile) -> Self {
        Self {
            member_id: value.member_id,
            organization_id: value.organization_id,
            work_email: value.work_email.map(|v| v.as_str().to_owned()),
            personal_email: value.personal_email.map(|v| v.as_str().to_owned()),
            mobile: value.mobile.map(|v| v.as_str().to_owned()),
            landline: value.landline.map(|v| v.as_str().to_owned()),
            address: MemberProfileAddressResponse {
                street: value.address.street,
                complement: value.address.complement,
                postal_code: value.address.postal_code.map(|v| v.as_str().to_owned()),
                city: value.address.city,
                country: value.address.country,
            },
            job_title: value.job_title,
            manager_member_id: value.manager_member_id,
            work_permit_end_date: value.work_permit_end_date,
            archived_at: value.archived_at,
        }
    }
}

#[derive(Debug)]
pub enum ProfileFailure {
    Api(ApiError),
    Refused(MemberProfileError),
}

impl From<ApiError> for ProfileFailure {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<MemberProfileError> for ProfileFailure {
    fn from(error: MemberProfileError) -> Self {
        Self::Refused(error)
    }
}

impl IntoResponse for ProfileFailure {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(error) => {
                let status = StatusCode::UNPROCESSABLE_ENTITY;
                (
                    status,
                    Json(json!({
                        "code": error.code(),
                        "message": error.to_string(),
                        "status": status.as_u16(),
                        "details": { "field": error.field().as_str() },
                    })),
                )
                    .into_response()
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/member-profiles",
    operation_id = "listMemberProfiles",
    tag = super::TAG,
    params(("organization_id" = OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Every HR profile written in the organization, members without one are absent", body = inline(DataEnvelope<Vec<MemberProfileResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_profiles(
    path: MemberProfilesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<MemberProfileResponse>>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let profiles = state
        .usecase
        .acting_as(user_id)
        .list_member_profiles(ListMemberProfilesCommand {
            actor,
            organization_id: path.organization_id,
        })
        .await?;

    Ok(Response::OK(profiles.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/profile",
    operation_id = "getMemberProfile",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 200, description = "HR profile, empty defaults when none was written", body = inline(DataEnvelope<MemberProfileResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_profile(
    path: MemberProfilePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MemberProfileResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let profile = state
        .usecase
        .acting_as(user_id)
        .get_member_profile(GetMemberProfileCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
        })
        .await?;

    Ok(Response::OK(profile.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/profile",
    operation_id = "putMemberProfile",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    request_body = PutMemberProfileRequest,
    responses(
        (status = 200, description = "HR profile created or replaced", body = inline(DataEnvelope<MemberProfileResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 422, description = "A field is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn put_profile(
    path: MemberProfilePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PutMemberProfileRequest>,
) -> Result<Response<MemberProfileResponse>, ProfileFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let profile = state
        .usecase
        .acting_as(user_id)
        .put_member_profile(PutMemberProfileCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            input: payload.into(),
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(profile.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/archive",
    operation_id = "archiveMember",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    request_body(content = ArchiveMemberRequest, description = "Optional, reassigns the direct reports in the same transaction"),
    responses(
        (status = 200, description = "Member archived, history kept", body = inline(DataEnvelope<MemberProfileResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
        (status = 422, description = "The reassignment target is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn archive(
    path: ArchiveMemberPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    body: Option<Json<ArchiveMemberRequest>>,
) -> Result<Response<MemberProfileResponse>, ProfileFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let reports = body
        .and_then(|Json(request)| request.reassign_reports_to)
        .map_or(ReportsPolicy::Keep, ReportsPolicy::ReassignTo);
    let profile = state
        .usecase
        .acting_as(user_id)
        .archive_member(ArchiveMemberCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
            reports,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::OK(profile.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/members/{member_id}/restore",
    operation_id = "restoreMember",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        ("member_id" = MemberId, Path, description = "Member identifier"),
    ),
    responses(
        (status = 200, description = "Member restored", body = inline(DataEnvelope<MemberProfileResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn restore(
    path: RestoreMemberPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<MemberProfileResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let profile = state
        .usecase
        .acting_as(user_id)
        .restore_member(RestoreMemberCommand {
            actor,
            organization_id: path.organization_id,
            member_id: path.member_id,
        })
        .await?;

    Ok(Response::OK(profile.into()))
}
