use std::collections::HashMap;

use authz::{Decision, MockAuthorizer, Subject};
use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};

use super::{
    ContactEmail, MemberProfile, MemberProfileError, MemberProfileInput, MemberProfileService,
    PhoneNumber, PostalCode,
    commands::{ArchiveMemberCommand, GetMemberProfileCommand, PutMemberProfileCommand},
    ports::MockMemberProfileRepository,
};
use crate::{Member, MemberId, MockMemberRepository, MockRoleRepository, OrganizationId};

fn member_id() -> MemberId {
    MemberId(generate_uuid_v7())
}

fn member(id: MemberId, organization_id: OrganizationId) -> Member {
    Member {
        id,
        organization_id,
        user_id: None,
        last_name: "Claire".to_owned(),
        first_name: None,
        joined_at: None,
        created_at: Utc::now(),
        deleted_at: None,
    }
}

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

struct World {
    members: Vec<Member>,
    profiles: HashMap<MemberId, MemberProfile>,
}

impl World {
    fn service(
        self,
    ) -> MemberProfileService<
        MockMemberProfileRepository,
        MockMemberRepository,
        MockRoleRepository,
        MockAuthorizer,
    > {
        let mut members = MockMemberRepository::new();
        let known = self.members;
        members.expect_find_by_id().returning(move |id| {
            let found = known.iter().find(|m| m.id == id).cloned();
            Box::pin(async move { Ok(found) })
        });
        let mut profiles = MockMemberProfileRepository::new();
        let stored = self.profiles.clone();
        profiles.expect_find_by_member().returning(move |id| {
            let found = stored.get(&id).cloned();
            Box::pin(async move { Ok(found) })
        });
        profiles.expect_upsert().returning(|profile| {
            let saved = profile.clone();
            Box::pin(async move { Ok(saved) })
        });
        MemberProfileService::new(profiles, members, MockRoleRepository::new(), allow_all())
    }
}

fn input_with_manager(manager: MemberId) -> MemberProfileInput {
    MemberProfileInput {
        manager_member_id: Some(manager),
        ..MemberProfileInput::default()
    }
}

fn managed_by(
    id: MemberId,
    organization_id: OrganizationId,
    manager: MemberId,
) -> (MemberId, MemberProfile) {
    let mut profile = MemberProfile::empty(id, organization_id, Utc::now());
    profile.manager_member_id = Some(manager);
    (id, profile)
}

#[test]
fn emails_need_one_at_sign_and_a_dotted_domain() {
    assert!(ContactEmail::try_from("a@b.fr").is_ok());
    assert!(ContactEmail::try_from("a@b").is_err());
    assert!(ContactEmail::try_from("a b@c.fr").is_err());
    assert!(ContactEmail::try_from("a@@c.fr").is_err());
    assert!(ContactEmail::try_from("@c.fr").is_err());
}

#[test]
fn phones_follow_the_prototype_pattern() {
    assert!(PhoneNumber::try_from("+33 6 12 34 56 78").is_ok());
    assert!(PhoneNumber::try_from("06.12.34.56.78").is_ok());
    assert!(PhoneNumber::try_from("12").is_err());
    assert!(PhoneNumber::try_from("abc123456").is_err());
}

#[test]
fn postal_codes_are_five_digits() {
    assert!(PostalCode::try_from("75011").is_ok());
    assert_eq!(
        PostalCode::try_from("7501"),
        Err(MemberProfileError::PostalCodeInvalid)
    );
}

#[tokio::test]
async fn spec_rh_1_1_a_member_without_profile_reads_empty_defaults() {
    let org = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let mut service = World {
        members: vec![member(id, org)],
        profiles: HashMap::new(),
    }
    .service();

    let profile = service
        .get_profile(GetMemberProfileCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
        })
        .await
        .unwrap();

    assert_eq!(profile.work_email, None);
    assert_eq!(profile.address.country, "France");
    assert_eq!(profile.archived_at, None);
}

#[tokio::test]
async fn spec_rh_1_1_a_put_stores_the_normalised_fields() {
    let org = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let mut service = World {
        members: vec![member(id, org)],
        profiles: HashMap::new(),
    }
    .service();

    let saved = service
        .put_profile(PutMemberProfileCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
            input: MemberProfileInput {
                work_email: Some("  a@b.fr ".to_owned()),
                mobile: Some("   ".to_owned()),
                postal_code: Some("75011".to_owned()),
                job_title: Some(" Chef de chantier ".to_owned()),
                work_permit_end_date: NaiveDate::from_ymd_opt(2027, 1, 31),
                ..MemberProfileInput::default()
            },
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(saved.work_email.unwrap().as_str(), "a@b.fr");
    assert_eq!(saved.mobile, None);
    assert_eq!(saved.job_title.as_deref(), Some("Chef de chantier"));
    assert_eq!(saved.address.postal_code.unwrap().as_str(), "75011");
}

#[tokio::test]
async fn spec_rh_1_1_an_invalid_email_is_refused_naming_the_field() {
    let org = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let mut service = World {
        members: vec![member(id, org)],
        profiles: HashMap::new(),
    }
    .service();

    let refusal = service
        .put_profile(PutMemberProfileCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
            input: MemberProfileInput {
                personal_email: Some("nope".to_owned()),
                ..MemberProfileInput::default()
            },
        })
        .await
        .unwrap();

    assert_eq!(refusal, Err(MemberProfileError::PersonalEmailInvalid));
}

async fn put_manager(
    world: World,
    org: OrganizationId,
    id: MemberId,
    manager: MemberId,
) -> Result<Result<MemberProfile, MemberProfileError>, CoreError> {
    world
        .service()
        .put_profile(PutMemberProfileCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
            input: input_with_manager(manager),
        })
        .await
}

#[tokio::test]
async fn spec_rh_1_2_the_member_itself_is_refused_as_manager() {
    let org = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let world = World {
        members: vec![member(id, org)],
        profiles: HashMap::new(),
    };

    assert_eq!(
        put_manager(world, org, id, id).await.unwrap(),
        Err(MemberProfileError::ManagerIsSelf)
    );
}

#[tokio::test]
async fn spec_rh_1_2_a_manager_from_another_organization_is_refused() {
    let org = OrganizationId(generate_uuid_v7());
    let other = OrganizationId(generate_uuid_v7());
    let (id, stranger) = (member_id(), member_id());
    let world = World {
        members: vec![member(id, org), member(stranger, other)],
        profiles: HashMap::new(),
    };

    assert_eq!(
        put_manager(world, org, id, stranger).await.unwrap(),
        Err(MemberProfileError::ManagerNotInOrganization)
    );
}

#[tokio::test]
async fn spec_rh_1_2_a_cycle_through_the_chain_is_refused() {
    let org = OrganizationId(generate_uuid_v7());
    let (a, b, c) = (member_id(), member_id(), member_id());
    let world = World {
        members: vec![member(a, org), member(b, org), member(c, org)],
        profiles: HashMap::from([managed_by(b, org, c), managed_by(c, org, a)]),
    };

    assert_eq!(
        put_manager(world, org, a, b).await.unwrap(),
        Err(MemberProfileError::ManagerCycle)
    );
}

#[tokio::test]
async fn spec_rh_1_2_a_manager_up_a_clean_chain_is_accepted() {
    let org = OrganizationId(generate_uuid_v7());
    let (a, b, c) = (member_id(), member_id(), member_id());
    let world = World {
        members: vec![member(a, org), member(b, org), member(c, org)],
        profiles: HashMap::from([managed_by(b, org, c)]),
    };

    let saved = put_manager(world, org, a, b).await.unwrap().unwrap();

    assert_eq!(saved.manager_member_id, Some(b));
}

#[tokio::test]
async fn spec_rh_1_3_archiving_sets_archived_at_and_restoring_clears_it() {
    let org = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let mut service = World {
        members: vec![member(id, org)],
        profiles: HashMap::new(),
    }
    .service();

    let archived = service
        .archive(ArchiveMemberCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
        })
        .await
        .unwrap();
    assert!(archived.archived_at.is_some());

    let restored = archived.restored(Utc::now());
    assert_eq!(restored.archived_at, None);
    assert_eq!(restored.created_at, archived.created_at);
}

#[tokio::test]
async fn spec_rh_1_4_a_member_of_another_organization_answers_not_found() {
    let org = OrganizationId(generate_uuid_v7());
    let other = OrganizationId(generate_uuid_v7());
    let id = member_id();
    let mut service = World {
        members: vec![member(id, other)],
        profiles: HashMap::new(),
    }
    .service();

    let result = service
        .get_profile(GetMemberProfileCommand {
            actor: Subject::system(),
            organization_id: org,
            member_id: id,
        })
        .await;

    assert!(matches!(result, Err(CoreError::NotFound)));
}
