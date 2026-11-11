use chrono::{DateTime, Utc};
use common::UserId;

pub const LOCK_SECONDS: i64 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditLock {
    pub holder: UserId,
    pub expires_at: DateTime<Utc>,
}

impl EditLock {
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        self.expires_at >= now
    }

    pub fn blocks(&self, user: UserId, now: DateTime<Utc>) -> bool {
        self.holder != user && self.is_active(now)
    }

    pub fn is_held_by(&self, user: UserId, now: DateTime<Utc>) -> bool {
        self.holder == user && self.is_active(now)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use common::generate_uuid_v7;

    use super::*;

    fn lock(holder: UserId, seconds_left: i64) -> EditLock {
        EditLock {
            holder,
            expires_at: Utc::now() + Duration::seconds(seconds_left),
        }
    }

    #[test]
    fn a_lock_lasts_ninety_seconds() {
        assert_eq!(LOCK_SECONDS, 90);
    }

    #[test]
    fn a_live_lock_blocks_everybody_but_its_holder() {
        let holder = UserId(generate_uuid_v7());
        let other = UserId(generate_uuid_v7());
        let live = lock(holder, 30);
        assert!(live.blocks(other, Utc::now()));
        assert!(!live.blocks(holder, Utc::now()));
        assert!(live.is_held_by(holder, Utc::now()));
        assert!(!live.is_held_by(other, Utc::now()));
    }

    #[test]
    fn an_expired_lock_is_free_for_everybody() {
        let holder = UserId(generate_uuid_v7());
        let other = UserId(generate_uuid_v7());
        let expired = lock(holder, -1);
        assert!(!expired.blocks(other, Utc::now()));
        assert!(!expired.is_held_by(holder, Utc::now()));
    }
}
