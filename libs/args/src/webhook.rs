#[derive(clap::Args, Debug, Clone)]
pub struct WebhookArgs {
    #[arg(
        long = "ferriskey-webhook-secret",
        env = "FERRISKEY_WEBHOOK_SECRET",
        name = "FERRISKEY_WEBHOOK_SECRET",
        long_help = "Shared HMAC-SHA256 secret used to verify FerrisKey webhook payloads"
    )]
    pub ferriskey_secret: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cmd {
        #[command(flatten)]
        webhook: WebhookArgs,
    }

    #[test]
    fn parse_from_arg() {
        let cmd =
            Cmd::try_parse_from(["cmd", "--ferriskey-webhook-secret", "supersecret"]).unwrap();
        assert_eq!(cmd.webhook.ferriskey_secret, "supersecret");
    }

    #[test]
    fn missing_secret_fails() {
        let result = Cmd::try_parse_from(["cmd"]);
        assert!(result.is_err());
    }
}
