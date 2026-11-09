use axum_extra::routing::TypedPath;
use mestier_core::{
    ClosureReasonId, CompanyTutorId, EstablishmentId, HostCompanyId, OrganizationId, ParticipantId,
    PromotionId, TrainingId,
};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/establishments")]
pub struct EstablishmentCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/establishments/{establishment_id}")]
pub struct EstablishmentPath {
    pub organization_id: OrganizationId,
    pub establishment_id: EstablishmentId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/establishments/{establishment_id}/archive"
)]
pub struct EstablishmentArchivePath {
    pub organization_id: OrganizationId,
    pub establishment_id: EstablishmentId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/trainings")]
pub struct TrainingCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/trainings/{training_id}")]
pub struct TrainingPath {
    pub organization_id: OrganizationId,
    pub training_id: TrainingId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/trainings/{training_id}/archive")]
pub struct TrainingArchivePath {
    pub organization_id: OrganizationId,
    pub training_id: TrainingId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/promotions")]
pub struct PromotionCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}")]
pub struct PromotionPath {
    pub organization_id: OrganizationId,
    pub promotion_id: PromotionId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/promotions/{promotion_id}/archive")]
pub struct PromotionArchivePath {
    pub organization_id: OrganizationId,
    pub promotion_id: PromotionId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/closure-reasons")]
pub struct ClosureReasonCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/closure-reasons/{closure_reason_id}")]
pub struct ClosureReasonPath {
    pub organization_id: OrganizationId,
    pub closure_reason_id: ClosureReasonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/closure-reasons/{closure_reason_id}/archive"
)]
pub struct ClosureReasonArchivePath {
    pub organization_id: OrganizationId,
    pub closure_reason_id: ClosureReasonId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/host-companies")]
pub struct HostCompanyCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}")]
pub struct HostCompanyPath {
    pub organization_id: OrganizationId,
    pub host_company_id: HostCompanyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/host-companies/{host_company_id}/archive"
)]
pub struct HostCompanyArchivePath {
    pub organization_id: OrganizationId,
    pub host_company_id: HostCompanyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/participants")]
pub struct ParticipantCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}")]
pub struct ParticipantPath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/invitation"
)]
pub struct ParticipantInvitationPath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/deactivate"
)]
pub struct ParticipantDeactivatePath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/reactivate"
)]
pub struct ParticipantReactivatePath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/apprentice"
)]
pub struct ParticipantApprenticePath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/participants/{participant_id}/company-tutors"
)]
pub struct ParticipantCompanyTutorsPath {
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/company-tutors/{company_tutor_id}")]
pub struct CompanyTutorPath {
    pub organization_id: OrganizationId,
    pub company_tutor_id: CompanyTutorId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/cursus/invitations/{token}/accept")]
pub struct AcceptInvitationPath {
    pub token: String,
}
