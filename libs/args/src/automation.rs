use common::AutomationConfig;

#[derive(clap::Args, Debug, Clone)]
pub struct AutomationArgs {
    #[arg(
        long = "automation-secret-key",
        env = "AUTOMATION_SECRET_KEY",
        name = "AUTOMATION_SECRET_KEY",
        long_help = "Base64-encoded 32-byte key sealing webhook secrets at rest. \
                     Without it the automation worker does not start and no webhook \
                     endpoint can be created. Keep it out of the store that holds the \
                     database: sealing protects against a leaked dump, not a leaked host."
    )]
    pub secret_key: Option<String>,

    #[arg(
        long = "automation-allow-private-network",
        env = "AUTOMATION_ALLOW_PRIVATE_NETWORK",
        name = "AUTOMATION_ALLOW_PRIVATE_NETWORK",
        default_value_t = false,
        long_help = "Let webhooks reach private, non-public addresses. Legitimate \
                     self-hosted, where the artisan's own LAN holds the integrations. \
                     Never enable it on an instance serving several organizations: a \
                     tenant would borrow the server's network rights."
    )]
    pub allow_private_network: bool,
}

impl From<AutomationArgs> for AutomationConfig {
    fn from(value: AutomationArgs) -> Self {
        Self {
            secret_key: value.secret_key,
            allow_private_network: value.allow_private_network,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[derive(Parser)]
    struct Cmd {
        #[command(flatten)]
        automation: AutomationArgs,
    }

    /// Absent by default rather than generated: a key invented at boot would
    /// make every secret sealed with it unreadable after the next restart.
    #[test]
    fn the_key_is_absent_by_default_and_the_private_network_closed() {
        let cmd = Cmd::try_parse_from(["cmd"]).unwrap();

        assert_eq!(cmd.automation.secret_key, None);
        assert!(!cmd.automation.allow_private_network);
    }

    #[test]
    fn parse_custom() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--automation-secret-key",
            "c2VjcmV0",
            "--automation-allow-private-network",
        ])
        .unwrap();

        assert_eq!(cmd.automation.secret_key.as_deref(), Some("c2VjcmV0"));
        assert!(cmd.automation.allow_private_network);
    }
}
