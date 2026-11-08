use std::fmt;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const DAYS_PER_WEEK: usize = 7;
pub const MAX_BLOCK_WEEKS: u32 = 52;
pub const MAX_BLOCK_MONTHS: u32 = 12;
pub const MAX_BLOCKS: usize = 12;
const LAST_MONTH_ANCHOR_DAY: u32 = 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum DayRefusal {
    #[error("times are written HH:MM")]
    TimeFormat,
    #[error("the end of the day must follow its start")]
    EndBeforeStart,
    #[error("the end of the break must follow its start")]
    LunchEndBeforeStart,
    #[error("the break must sit between the start and the end of the day")]
    LunchOutsideDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockTime(u16);

impl ClockTime {
    pub fn minutes(self) -> u16 {
        self.0
    }
}

impl TryFrom<&str> for ClockTime {
    type Error = DayRefusal;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();
        let well_formed = bytes.len() == 5
            && bytes[2] == b':'
            && [0, 1, 3, 4].iter().all(|&at| bytes[at].is_ascii_digit());
        if !well_formed {
            return Err(DayRefusal::TimeFormat);
        }
        let digit = |at: usize| u16::from(bytes[at] - b'0');
        let hours = digit(0) * 10 + digit(1);
        let minutes = digit(3) * 10 + digit(4);
        if hours < 24 && minutes < 60 {
            Ok(Self(hours * 60 + minutes))
        } else {
            Err(DayRefusal::TimeFormat)
        }
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LunchBreak {
    start: ClockTime,
    end: ClockTime,
}

impl LunchBreak {
    pub fn start(&self) -> ClockTime {
        self.start
    }

    pub fn end(&self) -> ClockTime {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkDay {
    start: ClockTime,
    end: ClockTime,
    lunch: Option<LunchBreak>,
}

impl WorkDay {
    pub fn start(&self) -> ClockTime {
        self.start
    }

    pub fn end(&self) -> ClockTime {
        self.end
    }

    pub fn lunch(&self) -> Option<LunchBreak> {
        self.lunch
    }

    pub fn worked_minutes(&self) -> u16 {
        let pause = self
            .lunch
            .map_or(0, |lunch| lunch.end.minutes() - lunch.start.minutes());
        self.end.minutes() - self.start.minutes() - pause
    }
}

impl TryFrom<&WorkDayInput> for WorkDay {
    type Error = DayRefusal;

    fn try_from(input: &WorkDayInput) -> Result<Self, Self::Error> {
        let start = ClockTime::try_from(input.start.as_str())?;
        let end = ClockTime::try_from(input.end.as_str())?;
        if end <= start {
            return Err(DayRefusal::EndBeforeStart);
        }
        let lunch = input
            .lunch
            .as_ref()
            .map(|lunch| {
                let from = ClockTime::try_from(lunch.start.as_str())?;
                let to = ClockTime::try_from(lunch.end.as_str())?;
                if to <= from {
                    return Err(DayRefusal::LunchEndBeforeStart);
                }
                if from <= start || to >= end {
                    return Err(DayRefusal::LunchOutsideDay);
                }
                Ok(LunchBreak {
                    start: from,
                    end: to,
                })
            })
            .transpose()?;
        Ok(Self { start, end, lunch })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct LunchInput {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct WorkDayInput {
    pub start: String,
    pub end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lunch: Option<LunchInput>,
}

impl From<&WorkDay> for WorkDayInput {
    fn from(day: &WorkDay) -> Self {
        Self {
            start: day.start.to_string(),
            end: day.end.to_string(),
            lunch: day.lunch.map(|lunch| LunchInput {
                start: lunch.start.to_string(),
                end: lunch.end.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleUnit {
    Weeks,
    Months,
}

impl ScheduleUnit {
    fn max_length(self) -> u32 {
        match self {
            Self::Weeks => MAX_BLOCK_WEEKS,
            Self::Months => MAX_BLOCK_MONTHS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BlockInput {
    pub length: u32,
    pub unit: ScheduleUnit,
    #[serde(default)]
    pub label: String,
    pub week: Vec<Option<WorkDayInput>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScheduleInput {
    Weekly {
        week: Vec<Option<WorkDayInput>>,
    },
    Rotation {
        #[schema(value_type = String, format = Date)]
        anchor: NaiveDate,
        blocks: Vec<BlockInput>,
    },
    Variable {
        default_day: WorkDayInput,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ScheduleRefusal {
    #[error("a week lists exactly {DAYS_PER_WEEK} days, Monday to Sunday")]
    WeekLength,
    #[error("{0}")]
    Day(DayRefusal),
    #[error("a rotation holds between 1 and {MAX_BLOCKS} blocks")]
    BlockCount,
    #[error("a block lasts between 1 and {max} {unit}")]
    BlockLength { max: u32, unit: &'static str },
    #[error("a rotation counted in months must start before the 29th")]
    MonthAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {reason}")]
pub struct ScheduleError {
    field: String,
    reason: ScheduleRefusal,
}

impl ScheduleError {
    fn new(field: impl Into<String>, reason: ScheduleRefusal) -> Self {
        Self {
            field: field.into(),
            reason,
        }
    }

    pub fn field(&self) -> &str {
        &self.field
    }

    pub fn reason(&self) -> &ScheduleRefusal {
        &self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeekSchedule([Option<WorkDay>; DAYS_PER_WEEK]);

impl WeekSchedule {
    pub fn days(&self) -> &[Option<WorkDay>; DAYS_PER_WEEK] {
        &self.0
    }

    fn parse(field: &str, days: &[Option<WorkDayInput>]) -> Result<Self, ScheduleError> {
        if days.len() != DAYS_PER_WEEK {
            return Err(ScheduleError::new(field, ScheduleRefusal::WeekLength));
        }
        let mut week = [None; DAYS_PER_WEEK];
        for (index, (slot, input)) in week.iter_mut().zip(days).enumerate() {
            *slot = input
                .as_ref()
                .map(WorkDay::try_from)
                .transpose()
                .map_err(|refusal| {
                    ScheduleError::new(format!("{field}[{index}]"), ScheduleRefusal::Day(refusal))
                })?;
        }
        Ok(Self(week))
    }

    fn to_input(self) -> Vec<Option<WorkDayInput>> {
        self.0
            .iter()
            .map(|day| day.as_ref().map(WorkDayInput::from))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleBlock {
    length: u32,
    unit: ScheduleUnit,
    label: String,
    week: WeekSchedule,
}

impl ScheduleBlock {
    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn unit(&self) -> ScheduleUnit {
        self.unit
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn week(&self) -> &WeekSchedule {
        &self.week
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractSchedule {
    Weekly(WeekSchedule),
    Rotation {
        anchor: NaiveDate,
        blocks: Vec<ScheduleBlock>,
    },
    Variable {
        default_day: WorkDay,
    },
}

impl TryFrom<ScheduleInput> for ContractSchedule {
    type Error = ScheduleError;

    fn try_from(input: ScheduleInput) -> Result<Self, Self::Error> {
        match input {
            ScheduleInput::Weekly { week } => {
                WeekSchedule::parse("schedule.week", &week).map(Self::Weekly)
            }
            ScheduleInput::Variable { default_day } => WorkDay::try_from(&default_day)
                .map(|default_day| Self::Variable { default_day })
                .map_err(|refusal| {
                    ScheduleError::new("schedule.default_day", ScheduleRefusal::Day(refusal))
                }),
            ScheduleInput::Rotation { anchor, blocks } => {
                if blocks.is_empty() || blocks.len() > MAX_BLOCKS {
                    return Err(ScheduleError::new(
                        "schedule.blocks",
                        ScheduleRefusal::BlockCount,
                    ));
                }
                let blocks = blocks
                    .into_iter()
                    .enumerate()
                    .map(|(index, block)| {
                        let max = block.unit.max_length();
                        if block.length < 1 || block.length > max {
                            let unit = match block.unit {
                                ScheduleUnit::Weeks => "weeks",
                                ScheduleUnit::Months => "months",
                            };
                            return Err(ScheduleError::new(
                                format!("schedule.blocks[{index}].length"),
                                ScheduleRefusal::BlockLength { max, unit },
                            ));
                        }
                        Ok(ScheduleBlock {
                            length: block.length,
                            unit: block.unit,
                            label: block.label.trim().to_owned(),
                            week: WeekSchedule::parse(
                                &format!("schedule.blocks[{index}].week"),
                                &block.week,
                            )?,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let counts_months = blocks
                    .iter()
                    .any(|block| block.unit == ScheduleUnit::Months);
                if counts_months && anchor.day() > LAST_MONTH_ANCHOR_DAY {
                    return Err(ScheduleError::new(
                        "schedule.anchor",
                        ScheduleRefusal::MonthAnchor,
                    ));
                }
                Ok(Self::Rotation { anchor, blocks })
            }
        }
    }
}

impl From<&ContractSchedule> for ScheduleInput {
    fn from(schedule: &ContractSchedule) -> Self {
        match schedule {
            ContractSchedule::Weekly(week) => Self::Weekly {
                week: week.to_input(),
            },
            ContractSchedule::Variable { default_day } => Self::Variable {
                default_day: WorkDayInput::from(default_day),
            },
            ContractSchedule::Rotation { anchor, blocks } => Self::Rotation {
                anchor: *anchor,
                blocks: blocks
                    .iter()
                    .map(|block| BlockInput {
                        length: block.length,
                        unit: block.unit,
                        label: block.label.clone(),
                        week: block.week.to_input(),
                    })
                    .collect(),
            },
        }
    }
}
