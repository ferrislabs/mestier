use common::RateLimitConfig;

#[derive(clap::Args, Debug, Clone)]
pub struct RateLimitArgs {
    #[arg(
        long = "rate-limit-redis-url",
        env = "RATE_LIMIT_REDIS_URL",
        name = "RATE_LIMIT_REDIS_URL",
        default_value = "redis://localhost:6379",
        long_help = "Redis URL backing the rate limiter"
    )]
    pub redis_url: String,

    #[arg(
        long = "rate-limit-per-minute",
        env = "RATE_LIMIT_PER_MINUTE",
        name = "RATE_LIMIT_PER_MINUTE",
        default_value_t = 120,
        long_help = "Allowed requests per minute per client IP"
    )]
    pub per_minute: u32,
}

impl From<RateLimitArgs> for RateLimitConfig {
    fn from(value: RateLimitArgs) -> Self {
        Self {
            redis_url: value.redis_url,
            per_minute: value.per_minute,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cmd {
        #[command(flatten)]
        rate_limit: RateLimitArgs,
    }

    #[test]
    fn parse_defaults() {
        let cmd = Cmd::try_parse_from(["cmd"]).unwrap();
        assert_eq!(cmd.rate_limit.redis_url, "redis://localhost:6379");
        assert_eq!(cmd.rate_limit.per_minute, 120);
    }

    #[test]
    fn parse_custom() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--rate-limit-redis-url",
            "redis://redis.prod:6379",
            "--rate-limit-per-minute",
            "1000",
        ])
        .unwrap();
        assert_eq!(cmd.rate_limit.redis_url, "redis://redis.prod:6379");
        assert_eq!(cmd.rate_limit.per_minute, 1000);
    }
}
