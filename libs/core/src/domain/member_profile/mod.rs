use chrono::{DateTime, NaiveDate, Utc};

use crate::{MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;
pub mod values;

#[cfg(test)]
mod tests;

pub use commands::MemberProfileInput;
pub use error::{MemberProfileError, MemberProfileField, MemberProfileOutcome};
pub use service::MemberProfileService;
pub use values::{ContactEmail, DEFAULT_COUNTRY, PhoneNumber, PostalAddress, PostalCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberProfile {
    pub member_id: MemberId,
    pub organization_id: OrganizationId,
    pub work_email: Option<ContactEmail>,
    pub personal_email: Option<ContactEmail>,
    pub mobile: Option<PhoneNumber>,
    pub landline: Option<PhoneNumber>,
    pub address: PostalAddress,
    pub job_title: Option<String>,
    pub manager_member_id: Option<MemberId>,
    pub work_permit_end_date: Option<NaiveDate>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn filled(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

fn parsed<T, E>(
    value: Option<&str>,
    refusal: MemberProfileError,
    parse: impl for<'a> FnOnce(&'a str) -> Result<T, E>,
) -> MemberProfileOutcome<Option<T>> {
    filled(value)
        .map(|text| parse(text).map_err(|_| refusal))
        .transpose()
}

impl MemberProfile {
    pub fn empty(member_id: MemberId, organization_id: OrganizationId, now: DateTime<Utc>) -> Self {
        Self {
            member_id,
            organization_id,
            work_email: None,
            personal_email: None,
            mobile: None,
            landline: None,
            address: PostalAddress::default(),
            job_title: None,
            manager_member_id: None,
            work_permit_end_date: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn replaced_with(
        &self,
        input: MemberProfileInput,
        now: DateTime<Utc>,
    ) -> MemberProfileOutcome<Self> {
        let postal_code = filled(input.postal_code.as_deref())
            .map(PostalCode::try_from)
            .transpose()?;
        Ok(Self {
            member_id: self.member_id,
            organization_id: self.organization_id,
            work_email: parsed(
                input.work_email.as_deref(),
                MemberProfileError::WorkEmailInvalid,
                ContactEmail::parse,
            )?,
            personal_email: parsed(
                input.personal_email.as_deref(),
                MemberProfileError::PersonalEmailInvalid,
                ContactEmail::parse,
            )?,
            mobile: parsed(
                input.mobile.as_deref(),
                MemberProfileError::MobileInvalid,
                PhoneNumber::parse,
            )?,
            landline: parsed(
                input.landline.as_deref(),
                MemberProfileError::LandlineInvalid,
                PhoneNumber::parse,
            )?,
            address: PostalAddress {
                street: filled(input.street.as_deref())
                    .unwrap_or_default()
                    .to_owned(),
                complement: filled(input.complement.as_deref())
                    .unwrap_or_default()
                    .to_owned(),
                postal_code,
                city: filled(input.city.as_deref()).unwrap_or_default().to_owned(),
                country: filled(input.country.as_deref())
                    .unwrap_or(DEFAULT_COUNTRY)
                    .to_owned(),
            },
            job_title: filled(input.job_title.as_deref()).map(str::to_owned),
            manager_member_id: input.manager_member_id,
            work_permit_end_date: input.work_permit_end_date,
            archived_at: self.archived_at,
            created_at: self.created_at,
            updated_at: now,
        })
    }

    pub fn archived(&self, now: DateTime<Utc>) -> Self {
        Self {
            archived_at: self.archived_at.or(Some(now)),
            updated_at: now,
            ..self.clone()
        }
    }

    pub fn restored(&self, now: DateTime<Utc>) -> Self {
        Self {
            archived_at: None,
            updated_at: now,
            ..self.clone()
        }
    }
}
