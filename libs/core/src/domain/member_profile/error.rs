use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberProfileField {
    WorkEmail,
    PersonalEmail,
    Mobile,
    Landline,
    PostalCode,
    ManagerMemberId,
}

impl MemberProfileField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WorkEmail => "work_email",
            Self::PersonalEmail => "personal_email",
            Self::Mobile => "mobile",
            Self::Landline => "landline",
            Self::PostalCode => "postal_code",
            Self::ManagerMemberId => "manager_member_id",
        }
    }
}

impl fmt::Display for MemberProfileField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MemberProfileError {
    #[error("work_email: not a valid e-mail address")]
    WorkEmailInvalid,
    #[error("personal_email: not a valid e-mail address")]
    PersonalEmailInvalid,
    #[error("mobile: not a valid phone number")]
    MobileInvalid,
    #[error("landline: not a valid phone number")]
    LandlineInvalid,
    #[error("postal_code: must be 5 digits")]
    PostalCodeInvalid,
    #[error("manager_member_id: must be a member of the organization")]
    ManagerNotInOrganization,
    #[error("manager_member_id: a member cannot be their own manager")]
    ManagerIsSelf,
    #[error("manager_member_id: would make the member their own superior")]
    ManagerCycle,
}

impl MemberProfileError {
    pub fn field(self) -> MemberProfileField {
        match self {
            Self::WorkEmailInvalid => MemberProfileField::WorkEmail,
            Self::PersonalEmailInvalid => MemberProfileField::PersonalEmail,
            Self::MobileInvalid => MemberProfileField::Mobile,
            Self::LandlineInvalid => MemberProfileField::Landline,
            Self::PostalCodeInvalid => MemberProfileField::PostalCode,
            Self::ManagerNotInOrganization | Self::ManagerIsSelf | Self::ManagerCycle => {
                MemberProfileField::ManagerMemberId
            }
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::ManagerCycle => "E_MEMBER_PROFILE_MANAGER_CYCLE",
            _ => "E_UNPROCESSABLE_ENTITY",
        }
    }
}

pub type MemberProfileOutcome<T> = Result<T, MemberProfileError>;
