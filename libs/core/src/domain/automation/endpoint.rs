use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use uuid::Uuid;

/// A webhook target an organization registered.
///
/// The secret is absent on purpose: it is sealed in the database and returned
/// exactly once, at creation. Carrying it on the aggregate would make leaking
/// it a matter of forgetting to strip a field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub url: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CreateWebhookEndpointCommand {
    pub org_id: OrganizationId,
    pub url: String,
    pub description: Option<String>,
    pub event_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateWebhookEndpointCommand {
    pub id: Uuid,
    pub url: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub event_names: Vec<String>,
}

/// Only `http` and `https` are targets. A `file://` or `gopher://` URL is not
/// a webhook, and letting one through would hand the client library a scheme
/// nobody audited.
pub fn validate_url(url: &str) -> Result<(), CoreError> {
    let trimmed = url.trim();

    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(CoreError::Conflict(
            "a webhook URL must start with http:// or https://".to_owned(),
        ));
    }

    if trimmed.len() > 2048 {
        return Err(CoreError::Conflict(
            "a webhook URL cannot exceed 2048 characters".to_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_and_https_are_accepted() {
        assert!(validate_url("https://example.com/hook").is_ok());
        assert!(validate_url("http://example.com/hook").is_ok());
    }

    /// The SSRF guard judges addresses; it never sees a scheme it cannot
    /// dial. Refusing here is what keeps that true.
    #[test]
    fn another_scheme_is_refused() {
        for url in [
            "file:///etc/passwd",
            "gopher://example.com",
            "ftp://example.com",
            "example.com/hook",
            "",
        ] {
            assert!(validate_url(url).is_err(), "{url} must be refused");
        }
    }

    #[test]
    fn an_absurdly_long_url_is_refused() {
        let url = format!("https://example.com/{}", "a".repeat(3000));

        assert!(validate_url(&url).is_err());
    }
}
