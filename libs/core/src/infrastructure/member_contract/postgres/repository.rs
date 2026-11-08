use chrono::NaiveDate;
use common::CoreError;
use mestier_macros::repository;

use crate::{
    MemberContract, MemberContractId, MemberId, OrganizationId, ScheduleInput,
    domain::member_contract::ports::MemberContractRepository,
    infrastructure::{
        member_contract::postgres::model::MemberContractRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = MemberContract, backend = Postgres)]
pub struct PgMemberContractRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMemberContractRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn schedule_json(contract: &MemberContract) -> Result<serde_json::Value, CoreError> {
    serde_json::to_value(ScheduleInput::from(&contract.schedule))
        .map_err(|error| CoreError::Internal(format!("member contract schedule: {error}")))
}

impl<'tx> MemberContractRepository for PgMemberContractRepository<'tx> {
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<MemberContract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberContractRow,
            r#"
            SELECT id, org_id, member_id, type AS contract_type, start_date, end_date,
                   trial_end_date, schedule, created_at, updated_at
            FROM member_contracts
            WHERE org_id = $1
            ORDER BY member_id, start_date
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(MemberContract::try_from).collect()
    }

    async fn list_overlapping(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<MemberContract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberContractRow,
            r#"
            SELECT id, org_id, member_id, type AS contract_type, start_date, end_date,
                   trial_end_date, schedule, created_at, updated_at
            FROM member_contracts
            WHERE org_id = $1
              AND start_date <= $3
              AND (end_date IS NULL OR end_date >= $2)
            ORDER BY member_id, start_date
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(MemberContract::try_from).collect()
    }

    async fn list_by_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<Vec<MemberContract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            MemberContractRow,
            r#"
            SELECT id, org_id, member_id, type AS contract_type, start_date, end_date,
                   trial_end_date, schedule, created_at, updated_at
            FROM member_contracts
            WHERE org_id = $1 AND member_id = $2
            ORDER BY start_date
            "#,
            organization_id.0,
            member_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(MemberContract::try_from).collect()
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: MemberContractId,
    ) -> Result<Option<MemberContract>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberContractRow,
            r#"
            SELECT id, org_id, member_id, type AS contract_type, start_date, end_date,
                   trial_end_date, schedule, created_at, updated_at
            FROM member_contracts
            WHERE org_id = $1 AND id = $2
            "#,
            organization_id.0,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(MemberContract::try_from).transpose()
    }

    async fn insert(&mut self, contract: &MemberContract) -> Result<MemberContract, CoreError> {
        let schedule = schedule_json(contract)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberContractRow,
            r#"
            INSERT INTO member_contracts (
                id, org_id, member_id, type, start_date, end_date, trial_end_date, schedule,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, org_id, member_id, type AS contract_type, start_date, end_date,
                      trial_end_date, schedule, created_at, updated_at
            "#,
            contract.id.0,
            contract.organization_id.0,
            contract.member_id.0,
            contract.contract_type.as_str(),
            contract.period.start(),
            contract.period.end(),
            contract.period.trial_end(),
            schedule,
            contract.created_at,
            contract.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        MemberContract::try_from(row)
    }

    async fn update(&mut self, contract: &MemberContract) -> Result<MemberContract, CoreError> {
        let schedule = schedule_json(contract)?;
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MemberContractRow,
            r#"
            UPDATE member_contracts
            SET type = $3, start_date = $4, end_date = $5, trial_end_date = $6, schedule = $7,
                updated_at = $8
            WHERE org_id = $1 AND id = $2
            RETURNING id, org_id, member_id, type AS contract_type, start_date, end_date,
                      trial_end_date, schedule, created_at, updated_at
            "#,
            contract.organization_id.0,
            contract.id.0,
            contract.contract_type.as_str(),
            contract.period.start(),
            contract.period.end(),
            contract.period.trial_end(),
            schedule,
            contract.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        MemberContract::try_from(row)
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: MemberContractId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM member_contracts WHERE org_id = $1 AND id = $2",
            organization_id.0,
            id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
