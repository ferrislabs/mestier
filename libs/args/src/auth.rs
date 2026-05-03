use common::AuthConfig;

#[derive(clap::Args, Debug, Clone)]
pub struct AuthArgs {
    #[arg(
        long = "auth-issuer",
        env = "AUTH_ISSUER",
        name = "AUTH_ISSUER",
        default_value = "http://localhost:3333/realms/mestier",
        long_help = "The issuer URL to use for authentication"
    )]
    pub issuer: String,

    #[arg(
        long = "auth-client-id",
        env = "AUTH_CLIENT_ID",
        name = "AUTH_CLIENT_ID",
        default_value = "mestier",
        long_help = "The client ID to use for authentication"
    )]
    pub client_id: String,

    #[arg(
        long = "auth-client-secret",
        env = "AUTH_CLIENT_SECRET",
        name = "AUTH_CLIENT_SECRET",
        default_value = "mestier",
        long_help = "The client secret to use for authentication"
    )]
    pub client_secret: String,
}

impl From<AuthArgs> for AuthConfig {
    fn from(value: AuthArgs) -> Self {
        Self {
            issuer: value.issuer,
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
        auth: AuthArgs,
    }

    #[test]
    fn parse_defaults() {
        let cmd = Cmd::try_parse_from(["cmd"]).unwrap();
        assert_eq!(cmd.auth.issuer, "http://localhost:3333/realms/mestier");
        assert_eq!(cmd.auth.client_id, "mestier");
        assert_eq!(cmd.auth.client_secret, "mestier");
    }

    #[test]
    fn parse_custom_issuer() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--auth-issuer",
            "https://auth.example.com/realms/prod",
        ])
        .unwrap();
        assert_eq!(cmd.auth.issuer, "https://auth.example.com/realms/prod");
    }

    #[test]
    fn parse_all_fields() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--auth-issuer",
            "https://sso.example.com",
            "--auth-client-id",
            "mestier",
            "--auth-client-secret",
            "supersecret",
        ])
        .unwrap();
        assert_eq!(cmd.auth.issuer, "https://sso.example.com");
        assert_eq!(cmd.auth.client_id, "mestier");
        assert_eq!(cmd.auth.client_secret, "supersecret");
    }
}
