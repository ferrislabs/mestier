/// Proof that the caller may run the news back-office. It is built only from
/// the realm roles of the caller's own token, so no back-office use case can
/// be reached without that check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewsEditor(());

impl NewsEditor {
    /// The FerrisKey realm role that opens the back-office. It is never given
    /// to the policy engine, where it would pass every organization's checks.
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

        assert!(NewsEditor::from_realm_roles(&roles(&["user", "mestier:admin"])).is_some());
        assert!(NewsEditor::from_realm_roles(&roles(&["user", "mestier:news"])).is_none());
        assert!(NewsEditor::from_realm_roles(&[]).is_none());
    }
}
