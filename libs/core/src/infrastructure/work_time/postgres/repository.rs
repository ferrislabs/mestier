use chrono::NaiveDate;
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    EmployeeId, EmployeeRhythm, EmployeeRhythmId, MemberId, OrganizationId, RhythmSlot, WorkSlot,
    domain::work_time::ports::{RhythmRepository, WorkSlotRepository},
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        work_time::postgres::model::{RhythmRow, RhythmSlotRow, WorkSlotRow},
    },
};

#[repository(domain = EmployeeRhythm, backend = Postgres)]
pub struct PgRhythmRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgRhythmRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> RhythmRepository for PgRhythmRepository<'tx> {
    async fn find_open_by_employee(
        &mut self,
        employee_id: EmployeeId,
    ) -> Result<Option<EmployeeRhythm>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RhythmRow,
            r#"
            SELECT id, org_id, employee_id, effective_from, effective_to, created_at, updated_at
            FROM employee_rhythms
            WHERE employee_id = $1 AND effective_to IS NULL
            ORDER BY effective_from DESC
            LIMIT 1
            "#,
            employee_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let id = EmployeeRhythmId(row.id);
                let slots = fetch_rhythm_slots(&mut tx, id).await?;
                Ok(Some(row.into_employee_rhythm(slots)))
            }
            None => Ok(None),
        }
    }

    async fn find_overlapping(
        &mut self,
        employee_id: EmployeeId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<EmployeeRhythm>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            RhythmRow,
            r#"
            SELECT id, org_id, employee_id, effective_from, effective_to, created_at, updated_at
            FROM employee_rhythms
            WHERE employee_id = $1
              AND effective_from <= $3
              AND (effective_to IS NULL OR effective_to > $2)
            ORDER BY effective_from ASC
            "#,
            employee_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut rhythms = Vec::with_capacity(rows.len());
        for row in rows {
            let id = EmployeeRhythmId(row.id);
            let slots = fetch_rhythm_slots(&mut tx, id).await?;
            rhythms.push(row.into_employee_rhythm(slots));
        }

        Ok(rhythms)
    }

    async fn insert(&mut self, rhythm: &EmployeeRhythm) -> Result<EmployeeRhythm, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RhythmRow,
            r#"
            INSERT INTO employee_rhythms (id, org_id, employee_id, effective_from, effective_to, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, employee_id, effective_from, effective_to, created_at, updated_at
            "#,
            rhythm.id.0,
            rhythm.organization_id.0,
            rhythm.employee_id.0,
            rhythm.effective_from,
            rhythm.effective_to,
            rhythm.created_at,
            rhythm.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        insert_rhythm_slots(&mut tx, rhythm.id, &rhythm.slots).await?;
        Ok(row.into_employee_rhythm(rhythm.slots.clone()))
    }

    async fn update(&mut self, rhythm: &EmployeeRhythm) -> Result<EmployeeRhythm, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RhythmRow,
            r#"
            UPDATE employee_rhythms
            SET effective_to = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, org_id, employee_id, effective_from, effective_to, created_at, updated_at
            "#,
            rhythm.id.0,
            rhythm.effective_to,
            rhythm.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

        replace_rhythm_slots(&mut tx, rhythm.id, &rhythm.slots).await?;
        Ok(row.into_employee_rhythm(rhythm.slots.clone()))
    }

    async fn set_effective_to(
        &mut self,
        id: EmployeeRhythmId,
        effective_to: NaiveDate,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE employee_rhythms
            SET effective_to = $2, updated_at = now()
            WHERE id = $1 AND effective_to IS NULL
            "#,
            id.0,
            effective_to,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

async fn fetch_rhythm_slots(
    conn: &mut PgConnection,
    rhythm_id: EmployeeRhythmId,
) -> Result<Vec<RhythmSlot>, CoreError> {
    let rows = sqlx::query_as!(
        RhythmSlotRow,
        r#"
        SELECT id, rhythm_id, weekday, starts_minute, ends_minute
        FROM employee_rhythm_slots
        WHERE rhythm_id = $1
        ORDER BY weekday ASC, starts_minute ASC
        "#,
        rhythm_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

async fn insert_rhythm_slots(
    conn: &mut PgConnection,
    rhythm_id: EmployeeRhythmId,
    slots: &[RhythmSlot],
) -> Result<(), CoreError> {
    for slot in slots {
        sqlx::query!(
            r#"
            INSERT INTO employee_rhythm_slots (id, rhythm_id, weekday, starts_minute, ends_minute)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            slot.id.0,
            rhythm_id.0,
            slot.weekday,
            slot.starts_minute,
            slot.ends_minute,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}

/// Replaces a rhythm version's complete slot list: physical delete of
/// whatever is there, then insert of the new set. A rhythm slot carries no
/// history worth keeping once removed (unlike the rhythm version itself,
/// whose row survives as history when a later version takes over), and the
/// `PUT` contract treats the slot list as never a delta.
async fn replace_rhythm_slots(
    conn: &mut PgConnection,
    rhythm_id: EmployeeRhythmId,
    slots: &[RhythmSlot],
) -> Result<(), CoreError> {
    sqlx::query!(
        r#"DELETE FROM employee_rhythm_slots WHERE rhythm_id = $1"#,
        rhythm_id.0,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    insert_rhythm_slots(conn, rhythm_id, slots).await
}

#[repository(domain = WorkSlot, backend = Postgres)]
pub struct PgWorkSlotRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWorkSlotRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> WorkSlotRepository for PgWorkSlotRepository<'tx> {
    async fn list_in_window(
        &mut self,
        member_id: MemberId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<WorkSlot>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkSlotRow,
            r#"
            SELECT id, org_id, member_id, work_date, starts_minute, ends_minute
            FROM work_slots
            WHERE member_id = $1 AND work_date BETWEEN $2 AND $3
            ORDER BY work_date ASC, starts_minute ASC
            "#,
            member_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn replace_window(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        from: NaiveDate,
        to: NaiveDate,
        slots: &[WorkSlot],
    ) -> Result<Vec<WorkSlot>, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"
            DELETE FROM work_slots
            WHERE member_id = $1 AND work_date BETWEEN $2 AND $3
            "#,
            member_id.0,
            from,
            to,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for slot in slots {
            sqlx::query!(
                r#"
                INSERT INTO work_slots (id, org_id, member_id, work_date, starts_minute, ends_minute)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                slot.id.0,
                organization_id.0,
                member_id.0,
                slot.work_date,
                slot.starts_minute,
                slot.ends_minute,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(slots.to_vec())
    }
}
