use common::OrganizationId;

use crate::ChannelId;
use crate::domain::OverwriteTarget;

#[derive(Debug, Clone)]
pub struct UpsertChannelOverwrite {
    pub channel_id: ChannelId,
    pub organization_id: OrganizationId,
    pub target: OverwriteTarget,
    pub allow: i64,
    pub deny: i64,
}

#[derive(Debug, Clone)]
pub struct DeleteChannelOverwrite {
    pub channel_id: ChannelId,
    pub target: OverwriteTarget,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChannelId;
    use crate::domain::OverwriteTarget;
    use common::OrganizationId;
    use uuid::Uuid;

    #[test]
    fn upsert_channel_overwrite_is_clone() {
        let cmd = UpsertChannelOverwrite {
            channel_id: ChannelId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            target: OverwriteTarget::Everyone,
            allow: 32,
            deny: 0,
        };
        let cloned = cmd.clone();
        assert_eq!(cmd.channel_id, cloned.channel_id);
        assert_eq!(cmd.allow, cloned.allow);
        assert_eq!(cmd.target, cloned.target);
    }

    #[test]
    fn delete_channel_overwrite_is_clone() {
        let cmd = DeleteChannelOverwrite {
            channel_id: ChannelId(Uuid::new_v4()),
            target: OverwriteTarget::Everyone,
        };
        let cloned = cmd.clone();
        assert_eq!(cmd.channel_id, cloned.channel_id);
        assert_eq!(cmd.target, cloned.target);
    }
}
