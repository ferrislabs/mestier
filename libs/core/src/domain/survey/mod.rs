pub mod error;
pub mod value;

#[cfg(test)]
mod tests;

pub use error::SurveyError;
pub use value::{Answer, EmailAddress, PhoneNumber, QuestionId, QuestionKind, Visibility};
