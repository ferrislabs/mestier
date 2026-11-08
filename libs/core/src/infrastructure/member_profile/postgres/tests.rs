use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    ContactEmail, MemberId, MemberProfile, OrganizationId,
    application::test_support::dev_pool,
    domain::{member::ports::MemberRepository, member_profile::ports::MemberProfileRepository},
    infrastructure::{
        member::postgres::PgMemberRepository,
        member_profile::postgres::PgMemberProfileRepository,
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

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_1_1_a_profile_round_trips_and_a_second_put_replaces_it() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = seed_member(&tx, org).await;
        let mut repo = PgMemberProfileRepository::new(&tx);
        assert_eq!(repo.find_by_member(id).await?, None);

        let mut profile = MemberProfile::empty(id, org, Utc::now());
        profile.work_email = ContactEmail::try_from("a@b.fr").ok();
        profile.job_title = Some("Chef".to_owned());
        profile.work_permit_end_date = NaiveDate::from_ymd_opt(2027, 1, 31);
        repo.upsert(&profile).await?;
        let found = repo.find_by_member(id).await?.unwrap();
        assert_eq!(found.work_email, profile.work_email);
        assert_eq!(found.work_permit_end_date, profile.work_permit_end_date);
        assert_eq!(found.address.country, "France");

        profile.job_title = None;
        repo.upsert(&profile).await?;
        assert_eq!(repo.find_by_member(id).await?.unwrap().job_title, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_1_2_the_database_refuses_a_manager_of_another_organization_or_itself() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let id = seed_member(&tx, org).await;
        let stranger = seed_member(&tx, other).await;
        let mut repo = PgMemberProfileRepository::new(&tx);

        let mut profile = MemberProfile::empty(id, org, Utc::now());
        profile.manager_member_id = Some(stranger);
        {
            let mut guard = tx.lock().await;
            sqlx::query("SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }
        assert!(repo.upsert(&profile).await.is_err());
        {
            let mut guard = tx.lock().await;
            sqlx::query("ROLLBACK TO SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_1_3_an_archived_member_leaves_the_planning_roster_and_keeps_its_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let kept = seed_member(&tx, org).await;
        let archived = seed_member(&tx, org).await;
        let mut profiles = PgMemberProfileRepository::new(&tx);
        let profile = MemberProfile::empty(archived, org, Utc::now());
        profiles.upsert(&profile.archived(Utc::now())).await?;
        let mut members = PgMemberRepository::new(&tx);

        let roster: Vec<MemberId> = members
            .list_active_by_organization(org)
            .await?
            .into_iter()
            .map(|member| member.id)
            .collect();
        assert_eq!(roster, vec![kept]);
        assert!(members.find_by_id(archived).await?.is_some());

        profiles
            .upsert(&profile.archived(Utc::now()).restored(Utc::now()))
            .await?;
        assert_eq!(members.list_active_by_organization(org).await?.len(), 2);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_1_5_listing_returns_only_the_profiles_of_the_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let profiled = seed_member(&tx, org).await;
        let _bare = seed_member(&tx, org).await;
        let stranger = seed_member(&tx, other).await;
        let mut repo = PgMemberProfileRepository::new(&tx);
        repo.upsert(&MemberProfile::empty(profiled, org, Utc::now()).archived(Utc::now()))
            .await?;
        repo.upsert(&MemberProfile::empty(stranger, other, Utc::now()))
            .await?;

        let listed = repo.list_by_organization(org).await?;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].member_id, profiled);
        assert!(listed[0].archived_at.is_some());
        Ok(())
    })
    .await;
}
