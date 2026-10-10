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

    #[arg(
        long = "auth-audience",
        env = "AUTH_AUDIENCE",
        name = "AUTH_AUDIENCE",
        default_value = "mestier-api",
        long_help = "The audience a token must name to be accepted by the API"
    )]
    pub audience: String,

    #[arg(
        long = "auth-trusted-clients",
        env = "AUTH_TRUSTED_CLIENTS",
        name = "AUTH_TRUSTED_CLIENTS",
        value_delimiter = ',',
        long_help = "Mestier's own OAuth clients whose tokens are accepted without the API audience. Defaults to AUTH_CLIENT_ID"
    )]
    pub trusted_clients: Vec<String>,
}

impl From<AuthArgs> for AuthConfig {
    fn from(value: AuthArgs) -> Self {
        let trusted_clients = if value.trusted_clients.is_empty() {
            vec![value.client_id]
        } else {
            value.trusted_clients
        };
        Self {
            issuer: value.issuer,
            audience: value.audience,
            trusted_clients,
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
