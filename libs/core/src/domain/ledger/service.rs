use authz::{Authorizer, Resource};
use chrono::{DateTime, NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};

use super::{
    Ledger, LedgerError, LedgerMonth, LedgerOutcome, Receipt, ReceiptId, build_ledger,
    commands::RecordReceiptCommand,
    ports::{LedgerRepository, ReceiptRepository},
};
use crate::{
    OrganizationId,
    application::policy,
    domain::{
        customer::ports::CustomerRepository, expense::ports::ExpenseRepository,
        member::ports::MemberRepository, role::ports::RoleRepository,
    },
};

pub struct LedgerService<L, E>
where
    L: LedgerRepository,
    E: ExpenseRepository,
{
    ledger: L,
    expenses: E,
}

impl<L, E> LedgerService<L, E>
where
    L: LedgerRepository,
    E: ExpenseRepository,
{
    pub fn new(ledger: L, expenses: E) -> Self {
        Self { ledger, expenses }
    }

    pub async fn read(
        &mut self,
        organization_id: OrganizationId,
        month: Option<LedgerMonth>,
        today: NaiveDate,
    ) -> Result<Ledger, CoreError> {
        let receipts = self.ledger.receipts(organization_id).await?;
        let documents = self.ledger.document_vat(organization_id).await?;
        let expenses = self.expenses.list_by_organization(organization_id).await?;
        Ok(build_ledger(receipts, &expenses, &documents, month, today))
    }
}

pub struct ReceiptService<R, Cu, M, Ro, A>
where
    R: ReceiptRepository,
    Cu: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    receipts: R,
    customers: Cu,
    member_repository: M,
    role_repository: Ro,
    authz: A,
}

impl<R, Cu, M, Ro, A> ReceiptService<R, Cu, M, Ro, A>
where
    R: ReceiptRepository,
    Cu: CustomerRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(
        receipts: R,
        customers: Cu,
        member_repository: M,
        role_repository: Ro,
        authz: A,
    ) -> Self {
        Self {
            receipts,
            customers,
            member_repository,
            role_repository,
            authz,
        }
    }

    pub async fn record_receipt(
        &mut self,
        command: RecordReceiptCommand,
        now: DateTime<Utc>,
    ) -> Result<LedgerOutcome<Receipt>, CoreError> {
        if command.amount_cents <= 0 {
            return Ok(Err(LedgerError::AmountNotPositive));
        }
        if command.paid_on > now.date_naive() {
            return Ok(Err(LedgerError::PaidOnInFuture));
        }

        let actor = policy::enrich_for_organization(
            command.actor,
            command.organization_id,
            &mut self.member_repository,
            &mut self.role_repository,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "invoice.manage",
            Resource::new("organization", command.organization_id.0.to_string()),
        )
        .await?;

        if let Some(customer_id) = command.customer_id {
            self.customers
                .find_by_id(customer_id)
                .await?
                .filter(|customer| customer.organization_id == command.organization_id)
                .ok_or(CoreError::NotFound)?;
        }

        let clean = |text: Option<String>| {
            text.map(|text| text.trim().to_owned())
                .filter(|text| !text.is_empty())
        };
        self.receipts
            .insert(&Receipt {
                id: ReceiptId(generate_uuid_v7()),
                organization_id: command.organization_id,
                customer_id: command.customer_id,
                amount_cents: command.amount_cents,
                paid_on: command.paid_on,
                method: command.method,
                reference: clean(command.reference),
                note: clean(command.note),
                recorded_by: command.recorded_by,
                created_at: now,
            })
            .await
            .map(Ok)
    }

    pub async fn get_receipt(&mut self, id: ReceiptId) -> Result<Receipt, CoreError> {
        self.receipts
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }
}

#[cfg(test)]
mod tests;
