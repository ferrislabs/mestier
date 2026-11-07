use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{oauth::OauthState, ports::OauthStateRepository, secret::SealedSecret},
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = OauthState, backend = Postgres)]
pub struct PgOauthStateRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgOauthStateRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> OauthStateRepository for PgOauthStateRepository<'tx> {
    async fn insert(&mut self, state_hash: &[u8], state: &OauthState) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"INSERT INTO automation.oauth_state
                   (state_hash, org_id, credential_id, member_id, redirect_uri, return_to,
                    payload_nonce, payload_cipher, created_at, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
            state_hash,
            state.org_id.0,
            state.credential_id,
            state.member_id,
            state.redirect_uri,
            state.return_to,
            state
                .payload
                .as_ref()
                .map(|payload| payload.nonce.as_slice()),
            state
                .payload
                .as_ref()
                .map(|payload| payload.ciphertext.as_slice()),
            state.created_at,
            state.expires_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn consume(
        &mut self,
        state_hash: &[u8],
        now: DateTime<Utc>,
    ) -> Result<Option<OauthState>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"DELETE FROM automation.oauth_state
               WHERE state_hash = $1 AND expires_at > $2
               RETURNING org_id, credential_id, member_id, redirect_uri, return_to,
                         payload_nonce, payload_cipher, created_at, expires_at"#,
            state_hash,
            now,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|row| OauthState {
            org_id: OrganizationId(row.org_id),
            credential_id: row.credential_id,
            member_id: row.member_id,
            redirect_uri: row.redirect_uri,
            return_to: row.return_to,
            payload: match (row.payload_nonce, row.payload_cipher) {
                (Some(nonce), Some(ciphertext)) => Some(SealedSecret { nonce, ciphertext }),
                _ => None,
            },
            created_at: row.created_at,
            expires_at: row.expires_at,
        }))
    }

    async fn purge_expired(&mut self, now: DateTime<Utc>) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM automation.oauth_state WHERE expires_at <= $1",
            now
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected())
    }

    async fn count_for_credential(&mut self, credential_id: Uuid) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM automation.oauth_state WHERE credential_id = $1"#,
            credential_id
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)
    }
}
