use chrono::{DateTime, Utc};
use common::{CoreError, RoleId};
use discord::{
    ChannelId, ChannelPermissionOverwrite, OrganizationId, OverwriteTarget, UserId,
    ids::OverwriteId,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChannelPermissionOverwriteRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub org_id: Uuid,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub allow: i64,
    pub deny: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ChannelPermissionOverwriteRow> for ChannelPermissionOverwrite {
    type Error = CoreError;

    fn try_from(r: ChannelPermissionOverwriteRow) -> Result<Self, Self::Error> {
        let target = match (r.target_type.as_str(), r.target_id) {
            ("EVERYONE", None) => OverwriteTarget::Everyone,
            ("ROLE", Some(id)) => OverwriteTarget::Role(RoleId(id)),
            ("MEMBER", Some(id)) => OverwriteTarget::Member(UserId(id)),
            other => {
                return Err(CoreError::Internal(format!(
                    "invalid overwrite target: ({:?}, {:?})",
                    other.0, other.1
                )));
            }
        };
        Ok(ChannelPermissionOverwrite {
            id: OverwriteId(r.id),
            channel_id: ChannelId(r.channel_id),
            organization_id: OrganizationId(r.org_id),
            target,
            allow: r.allow,
            deny: r.deny,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn base_row(target_type: &str, target_id: Option<Uuid>) -> ChannelPermissionOverwriteRow {
        ChannelPermissionOverwriteRow {
            id: Uuid::from_u128(1),
            channel_id: Uuid::from_u128(2),
            org_id: Uuid::from_u128(3),
            target_type: target_type.to_string(),
            target_id,
            allow: 32,
            deny: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn row_maps_everyone_to_everyone_target() {
        let row = base_row("EVERYONE", None);
        let ow: ChannelPermissionOverwrite = row.try_into().unwrap();
        assert!(matches!(ow.target, OverwriteTarget::Everyone));
        assert_eq!(ow.organization_id, OrganizationId(Uuid::from_u128(3)));
    }

    #[test]
    fn row_maps_role_to_role_target() {
        let role = Uuid::from_u128(10);
        let row = base_row("ROLE", Some(role));
        let ow: ChannelPermissionOverwrite = row.try_into().unwrap();
        assert!(matches!(ow.target, OverwriteTarget::Role(r) if r == RoleId(role)));
    }

    #[test]
    fn row_maps_member_to_member_target() {
        let user = Uuid::from_u128(20);
        let row = base_row("MEMBER", Some(user));
        let ow: ChannelPermissionOverwrite = row.try_into().unwrap();
        assert!(matches!(ow.target, OverwriteTarget::Member(u) if u == UserId(user)));
    }

    #[test]
    fn row_rejects_invalid_combination() {
        let row = base_row("EVERYONE", Some(Uuid::from_u128(99)));
        assert!(ChannelPermissionOverwrite::try_from(row).is_err());
    }
}
