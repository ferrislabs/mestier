use std::fmt;

#[derive(Debug, Clone)]
pub struct RateLimitKey {
    pub scope: String,
    pub identifier: String,
}

impl RateLimitKey {
    pub fn organization(org_id: impl Into<String>) -> Self {
        Self {
            scope: "org".to_string(),
            identifier: org_id.into(),
        }
    }

    pub fn ip(addr: impl Into<String>) -> Self {
        Self {
            scope: "ip".to_string(),
            identifier: addr.into(),
        }
    }
}

impl fmt::Display for RateLimitKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ratelimit:{}:{}", self.scope, self.identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn organization_key_formats_with_org_scope() {
        let key = RateLimitKey::organization("org-123");
        assert_eq!(key.scope, "org");
        assert_eq!(key.to_string(), "ratelimit:org:org-123");
    }

    #[test]
    fn ip_key_formats_with_ip_scope() {
        let key = RateLimitKey::ip("10.0.0.1");
        assert_eq!(key.scope, "ip");
        assert_eq!(key.to_string(), "ratelimit:ip:10.0.0.1");
    }
}
