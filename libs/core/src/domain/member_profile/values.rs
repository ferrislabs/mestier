use super::error::MemberProfileError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFormat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactEmail(String);

impl ContactEmail {
    pub fn parse(value: &str) -> Result<Self, InvalidFormat> {
        Self::try_from(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ContactEmail {
    type Error = InvalidFormat;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let mut parts = trimmed.split('@');
        let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(InvalidFormat);
        };
        let chars: Vec<char> = domain.chars().collect();
        let inner_dot = chars.len() >= 3 && chars[1..chars.len() - 1].contains(&'.');
        if trimmed.chars().any(char::is_whitespace) || local.is_empty() || !inner_dot {
            return Err(InvalidFormat);
        }
        Ok(Self(trimmed.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn parse(value: &str) -> Result<Self, InvalidFormat> {
        Self::try_from(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for PhoneNumber {
    type Error = InvalidFormat;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let rest = trimmed.strip_prefix('+').unwrap_or(trimmed);
        let mut chars = rest.chars();
        let starts_with_digit = chars.next().is_some_and(|c| c.is_ascii_digit());
        let tail: Vec<char> = chars.collect();
        let tail_ok = (5..=18).contains(&tail.len())
            && tail
                .iter()
                .all(|c| c.is_ascii_digit() || matches!(c, ' ' | '.' | '-'));
        if starts_with_digit && tail_ok {
            Ok(Self(trimmed.to_owned()))
        } else {
            Err(InvalidFormat)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalCode(String);

impl PostalCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for PostalCode {
    type Error = MemberProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.len() == 5 && trimmed.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(trimmed.to_owned()))
        } else {
            Err(MemberProfileError::PostalCodeInvalid)
        }
    }
}

pub const DEFAULT_COUNTRY: &str = "France";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostalAddress {
    pub street: String,
    pub complement: String,
    pub postal_code: Option<PostalCode>,
    pub city: String,
    pub country: String,
}

impl Default for PostalAddress {
    fn default() -> Self {
        Self {
            street: String::new(),
            complement: String::new(),
            postal_code: None,
            city: String::new(),
            country: DEFAULT_COUNTRY.to_owned(),
        }
    }
}
