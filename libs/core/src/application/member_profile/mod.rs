use common::CoreError;
use mestier_macros::transactional;

use crate::{
    MemberProfile, MemberProfileOutcome,
    application::MestierUseCase,
    domain::member_profile::{
        MemberProfileService,
        commands::{
            ArchiveMemberCommand, GetMemberProfileCommand, PutMemberProfileCommand,
            RestoreMemberCommand,
        },
    },
};

impl MestierUseCase {
    #[transactional(member_profile, member, role, authz)]
    pub async fn get_member_profile(
        &self,
        command: GetMemberProfileCommand,
    ) -> Result<MemberProfile, CoreError> {
        let mut service = MemberProfileService::new(
            member_profile_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_profile(command).await
    }

    #[transactional(member_profile, member, role, authz)]
    pub async fn put_member_profile(
        &self,
        command: PutMemberProfileCommand,
    ) -> Result<MemberProfileOutcome<MemberProfile>, CoreError> {
        let mut service = MemberProfileService::new(
            member_profile_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.put_profile(command).await
    }

    #[transactional(member_profile, member, role, authz)]
    pub async fn archive_member(
        &self,
        command: ArchiveMemberCommand,
    ) -> Result<MemberProfile, CoreError> {
        let mut service = MemberProfileService::new(
            member_profile_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.archive(command).await
    }

    #[transactional(member_profile, member, role, authz)]
    pub async fn restore_member(
        &self,
        command: RestoreMemberCommand,
    ) -> Result<MemberProfile, CoreError> {
        let mut service = MemberProfileService::new(
            member_profile_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.restore(command).await
    }
}
