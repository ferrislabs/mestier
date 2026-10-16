//! The maintenance contract an assistant drafts from a paper sheet.
//!
//! The assistant proposes, a person validates: the request is built here and
//! always carries the status `DRAFT`. The model has no field to change that,
//! and the API refuses anything else from a delegated token.

use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::api::Fault;

const DRAFT: &str = "DRAFT";

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct Season {
    #[schemars(description = "Calendar year of the season, for example 2026.")]
    pub year: i32,
    #[schemars(description = "First ISO week of the season, 1 to 53.")]
    pub first_week: u8,
    #[schemars(
        description = "Last ISO week of the season, 1 to 53, not before first_week and existing in the year."
    )]
    pub last_week: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct Window {
    #[schemars(description = "First ISO week of the window.")]
    pub from_week: u8,
    #[schemars(description = "Last ISO week of the window, not before from_week.")]
    pub to_week: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Cadence {
    /// Due `every_days` after the last visit a person validated, for example
    /// "tonte toutes les 2 semaines" is 14.
    Rolling {
        #[schemars(description = "Days between two visits, 1 to 365.")]
        every_days: u16,
    },
    /// One or more periods of weeks, for example "taille printemps / taille
    /// automne" or "passage avril / juillet / novembre".
    Windows {
        #[schemars(description = "1 to 12 windows, none overlapping, all inside the season.")]
        windows: Vec<Window>,
    },
    /// Never due, visits are recorded when they happen (the "ND" lines).
    OnDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct Line {
    #[schemars(
        description = "The service as written on the sheet, for example \"Tonte avec ramassage\"."
    )]
    pub label: String,
    pub cadence: Cadence,
    #[schemars(description = "Planned minutes for one visit.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes_per_intervention: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ContractDraft {
    #[schemars(
        description = "UUID of the customer. It must already exist; take it from get_maintenance_contract, list_projects or get_project."
    )]
    pub customer_id: String,
    #[schemars(
        description = "UUID of the site (customer context) the contract covers. It must belong to the customer."
    )]
    pub customer_context_id: String,
    #[schemars(description = "The \"code affaire\" of the sheet, for example C25029.")]
    pub reference: Option<String>,
    pub season: Season,
    #[schemars(description = "UUID of the member in charge of the contract (the conducteur).")]
    pub manager_member_id: Option<String>,
    #[schemars(description = "Contract amount in cents.")]
    pub amount_cents: Option<i64>,
    #[schemars(description = "Free notes, for example the contact written on the sheet.")]
    pub notes: Option<String>,
    #[schemars(description = "The lines of the sheet, one per service.")]
    pub lines: Vec<Line>,
}

impl ContractDraft {
    /// The body of `POST .../maintenance/contracts`. Identifiers are checked
    /// before anything leaves the server, and the status is not a choice.
    pub fn request_body(&self) -> Result<Value, Fault> {
        let mut body = json!({
            "customer_id": uuid(&self.customer_id)?,
            "customer_context_id": uuid(&self.customer_context_id)?,
            "season": self.season,
            "lines": self.lines,
            "status": DRAFT,
        });
        if let Some(reference) = &self.reference {
            body["reference"] = json!(reference);
        }
        if let Some(manager) = &self.manager_member_id {
            body["manager_member_id"] = json!(uuid(manager)?);
        }
        if let Some(amount) = self.amount_cents {
            body["amount_cents"] = json!(amount);
        }
        if let Some(notes) = &self.notes {
            body["notes"] = json!(notes);
        }
        Ok(body)
    }
}

fn uuid(raw: &str) -> Result<String, Fault> {
    Uuid::parse_str(raw)
        .map(|id| id.to_string())
        .map_err(|_| Fault::new(format!("identifiant invalide : « {raw} »")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> ContractDraft {
        serde_json::from_value(json!({
            "customer_id": "f2222222-2222-4222-8222-222222222222",
            "customer_context_id": "f3333333-3333-4333-8333-333333333333",
            "season": {"year": 2026, "first_week": 12, "last_week": 48},
            "lines": [{"label": "Tonte", "cadence": {"kind": "ROLLING", "every_days": 14}}],
        }))
        .unwrap()
    }

    #[test]
    fn the_request_is_always_a_draft() {
        assert_eq!(draft().request_body().unwrap()["status"], "DRAFT");
    }

    #[test]
    fn what_the_sheet_does_not_say_is_left_out() {
        let body = draft().request_body().unwrap();
        for absent in ["reference", "manager_member_id", "amount_cents", "notes"] {
            assert!(body.get(absent).is_none(), "{absent}");
        }
    }

    #[test]
    fn a_malformed_manager_id_is_refused() {
        let mut draft = draft();
        draft.manager_member_id = Some("Bernard".to_owned());

        assert!(draft.request_body().is_err());
    }
}
