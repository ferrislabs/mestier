use super::{AcquisitionDraft, LeaveAccountInput};

#[derive(Debug, Clone, Copy)]
pub struct DefaultLeaveAccount {
    pub name: &'static str,
    pub acquisition: AcquisitionDraft,
    pub period_start_month: u8,
    pub carry_over_max: f64,
}

impl From<&DefaultLeaveAccount> for LeaveAccountInput<'static> {
    fn from(value: &DefaultLeaveAccount) -> Self {
        Self {
            name: value.name,
            acquisition: value.acquisition,
            period_start_month: value.period_start_month,
            carry_over_max: value.carry_over_max,
        }
    }
}

const DEFAULT_LEAVE_ACCOUNTS: [DefaultLeaveAccount; 2] = [
    DefaultLeaveAccount {
        name: "Congés payés",
        acquisition: AcquisitionDraft::Monthly { per_month: 2.08 },
        period_start_month: 6,
        carry_over_max: 5.0,
    },
    DefaultLeaveAccount {
        name: "RTT",
        acquisition: AcquisitionDraft::Annual { days: 10.0 },
        period_start_month: 1,
        carry_over_max: 0.0,
    },
];

pub fn default_leave_accounts() -> &'static [DefaultLeaveAccount] {
    &DEFAULT_LEAVE_ACCOUNTS
}
