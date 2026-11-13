use axum_extra::routing::TypedPath;
use mestier_core::{
    ClosureReasonId, CompanyTutorId, ContractFileKind, ContractId, EstablishmentId, HostCompanyId,
    OrganizationId, ParticipantId, PromotionId, SchoolTutorId, TrainingId, TriadId,
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
#[typed_path("/api/v1/organizations/{organization_id}/cursus/school-tutors")]
pub struct SchoolTutorCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}")]
pub struct SchoolTutorPath {
    pub organization_id: OrganizationId,
    pub school_tutor_id: SchoolTutorId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}/archive"
)]
pub struct SchoolTutorArchivePath {
    pub organization_id: OrganizationId,
    pub school_tutor_id: SchoolTutorId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path(
    "/api/v1/organizations/{organization_id}/cursus/school-tutors/{school_tutor_id}/restore"
)]
pub struct SchoolTutorRestorePath {
    pub organization_id: OrganizationId,
    pub school_tutor_id: SchoolTutorId,
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/triads")]
pub struct TriadCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}")]
pub struct TriadPath {
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/close")]
pub struct TriadClosePath {
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/triads/{triad_id}/contracts")]
pub struct TriadContractsPath {
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}")]
pub struct ContractPath {
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/close")]
pub struct ContractClosePath {
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/tutors")]
pub struct ContractTutorsPath {
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/contracts/{contract_id}/files/{kind}")]
pub struct ContractFilePath {
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
    pub kind: ContractFileKind,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/cursus/me/spaces")]
pub struct MySpacesPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/me/triads")]
pub struct MyTriadCollectionPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/cursus/me/triads/{triad_id}")]
pub struct MyTriadPath {
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
}
