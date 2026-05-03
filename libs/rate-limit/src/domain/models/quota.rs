use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct Quota {
    pub limit: u32,
    pub window: Duration,
}

impl Quota {
    pub fn per_minute(limit: u32) -> Self {
        Self {
            limit,
            window: Duration::from_secs(60),
        }
    }

    pub fn per_hour(limit: u32) -> Self {
        Self {
            limit,
            window: Duration::from_secs(3600),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_minute_uses_60s_window() {
        let q = Quota::per_minute(100);
        assert_eq!(q.limit, 100);
        assert_eq!(q.window, Duration::from_secs(60));
    }

    #[test]
    fn per_hour_uses_3600s_window() {
        let q = Quota::per_hour(5_000);
        assert_eq!(q.limit, 5_000);
        assert_eq!(q.window, Duration::from_secs(3600));
    }
}
