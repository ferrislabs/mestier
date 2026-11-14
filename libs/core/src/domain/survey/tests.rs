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

#[test]
fn an_invalid_email_cannot_be_deserialized() {
    assert!(serde_json::from_str::<EmailAddress>("\"x\"").is_err());
}

#[test]
fn an_invalid_phone_cannot_be_deserialized() {
    assert!(serde_json::from_str::<PhoneNumber>("\"x\"").is_err());
}

#[test]
fn an_email_survives_a_serde_round_trip() {
    let email = EmailAddress::try_from("jane@example.com").unwrap();
    let json = serde_json::to_string(&email).unwrap();
    assert_eq!(serde_json::from_str::<EmailAddress>(&json).unwrap(), email);
}

#[test]
fn a_phone_survives_a_serde_round_trip() {
    let phone = PhoneNumber::try_from("+33 6 12 34 56 78").unwrap();
    let json = serde_json::to_string(&phone).unwrap();
    assert_eq!(serde_json::from_str::<PhoneNumber>(&json).unwrap(), phone);
}

#[test]
fn a_phone_with_six_digits_is_accepted() {
    assert!(PhoneNumber::try_from("123456").is_ok());
}

#[test]
fn a_phone_with_fifteen_digits_is_accepted() {
    assert!(PhoneNumber::try_from("123456789012345").is_ok());
}

#[test]
fn a_phone_with_sixteen_digits_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("1234567890123456"),
        Err(SurveyError::PhoneInvalid)
    );
}

#[test]
fn a_plus_that_is_not_first_is_refused() {
    assert_eq!(
        PhoneNumber::try_from("12+345678"),
        Err(SurveyError::PhoneInvalid)
    );
}

proptest! {
    #[test]
    fn an_email_normalization_is_idempotent(raw in "[ \\t]{0,3}[A-Za-z0-9.]{1,8}@[A-Za-z0-9.]{1,8}[ \\t]{0,3}") {
        let once = EmailAddress::try_from(raw.as_str()).unwrap();
        let twice = EmailAddress::try_from(once.as_str());
        prop_assert_eq!(twice, Ok(once));
    }
}
