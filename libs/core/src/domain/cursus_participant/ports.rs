use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    Apprentice, ApprenticeId, CompanyTutor, CompanyTutorId, CompanyTutorRecord, OrganizationId,
    Participant, ParticipantId, ParticipantKind, ParticipantStatus, UserId,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CursusParticipantRepository: Send {
    fn insert_participant(
        &mut self,
        participant: &Participant,
    ) -> impl Future<Output = Result<Participant, CoreError>> + Send;

    fn find_participant(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> impl Future<Output = Result<Option<Participant>, CoreError>> + Send;

    fn find_participant_by_email(
        &mut self,
        organization_id: OrganizationId,
        email: &str,
    ) -> impl Future<Output = Result<Option<Participant>, CoreError>> + Send;

    fn find_participant_by_user(
        &mut self,
        organization_id: OrganizationId,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Participant>, CoreError>> + Send;

    fn list_active_participants_by_user(
        &mut self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Vec<Participant>, CoreError>> + Send;

    fn find_participant_by_token_hash(
        &mut self,
        token_hash: &[u8],
    ) -> impl Future<Output = Result<Option<Participant>, CoreError>> + Send;

    fn list_participants(
        &mut self,
        organization_id: OrganizationId,
        kind: Option<ParticipantKind>,
        status: Option<ParticipantStatus>,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Participant>, u64), CoreError>> + Send;

    fn update_participant(
        &mut self,
        participant: &Participant,
    ) -> impl Future<Output = Result<Participant, CoreError>> + Send;

    fn set_invitation(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        token_hash: &[u8],
        expires_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Participant, CoreError>> + Send;

    fn link_user(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        user_id: UserId,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Participant, CoreError>> + Send;

    fn set_deactivated_at(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
        deactivated_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Participant, CoreError>> + Send;

    fn upsert_apprentice(
        &mut self,
        apprentice: &Apprentice,
    ) -> impl Future<Output = Result<Apprentice, CoreError>> + Send;

    fn find_apprentice(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> impl Future<Output = Result<Option<Apprentice>, CoreError>> + Send;

    fn find_apprentice_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: ApprenticeId,
    ) -> impl Future<Output = Result<Option<Apprentice>, CoreError>> + Send;

    fn find_company_tutor(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> impl Future<Output = Result<Option<CompanyTutor>, CoreError>> + Send;

    fn insert_company_tutor(
        &mut self,
        tutor: &CompanyTutor,
    ) -> impl Future<Output = Result<CompanyTutor, CoreError>> + Send;

    fn list_company_tutors(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> impl Future<Output = Result<Vec<CompanyTutorRecord>, CoreError>> + Send;

    fn delete_company_tutor(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
