use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerId, MemberId, OrganizationId, PaymentMethod, SupplierId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;

pub use error::{ExpenseError, ExpenseField, ExpenseOutcome};
pub use service::{ExpenseCategoryService, ExpenseService};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ExpenseId(pub Uuid);

impl FromStr for ExpenseId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ExpenseId)
    }
}

impl Display for ExpenseId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ExpenseCategoryId(pub Uuid);

impl FromStr for ExpenseCategoryId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ExpenseCategoryId)
    }
}

impl Display for ExpenseCategoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpenseStatus {
    ToPay,
    Paid,
}

impl ExpenseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ToPay => "TO_PAY",
            Self::Paid => "PAID",
        }
    }
}

impl FromStr for ExpenseStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TO_PAY" => Ok(Self::ToPay),
            "PAID" => Ok(Self::Paid),
            other => Err(format!("invalid expense status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpenseDisplayState {
    ToPay,
    Overdue,
    Paid,
    ToReimburse,
    Reimbursed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpenseCategory {
    pub id: ExpenseCategoryId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hue: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expense {
    pub id: ExpenseId,
    pub organization_id: OrganizationId,
    pub date: NaiveDate,
    pub label: String,
    pub supplier_id: Option<SupplierId>,
    pub category_id: ExpenseCategoryId,
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    pub vat_cents: i32,
    pub due_date: Option<NaiveDate>,
    pub status: ExpenseStatus,
    pub paid_on: Option<NaiveDate>,
    pub method: Option<PaymentMethod>,
    pub advanced_by_member_id: Option<MemberId>,
    pub reimbursed_on: Option<NaiveDate>,
    pub customer_id: Option<CustomerId>,
    pub proof_file_key: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub supplier_name: Option<String>,
    pub customer_name: Option<String>,
}

impl Expense {
    pub fn total_cents(&self) -> i64 {
        i64::from(self.amount_ht_cents) + i64::from(self.vat_cents)
    }

    pub fn display_state(&self, today: NaiveDate) -> ExpenseDisplayState {
        if self.advanced_by_member_id.is_some() {
            return if self.reimbursed_on.is_none() {
                ExpenseDisplayState::ToReimburse
            } else {
                ExpenseDisplayState::Reimbursed
            };
        }
        match self.status {
            ExpenseStatus::Paid => ExpenseDisplayState::Paid,
            ExpenseStatus::ToPay if self.days_late(today) > 0 => ExpenseDisplayState::Overdue,
            ExpenseStatus::ToPay => ExpenseDisplayState::ToPay,
        }
    }

    pub fn days_late(&self, today: NaiveDate) -> i64 {
        self.due_date
            .filter(|due| *due < today)
            .map_or(0, |due| (today - due).num_days())
    }
}

pub(crate) fn clean_text(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}
