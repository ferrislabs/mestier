use common::CoreError;
use mestier_macros::repository;

use crate::{
    MemberDocument, MemberDocumentId, OrganizationId,
    domain::member_document::ports::MemberDocumentRepository,
    infrastructure::{
        member_document::postgres::model::MemberDocumentRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = MemberDocument, backend = Postgres)]
pub struct PgMemberDocumentRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMemberDocumentRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> MemberDocumentRepository for PgMemberDocumentRepository<'tx> {
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<MemberDocument>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberDocumentRow,
            r#"
            SELECT id, org_id, member_id, name, kind, file_key, added_at
            FROM member_documents
            WHERE org_id = $1
            ORDER BY added_at, id
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(MemberDocument::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: MemberDocumentId,
    ) -> Result<Option<MemberDocument>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberDocumentRow,
            r#"
            SELECT id, org_id, member_id, name, kind, file_key, added_at
            FROM member_documents
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(MemberDocument::try_from).transpose()
    }

    async fn insert(&mut self, document: &MemberDocument) -> Result<MemberDocument, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberDocumentRow,
            r#"
            INSERT INTO member_documents (id, org_id, member_id, name, kind, file_key, added_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, member_id, name, kind, file_key, added_at
            "#,
            document.id.0,
            document.organization_id.0,
            document.member_id.0,
            document.name.as_str(),
            document.kind.as_str(),
            document.file_key.as_str(),
            document.added_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        MemberDocument::try_from(row)
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: MemberDocumentId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM member_documents WHERE org_id = $1 AND id = $2",
            organization_id.0,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
