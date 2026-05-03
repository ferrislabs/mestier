use std::{path::PathBuf, vec};

#[derive(clap::Args, Debug, Clone)]
pub struct ServerArgs {
    #[arg(
        short,
        long,
        env,
        num_args = 0..,
        value_delimiter = ',',
        long_help = "The port to run the application on",
    )]
    pub allowed_origins: Vec<String>,
    #[arg(
        short = 'H',
        long = "server-host",
        env = "SERVER_HOST",
        name = "SERVER_HOST",
        default_value = "0.0.0.0",
        long_help = "The host to run the application on"
    )]
    pub host: String,
    #[arg(
        short = 'P',
        long = "server-port",
        env = "SERVER_PORT",
        name = "SERVER_PORT",
        default_value_t = 3456,
        long_help = "The port to run the application on"
    )]
    pub port: u16,
    #[arg(
        long = "server-internal-port",
        env = "SERVER_INTERNAL_PORT",
        name = "SERVER_INTERNAL_PORT",
        default_value_t = 3457,
        long_help = "The port to run the internal application on (health, metrics, ...)"
    )]
    pub internal_port: u16,
    #[command(flatten)]
    pub tls: Option<ServerTlsArgs>,
}

#[derive(clap::Args, Debug, Clone)]
#[group(requires_all = ["SERVER_TLS_CERT", "SERVER_TLS_KEY"])]
pub struct ServerTlsArgs {
    #[arg(
        long = "server-tls-cert",
        env = "SERVER_TLS_CERT",
        name = "SERVER_TLS_CERT",
        long_help = "Path to the TLS cert file in PEM format",
        required = false
    )]
    pub cert: PathBuf,
    #[arg(
        long = "server-tls-key",
        env = "SERVER_TLS_KEY",
        name = "SERVER_TLS_KEY",
        long_help = "Path to the TLS key file in PEM format",
        required = false
    )]
    pub key: PathBuf,
}

impl Default for ServerArgs {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            host: "0.0.0.0".to_string(),
            port: 3333,
            internal_port: 3334,
            tls: None,
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
        server: ServerArgs,
    }

    #[test]
    fn default_values() {
        let args = ServerArgs::default();
        assert_eq!(args.host, "0.0.0.0");
        assert_eq!(args.port, 3333);
        assert_eq!(args.internal_port, 3334);
        assert!(args.allowed_origins.is_empty());
        assert!(args.tls.is_none());
    }

    #[test]
    fn parse_host_and_port() {
        let cmd =
            Cmd::try_parse_from(["cmd", "--server-host", "127.0.0.1", "--server-port", "8080"])
                .unwrap();
        assert_eq!(cmd.server.host, "127.0.0.1");
        assert_eq!(cmd.server.port, 8080);
    }

    #[test]
    fn parse_allowed_origins_comma_separated() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--allowed-origins",
            "https://app.example.com,https://admin.example.com",
        ])
        .unwrap();
        assert_eq!(
            cmd.server.allowed_origins,
            vec!["https://app.example.com", "https://admin.example.com"]
        );
    }

    #[test]
    fn parse_allowed_origins_repeated_flag() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--allowed-origins",
            "https://a.example.com",
            "--allowed-origins",
            "https://b.example.com",
        ])
        .unwrap();
        assert_eq!(cmd.server.allowed_origins.len(), 2);
    }

    #[test]
    fn parse_tls_args() {
        let cmd = Cmd::try_parse_from([
            "cmd",
            "--server-tls-cert",
            "/etc/ssl/cert.pem",
            "--server-tls-key",
            "/etc/ssl/key.pem",
        ])
        .unwrap();
        let tls = cmd.server.tls.unwrap();
        assert_eq!(tls.cert.to_str().unwrap(), "/etc/ssl/cert.pem");
        assert_eq!(tls.key.to_str().unwrap(), "/etc/ssl/key.pem");
    }
}
