use std::net::SocketAddr;
use std::time::Duration;

use common::OrganizationId;
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{Address, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde_json::{Map, Value, json};
use uuid::Uuid;

use crate::application::MestierUseCase;
use crate::domain::automation::connector::smtp::{BodyFormat, Draft};
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::infrastructure::automation::webhook::address_policy::{
    AddressVerdict, PrivateNetworkAccess, judge,
};

pub const SEND_TIMEOUT: Duration = Duration::from_secs(30);

enum AddressRule {
    Policy(PrivateNetworkAccess),
    #[cfg(test)]
    Open,
}

impl AddressRule {
    fn verdict(&self, address: std::net::IpAddr) -> AddressVerdict {
        match self {
            Self::Policy(access) => judge(address, *access),
            #[cfg(test)]
            Self::Open => AddressVerdict::Allowed,
        }
    }
}

pub struct SmtpSendConnector {
    usecase: MestierUseCase,
    rule: AddressRule,
    timeout: Duration,
}

impl SmtpSendConnector {
    pub fn new(usecase: MestierUseCase, access: PrivateNetworkAccess) -> Self {
        Self {
            usecase,
            rule: AddressRule::Policy(access),
            timeout: SEND_TIMEOUT,
        }
    }

    #[cfg(test)]
    fn reaching_anything(usecase: MestierUseCase, timeout: Duration) -> Self {
        Self {
            usecase,
            rule: AddressRule::Open,
            timeout,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Security {
    Ssl,
    StartTls,
    None,
}

struct Server {
    host: String,
    port: u16,
    security: Security,
    login: Option<(String, String)>,
    sender: Address,
}

fn text<'a>(fields: &'a Map<String, Value>, name: &str) -> Option<&'a str> {
    fields
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn security_of(fields: &Map<String, Value>) -> Result<Security, String> {
    match text(fields, "security") {
        None | Some("ssl") => Ok(Security::Ssl),
        Some("starttls") => Ok(Security::StartTls),
        Some("none") => Ok(Security::None),
        Some(_) => Err("the SMTP credential's security is not usable".to_owned()),
    }
}

fn port_of(fields: &Map<String, Value>, security: Security) -> Result<u16, String> {
    let default = match security {
        Security::Ssl => 465,
        Security::StartTls => 587,
        Security::None => 25,
    };
    match fields.get("port") {
        None | Some(Value::Null) => Some(default),
        Some(Value::Number(number)) => number.as_u64().and_then(|port| u16::try_from(port).ok()),
        Some(Value::String(port)) if port.trim().is_empty() => Some(default),
        Some(Value::String(port)) => port.trim().parse().ok(),
        Some(_) => None,
    }
    .filter(|port| *port > 0)
    .ok_or_else(|| "the SMTP credential's port is not usable".to_owned())
}

fn server_from(fields: &Map<String, Value>) -> Result<Server, String> {
    let host = text(fields, "host")
        .filter(|host| !host.contains(char::is_whitespace))
        .ok_or_else(|| "the SMTP credential's host is not usable".to_owned())?;
    let security = security_of(fields)?;
    let port = port_of(fields, security)?;
    let username = text(fields, "username").ok_or_else(|| {
        "the SMTP credential needs a user, which is also the sender address".to_owned()
    })?;
    let sender = username.parse::<Address>().map_err(|_| {
        "the SMTP credential's user must be an e-mail address to serve as the sender".to_owned()
    })?;
    let password = fields
        .get("password")
        .and_then(Value::as_str)
        .filter(|password| !password.is_empty());

    Ok(Server {
        host: host.to_owned(),
        port,
        security,
        login: password.map(|password| (username.to_owned(), password.to_owned())),
        sender,
    })
}

async fn pick_address(host: &str, port: u16, rule: &AddressRule) -> Result<SocketAddr, String> {
    let resolved = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| format!("the SMTP host `{host}` does not resolve"))?;

    let mut refusal: Option<&'static str> = None;
    for address in resolved {
        match rule.verdict(address.ip()) {
            AddressVerdict::Allowed => return Ok(address),
            AddressVerdict::Refused(reason) => refusal = Some(reason),
        }
    }
    Err(format!(
        "the SMTP host `{host}` resolves to {}",
        refusal.unwrap_or("no address at all")
    ))
}

fn transport(
    server: &Server,
    address: SocketAddr,
    timeout: Duration,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let tls = match server.security {
        Security::None => Tls::None,
        Security::Ssl => Tls::Wrapper(tls_parameters(&server.host)?),
        Security::StartTls => Tls::Required(tls_parameters(&server.host)?),
    };
    let mut builder =
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(address.ip().to_string())
            .port(address.port())
            .tls(tls)
            .timeout(Some(timeout));
    if let Some((username, password)) = &server.login {
        builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }
    Ok(builder.build())
}

fn tls_parameters(host: &str) -> Result<TlsParameters, String> {
    TlsParameters::new(host.to_owned())
        .map_err(|_| "the SMTP host cannot be used as a TLS server name".to_owned())
}

fn mailbox(field: &str, address: &str) -> Result<Mailbox, String> {
    address
        .parse::<Address>()
        .map(|address| Mailbox::new(None, address))
        .map_err(|_| format!("`{field}` holds an address that is not valid"))
}

fn message_from(
    sender: &Address,
    draft: &Draft<'_>,
    message_id: String,
) -> Result<Message, String> {
    let mut builder = Message::builder()
        .from(Mailbox::new(None, sender.clone()))
        .subject(draft.subject)
        .message_id(Some(message_id))
        .header(match draft.format {
            BodyFormat::Text => ContentType::TEXT_PLAIN,
            BodyFormat::Html => ContentType::TEXT_HTML,
        });
    for address in &draft.to {
        builder = builder.to(mailbox("to", address)?);
    }
    for address in &draft.cc {
        builder = builder.cc(mailbox("cc", address)?);
    }
    builder
        .body(draft.body.to_owned())
        .map_err(|_| "the message cannot be built".to_owned())
}

fn scrub(error: String, secrets: &[&str]) -> String {
    secrets
        .iter()
        .filter(|secret| !secret.is_empty())
        .fold(error, |error, secret| error.replace(secret, "***"))
}

async fn deliver(
    server: &Server,
    draft: &Draft<'_>,
    rule: &AddressRule,
    timeout: Duration,
) -> Result<Value, String> {
    let message_id = format!("<{}@{}>", Uuid::now_v7().simple(), server.sender.domain());
    let message = message_from(&server.sender, draft, message_id.clone())?;
    let address = pick_address(&server.host, server.port, rule).await?;
    let transport = transport(server, address, timeout)?;

    let password = server
        .login
        .as_ref()
        .map(|(_, password)| password.as_str())
        .unwrap_or_default();
    transport
        .send(message)
        .await
        .map_err(|error| scrub(format!("sending the e-mail failed: {error}"), &[password]))?;

    Ok(json!({
        "message_id": message_id,
        "accepted": draft.recipients().collect::<Vec<_>>(),
    }))
}

async fn open_server(
    usecase: &MestierUseCase,
    organization_id: OrganizationId,
    credential_id: Option<Uuid>,
) -> Result<Server, String> {
    let credential_id = credential_id
        .ok_or_else(|| "an SMTP credential is required to send an e-mail".to_owned())?;
    let (credential, plaintext) = usecase
        .open_credential_for_run(organization_id, credential_id)
        .await?;
    if credential.kind != "smtp" {
        return Err(format!(
            "credential `{}` is not an SMTP credential",
            credential.name
        ));
    }
    let fields = match serde_json::from_slice(&plaintext) {
        Ok(Value::Object(fields)) => fields,
        _ => return Err("the SMTP credential's data is not usable".to_owned()),
    };
    server_from(&fields)
}

impl Connector for SmtpSendConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let draft = match Draft::from_config(input.config) {
            Ok(draft) => draft,
            Err(fault) => {
                return ConnectorOutcome::Failed {
                    error: fault.to_string(),
                };
            }
        };
        let server = match open_server(&self.usecase, input.org_id, input.credential_id).await {
            Ok(server) => server,
            Err(error) => return ConnectorOutcome::Failed { error },
        };

        match tokio::time::timeout(
            self.timeout,
            deliver(&server, &draft, &self.rule, self.timeout),
        )
        .await
        {
            Ok(Ok(output)) => ConnectorOutcome::Produced(output),
            Ok(Err(error)) => ConnectorOutcome::Failed { error },
            Err(_) => ConnectorOutcome::Failed {
                error: format!(
                    "the SMTP server did not answer within {} seconds",
                    self.timeout.as_secs().max(1)
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use base64::Engine;
    use serde_json::json;
    use sqlx::PgPool;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{TcpListener, TcpStream};

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::credential::{CreateCredentialCommand, CredentialOrigin};
    use crate::domain::automation::run::DryRunInput;
    use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
    use crate::infrastructure::automation::connectors::world_support;
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::realtime::EventHub;

    const PASSWORD: &str = "hunter2-do-not-leak";

    #[derive(Clone, Default)]
    struct Script {
        offer_starttls: bool,
        refuse_login_echoing_it: bool,
        refuse_recipient: Option<&'static str>,
        silent: bool,
        implicit_tls: bool,
    }

    #[derive(Default)]
    struct Captured {
        connections: usize,
        commands: Vec<String>,
        logins: Vec<String>,
        messages: Vec<String>,
        tls_first_byte: Option<u8>,
    }

    struct FakeSmtp {
        port: u16,
        captured: Arc<Mutex<Captured>>,
    }

    impl FakeSmtp {
        async fn start(script: Script) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let captured = Arc::new(Mutex::new(Captured::default()));
            let shared = Arc::clone(&captured);
            tokio::spawn(async move {
                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(serve(stream, script.clone(), Arc::clone(&shared)));
                }
            });
            Self { port, captured }
        }

        fn connections(&self) -> usize {
            self.captured.lock().unwrap().connections
        }

        fn commands(&self) -> Vec<String> {
            self.captured.lock().unwrap().commands.clone()
        }

        fn logins(&self) -> Vec<String> {
            self.captured.lock().unwrap().logins.clone()
        }

        fn messages(&self) -> Vec<String> {
            self.captured.lock().unwrap().messages.clone()
        }

        fn tls_first_byte(&self) -> Option<u8> {
            self.captured.lock().unwrap().tls_first_byte
        }

        fn saw(&self, prefix: &str) -> usize {
            self.commands()
                .iter()
                .filter(|command| command.to_ascii_uppercase().starts_with(prefix))
                .count()
        }
    }

    async fn serve(stream: TcpStream, script: Script, captured: Arc<Mutex<Captured>>) {
        captured.lock().unwrap().connections += 1;
        let (read, mut write) = stream.into_split();
        let mut reader = BufReader::new(read);

        if script.silent {
            std::future::pending::<()>().await;
        }
        if script.implicit_tls {
            let mut first = [0u8; 1];
            if reader.read_exact(&mut first).await.is_ok() {
                captured.lock().unwrap().tls_first_byte = Some(first[0]);
            }
            return;
        }

        write.write_all(b"220 fake.test ESMTP\r\n").await.ok();
        loop {
            let mut line = String::new();
            if !matches!(reader.read_line(&mut line).await, Ok(read) if read > 0) {
                return;
            }
            let command = line.trim_end().to_owned();
            captured.lock().unwrap().commands.push(command.clone());
            let upper = command.to_ascii_uppercase();

            let reply: String = if upper.starts_with("EHLO") || upper.starts_with("HELO") {
                let starttls = if script.offer_starttls {
                    "250-STARTTLS\r\n"
                } else {
                    ""
                };
                format!("250-fake.test\r\n250-AUTH PLAIN LOGIN\r\n{starttls}250 8BITMIME\r\n")
            } else if upper == "STARTTLS" {
                write.write_all(b"220 go ahead\r\n").await.ok();
                let mut first = [0u8; 1];
                if reader.read_exact(&mut first).await.is_ok() {
                    captured.lock().unwrap().tls_first_byte = Some(first[0]);
                }
                return;
            } else if upper.starts_with("AUTH PLAIN") {
                let encoded = command.split_whitespace().nth(2).unwrap_or_default();
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .unwrap_or_default();
                let login = String::from_utf8_lossy(&decoded).replace('\0', "|");
                captured.lock().unwrap().logins.push(login.clone());
                if script.refuse_login_echoing_it {
                    format!("535 5.7.8 authentication failed for {login}\r\n")
                } else {
                    "235 2.7.0 welcome\r\n".to_owned()
                }
            } else if upper.starts_with("MAIL FROM") {
                "250 2.1.0 sender ok\r\n".to_owned()
            } else if upper.starts_with("RCPT TO") {
                match script.refuse_recipient {
                    Some(refused) if command.contains(refused) => {
                        "550 5.1.1 no such mailbox\r\n".to_owned()
                    }
                    _ => "250 2.1.5 recipient ok\r\n".to_owned(),
                }
            } else if upper == "DATA" {
                write.write_all(b"354 go on\r\n").await.ok();
                let mut message = String::new();
                loop {
                    let mut data_line = String::new();
                    if !matches!(reader.read_line(&mut data_line).await, Ok(read) if read > 0) {
                        return;
                    }
                    if data_line == ".\r\n" {
                        break;
                    }
                    message.push_str(&data_line);
                }
                captured.lock().unwrap().messages.push(message);
                "250 2.0.0 queued\r\n".to_owned()
            } else if upper == "QUIT" {
                write.write_all(b"221 bye\r\n").await.ok();
                return;
            } else {
                "250 ok\r\n".to_owned()
            };
            write.write_all(reply.as_bytes()).await.ok();
        }
    }

    fn lazy_usecase() -> MestierUseCase {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
            .with_cipher(Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap())))
    }

    fn config(entries: &[(&str, Value)]) -> Map<String, Value> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect()
    }

    fn mail() -> Map<String, Value> {
        config(&[
            ("to", json!("claire@fontaine-immo.fr")),
            ("subject", json!("Votre dossier")),
            ("body", json!("Bonjour,\n\nCordialement")),
        ])
    }

    fn with(mut base: Map<String, Value>, field: &str, value: Value) -> Map<String, Value> {
        base.insert(field.to_owned(), value);
        base
    }

    fn input(
        org: OrganizationId,
        credential_id: Option<Uuid>,
        config: &Map<String, Value>,
    ) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: org,
            run_id: Uuid::from_u128(2),
            config,
            credential_id,
            operation: None,
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        match outcome {
            ConnectorOutcome::Failed { error } => error,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    fn sent(outcome: ConnectorOutcome) -> Value {
        match outcome {
            ConnectorOutcome::Produced(output) => output,
            other => panic!("expected an output, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn a_value_that_would_inject_a_header_or_overflow_the_recipients_fails_before_any_lookup()
    {
        let connector = SmtpSendConnector::new(lazy_usecase(), PrivateNetworkAccess::Denied);
        let fifty_one = (0..51)
            .map(|n| format!("u{n}@x.fr"))
            .collect::<Vec<_>>()
            .join(",");
        let cases = [
            (
                with(mail(), "to", json!("a@x.fr\r\nBcc: spy@evil.fr")),
                "`to` must not contain a line break",
            ),
            (
                with(mail(), "cc", json!(["b@x.fr", "c@x.fr\nBcc: spy@evil.fr"])),
                "`cc` must not contain a line break",
            ),
            (
                with(mail(), "subject", json!("Hi\r\nBcc: spy@evil.fr")),
                "`subject` must not contain a line break",
            ),
            (
                with(mail(), "to", json!(fifty_one)),
                "an e-mail goes to at most 50 recipients, `to` and `cc` together",
            ),
        ];

        for (config, expected) in cases {
            let outcome = connector
                .execute(input(
                    OrganizationId(Uuid::from_u128(1)),
                    Some(Uuid::from_u128(9)),
                    &config,
                ))
                .await;

            assert_eq!(failure(outcome), expected);
        }
    }

    #[tokio::test]
    async fn a_node_without_a_credential_fails_naming_it() {
        let connector = SmtpSendConnector::new(lazy_usecase(), PrivateNetworkAccess::Denied);
        let config = mail();

        let outcome = connector
            .execute(input(OrganizationId(Uuid::from_u128(1)), None, &config))
            .await;

        assert_eq!(
            failure(outcome),
            "an SMTP credential is required to send an e-mail"
        );
    }

    #[test]
    fn the_server_is_read_from_the_credential_with_a_port_per_security() {
        let fields = |security: Option<&str>, port: Option<Value>| {
            let mut fields = config(&[
                ("host", json!("smtp.exemple.fr")),
                ("username", json!("atelier@duval.fr")),
                ("password", json!(PASSWORD)),
            ]);
            if let Some(security) = security {
                fields.insert("security".to_owned(), json!(security));
            }
            if let Some(port) = port {
                fields.insert("port".to_owned(), port);
            }
            fields
        };

        let ssl = server_from(&fields(None, None)).unwrap();
        let starttls = server_from(&fields(Some("starttls"), None)).unwrap();
        let plain = server_from(&fields(Some("none"), Some(json!("2525")))).unwrap();

        assert_eq!((ssl.security, ssl.port), (Security::Ssl, 465));
        assert_eq!(
            (starttls.security, starttls.port),
            (Security::StartTls, 587)
        );
        assert_eq!((plain.security, plain.port), (Security::None, 2525));
        assert_eq!(ssl.sender.to_string(), "atelier@duval.fr");
        assert_eq!(
            ssl.login,
            Some(("atelier@duval.fr".to_owned(), PASSWORD.to_owned()))
        );
    }

    #[test]
    fn an_unusable_credential_is_refused_without_echoing_any_of_it() {
        let base = || {
            config(&[
                ("host", json!("smtp.exemple.fr")),
                ("username", json!("atelier@duval.fr")),
                ("password", json!(PASSWORD)),
            ])
        };
        let cases = [
            (
                with(base(), "host", json!("bad host")),
                "the SMTP credential's host is not usable",
            ),
            (
                with(base(), "port", json!(70_000)),
                "the SMTP credential's port is not usable",
            ),
            (
                with(base(), "port", json!(0)),
                "the SMTP credential's port is not usable",
            ),
            (
                with(base(), "security", json!("carrier-pigeon")),
                "the SMTP credential's security is not usable",
            ),
            (
                with(base(), "username", json!("not-an-address")),
                "the SMTP credential's user must be an e-mail address to serve as the sender",
            ),
        ];

        for (fields, expected) in cases {
            let error = server_from(&fields).err().expect("refused");

            assert_eq!(error, expected);
            assert!(!error.contains(PASSWORD));
        }
        let mut without_user = base();
        without_user.remove("username");
        assert!(server_from(&without_user).is_err());
    }

    #[test]
    fn a_secret_is_scrubbed_from_a_message() {
        let scrubbed = scrub(
            format!("bad login for {PASSWORD}, {PASSWORD}"),
            &[PASSWORD, ""],
        );

        assert_eq!(scrubbed, "bad login for ***, ***");
    }

    #[test]
    fn the_message_carries_the_headers_the_draft_names_and_nothing_else() {
        let sender: Address = "atelier@duval.fr".parse().unwrap();
        let config = with(
            with(mail(), "cc", json!("marc@duval.fr")),
            "format",
            json!("html"),
        );
        let draft = Draft::from_config(&config).unwrap();

        let message = message_from(&sender, &draft, "<abc@duval.fr>".to_owned()).unwrap();
        let text = String::from_utf8(message.formatted()).unwrap();

        assert!(text.contains("From: atelier@duval.fr"), "{text}");
        assert!(text.contains("To: claire@fontaine-immo.fr"), "{text}");
        assert!(text.contains("Cc: marc@duval.fr"), "{text}");
        assert!(text.contains("Subject: Votre dossier"), "{text}");
        assert!(text.contains("Message-ID: <abc@duval.fr>"), "{text}");
        assert!(text.contains("Content-Type: text/html"), "{text}");
        assert!(!text.contains("Bcc"), "{text}");
    }

    #[test]
    fn an_address_that_is_not_valid_is_refused_naming_the_field_without_echoing_it() {
        let sender: Address = "atelier@duval.fr".parse().unwrap();
        let config = with(mail(), "cc", json!("Name <nobody>"));
        let draft = Draft::from_config(&config).unwrap();

        let error = message_from(&sender, &draft, "<x@y>".to_owned()).unwrap_err();

        assert_eq!(error, "`cc` holds an address that is not valid");
    }

    #[test]
    fn the_dry_run_answers_with_the_sample_and_the_sample_names_what_a_run_returns() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("smtp.send", 1).unwrap();
        let config = mail();

        let outcome = SmtpSendConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
        assert_same_shape(
            &json!({ "message_id": "<x@y>", "accepted": ["a@x.fr"] }),
            "smtp.send",
        );
    }

    async fn credential(
        usecase: &MestierUseCase,
        org: OrganizationId,
        port: u16,
        security: &str,
        host: &str,
    ) -> Uuid {
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id: org,
                kind: "smtp".to_owned(),
                name: format!("Atelier {host} {security} {port}"),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({
                    "host": host,
                    "port": port,
                    "security": security,
                    "username": "atelier@duval.fr",
                    "password": PASSWORD,
                })),
            })
            .await
            .unwrap();
        credential.id
    }

    struct World {
        pool: PgPool,
        usecase: MestierUseCase,
        org: OrganizationId,
    }

    async fn world(label: &str) -> World {
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, label).await;
        World {
            usecase: usecase(pool.clone()),
            pool,
            org,
        }
    }

    fn connector_for(world: &World) -> SmtpSendConnector {
        SmtpSendConnector::reaching_anything(world.usecase.clone(), Duration::from_secs(10))
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_mail_is_sent_through_the_credential_and_reports_its_id_and_recipients() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-send").await;
        let server = FakeSmtp::start(Script::default()).await;
        let credential =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let config = with(
            with(
                mail(),
                "to",
                json!("claire@fontaine-immo.fr; paul@fontaine-immo.fr"),
            ),
            "cc",
            json!(["marc@duval.fr"]),
        );

        let outcome = connector_for(&world)
            .execute(input(world.org, Some(credential), &config))
            .await;

        let output = sent(outcome);
        assert_same_shape(&output, "smtp.send");
        assert_eq!(
            output["accepted"],
            json!([
                "claire@fontaine-immo.fr",
                "paul@fontaine-immo.fr",
                "marc@duval.fr"
            ])
        );
        let message_id = output["message_id"].as_str().unwrap();
        assert!(message_id.starts_with('<') && message_id.ends_with("@duval.fr>"));
        assert_eq!(server.logins(), [format!("|atelier@duval.fr|{PASSWORD}")]);
        assert_eq!(server.saw("MAIL FROM"), 1);
        assert_eq!(server.saw("RCPT TO"), 3);
        let messages = server.messages();
        assert_eq!(messages.len(), 1);
        assert!(
            messages[0].contains("Subject: Votre dossier"),
            "{}",
            messages[0]
        );
        assert!(messages[0].contains(&format!("Message-ID: {message_id}")));
        assert!(messages[0].contains("Content-Type: text/plain"));
        assert!(messages[0].contains("Cc: marc@duval.fr"));
        assert!(messages[0].contains("Cordialement"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_html_body_is_sent_as_html() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-html").await;
        let server = FakeSmtp::start(Script::default()).await;
        let credential =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let config = with(
            with(mail(), "body", json!("<p>Bonjour</p>")),
            "format",
            json!("html"),
        );

        let outcome = connector_for(&world)
            .execute(input(world.org, Some(credential), &config))
            .await;

        sent(outcome);
        assert!(server.messages()[0].contains("Content-Type: text/html"));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_refused_recipient_fails_the_node_and_queues_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-rcpt").await;
        let server = FakeSmtp::start(Script {
            refuse_recipient: Some("ghost@"),
            ..Script::default()
        })
        .await;
        let credential =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let config = with(
            mail(),
            "to",
            json!("claire@fontaine-immo.fr, ghost@fontaine-immo.fr"),
        );

        let outcome = connector_for(&world)
            .execute(input(world.org, Some(credential), &config))
            .await;

        let error = failure(outcome);
        assert!(error.starts_with("sending the e-mail failed:"), "{error}");
        assert!(!error.contains(PASSWORD));
        assert!(server.messages().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_server_echoing_the_login_back_never_puts_the_secret_in_the_error() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-echo").await;
        let server = FakeSmtp::start(Script {
            refuse_login_echoing_it: true,
            ..Script::default()
        })
        .await;
        let credential =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let config = mail();

        let outcome = connector_for(&world)
            .execute(input(world.org, Some(credential), &config))
            .await;

        let error = failure(outcome);
        assert!(error.contains("***"), "{error}");
        assert!(!error.contains(PASSWORD), "{error}");
        assert!(server.messages().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_host_on_a_refused_address_is_never_connected_to() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-policy").await;
        let server = FakeSmtp::start(Script::default()).await;
        let loopback =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let private = credential(&world.usecase, world.org, 25, "none", "10.255.255.1").await;
        let metadata = credential(&world.usecase, world.org, 25, "none", "169.254.169.254").await;
        let config = mail();
        let guarded = SmtpSendConnector::new(world.usecase.clone(), PrivateNetworkAccess::Allowed);
        let strict = SmtpSendConnector::new(world.usecase.clone(), PrivateNetworkAccess::Denied);

        let to_loopback = guarded
            .execute(input(world.org, Some(loopback), &config))
            .await;
        let to_private = strict
            .execute(input(world.org, Some(private), &config))
            .await;
        let to_metadata = guarded
            .execute(input(world.org, Some(metadata), &config))
            .await;

        let loopback_error = failure(to_loopback);
        assert!(
            loopback_error.contains("`127.0.0.1` resolves to a loopback address"),
            "{loopback_error}"
        );
        let private_error = failure(to_private);
        assert!(
            private_error.contains("`10.255.255.1` resolves to"),
            "{private_error}"
        );
        assert!(failure(to_metadata).contains("cloud metadata"));
        assert_eq!(server.connections(), 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn starttls_is_required_before_anything_sensitive_is_sent() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-starttls").await;
        let not_offered = FakeSmtp::start(Script::default()).await;
        let offered_but_broken = FakeSmtp::start(Script {
            offer_starttls: true,
            ..Script::default()
        })
        .await;
        let without = credential(
            &world.usecase,
            world.org,
            not_offered.port,
            "starttls",
            "127.0.0.1",
        )
        .await;
        let with_tls = credential(
            &world.usecase,
            world.org,
            offered_but_broken.port,
            "starttls",
            "127.0.0.1",
        )
        .await;
        let config = mail();

        let refused = connector_for(&world)
            .execute(input(world.org, Some(without), &config))
            .await;
        let failed_handshake = connector_for(&world)
            .execute(input(world.org, Some(with_tls), &config))
            .await;

        assert!(!failure(refused).contains(PASSWORD));
        assert!(!failure(failed_handshake).contains(PASSWORD));
        for server in [&not_offered, &offered_but_broken] {
            assert_eq!(server.saw("AUTH"), 0, "{:?}", server.commands());
            assert_eq!(server.saw("MAIL"), 0);
            assert_eq!(server.saw("RCPT"), 0);
            assert!(server.logins().is_empty());
        }
        assert_eq!(offered_but_broken.saw("STARTTLS"), 1);
        assert_eq!(offered_but_broken.tls_first_byte(), Some(0x16));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn implicit_tls_opens_with_a_handshake_and_sends_nothing_in_clear() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-ssl").await;
        let server = FakeSmtp::start(Script {
            implicit_tls: true,
            ..Script::default()
        })
        .await;
        let credential =
            credential(&world.usecase, world.org, server.port, "ssl", "127.0.0.1").await;
        let config = mail();

        let outcome = connector_for(&world)
            .execute(input(world.org, Some(credential), &config))
            .await;

        assert!(!failure(outcome).contains(PASSWORD));
        assert_eq!(server.tls_first_byte(), Some(0x16));
        assert!(server.commands().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_server_that_never_answers_ends_the_node_at_the_timeout() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let world = world("smtp-timeout").await;
        let server = FakeSmtp::start(Script {
            silent: true,
            ..Script::default()
        })
        .await;
        let credential =
            credential(&world.usecase, world.org, server.port, "none", "127.0.0.1").await;
        let config = mail();
        let impatient =
            SmtpSendConnector::reaching_anything(world.usecase.clone(), Duration::from_millis(400));
        let started = std::time::Instant::now();

        let outcome = impatient
            .execute(input(world.org, Some(credential), &config))
            .await;

        assert!(started.elapsed() < Duration::from_secs(5));
        let error = failure(outcome);
        assert!(!error.is_empty() && !error.contains(PASSWORD), "{error}");
        assert_eq!(SEND_TIMEOUT, Duration::from_secs(30));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_of_another_kind_or_another_organization_is_refused() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let acme = world("smtp-acme").await;
        let globex_org = world_support::organization(&acme.pool, "smtp-globex").await;
        let server = FakeSmtp::start(Script::default()).await;
        let foreign = credential(&acme.usecase, globex_org, server.port, "none", "127.0.0.1").await;
        let (bearer, _) = acme
            .usecase
            .create_credential(CreateCredentialCommand {
                description: None,
                expires_at: None,
                org_id: acme.org,
                kind: "bearer_token".to_owned(),
                name: "Not smtp".to_owned(),
                origin: CredentialOrigin::Supplied,
                data: Some(json!({ "token": "abc" })),
            })
            .await
            .unwrap();
        let config = mail();

        let of_another_organization = connector_for(&acme)
            .execute(input(acme.org, Some(foreign), &config))
            .await;
        let of_another_kind = connector_for(&acme)
            .execute(input(acme.org, Some(bearer.id), &config))
            .await;

        assert!(failure(of_another_organization).contains("no longer exists"));
        assert_eq!(
            failure(of_another_kind),
            "credential `Not smtp` is not an SMTP credential"
        );
        assert_eq!(server.connections(), 0);
    }
}
