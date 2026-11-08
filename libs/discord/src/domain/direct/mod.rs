pub mod ports;
pub mod service;

use common::{CoreError, UserId};

use crate::Channel;

pub use ports::DirectChannelRepository;
pub use service::DirectChannelService;

/// Two distinct people, stored in a fixed order so that the same pair always
/// reads the same way whoever asks. `user_low < user_high` is also a CHECK in
/// `chat.direct_channels`: the schema and the type state one rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectPair {
    low: UserId,
    high: UserId,
}

impl DirectPair {
    pub fn new(a: UserId, b: UserId) -> Result<Self, CoreError> {
        match a.0.cmp(&b.0) {
            std::cmp::Ordering::Less => Ok(Self { low: a, high: b }),
            std::cmp::Ordering::Greater => Ok(Self { low: b, high: a }),
            std::cmp::Ordering::Equal => Err(CoreError::Conflict(
                "a direct message needs two different people".to_owned(),
            )),
        }
    }

    pub fn low(&self) -> UserId {
        self.low
    }

    pub fn high(&self) -> UserId {
        self.high
    }

    pub fn includes(&self, user: UserId) -> bool {
        self.low == user || self.high == user
    }

    /// The other person, or `None` when `user` is not in the pair.
    pub fn peer_of(&self, user: UserId) -> Option<UserId> {
        if user == self.low {
            Some(self.high)
        } else if user == self.high {
            Some(self.low)
        } else {
            None
        }
    }
}

/// A `DIRECT` channel together with the two people it belongs to.
#[derive(Debug, Clone)]
pub struct DirectChannel {
    pub channel: Channel,
    pub pair: DirectPair,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn user(n: u128) -> UserId {
        UserId(Uuid::from_u128(n))
    }

    #[test]
    fn pair_is_ordered_whatever_the_argument_order() {
        let forward = DirectPair::new(user(1), user(2)).unwrap();
        let backward = DirectPair::new(user(2), user(1)).unwrap();

        assert_eq!(forward, backward);
        assert_eq!(forward.low(), user(1));
        assert_eq!(forward.high(), user(2));
    }

    #[test]
    fn pair_refuses_the_same_person_twice() {
        assert!(matches!(
            DirectPair::new(user(1), user(1)),
            Err(CoreError::Conflict(_))
        ));
    }

    #[test]
    fn peer_of_returns_the_other_person_only_for_a_member() {
        let pair = DirectPair::new(user(1), user(2)).unwrap();

        assert_eq!(pair.peer_of(user(1)), Some(user(2)));
        assert_eq!(pair.peer_of(user(2)), Some(user(1)));
        assert_eq!(pair.peer_of(user(3)), None);
        assert!(pair.includes(user(2)));
        assert!(!pair.includes(user(3)));
    }
}
