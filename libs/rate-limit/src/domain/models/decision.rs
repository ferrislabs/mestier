use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub retry_after: Option<Duration>,
}
