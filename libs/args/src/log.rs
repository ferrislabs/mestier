#[derive(clap::Args, Debug, Clone)]
pub struct LogArgs {
    #[arg(
        long = "log-filter",
        env = "LOG_FILTER",
        name = "LOG_FILTER",
        long_help = "The log filter to use\nhttps://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives",
        default_value = "info"
    )]
    pub filter: String,

    #[arg(
        long = "log-json",
        env = "LOG_JSON",
        name = "LOG_JSON",
        long_help = "Whether to log in JSON format"
    )]
    pub json: bool,
}

impl Default for LogArgs {
    fn default() -> Self {
        Self {
            filter: "info".to_string(),
            json: false,
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
        log: LogArgs,
    }

    #[test]
    fn default_values() {
        let args = LogArgs::default();
        assert_eq!(args.filter, "info");
        assert!(!args.json);
    }

    #[test]
    fn parse_custom_filter() {
        let cmd = Cmd::try_parse_from(["cmd", "--log-filter", "debug,tower=warn"]).unwrap();
        assert_eq!(cmd.log.filter, "debug,tower=warn");
    }

    #[test]
    fn parse_json_flag() {
        let cmd = Cmd::try_parse_from(["cmd", "--log-json"]).unwrap();
        assert!(cmd.log.json);
    }

    #[test]
    fn parse_defaults_when_no_args() {
        let cmd = Cmd::try_parse_from(["cmd"]).unwrap();
        assert_eq!(cmd.log.filter, "info");
        assert!(!cmd.log.json);
    }
}
