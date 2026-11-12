#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformAdmin(());

impl PlatformAdmin {
    pub const ROLE: &'static str = "mestier:admin";

    pub fn from_realm_roles(roles: &[String]) -> Option<Self> {
        roles
            .iter()
            .any(|role| role == Self::ROLE)
            .then_some(Self(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_admin_realm_role_opens_the_back_office() {
        let roles = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();

        assert!(PlatformAdmin::from_realm_roles(&roles(&["user", "mestier:admin"])).is_some());
        assert!(PlatformAdmin::from_realm_roles(&roles(&["user", "mestier:news"])).is_none());
        assert!(PlatformAdmin::from_realm_roles(&[]).is_none());
    }
}
