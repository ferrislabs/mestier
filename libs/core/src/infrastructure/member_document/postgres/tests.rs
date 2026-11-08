use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    DocumentKind, MemberDocument, MemberDocumentId, MemberId, OrganizationId,
    application::test_support::dev_pool,
    domain::member_document::ports::MemberDocumentRepository,
    infrastructure::{
        member_document::postgres::PgMemberDocumentRepository,
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn seed_member(tx: &SharedTx<'_>, organization_id: OrganizationId) -> MemberId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)"#,
        id,
        organization_id.0,
        "Claire",
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    MemberId(id)
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(matches!(outcome, Err(CoreError::Internal(message)) if message == "rollback"));
}

fn document(org: OrganizationId, member: MemberId, name: &str, key: &str) -> MemberDocument {
    MemberDocument::new(
        MemberDocumentId(generate_uuid_v7()),
        org,
        member,
        name,
        DocumentKind::Medical,
        key,
        Utc::now(),
    )
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_2_5_a_document_keeps_its_file_key_and_leaves_with_its_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let stranger = seed_member(&tx, other).await;
        let mut repo = PgMemberDocumentRepository::new(&tx);

        let stored = repo
            .insert(&document(org, member, "Visite", "documents/abc"))
            .await?;
        repo.insert(&document(other, stranger, "Autre", "documents/zzz"))
            .await?;
        let found = repo.find(org, stored.id).await?.unwrap();
        assert_eq!(found.file_key.as_str(), "documents/abc");
        assert_eq!(found.kind, DocumentKind::Medical);
        assert_eq!(repo.list_by_organization(org).await?.len(), 1);
        assert_eq!(repo.find(other, stored.id).await?, None);

        repo.delete(other, stored.id).await?;
        assert!(repo.find(org, stored.id).await?.is_some());
        repo.delete(org, stored.id).await?;
        assert_eq!(repo.find(org, stored.id).await?, None);
        assert!(repo.list_by_organization(org).await?.is_empty());
        Ok(())
    })
    .await;
}
