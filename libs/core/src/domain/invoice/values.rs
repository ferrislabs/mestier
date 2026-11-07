use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use common::UserId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethod {
    Card,
    Transfer,
    Check,
    Cash,
    DirectDebit,
}

impl PaymentMethod {
    pub const ALL: [PaymentMethod; 5] = [
        Self::Card,
        Self::Transfer,
        Self::Check,
        Self::Cash,
        Self::DirectDebit,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Card => "CARD",
            Self::Transfer => "TRANSFER",
            Self::Check => "CHECK",
            Self::Cash => "CASH",
            Self::DirectDebit => "DIRECT_DEBIT",
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PaymentMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|method| method.as_str() == s)
            .ok_or_else(|| format!("invalid payment method `{s}`"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceDisplayState {
    Draft,
    Unpaid,
    Partial,
    Overdue,
    Paid,
    Cancelled,
    RefundDue,
}

impl InvoiceDisplayState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Unpaid => "UNPAID",
            Self::Partial => "PARTIAL",
            Self::Overdue => "OVERDUE",
            Self::Paid => "PAID",
            Self::Cancelled => "CANCELLED",
            Self::RefundDue => "REFUND_DUE",
        }
    }
}

impl fmt::Display for InvoiceDisplayState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InvoiceEventKind {
    Sent,
    Reminder,
    Note,
}

impl InvoiceEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sent => "SENT",
            Self::Reminder => "REMINDER",
            Self::Note => "NOTE",
        }
    }
}

impl FromStr for InvoiceEventKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SENT" => Ok(Self::Sent),
            "REMINDER" => Ok(Self::Reminder),
            "NOTE" => Ok(Self::Note),
            other => Err(format!("invalid invoice event kind `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct InvoiceEventId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvoiceEvent {
    pub id: InvoiceEventId,
    pub kind: InvoiceEventKind,
    pub detail: String,
    pub at: DateTime<Utc>,
    pub actor: Option<UserId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InvoiceSettlement {
    pub paid_cents: i64,
    pub credited_cents: i64,
    pub balance_cents: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_payment_method_round_trips_through_its_wire_form() {
        for method in PaymentMethod::ALL {
            assert_eq!(method.as_str().parse::<PaymentMethod>(), Ok(method));
        }
        assert!("Virement".parse::<PaymentMethod>().is_err());
    }

    #[test]
    fn an_event_kind_round_trips_through_its_wire_form() {
        for kind in [
            InvoiceEventKind::Sent,
            InvoiceEventKind::Reminder,
            InvoiceEventKind::Note,
        ] {
            assert_eq!(kind.as_str().parse::<InvoiceEventKind>(), Ok(kind));
        }
        assert!("CALL".parse::<InvoiceEventKind>().is_err());
    }
}
