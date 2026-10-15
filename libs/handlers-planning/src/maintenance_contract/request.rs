use handlers::ApiError;
use mestier_core::{
    Cadence, ContractStatus, IntervalDays, IsoWeekNumber, MaintenanceLineId, MaintenanceLineInput,
    MemberId, QuoteId, Season, Window, Windows,
};
use serde::Deserialize;
use utoipa::ToSchema;

/// Read as plain numbers and validated here, so an invalid value is a 409
/// like everywhere else and not a deserialization failure.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SeasonRequest {
    pub year: i32,
    /// ISO week, 1 to 53.
    pub first_week: u8,
    /// ISO week, 1 to 53, not before `first_week`, and existing in `year`.
    pub last_week: u8,
}

impl TryFrom<SeasonRequest> for Season {
    type Error = ApiError;

    fn try_from(request: SeasonRequest) -> Result<Self, Self::Error> {
        Ok(Season::new(
            request.year,
            IsoWeekNumber::try_from(request.first_week)?,
            IsoWeekNumber::try_from(request.last_week)?,
        )?)
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct WindowRequest {
    pub from_week: u8,
    pub to_week: u8,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CadenceRequest {
    /// Due `every_days` (1 to 365) after the last validated intervention.
    Rolling { every_days: u16 },
    /// 1 to 12 windows, none overlapping, all inside the season.
    Windows { windows: Vec<WindowRequest> },
    /// Never due.
    OnDemand,
}

impl TryFrom<CadenceRequest> for Cadence {
    type Error = ApiError;

    fn try_from(request: CadenceRequest) -> Result<Self, Self::Error> {
        match request {
            CadenceRequest::Rolling { every_days } => Ok(Cadence::Rolling {
                every_days: IntervalDays::try_from(every_days)?,
            }),
            CadenceRequest::Windows { windows } => {
                let windows = windows
                    .into_iter()
                    .map(|window| {
                        Ok(Window::new(
                            IsoWeekNumber::try_from(window.from_week)?,
                            IsoWeekNumber::try_from(window.to_week)?,
                        )?)
                    })
                    .collect::<Result<Vec<_>, ApiError>>()?;
                Ok(Cadence::Windows {
                    windows: Windows::try_from(windows)?,
                })
            }
            CadenceRequest::OnDemand => Ok(Cadence::OnDemand),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MaintenanceLineRequest {
    /// Absent for a new line. An id keeps the line, and its interventions.
    #[serde(default)]
    pub id: Option<MaintenanceLineId>,
    pub label: String,
    pub cadence: CadenceRequest,
    #[serde(default)]
    pub minutes_per_intervention: Option<u32>,
}

impl TryFrom<MaintenanceLineRequest> for MaintenanceLineInput {
    type Error = ApiError;

    fn try_from(request: MaintenanceLineRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            id: request.id,
            label: request.label,
            cadence: request.cadence.try_into()?,
            minutes_per_intervention: request.minutes_per_intervention,
        })
    }
}

/// The part of a contract the form edits. A `PUT` replaces all of it, lines
/// included.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ContractBody {
    /// The "code affaire". Blank is stored as absent.
    #[serde(default)]
    pub reference: Option<String>,
    pub season: SeasonRequest,
    #[serde(default)]
    pub manager_member_id: Option<MemberId>,
    #[serde(default)]
    pub amount_cents: Option<i64>,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
    #[serde(default)]
    pub notes: Option<String>,
    /// `ACTIVE` when absent.
    #[serde(default)]
    pub status: Option<ContractStatus>,
    pub lines: Vec<MaintenanceLineRequest>,
}

pub struct ContractFields {
    pub reference: Option<String>,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<QuoteId>,
    pub notes: Option<String>,
    pub status: ContractStatus,
    pub lines: Vec<MaintenanceLineInput>,
}

impl TryFrom<ContractBody> for ContractFields {
    type Error = ApiError;

    fn try_from(body: ContractBody) -> Result<Self, Self::Error> {
        Ok(Self {
            reference: body.reference,
            season: body.season.try_into()?,
            manager_member_id: body.manager_member_id,
            amount_cents: body.amount_cents,
            quote_id: body.quote_id,
            notes: body.notes,
            status: body.status.unwrap_or(ContractStatus::Active),
            lines: body
                .lines
                .into_iter()
                .map(MaintenanceLineInput::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
