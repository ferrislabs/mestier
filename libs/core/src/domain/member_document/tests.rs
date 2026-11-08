use std::sync::{Arc, Mutex};

use authz::{Decision, MockAuthorizer, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    DocumentKind, DocumentName, FileKey, MemberDocument, MemberDocumentError, MemberDocumentId,
    MemberDocumentService,
    commands::{AddMemberDocumentCommand, DeleteMemberDocumentCommand, ListMemberDocumentsCommand},
    ports::MockMemberDocumentRepository,
};
use crate::{Member, MemberId, MockMemberRepository, MockRoleRepository, OrganizationId};

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

type Service = MemberDocumentService<
    MockMemberDocumentRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

fn service(
    organization: OrganizationId,
    owner: MemberId,
    stored: Vec<MemberDocument>,
) -> (Service, Arc<Mutex<Vec<MemberDocument>>>) {
    let mut members = MockMemberRepository::new();
    let known = member(owner, organization);
    members.expect_find_by_id().returning(move |id| {
        let found = (id == known.id).then(|| known.clone());
        Box::pin(async move { Ok(found) })
    });
    let mut documents = MockMemberDocumentRepository::new();
    let listed = stored.clone();
    documents.expect_list_by_organization().returning(move |_| {
        let all = listed.clone();
        Box::pin(async move { Ok(all) })
    });
    documents.expect_find().returning(move |_, id| {
        let hit = stored.iter().find(|document| document.id == id).cloned();
        Box::pin(async move { Ok(hit) })
    });
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&log);
    documents.expect_insert().returning(move |document| {
        let saved = document.clone();
        sink.lock().expect("log").push(saved.clone());
        Box::pin(async move { Ok(saved) })
    });
    documents
        .expect_delete()
        .returning(|_, _| Box::pin(async { Ok(()) }));
    (
        MemberDocumentService::new(documents, members, MockRoleRepository::new(), allow_all()),
        log,
    )
}

fn add(
    organization: OrganizationId,
    member: MemberId,
    name: &str,
    key: &str,
) -> AddMemberDocumentCommand {
    AddMemberDocumentCommand {
        actor: Subject::system(),
        organization_id: organization,
        member_id: member,
        name: name.to_owned(),
        kind: DocumentKind::Identity,
        file_key: key.to_owned(),
    }
}

#[test]
fn a_document_needs_a_name_and_a_safe_file_key() {
    assert!(DocumentName::try_from("  CNI  ").is_ok_and(|name| name.as_str() == "CNI"));
    assert_eq!(
        DocumentName::try_from("  "),
        Err(MemberDocumentError::NameEmpty)
    );
    assert!(FileKey::try_from("attachments/abc").is_ok());
    for bad in ["", " ", "/etc/passwd", "a/../b", "a\nb"] {
        assert_eq!(
            FileKey::try_from(bad),
            Err(MemberDocumentError::FileKeyInvalid),
            "{bad:?}"
        );
    }
}

#[tokio::test]
async fn spec_rh_2_5_a_document_is_stored_with_its_file_key() {
    let organization = OrganizationId(generate_uuid_v7());
    let owner = MemberId(generate_uuid_v7());
    let (mut service, log) = service(organization, owner, Vec::new());
    let stored = service
        .add(add(organization, owner, "CNI", "documents/abc"))
        .await
        .expect("stored")
        .expect("accepted");
    assert_eq!(stored.file_key.as_str(), "documents/abc");
    assert_eq!(stored.name.as_str(), "CNI");
    assert_eq!(log.lock().expect("log").len(), 1);

    let refused = service
        .add(add(organization, owner, "", "documents/abc"))
        .await
        .expect("no storage failure");
    assert_eq!(refused.unwrap_err(), MemberDocumentError::NameEmpty);
    assert_eq!(log.lock().expect("log").len(), 1);
}

#[tokio::test]
async fn spec_rh_2_5_deleting_returns_the_document_so_its_file_can_follow() {
    let organization = OrganizationId(generate_uuid_v7());
    let owner = MemberId(generate_uuid_v7());
    let document = MemberDocument::new(
        MemberDocumentId(generate_uuid_v7()),
        organization,
        owner,
        "CNI",
        DocumentKind::Identity,
        "documents/abc",
        Utc::now(),
    )
    .expect("valid");
    let id = document.id;
    let (mut service, _) = service(organization, owner, vec![document]);
    let command = |member| DeleteMemberDocumentCommand {
        actor: Subject::system(),
        organization_id: organization,
        member_id: member,
        document_id: id,
    };
    assert!(matches!(
        service.delete(command(MemberId(generate_uuid_v7()))).await,
        Err(CoreError::NotFound)
    ));
    let removed = service.delete(command(owner)).await.expect("deleted");
    assert_eq!(removed.file_key.as_str(), "documents/abc");
}

#[tokio::test]
async fn the_organization_list_and_unknown_members_follow_the_organization() {
    let organization = OrganizationId(generate_uuid_v7());
    let owner = MemberId(generate_uuid_v7());
    let (mut service, _) = service(organization, owner, Vec::new());
    let listed = service
        .list(ListMemberDocumentsCommand {
            actor: Subject::system(),
            organization_id: organization,
        })
        .await
        .expect("listed");
    assert!(listed.is_empty());
    let stranger = MemberId(generate_uuid_v7());
    assert!(matches!(
        service.add(add(organization, stranger, "CNI", "k")).await,
        Err(CoreError::NotFound)
    ));
}
