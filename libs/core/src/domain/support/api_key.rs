use std::{fmt, fmt::Display, str::FromStr};

use aws_lc_rs::{
    constant_time, digest,
    rand::{SecureRandom, SystemRandom},
};
use chrono::{DateTime, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub const MAX_API_KEY_NAME_CHARS: usize = 100;

const PREFIX_BYTES: usize = 4;
const SECRET_BYTES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SupportApiKeyId(pub Uuid);

impl FromStr for SupportApiKeyId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SupportApiKeyId)
    }
}

impl Display for SupportApiKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct SupportApiKey {
    pub id: SupportApiKeyId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub prefix: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl fmt::Debug for SupportApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SupportApiKey")
            .field("id", &self.id)
            .field("organization_id", &self.organization_id)
            .field("name", &self.name)
            .field("prefix", &self.prefix)
            .field("secret_hash", &format_args!("<redacted>"))
            .field("created_at", &self.created_at)
            .field("last_used_at", &self.last_used_at)
            .field("revoked_at", &self.revoked_at)
            .finish()
    }
}

pub struct GeneratedApiKey {
    pub plaintext: String,
    pub prefix: String,
    pub secret_hash: Vec<u8>,
}

impl fmt::Debug for GeneratedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedApiKey")
            .field("plaintext", &format_args!("<redacted>"))
            .field("prefix", &self.prefix)
            .field("secret_hash", &format_args!("<redacted>"))
            .finish()
    }
}

pub struct PresentedApiKey {
    pub prefix: String,
    secret: String,
}

impl fmt::Debug for PresentedApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresentedApiKey")
            .field("prefix", &self.prefix)
            .field("secret", &format_args!("<redacted>"))
            .finish()
    }
}

pub fn generate_api_key() -> Result<GeneratedApiKey, CoreError> {
    let random = SystemRandom::new();
    let mut prefix_bytes = [0u8; PREFIX_BYTES];
    let mut secret_bytes = [0u8; SECRET_BYTES];
    random.fill(&mut prefix_bytes).map_err(random_failure)?;
    random.fill(&mut secret_bytes).map_err(random_failure)?;

    let prefix = hex(&prefix_bytes);
    let secret = hex(&secret_bytes);
    let secret_hash = hash_secret(&secret);

    Ok(GeneratedApiKey {
        plaintext: format!("{prefix}.{secret}"),
        prefix,
        secret_hash,
    })
}

impl PresentedApiKey {
    pub fn parse(plaintext: &str) -> Option<Self> {
        let (prefix, secret) = plaintext.split_once('.')?;
        if !is_lowercase_hex(prefix, PREFIX_BYTES * 2)
            || !is_lowercase_hex(secret, SECRET_BYTES * 2)
        {
            return None;
        }

        Some(Self {
            prefix: prefix.to_owned(),
            secret: secret.to_owned(),
        })
    }

    pub fn secret_hash(&self) -> Vec<u8> {
        hash_secret(&self.secret)
    }
}

pub fn api_key_matches(stored_hash: &[u8], presented: &PresentedApiKey) -> bool {
    constant_time::verify_slices_are_equal(stored_hash, &presented.secret_hash()).is_ok()
}

pub fn validate_api_key_name(name: &str) -> Result<String, CoreError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CoreError::Conflict(
            "api key name must not be empty".to_owned(),
        ));
    }
    if name.chars().count() > MAX_API_KEY_NAME_CHARS {
        return Err(CoreError::Conflict(format!(
            "api key name must be at most {MAX_API_KEY_NAME_CHARS} characters"
        )));
    }

    Ok(name.to_owned())
}

fn random_failure(_: aws_lc_rs::error::Unspecified) -> CoreError {
    CoreError::Internal("the system random source failed".to_owned())
}

fn hash_secret(secret: &str) -> Vec<u8> {
    digest::digest(&digest::SHA256, secret.as_bytes())
        .as_ref()
        .to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn is_lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use aws_lc_rs::digest;
    use chrono::TimeZone;

    use super::*;

    const ZERO_SECRET: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const ZERO_SECRET_SHA256: [u8; 32] = [
        0x60, 0xe0, 0x5b, 0xd1, 0xb1, 0x95, 0xaf, 0x2f, 0x94, 0x11, 0x2f, 0xa7, 0x19, 0x7a, 0x5c,
        0x88, 0x28, 0x90, 0x58, 0x84, 0x0c, 0xe7, 0xc6, 0xdf, 0x96, 0x93, 0x75, 0x6b, 0xc6, 0x25,
        0x0f, 0x55,
    ];

    fn generated() -> GeneratedApiKey {
        generate_api_key().unwrap()
    }

    fn secret_of(key: &GeneratedApiKey) -> &str {
        key.plaintext.split_once('.').unwrap().1
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn is_lower_hex(value: &str) -> bool {
        value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    }

    fn stored(key: &GeneratedApiKey) -> SupportApiKey {
        SupportApiKey {
            id: SupportApiKeyId(Uuid::from_u128(1)),
            organization_id: OrganizationId(Uuid::from_u128(2)),
            name: "Intake".to_owned(),
            prefix: key.prefix.clone(),
            secret_hash: key.secret_hash.clone(),
            created_at: Utc.with_ymd_and_hms(2026, 11, 4, 9, 0, 0).unwrap(),
            last_used_at: None,
            revoked_at: None,
        }
    }

    #[test]
    fn a_generated_key_is_an_eight_hex_prefix_and_a_sixty_four_hex_secret_joined_by_a_dot() {
        let key = generated();

        assert_eq!(key.prefix.len(), 8);
        assert!(is_lower_hex(&key.prefix));
        assert_eq!(secret_of(&key).len(), 64);
        assert!(is_lower_hex(secret_of(&key)));
        assert_eq!(key.plaintext, format!("{}.{}", key.prefix, secret_of(&key)));
        assert_eq!(key.plaintext.matches('.').count(), 1);
    }

    #[test]
    fn generated_keys_never_repeat() {
        let keys: Vec<GeneratedApiKey> = (0..64).map(|_| generated()).collect();

        for (index, key) in keys.iter().enumerate() {
            for other in &keys[index + 1..] {
                assert_ne!(key.plaintext, other.plaintext);
                assert_ne!(key.secret_hash, other.secret_hash);
            }
        }
    }

    #[test]
    fn the_stored_hash_is_the_sha256_of_the_secret() {
        let key = generated();

        let expected = digest::digest(&digest::SHA256, secret_of(&key).as_bytes());

        assert_eq!(key.secret_hash.len(), 32);
        assert_eq!(key.secret_hash.as_slice(), expected.as_ref());
    }

    #[test]
    fn the_hash_of_a_known_secret_matches_a_published_vector() {
        let presented = PresentedApiKey::parse(&format!("abcdef01.{ZERO_SECRET}")).unwrap();

        assert_eq!(presented.secret_hash(), ZERO_SECRET_SHA256);
    }

    #[test]
    fn a_generated_key_parses_back_to_its_prefix_and_hash() {
        let key = generated();

        let presented = PresentedApiKey::parse(&key.plaintext).unwrap();

        assert_eq!(presented.prefix, key.prefix);
        assert_eq!(presented.secret_hash(), key.secret_hash);
    }

    #[test]
    fn malformed_keys_do_not_parse() {
        let secret = "ab".repeat(32);
        let cases = [
            String::new(),
            ".".to_owned(),
            format!("abcdef01{secret}"),
            format!("abcdef01.{secret}.{secret}"),
            format!("abcdef01..{secret}"),
            format!("abcdef0.{secret}"),
            format!("abcdef012.{secret}"),
            format!("ABCDEF01.{secret}"),
            format!("abcdefgh.{secret}"),
            format!("abcdef0é.{secret}"),
            format!("abcdeé0.{secret}"),
            format!(".{secret}"),
            format!("abcdef01.{}", &secret[..63]),
            format!("abcdef01.{secret}0"),
            format!("abcdef01.{}", secret.to_uppercase()),
            format!("abcdef01.{}g", &secret[..63]),
            "abcdef01.".to_owned(),
            format!("abcdef01.{secret} "),
            format!("abcdef01.{secret}\n"),
            format!(" abcdef01.{secret}"),
            format!("abcdef01 .{secret}"),
            format!("abcdef01.{}é", &secret[..62]),
        ];

        for case in cases {
            assert!(PresentedApiKey::parse(&case).is_none(), "{case:?}");
        }
    }

    #[test]
    fn a_generated_key_matches_its_own_hash() {
        let key = generated();
        let presented = PresentedApiKey::parse(&key.plaintext).unwrap();

        assert!(api_key_matches(&key.secret_hash, &presented));
    }

    #[test]
    fn changing_any_single_character_of_the_secret_breaks_the_match() {
        let key = generated();

        for position in 0..64 {
            let mut secret: Vec<u8> = secret_of(&key).bytes().collect();
            let digit = (secret[position] as char).to_digit(16).unwrap();
            secret[position] = char::from_digit((digit + 1) % 16, 16).unwrap() as u8;
            let tampered = format!("{}.{}", key.prefix, String::from_utf8(secret).unwrap());

            let presented = PresentedApiKey::parse(&tampered).unwrap();

            assert!(
                !api_key_matches(&key.secret_hash, &presented),
                "position {position}"
            );
        }
    }

    #[test]
    fn a_hash_of_the_wrong_length_never_matches() {
        let key = generated();
        let presented = PresentedApiKey::parse(&key.plaintext).unwrap();

        let mut longer = key.secret_hash.clone();
        longer.push(0);

        assert!(!api_key_matches(&[], &presented));
        assert!(!api_key_matches(&key.secret_hash[..31], &presented));
        assert!(!api_key_matches(&longer, &presented));
        assert!(!api_key_matches(&[0u8; 32], &presented));
    }

    fn assert_never_prints(rendered: &str, key: &GeneratedApiKey) {
        assert!(!rendered.contains(&key.plaintext), "{rendered}");
        assert!(!rendered.contains(secret_of(key)), "{rendered}");
        assert!(!rendered.contains(&hex(&key.secret_hash)), "{rendered}");
        assert!(
            !rendered.contains(&format!("{:?}", key.secret_hash)),
            "{rendered}"
        );
        assert!(rendered.contains("<redacted>"), "{rendered}");
    }

    #[test]
    fn the_stored_key_never_prints_its_hash() {
        let key = generated();

        assert_never_prints(&format!("{:?}", stored(&key)), &key);
        assert_never_prints(&format!("{:#?}", stored(&key)), &key);
    }

    #[test]
    fn a_generated_key_never_prints_its_plaintext_or_hash() {
        let key = generated();

        assert_never_prints(&format!("{key:?}"), &key);
        assert_never_prints(&format!("{key:#?}"), &key);
    }

    #[test]
    fn a_presented_key_never_prints_its_secret() {
        let key = generated();
        let presented = PresentedApiKey::parse(&key.plaintext).unwrap();

        assert_never_prints(&format!("{presented:?}"), &key);
    }

    #[test]
    fn the_stored_key_debug_still_shows_what_is_public() {
        let key = generated();
        let rendered = format!("{:?}", stored(&key));

        assert!(rendered.contains(&key.prefix));
        assert!(rendered.contains("Intake"));
    }

    #[test]
    fn a_name_is_trimmed() {
        assert_eq!(
            validate_api_key_name("  Intake form \n").unwrap(),
            "Intake form"
        );
    }

    #[test]
    fn an_empty_or_blank_name_is_refused() {
        assert!(matches!(
            validate_api_key_name(""),
            Err(CoreError::Conflict(_))
        ));
        assert!(matches!(
            validate_api_key_name(" \t\n "),
            Err(CoreError::Conflict(_))
        ));
    }

    #[test]
    fn a_name_of_one_hundred_characters_is_accepted_and_one_more_is_refused() {
        let accepted = "x".repeat(MAX_API_KEY_NAME_CHARS);

        assert_eq!(validate_api_key_name(&accepted).unwrap(), accepted);
        assert!(matches!(
            validate_api_key_name(&"x".repeat(MAX_API_KEY_NAME_CHARS + 1)),
            Err(CoreError::Conflict(_))
        ));
    }

    #[test]
    fn the_length_is_counted_after_trimming() {
        let padded = format!("  {}  ", "x".repeat(MAX_API_KEY_NAME_CHARS));

        assert_eq!(
            validate_api_key_name(&padded).unwrap().chars().count(),
            MAX_API_KEY_NAME_CHARS
        );
    }

    #[test]
    fn multi_byte_characters_count_once() {
        let accepted = "é".repeat(MAX_API_KEY_NAME_CHARS);
        let refused = "é".repeat(MAX_API_KEY_NAME_CHARS + 1);

        assert_eq!(validate_api_key_name(&accepted).unwrap(), accepted);
        assert!(validate_api_key_name(&refused).is_err());
    }
}
