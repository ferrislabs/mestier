use std::{fmt, fmt::Display, str::FromStr};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use common::UserId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerId, OrganizationId, PaymentMethod};

pub mod commands;
pub mod model;
pub mod ports;
pub mod service;

pub use model::{
    DocumentVatRow, Ledger, LedgerEntry, LedgerKind, LedgerStatus, LedgerSummary, MonthTrend,
    Payables, ReceiptRow, VatSummary, build_ledger,
};
pub use service::{LedgerService, ReceiptService};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ReceiptId(pub Uuid);

impl FromStr for ReceiptId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ReceiptId)
    }
}

impl Display for ReceiptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Receipt {
    pub id: ReceiptId,
    pub organization_id: OrganizationId,
    pub customer_id: Option<CustomerId>,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub recorded_by: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LedgerMonth {
    first: NaiveDate,
}

impl LedgerMonth {
    pub fn of(date: NaiveDate) -> Self {
        Self {
            first: date.with_day(1).unwrap_or(date),
        }
    }

    pub fn contains(self, date: NaiveDate) -> bool {
        Self::of(date) == self
    }

    pub fn key(self) -> String {
        format!("{:04}-{:02}", self.first.year(), self.first.month())
    }

    pub fn previous(self) -> Self {
        let last_of_previous = self.first.pred_opt().unwrap_or(self.first);
        Self::of(last_of_previous)
    }
}

impl Display for LedgerMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("month: must be formatted YYYY-MM")]
pub struct LedgerMonthError;

impl FromStr for LedgerMonth {
    type Err = LedgerMonthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (year, month) = s.split_once('-').ok_or(LedgerMonthError)?;
        if year.len() != 4 || month.len() != 2 {
            return Err(LedgerMonthError);
        }
        let year: i32 = year.parse().map_err(|_| LedgerMonthError)?;
        let month: u32 = month.parse().map_err(|_| LedgerMonthError)?;
        NaiveDate::from_ymd_opt(year, month, 1)
            .map(|first| Self { first })
            .ok_or(LedgerMonthError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LedgerError {
    #[error("amount_cents: must be greater than zero")]
    AmountNotPositive,
    #[error("paid_on: must not be in the future")]
    PaidOnInFuture,
}

impl LedgerError {
    pub fn field(self) -> &'static str {
        match self {
            Self::AmountNotPositive => "amount_cents",
            Self::PaidOnInFuture => "paid_on",
        }
    }
}

pub type LedgerOutcome<T> = Result<T, LedgerError>;
