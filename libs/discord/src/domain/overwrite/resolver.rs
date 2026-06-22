use crate::domain::ChannelPermissionOverwrite;

pub fn resolve_channel_permissions(
    base: i64,
    everyone: Option<&ChannelPermissionOverwrite>,
    role_overwrites: &[ChannelPermissionOverwrite],
    member: Option<&ChannelPermissionOverwrite>,
) -> i64 {
    let _ = (everyone, role_overwrites, member);
    base // stub — implemented in Task 5
}
