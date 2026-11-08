use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::test_support::scratch_pool;

static MIGRATION_SCRATCH: OnceCell<String> = OnceCell::const_new();

const BEFORE_ABSENCE_TYPES: i64 = 20261110000002;

async fn count(pool: &PgPool, statement: &str) -> i64 {
    sqlx::query_scalar(statement)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|error| panic!("`{statement}` failed: {error}"))
}

async fn seed_legacy_absences(pool: &PgPool) -> Uuid {
    let owner = Uuid::now_v7();
    let organization = Uuid::now_v7();
    let member = Uuid::now_v7();
    sqlx::query("INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Owner', $4)")
        .bind(owner)
        .bind(format!("owner-{owner}@example.com"))
        .bind(format!("owner-{owner}"))
        .bind(format!("sub-{owner}"))
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id, timezone) VALUES ($1, 'Org', $2, $3, 'Europe/Paris')")
        .bind(organization)
        .bind(format!("org-{organization}"))
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Alice')")
        .bind(member)
        .bind(organization)
        .execute(pool)
        .await
        .unwrap();
    for (kind, starts_at, ends_at, all_day) in [
        ("LEAVE", "2026-07-05 22:00+00", "2026-07-07 22:00+00", true),
        ("SICK", "2026-01-12 08:00+00", "2026-01-12 09:00+00", false),
        (
            "UNAVAILABLE",
            "2026-01-12 13:00+00",
            "2026-01-13 11:00+00",
            false,
        ),
    ] {
        sqlx::query(
            "INSERT INTO absences (id, org_id, member_id, kind, starts_at, ends_at, all_day)
             VALUES ($1, $2, $3, $4::absence_kind, $5::timestamptz, $6::timestamptz, $7)",
        )
        .bind(Uuid::now_v7())
        .bind(organization)
        .bind(member)
        .bind(kind)
        .bind(starts_at)
        .bind(ends_at)
        .bind(all_day)
        .execute(pool)
        .await
        .unwrap();
    }
    organization
}

async fn assert_migrated(pool: &PgPool, organization: Uuid) {
    let rows: Vec<(String, String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT a.kind::text, t.name, a.status, a.start_date::text, a.start_part, a.end_date::text, a.end_part
         FROM absences a JOIN absence_types t ON t.id = a.type_id AND t.org_id = a.org_id
         WHERE a.org_id = $1 ORDER BY a.starts_at, a.kind::text",
    )
    .bind(organization)
    .fetch_all(pool)
    .await
    .unwrap();
    let expected = [
        (
            "SICK",
            "Arrêt maladie",
            "2026-01-12",
            "MORNING",
            "2026-01-12",
            "MORNING",
        ),
        (
            "UNAVAILABLE",
            "Intempéries",
            "2026-01-12",
            "AFTERNOON",
            "2026-01-13",
            "MORNING",
        ),
        (
            "LEAVE",
            "Congés payés",
            "2026-07-06",
            "MORNING",
            "2026-07-07",
            "AFTERNOON",
        ),
    ];
    assert_eq!(rows.len(), expected.len());
    for (row, (kind, name, start, start_part, end, end_part)) in rows.iter().zip(expected) {
        assert_eq!(
            (row.0.as_str(), row.1.as_str(), row.2.as_str()),
            (kind, name, "APPROVED")
        );
        assert_eq!(
            (
                row.3.as_str(),
                row.4.as_str(),
                row.5.as_str(),
                row.6.as_str()
            ),
            (start, start_part, end, end_part)
        );
    }
    let types: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM absence_types WHERE org_id = $1")
        .bind(organization)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(types, 7);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_rh_3_7_existing_absences_migrate_to_approved_with_the_default_type_and_the_migration_reverts()
 {
    let pool = scratch_pool("mestier_absence_migration_test_", &MIGRATION_SCRATCH).await;
    let migrator = sqlx::migrate!("../../migrations");

    migrator.undo(&pool, BEFORE_ABSENCE_TYPES).await.unwrap();
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'absence_types'"
        )
        .await,
        0
    );
    let organization = seed_legacy_absences(&pool).await;

    migrator.run(&pool).await.unwrap();
    assert_migrated(&pool, organization).await;

    migrator.undo(&pool, BEFORE_ABSENCE_TYPES).await.unwrap();
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'absence_types'"
        )
        .await,
        0
    );
    assert_eq!(
        count(&pool, "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'absences' AND column_name IN ('type_id', 'status', 'start_part')").await,
        0
    );
    assert_eq!(
        count(
            &pool,
            "SELECT COUNT(*) FROM absences WHERE deleted_at IS NULL"
        )
        .await,
        3
    );

    migrator.run(&pool).await.unwrap();
    assert_migrated(&pool, organization).await;
}
