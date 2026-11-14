use proptest::prelude::*;

use super::{EmailAddress, PhoneNumber, SurveyError};

#[test]
fn an_email_is_trimmed_and_lowercased() {
    let email = EmailAddress::try_from("  Jane.Doe@Example.COM ").unwrap();
    assert_eq!(email.as_str(), "jane.doe@example.com");
}

#[test]
fn an_email_without_an_at_sign_is_refused() {
    assert_eq!(
        EmailAddress::try_from("jane.example.com"),
        Err(SurveyError::EmailInvalid)
    );
}

#[test]
fn a_phone_keeps_only_a_plus_and_digits() {
    let phone = PhoneNumber::try_from(" +33 (0)6-12 34 56 78 ").unwrap();
    assert_eq!(phone.as_str(), "+330612345678");
}

#[test]
fn a_phone_with_five_digits_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("12 345"),
        Err(SurveyError::PhoneInvalid)
    );
}

#[test]
fn a_phone_with_letters_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("06 12 34 56 7a"),
        Err(SurveyError::PhoneInvalid)
    );
}

proptest! {
    #[test]
    fn an_email_normalization_is_idempotent(raw in "\\PC{0,40}") {
        if let Ok(once) = EmailAddress::try_from(raw.as_str()) {
            let twice = EmailAddress::try_from(once.as_str());
            prop_assert_eq!(twice, Ok(once));
        }
    }
}
