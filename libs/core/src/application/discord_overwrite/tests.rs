use common::{CoreError, OrganizationId, RoleId, UserId, generate_uuid_v7};
use discord::{ChannelId, ChannelType, CreateCategoryCommand, CreateChannelCommand};
use sqlx::PgPool;

use crate::Permissions;
use crate::application::test_support::{dev_pool, purge};
use crate::application::{MestierUseCase, default_authorizer};
use crate::infrastructure::realtime::EventHub;

struct Fixture {
    org: OrganizationId,
    creator: UserId,
    in_role: UserId,
    outside: UserId,
    role: RoleId,
    other_org_role: RoleId,
}

async fn seed_user(pool: &PgPool, label: &str) -> UserId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(format!("{label}-{id}@example.com"))
    .bind(format!("{label}-{id}"))
    .bind(label)
    .bind(format!("sub-{label}-{id}"))
    .execute(pool)
    .await
    .unwrap();
    UserId(id)
}

async fn seed_role(pool: &PgPool, org: OrganizationId, permissions: i64) -> RoleId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(org.0)
    .bind(format!("role-{id}"))
    .bind(permissions)
    .execute(pool)
    .await
    .unwrap();
    RoleId(id)
}

async fn seat(pool: &PgPool, org: OrganizationId, user: UserId, role: RoleId) {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(org.0)
    .bind(user.0)
    .bind("Dupont")
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(generate_uuid_v7())
        .bind(member_id)
        .bind(role.0)
        .execute(pool)
        .await
        .unwrap();
}

async fn seed(pool: &PgPool) -> Fixture {
    let creator = seed_user(pool, "creator").await;
    let in_role = seed_user(pool, "in-role").await;
    let outside = seed_user(pool, "outside").await;
    let org_id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(org_id)
        .bind("Test Org")
        .bind(format!("test-org-{org_id}"))
        .bind(creator.0)
        .execute(pool)
        .await
        .unwrap();
    let org = OrganizationId(org_id);
    let manager_role = seed_role(pool, org, Permissions::MANAGE_CHANNELS.bits()).await;
    let role = seed_role(pool, org, 0).await;
    let plain_role = seed_role(pool, org, 0).await;
    seat(pool, org, creator, manager_role).await;
    seat(pool, org, in_role, role).await;
    seat(pool, org, outside, plain_role).await;

    let foreign_org_id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(foreign_org_id)
        .bind("Other Org")
        .bind(format!("other-org-{foreign_org_id}"))
        .bind(creator.0)
        .execute(pool)
        .await
        .unwrap();
    let other_org_role = seed_role(pool, OrganizationId(foreign_org_id), 0).await;

    Fixture {
        org,
        creator,
        in_role,
        outside,
        role,
        other_org_role,
    }
}

async fn cleanup(pool: &PgPool, fixture: &Fixture) {
    purge(
        pool,
        "DELETE FROM organizations WHERE owner_id = $1",
        fixture.creator.0,
    )
    .await;
    for user in [fixture.creator, fixture.in_role, fixture.outside] {
        purge(pool, "DELETE FROM users WHERE id = $1", user.0).await;
    }
}

fn usecase(pool: PgPool) -> MestierUseCase {
    MestierUseCase::new(pool, default_authorizer(), EventHub::new())
}

fn command(org: OrganizationId, channel_type: ChannelType) -> CreateChannelCommand {
    CreateChannelCommand {
        organization_id: org,
        channel_type,
        category_id: None,
        name: format!("canal-{}", generate_uuid_v7()),
        topic: None,
        position: 0,
    }
}

async fn can(usecase: &MestierUseCase, user: UserId, channel: ChannelId, bit: Permissions) -> bool {
    usecase
        .resolve_channel_permissions(user, channel)
        .await
        .map(|permissions| permissions.contains(bit))
        .unwrap_or(false)
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_private_channel_is_open_to_its_roles_and_its_creator_only() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let channel = usecase
        .create_private_channel(
            command(fixture.org, ChannelType::Text),
            fixture.creator,
            vec![fixture.role],
        )
        .await
        .unwrap();

    for user in [fixture.in_role, fixture.creator] {
        assert!(can(&usecase, user, channel.id, Permissions::VIEW_CHANNEL).await);
        assert!(can(&usecase, user, channel.id, Permissions::SEND_MESSAGES).await);
    }
    assert!(
        !can(
            &usecase,
            fixture.outside,
            channel.id,
            Permissions::VIEW_CHANNEL
        )
        .await
    );
    assert!(
        !can(
            &usecase,
            fixture.outside,
            channel.id,
            Permissions::SEND_MESSAGES
        )
        .await
    );

    let listed_for_outside = usecase
        .list_visible_channels(fixture.outside, fixture.org)
        .await
        .unwrap();
    let listed_for_role = usecase
        .list_visible_channels(fixture.in_role, fixture.org)
        .await
        .unwrap();
    assert!(listed_for_outside.iter().all(|c| c.id != channel.id));
    assert!(listed_for_role.iter().any(|c| c.id == channel.id));

    cleanup(&pool, &fixture).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_private_channel_needs_roles_of_its_own_organization() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let none = usecase
        .create_private_channel(
            command(fixture.org, ChannelType::Text),
            fixture.creator,
            vec![],
        )
        .await;
    let foreign = usecase
        .create_private_channel(
            command(fixture.org, ChannelType::Text),
            fixture.creator,
            vec![fixture.other_org_role],
        )
        .await;

    assert!(matches!(none, Err(CoreError::Conflict(_))));
    assert!(matches!(foreign, Err(CoreError::Conflict(_))));
    let channels = usecase.list_channels(fixture.org).await.unwrap();
    assert!(
        channels.is_empty(),
        "a refused channel must leave nothing behind"
    );

    cleanup(&pool, &fixture).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_announcement_channel_is_written_by_managers_only() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let channel = usecase
        .create_channel(command(fixture.org, ChannelType::Announcement))
        .await
        .unwrap();

    assert!(
        can(
            &usecase,
            fixture.in_role,
            channel.id,
            Permissions::VIEW_CHANNEL
        )
        .await
    );
    assert!(
        !can(
            &usecase,
            fixture.in_role,
            channel.id,
            Permissions::SEND_MESSAGES
        )
        .await
    );
    assert!(
        can(
            &usecase,
            fixture.creator,
            channel.id,
            Permissions::SEND_MESSAGES
        )
        .await
    );

    cleanup(&pool, &fixture).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn nobody_sends_in_a_voice_channel_and_it_is_listed() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let channel = usecase
        .create_channel(command(fixture.org, ChannelType::Voice))
        .await
        .unwrap();

    assert!(
        can(
            &usecase,
            fixture.creator,
            channel.id,
            Permissions::VIEW_CHANNEL
        )
        .await
    );
    assert!(
        !can(
            &usecase,
            fixture.creator,
            channel.id,
            Permissions::SEND_MESSAGES
        )
        .await
    );
    let listed = usecase.list_channels(fixture.org).await.unwrap();
    assert!(listed.iter().any(|c| c.id == channel.id));

    cleanup(&pool, &fixture).await;
}

fn category_command(org: OrganizationId) -> CreateCategoryCommand {
    CreateCategoryCommand {
        organization_id: org,
        name: format!("categorie-{}", generate_uuid_v7()),
        position: 0,
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_private_category_closes_the_channels_in_it_to_everyone_else() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let category = usecase
        .create_private_category(
            category_command(fixture.org),
            fixture.creator,
            vec![fixture.role],
        )
        .await
        .unwrap();
    let mut in_category = command(fixture.org, ChannelType::Text);
    in_category.category_id = Some(category.id);
    let channel = usecase.create_channel(in_category).await.unwrap();

    for user in [fixture.in_role, fixture.creator] {
        assert!(can(&usecase, user, channel.id, Permissions::VIEW_CHANNEL).await);
    }
    assert!(
        !can(
            &usecase,
            fixture.outside,
            channel.id,
            Permissions::VIEW_CHANNEL
        )
        .await
    );
    assert!(
        !can(
            &usecase,
            fixture.outside,
            channel.id,
            Permissions::SEND_MESSAGES
        )
        .await
    );

    let channels_for_outside = usecase
        .list_visible_channels(fixture.outside, fixture.org)
        .await
        .unwrap();
    let categories_for_outside = usecase
        .list_visible_categories(fixture.outside, fixture.org)
        .await
        .unwrap();
    let categories_for_role = usecase
        .list_visible_categories(fixture.in_role, fixture.org)
        .await
        .unwrap();
    assert!(channels_for_outside.iter().all(|c| c.id != channel.id));
    assert!(categories_for_outside.iter().all(|c| c.id != category.id));
    assert!(categories_for_role.iter().any(|c| c.id == category.id));

    cleanup(&pool, &fixture).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_open_category_stays_open_and_a_private_one_needs_valid_roles() {
    let pool = dev_pool().await;
    let fixture = seed(&pool).await;
    let usecase = usecase(pool.clone());

    let open = usecase
        .create_category(category_command(fixture.org))
        .await
        .unwrap();
    let mut in_open = command(fixture.org, ChannelType::Text);
    in_open.category_id = Some(open.id);
    let channel = usecase.create_channel(in_open).await.unwrap();
    assert!(
        can(
            &usecase,
            fixture.outside,
            channel.id,
            Permissions::VIEW_CHANNEL
        )
        .await
    );

    let none = usecase
        .create_private_category(category_command(fixture.org), fixture.creator, vec![])
        .await;
    let foreign = usecase
        .create_private_category(
            category_command(fixture.org),
            fixture.creator,
            vec![fixture.other_org_role],
        )
        .await;
    assert!(matches!(none, Err(CoreError::Conflict(_))));
    assert!(matches!(foreign, Err(CoreError::Conflict(_))));
    let categories = usecase.list_categories(fixture.org).await.unwrap();
    assert_eq!(
        categories.len(),
        1,
        "a refused category must leave nothing behind"
    );

    cleanup(&pool, &fixture).await;
}
