use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    domain::automation::{
        collection::{
            Collection, CollectionDetail, CollectionSource, CollectionSummary, CollectionVersion,
            MAX_HISTORY_ENTRIES, Operation, SecurityScheme,
        },
        ports::CollectionRepository,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Collection, backend = Postgres)]
pub struct PgCollectionRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgCollectionRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct CollectionRow {
    id: Uuid,
    org_id: Uuid,
    slug: String,
    title: String,
    version: String,
    description: String,
    base_url: String,
    source_kind: String,
    source_value: String,
    imported_at: DateTime<Utc>,
    security: Value,
    operations: Value,
    schemas: Value,
}

struct SummaryRow {
    id: Uuid,
    org_id: Uuid,
    slug: String,
    title: String,
    version: String,
    base_url: String,
    source_kind: String,
    source_value: String,
    imported_at: DateTime<Utc>,
    security: Value,
    operation_count: i32,
}

struct VersionRow {
    version: String,
    imported_at: DateTime<Utc>,
    source_kind: String,
    source_value: String,
    base_url: String,
    security: Value,
}

struct DetailRow {
    id: Uuid,
    org_id: Uuid,
    slug: String,
    title: String,
    version: String,
    description: String,
    base_url: String,
    source_kind: String,
    source_value: String,
    imported_at: DateTime<Utc>,
    security: Value,
    operations: Value,
    schemas: Value,
    history: Value,
}

#[derive(serde::Deserialize)]
struct HistoryJson {
    version: String,
    imported_at: DateTime<Utc>,
    source_kind: String,
    source_value: String,
    base_url: String,
    security: Value,
}

fn corrupt(what: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("invalid collection {what} in database: {error}"))
}

fn source_of(kind: &str, value: String) -> Result<CollectionSource, CoreError> {
    CollectionSource::from_parts(kind, value).ok_or_else(|| corrupt("source", kind))
}

impl TryFrom<CollectionRow> for Collection {
    type Error = CoreError;

    fn try_from(row: CollectionRow) -> Result<Self, Self::Error> {
        let schemas: Map<String, Value> =
            serde_json::from_value(row.schemas).map_err(|error| corrupt("schemas", error))?;
        let security: Vec<SecurityScheme> =
            serde_json::from_value(row.security).map_err(|error| corrupt("security", error))?;
        let operations: Vec<Operation> =
            serde_json::from_value(row.operations).map_err(|error| corrupt("operations", error))?;

        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            slug: row.slug,
            title: row.title,
            version: row.version,
            description: row.description,
            base_url: row.base_url,
            source: source_of(&row.source_kind, row.source_value)?,
            imported_at: row.imported_at,
            security,
            operations,
            schemas,
        })
    }
}

impl TryFrom<SummaryRow> for CollectionSummary {
    type Error = CoreError;

    fn try_from(row: SummaryRow) -> Result<Self, Self::Error> {
        let security: Vec<SecurityScheme> =
            serde_json::from_value(row.security).map_err(|error| corrupt("security", error))?;

        Ok(Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            slug: row.slug,
            title: row.title,
            version: row.version,
            base_url: row.base_url,
            source: source_of(&row.source_kind, row.source_value)?,
            imported_at: row.imported_at,
            security,
            operation_count: usize::try_from(row.operation_count).unwrap_or_default(),
        })
    }
}

impl TryFrom<VersionRow> for CollectionVersion {
    type Error = CoreError;

    fn try_from(row: VersionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            version: row.version,
            imported_at: row.imported_at,
            source: source_of(&row.source_kind, row.source_value)?,
            base_url: row.base_url,
            security: serde_json::from_value(row.security)
                .map_err(|error| corrupt("history security", error))?,
        })
    }
}

impl TryFrom<HistoryJson> for CollectionVersion {
    type Error = CoreError;

    fn try_from(row: HistoryJson) -> Result<Self, Self::Error> {
        VersionRow {
            version: row.version,
            imported_at: row.imported_at,
            source_kind: row.source_kind,
            source_value: row.source_value,
            base_url: row.base_url,
            security: row.security,
        }
        .try_into()
    }
}

impl TryFrom<DetailRow> for CollectionDetail {
    type Error = CoreError;

    fn try_from(row: DetailRow) -> Result<Self, Self::Error> {
        let history: Vec<HistoryJson> =
            serde_json::from_value(row.history).map_err(|error| corrupt("history", error))?;
        let collection = Collection::try_from(CollectionRow {
            id: row.id,
            org_id: row.org_id,
            slug: row.slug,
            title: row.title,
            version: row.version,
            description: row.description,
            base_url: row.base_url,
            source_kind: row.source_kind,
            source_value: row.source_value,
            imported_at: row.imported_at,
            security: row.security,
            operations: row.operations,
            schemas: row.schemas,
        })?;

        Ok(Self {
            collection,
            history: history
                .into_iter()
                .map(CollectionVersion::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

fn history_limit() -> i64 {
    i64::try_from(MAX_HISTORY_ENTRIES).unwrap_or(i64::MAX)
}

struct Stored {
    security: Value,
    operations: Value,
    schemas: Value,
}

fn stored(collection: &Collection) -> Result<Stored, CoreError> {
    Ok(Stored {
        security: serde_json::to_value(&collection.security)
            .map_err(|error| corrupt("security", error))?,
        operations: serde_json::to_value(&collection.operations)
            .map_err(|error| corrupt("operations", error))?,
        schemas: serde_json::to_value(&collection.schemas)
            .map_err(|error| corrupt("schemas", error))?,
    })
}

impl<'tx> CollectionRepository for PgCollectionRepository<'tx> {
    async fn lock_slugs(&mut self, org_id: OrganizationId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1::text, 3))",
            org_id.0.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn slugs_matching(
        &mut self,
        org_id: OrganizationId,
        base: &str,
    ) -> Result<Vec<String>, CoreError> {
        let mut tx = self.tx.lock().await;
        let slugs = sqlx::query_scalar!(
            r#"SELECT slug
               FROM automation.collection
               WHERE org_id = $1 AND (slug = $2 OR starts_with(slug, $2 || '-'))"#,
            org_id.0,
            base,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(slugs)
    }

    async fn insert(&mut self, collection: &Collection) -> Result<Collection, CoreError> {
        let model = stored(collection)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CollectionRow,
            r#"INSERT INTO automation.collection
                   (id, org_id, slug, title, version, description, base_url,
                    source_kind, source_value, imported_at, security, operations, schemas)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
               RETURNING id, org_id, slug, title, version, description, base_url,
                         source_kind, source_value, imported_at, security, operations, schemas"#,
            collection.id,
            collection.org_id.0,
            collection.slug,
            collection.title,
            collection.version,
            collection.description,
            collection.base_url,
            collection.source.kind_str(),
            collection.source.value(),
            collection.imported_at,
            model.security,
            model.operations,
            model.schemas,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Collection>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            CollectionRow,
            r#"SELECT id, org_id, slug, title, version, description, base_url,
                      source_kind, source_value, imported_at, security, operations, schemas
               FROM automation.collection
               WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Collection::try_from).transpose()
    }

    async fn find_many(
        &mut self,
        org_id: OrganizationId,
        ids: &[Uuid],
    ) -> Result<Vec<Collection>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            CollectionRow,
            r#"SELECT id, org_id, slug, title, version, description, base_url,
                      source_kind, source_value, imported_at, security, operations, schemas
               FROM automation.collection
               WHERE org_id = $1 AND id = ANY($2)
               ORDER BY id"#,
            org_id.0,
            ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Collection::try_from).collect()
    }

    async fn find_detail(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<CollectionDetail>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            DetailRow,
            r#"SELECT c.id, c.org_id, c.slug, c.title, c.version, c.description, c.base_url,
                      c.source_kind, c.source_value, c.imported_at, c.security,
                      c.operations, c.schemas,
                      COALESCE((
                          SELECT jsonb_agg(
                                     jsonb_build_object(
                                         'version', recent.version,
                                         'imported_at', recent.imported_at,
                                         'source_kind', recent.source_kind,
                                         'source_value', recent.source_value,
                                         'base_url', recent.base_url,
                                         'security', recent.security
                                     )
                                     ORDER BY recent.id DESC
                                 )
                          FROM (
                              SELECT v.id, v.version, v.imported_at, v.source_kind,
                                     v.source_value, v.base_url, v.security
                              FROM automation.collection_version v
                              WHERE v.collection_id = c.id
                              ORDER BY v.id DESC
                              LIMIT $3
                          ) recent
                      ), '[]'::jsonb) AS "history!"
               FROM automation.collection c
               WHERE c.org_id = $1 AND c.id = $2"#,
            org_id.0,
            id,
            history_limit(),
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(CollectionDetail::try_from).transpose()
    }

    async fn count_by_organization(&mut self, org_id: OrganizationId) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;
        let count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM automation.collection WHERE org_id = $1"#,
            org_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(count)
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<CollectionSummary>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            SummaryRow,
            r#"SELECT id, org_id, slug, title, version, base_url,
                      source_kind, source_value, imported_at, security,
                      jsonb_array_length(operations) AS "operation_count!"
               FROM automation.collection
               WHERE org_id = $1
               ORDER BY lower(title), id"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(CollectionSummary::try_from).collect()
    }

    async fn history(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Vec<CollectionVersion>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            VersionRow,
            r#"SELECT v.version, v.imported_at, v.source_kind, v.source_value,
                      v.base_url, v.security
               FROM automation.collection_version v
               JOIN automation.collection c ON c.id = v.collection_id
               WHERE c.org_id = $1 AND c.id = $2
               ORDER BY v.id DESC
               LIMIT $3"#,
            org_id.0,
            id,
            history_limit(),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(CollectionVersion::try_from).collect()
    }

    async fn replace(&mut self, collection: &Collection) -> Result<Collection, CoreError> {
        let model = stored(collection)?;
        let mut tx = self.tx.lock().await;

        let previous = sqlx::query_as!(
            VersionRow,
            r#"SELECT version, imported_at, source_kind, source_value, base_url, security
               FROM automation.collection
               WHERE org_id = $1 AND id = $2
               FOR UPDATE"#,
            collection.org_id.0,
            collection.id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        sqlx::query!(
            r#"INSERT INTO automation.collection_version
                   (collection_id, version, imported_at, source_kind, source_value,
                    base_url, security)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            collection.id,
            previous.version,
            previous.imported_at,
            previous.source_kind,
            previous.source_value,
            previous.base_url,
            previous.security,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = sqlx::query_as!(
            CollectionRow,
            r#"UPDATE automation.collection
               SET title = $3, version = $4, description = $5, base_url = $6,
                   source_kind = $7, source_value = $8, imported_at = $9,
                   security = $10, operations = $11, schemas = $12
               WHERE org_id = $1 AND id = $2
               RETURNING id, org_id, slug, title, version, description, base_url,
                         source_kind, source_value, imported_at, security, operations, schemas"#,
            collection.org_id.0,
            collection.id,
            collection.title,
            collection.version,
            collection.description,
            collection.base_url,
            collection.source.kind_str(),
            collection.source.value(),
            collection.imported_at,
            model.security,
            model.operations,
            model.schemas,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn delete(&mut self, org_id: OrganizationId, id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let outcome = sqlx::query!(
            "DELETE FROM automation.collection WHERE org_id = $1 AND id = $2",
            org_id.0,
            id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if outcome.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use serde_json::json;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::{automation_pool, now_storable};
    use crate::domain::automation::collection::{
        ApiKeyLocation, HttpMethod, Parameter, ParameterLocation, RequestBody, SecurityKind,
    };
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn operations() -> Vec<Operation> {
        vec![
            Operation {
                id: "getAddress".to_owned(),
                method: HttpMethod::Get,
                path: "/addresses/{id}".to_owned(),
                summary: "Read one address".to_owned(),
                tag: "Addresses".to_owned(),
                parameters: vec![
                    Parameter {
                        name: "id".to_owned(),
                        location: ParameterLocation::Path,
                        required: true,
                        description: "The address".to_owned(),
                        schema: json!({ "type": "string" }),
                    },
                    Parameter {
                        name: "limit".to_owned(),
                        location: ParameterLocation::Query,
                        required: false,
                        description: String::new(),
                        schema: json!({ "type": "integer" }),
                    },
                ],
                request_body: None,
                response: Some(json!({ "$ref": "#/components/schemas/Address" })),
                response_status: Some("200".to_owned()),
            },
            Operation {
                id: "createAddress".to_owned(),
                method: HttpMethod::Post,
                path: "/addresses".to_owned(),
                summary: "Create an address".to_owned(),
                tag: "Addresses".to_owned(),
                parameters: Vec::new(),
                request_body: Some(RequestBody {
                    schema: json!({ "type": "object", "required": ["street"] }),
                }),
                response: None,
                response_status: Some("201".to_owned()),
            },
        ]
    }

    fn collection(org_id: OrganizationId, title: &str, slug: &str) -> Collection {
        let mut schemas = Map::new();
        schemas.insert(
            "Address".to_owned(),
            json!({ "type": "object", "properties": { "street": { "type": "string" } } }),
        );
        Collection {
            id: generate_uuid_v7(),
            org_id,
            slug: slug.to_owned(),
            title: title.to_owned(),
            version: "1.0.0".to_owned(),
            description: "Addresses".to_owned(),
            base_url: "https://api.example.com/v1".to_owned(),
            source: CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned(),
            },
            imported_at: now_storable(),
            security: vec![
                SecurityScheme {
                    name: "bearerAuth".to_owned(),
                    kind: SecurityKind::Http {
                        scheme: "bearer".to_owned(),
                    },
                },
                SecurityScheme {
                    name: "keyAuth".to_owned(),
                    kind: SecurityKind::ApiKey {
                        location: ApiKeyLocation::Header,
                        parameter_name: "X-API-Key".to_owned(),
                    },
                },
            ],
            operations: operations(),
            schemas,
        }
    }

    async fn insert(pool: &PgPool, collection: &Collection) -> Result<Collection, CoreError> {
        with_tx(pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.insert(collection).await
        })
        .await
    }

    async fn history_of(pool: &PgPool, org_id: OrganizationId, id: Uuid) -> Vec<CollectionVersion> {
        with_tx(pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.history(org_id, id).await
        })
        .await
        .unwrap()
    }

    async fn find(pool: &PgPool, org_id: OrganizationId, id: Uuid) -> Option<Collection> {
        with_tx(pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.find_by_id(org_id, id).await
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_inserted_collection_reads_back_with_its_whole_model() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-roundtrip").await;
        let written = collection(org_id, "Adresses", "adresses");

        let inserted = insert(&pool, &written).await.unwrap();

        assert_eq!(inserted, written);
        assert_eq!(find(&pool, org_id, written.id).await, Some(written));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_collection_never_leaves_its_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-isolation").await;
        let stranger = seed_organization(&pool, "collection-isolation-stranger").await;
        let written = collection(org_id, "Adresses", "adresses");
        insert(&pool, &written).await.unwrap();

        let (found, many, listed, history) = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            let found = repo.find_by_id(stranger, written.id).await?;
            let many = repo.find_many(stranger, &[written.id]).await?;
            let listed = repo.list_by_organization(stranger).await?;
            let history = repo.history(stranger, written.id).await?;
            Ok::<_, CoreError>((found, many, listed, history))
        })
        .await
        .unwrap();

        assert_eq!(found, None);
        assert!(many.is_empty());
        assert!(listed.is_empty());
        assert!(history.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_list_is_light_scoped_and_ordered_by_title() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-list").await;
        let other = seed_organization(&pool, "collection-list-other").await;
        insert(&pool, &collection(org_id, "zeta", "zeta"))
            .await
            .unwrap();
        insert(&pool, &collection(org_id, "Alpha", "alpha"))
            .await
            .unwrap();
        insert(&pool, &collection(other, "Not mine", "not-mine"))
            .await
            .unwrap();

        let listed = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.list_by_organization(org_id).await
        })
        .await
        .unwrap();

        let titles: Vec<&str> = listed.iter().map(|item| item.title.as_str()).collect();
        assert_eq!(titles, ["Alpha", "zeta"]);
        assert_eq!(listed[0].operation_count, 2);
        assert_eq!(listed[0].slug, "alpha");
        assert_eq!(listed[0].security.len(), 2);
        assert_eq!(listed[0].org_id, org_id);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_many_returns_only_the_requested_collections_of_the_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-many").await;
        let first = insert(&pool, &collection(org_id, "One", "one"))
            .await
            .unwrap();
        insert(&pool, &collection(org_id, "Two", "two"))
            .await
            .unwrap();
        let third = insert(&pool, &collection(org_id, "Three", "three"))
            .await
            .unwrap();

        let mut found = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.find_many(org_id, &[first.id, third.id, Uuid::from_u128(9)])
                .await
        })
        .await
        .unwrap();
        found.sort_by(|a, b| a.title.cmp(&b.title));

        assert_eq!(found, vec![first, third]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn slugs_matching_returns_the_base_and_its_suffixes_of_this_organization_only() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-slugs").await;
        let other = seed_organization(&pool, "collection-slugs-other").await;
        for slug in ["api", "api-2", "api-extra", "apis", "other"] {
            insert(&pool, &collection(org_id, slug, slug))
                .await
                .unwrap();
        }
        insert(&pool, &collection(other, "api", "api-3"))
            .await
            .unwrap();

        let mut slugs = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.slugs_matching(org_id, "api").await
        })
        .await
        .unwrap();
        slugs.sort();

        assert_eq!(slugs, ["api", "api-2", "api-extra"]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_collections_cannot_share_a_slug_in_one_organization_but_can_across_two() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-unique").await;
        let other = seed_organization(&pool, "collection-unique-other").await;
        insert(&pool, &collection(org_id, "Api", "api"))
            .await
            .unwrap();

        let clash = insert(&pool, &collection(org_id, "Api again", "api")).await;
        let elsewhere = insert(&pool, &collection(other, "Api", "api")).await;

        assert!(matches!(clash, Err(CoreError::Conflict(_))), "{clash:?}");
        assert!(elsewhere.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_appends_the_previous_version_to_the_history_newest_first() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-replace").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();

        let mut second = first.clone();
        second.version = "2.0.0".to_owned();
        second.imported_at = now_storable();
        second.source = CollectionSource::File {
            file_name: "adresses-v2.yaml".to_owned(),
        };
        second.operations.truncate(1);
        let replaced = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&second).await
        })
        .await
        .unwrap();
        assert_eq!(replaced, second);

        let mut third = second.clone();
        third.version = "3.0.0".to_owned();
        third.imported_at = now_storable();
        with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&third).await
        })
        .await
        .unwrap();

        let history = history_of(&pool, org_id, first.id).await;
        assert_eq!(history, vec![second.version_entry(), first.version_entry()]);
        assert_eq!(find(&pool, org_id, first.id).await, Some(third));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_keeps_the_replaced_versions_base_url_and_security_in_the_history() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-history-fields").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let mut second = first.clone();
        second.version = "2.0.0".to_owned();
        second.base_url = "https://api.example.com/v2".to_owned();
        second.security = Vec::new();
        with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&second).await
        })
        .await
        .unwrap();

        let history = history_of(&pool, org_id, first.id).await;

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].base_url, "https://api.example.com/v1");
        assert_eq!(history[0].security, first.security);
        assert_eq!(history[0], first.version_entry());
    }

    async fn replace_n_times(pool: &PgPool, first: &Collection, count: usize) -> Collection {
        let mut current = first.clone();
        for n in 1..=count {
            current.version = format!("{n}");
            current.imported_at = now_storable();
            current = with_tx(pool, async |tx| {
                let mut repo = PgCollectionRepository::new(&tx);
                repo.replace(&current).await
            })
            .await
            .unwrap();
        }
        current
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_history_read_keeps_the_fifty_latest_versions_while_the_table_keeps_all() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-history-cap").await;
        let mut first = collection(org_id, "Adresses", "adresses");
        first.version = "0".to_owned();
        let first = insert(&pool, &first).await.unwrap();

        replace_n_times(&pool, &first, 55).await;

        let history = history_of(&pool, org_id, first.id).await;
        let versions: Vec<&str> = history.iter().map(|entry| entry.version.as_str()).collect();
        assert_eq!(versions.len(), 50);
        assert_eq!(versions.first(), Some(&"54"));
        assert_eq!(versions.last(), Some(&"5"));
        let stored: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM automation.collection_version WHERE collection_id = $1",
        )
        .bind(first.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(stored, 55);
        let detail = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.find_detail(org_id, first.id).await
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(detail.history, history);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn collections_are_counted_per_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-count").await;
        let other = seed_organization(&pool, "collection-count-other").await;
        insert(&pool, &collection(org_id, "One", "one"))
            .await
            .unwrap();
        insert(&pool, &collection(org_id, "Two", "two"))
            .await
            .unwrap();
        insert(&pool, &collection(other, "Three", "three"))
            .await
            .unwrap();

        let (mine, theirs, nobody) = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            let mine = repo.count_by_organization(org_id).await?;
            let theirs = repo.count_by_organization(other).await?;
            let nobody = repo
                .count_by_organization(OrganizationId(Uuid::from_u128(5)))
                .await?;
            Ok::<_, CoreError>((mine, theirs, nobody))
        })
        .await
        .unwrap();

        assert_eq!((mine, theirs, nobody), (2, 1, 0));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_detail_is_the_collection_and_its_history_and_never_crosses_organizations() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-detail").await;
        let stranger = seed_organization(&pool, "collection-detail-stranger").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let latest = replace_n_times(&pool, &first, 3).await;

        let (mine, theirs) = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            let mine = repo.find_detail(org_id, first.id).await?;
            let theirs = repo.find_detail(stranger, first.id).await?;
            Ok::<_, CoreError>((mine, theirs))
        })
        .await
        .unwrap();

        let detail = mine.expect("the owner reads it");
        assert_eq!(detail.collection, latest);
        assert_eq!(detail.history, history_of(&pool, org_id, first.id).await);
        assert_eq!(detail.history.len(), 3);
        assert_eq!(theirs, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires live postgres"]
    async fn a_detail_read_during_replacements_always_sees_one_consistent_snapshot() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-snapshot").await;
        let mut first = collection(org_id, "Adresses", "adresses");
        first.version = "0".to_owned();
        let first = insert(&pool, &first).await.unwrap();

        let writer = async {
            replace_n_times(&pool, &first, 40).await;
        };
        let reader = async {
            let mut inconsistent = Vec::new();
            for _ in 0..300 {
                let detail = with_tx(&pool, async |tx| {
                    let mut repo = PgCollectionRepository::new(&tx);
                    repo.find_detail(org_id, first.id).await
                })
                .await
                .unwrap()
                .unwrap();
                let version: usize = detail.collection.version.parse().unwrap();
                if detail.history.len() != version {
                    inconsistent.push((version, detail.history.len()));
                }
            }
            inconsistent
        };
        let (_, inconsistent) = tokio::join!(writer, reader);

        assert_eq!(inconsistent, Vec::new());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_replacement_that_fails_leaves_no_history_and_keeps_the_collection() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-atomic").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();

        let mut broken = first.clone();
        broken.version = "2.0.0".to_owned();
        broken.title = "t".repeat(250);
        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&broken).await
        })
        .await;

        assert!(outcome.is_err());
        assert!(history_of(&pool, org_id, first.id).await.is_empty());
        assert_eq!(find(&pool, org_id, first.id).await, Some(first));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_rolled_back_replacement_takes_its_history_entry_with_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-rollback").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let mut next = first.clone();
        next.version = "2.0.0".to_owned();

        let outcome: Result<(), CoreError> = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&next).await?;
            Err(CoreError::Internal(
                "abort after the replacement".to_owned(),
            ))
        })
        .await;

        assert!(outcome.is_err());
        assert!(history_of(&pool, org_id, first.id).await.is_empty());
        assert_eq!(find(&pool, org_id, first.id).await, Some(first));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_a_collection_of_another_organization_is_not_found_and_changes_nothing() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-replace-owner").await;
        let stranger = seed_organization(&pool, "collection-replace-stranger").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let mut hijack = first.clone();
        hijack.org_id = stranger;
        hijack.title = "Hijacked".to_owned();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&hijack).await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)), "{outcome:?}");
        assert!(history_of(&pool, org_id, first.id).await.is_empty());
        assert_eq!(find(&pool, org_id, first.id).await, Some(first));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_removes_the_collection_and_its_history() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-delete").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let mut next = first.clone();
        next.version = "2.0.0".to_owned();
        with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&next).await
        })
        .await
        .unwrap();
        assert_eq!(history_of(&pool, org_id, first.id).await.len(), 1);

        with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.delete(org_id, first.id).await
        })
        .await
        .unwrap();

        assert_eq!(find(&pool, org_id, first.id).await, None);
        let leftovers: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM automation.collection_version WHERE collection_id = $1",
        )
        .bind(first.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(leftovers, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_collection_of_another_organization_is_not_found_and_removes_nothing() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-delete-owner").await;
        let stranger = seed_organization(&pool, "collection-delete-stranger").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.delete(stranger, first.id).await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)), "{outcome:?}");
        assert_eq!(find(&pool, org_id, first.id).await, Some(first));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_history_is_append_only() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-append-only").await;
        let first = insert(&pool, &collection(org_id, "Adresses", "adresses"))
            .await
            .unwrap();
        let mut next = first.clone();
        next.version = "2.0.0".to_owned();
        with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.replace(&next).await
        })
        .await
        .unwrap();

        let rewrite = sqlx::query(
            "UPDATE automation.collection_version SET version = 'forged' WHERE collection_id = $1",
        )
        .bind(first.id)
        .execute(&pool)
        .await;

        assert!(rewrite.is_err());
        assert_eq!(
            history_of(&pool, org_id, first.id).await,
            vec![first.version_entry()]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_lock_and_the_slug_lookup_work_inside_one_transaction() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "collection-lock").await;

        let slugs = with_tx(&pool, async |tx| {
            let mut repo = PgCollectionRepository::new(&tx);
            repo.lock_slugs(org_id).await?;
            repo.slugs_matching(org_id, "anything").await
        })
        .await
        .unwrap();

        assert!(slugs.is_empty());
    }
}
