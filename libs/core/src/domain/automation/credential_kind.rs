#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialCategory {
    Http,
    Webhook,
    Email,
    Servers,
    Other,
}

impl CredentialCategory {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Webhook => "webhook",
            Self::Email => "email",
            Self::Servers => "servers",
            Self::Other => "other",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Http => "Requêtes HTTP",
            Self::Webhook => "Webhooks entrants",
            Self::Email => "E-mail",
            Self::Servers => "Serveurs et bases de données",
            Self::Other => "Autres",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialUsage {
    Http,
    Webhook,
    Smtp,
    Imap,
    Ssh,
    Ftp,
    Database,
    Odoo,
}

impl CredentialUsage {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Webhook => "webhook",
            Self::Smtp => "smtp",
            Self::Imap => "imap",
            Self::Ssh => "ssh",
            Self::Ftp => "ftp",
            Self::Database => "database",
            Self::Odoo => "odoo",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Http => "Requête HTTP et opérations de collection",
            Self::Webhook => "Webhook",
            Self::Smtp => "E-mail SMTP",
            Self::Imap => "Lecture d’e-mails",
            Self::Ssh => "Commandes SSH",
            Self::Ftp => "Transfert de fichiers",
            Self::Database => "Requêtes SQL",
            Self::Odoo => "Actions Odoo",
        }
    }

    pub const fn has_node(self) -> bool {
        matches!(self, Self::Http | Self::Webhook | Self::Odoo | Self::Smtp)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialFieldKind {
    Text,
    Password,
    Textarea,
    Select,
    Number,
    Boolean,
    Json,
}

impl CredentialFieldKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Textarea => "textarea",
            Self::Select => "select",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Json => "json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowWhen {
    pub field: &'static str,
    pub one_of: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialField {
    pub name: &'static str,
    pub label: &'static str,
    pub kind: CredentialFieldKind,
    pub secret: bool,
    pub required: bool,
    pub default: Option<&'static str>,
    pub placeholder: Option<&'static str>,
    pub hint: Option<&'static str>,
    pub options: &'static [CredentialOption],
    pub show_when: Option<ShowWhen>,
}

impl CredentialField {
    const fn new(name: &'static str, label: &'static str, kind: CredentialFieldKind) -> Self {
        Self {
            name,
            label,
            kind,
            secret: false,
            required: false,
            default: None,
            placeholder: None,
            hint: None,
            options: &[],
            show_when: None,
        }
    }

    const fn required(mut self) -> Self {
        self.required = true;
        self
    }

    const fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    const fn default_value(mut self, value: &'static str) -> Self {
        self.default = Some(value);
        self
    }

    const fn placeholder(mut self, value: &'static str) -> Self {
        self.placeholder = Some(value);
        self
    }

    const fn hint(mut self, value: &'static str) -> Self {
        self.hint = Some(value);
        self
    }

    const fn options(mut self, options: &'static [CredentialOption]) -> Self {
        self.options = options;
        self
    }

    const fn show_when(mut self, field: &'static str, one_of: &'static [&'static str]) -> Self {
        self.show_when = Some(ShowWhen { field, one_of });
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CredentialKind {
    pub kind: &'static str,
    pub label: &'static str,
    pub category: CredentialCategory,
    pub description: &'static str,
    pub usages: &'static [CredentialUsage],
    pub origin_choices: &'static [&'static str],
    pub fields: &'static [CredentialField],
    pub executable: bool,
    pub connectable: bool,
}

impl CredentialKind {
    pub fn field(&self, name: &str) -> Option<&'static CredentialField> {
        self.fields.iter().find(|field| field.name == name)
    }

    pub fn secret_names(&self) -> Vec<&'static str> {
        self.fields
            .iter()
            .filter(|field| field.secret)
            .map(|field| field.name)
            .collect()
    }

    pub fn accepts_usage(&self, usage: CredentialUsage) -> bool {
        self.usages.contains(&usage)
    }
}

use CredentialFieldKind::{Json, Number, Password, Select, Text, Textarea};

const fn option(value: &'static str, label: &'static str) -> CredentialOption {
    CredentialOption { value, label }
}

const fn user() -> CredentialField {
    CredentialField::new("username", "Utilisateur", Text)
}

const fn password() -> CredentialField {
    CredentialField::new("password", "Mot de passe", Password).secret()
}

const fn host(placeholder: &'static str) -> CredentialField {
    CredentialField::new("host", "Hôte", Text)
        .required()
        .placeholder(placeholder)
}

const fn port(default: &'static str) -> CredentialField {
    CredentialField::new("port", "Port", Number)
        .required()
        .default_value(default)
}

const fn scopes() -> CredentialField {
    CredentialField::new("scopes", "Scopes", Text)
        .placeholder("openid profile")
        .hint("Séparés par des espaces.")
}

const HTTP_HEADER_FIELDS: &[CredentialField] = &[
    CredentialField::new("header_name", "Nom de l’en-tête", Text)
        .required()
        .default_value("X-API-Key"),
    CredentialField::new("header_value", "Valeur", Password)
        .required()
        .secret(),
];

const HTTP_BASIC_FIELDS: &[CredentialField] = &[user().required(), password().required()];

const BEARER_TOKEN_FIELDS: &[CredentialField] = &[CredentialField::new("token", "Jeton", Password)
    .required()
    .secret()];

const API_KEY_QUERY_FIELDS: &[CredentialField] = &[
    CredentialField::new("param_name", "Nom du paramètre", Text)
        .required()
        .default_value("api_key"),
    CredentialField::new("param_value", "Valeur", Password)
        .required()
        .secret(),
];

const DIGEST_FIELDS: &[CredentialField] = &[user().required(), password().required()];

const OAUTH2_AUTHORIZATION_CODE_FIELDS: &[CredentialField] = &[
    CredentialField::new("authorization_url", "URL d’autorisation", Text)
        .required()
        .placeholder("https://auth.exemple.fr/authorize"),
    CredentialField::new("token_url", "URL du jeton", Text)
        .required()
        .placeholder("https://auth.exemple.fr/token"),
    CredentialField::new("userinfo_url", "URL des informations du compte", Text)
        .placeholder("https://auth.exemple.fr/userinfo")
        .hint("Facultative : sert à afficher le compte connecté."),
    CredentialField::new("client_id", "Client ID", Text).required(),
    CredentialField::new("client_secret", "Client secret", Password)
        .required()
        .secret(),
    scopes(),
    CredentialField::new("pkce", "PKCE", Select)
        .default_value("yes")
        .options(&[
            option("yes", "Activé (recommandé)"),
            option("no", "Désactivé"),
        ]),
];

const OAUTH2_CLIENT_CREDENTIALS_FIELDS: &[CredentialField] = &[
    CredentialField::new("token_url", "URL du jeton", Text)
        .required()
        .placeholder("https://auth.exemple.fr/token"),
    CredentialField::new("client_id", "Client ID", Text).required(),
    CredentialField::new("client_secret", "Client secret", Password)
        .required()
        .secret(),
    scopes(),
    CredentialField::new(
        "client_authentication",
        "Envoi des identifiants du client",
        Select,
    )
    .default_value("header")
    .options(&[
        option("header", "En-tête Authorization (Basic)"),
        option("body", "Corps de la requête"),
    ]),
];

const OAUTH1_FIELDS: &[CredentialField] = &[
    CredentialField::new("consumer_key", "Consumer key", Text).required(),
    CredentialField::new("consumer_secret", "Consumer secret", Password)
        .required()
        .secret(),
    CredentialField::new("request_token_url", "URL du jeton de requête", Text).required(),
    CredentialField::new("authorization_url", "URL d’autorisation", Text).required(),
    CredentialField::new("access_token_url", "URL du jeton d’accès", Text).required(),
    CredentialField::new("signature_method", "Méthode de signature", Select)
        .default_value("HMAC-SHA1")
        .options(&[
            option("HMAC-SHA1", "HMAC-SHA1"),
            option("HMAC-SHA256", "HMAC-SHA256"),
            option("PLAINTEXT", "PLAINTEXT"),
        ]),
];

const JWT_FIELDS: &[CredentialField] = &[
    CredentialField::new("algorithm", "Algorithme", Select)
        .default_value("HS256")
        .options(&[
            option("HS256", "HS256"),
            option("HS384", "HS384"),
            option("HS512", "HS512"),
            option("RS256", "RS256"),
            option("ES256", "ES256"),
        ]),
    CredentialField::new("key", "Secret ou clé privée (PEM)", Textarea)
        .required()
        .secret(),
];

const MTLS_FIELDS: &[CredentialField] = &[
    CredentialField::new("certificate", "Certificat (PEM)", Textarea)
        .required()
        .placeholder("-----BEGIN CERTIFICATE-----"),
    CredentialField::new("private_key", "Clé privée (PEM)", Textarea)
        .required()
        .secret(),
    CredentialField::new("passphrase", "Phrase de passe", Password)
        .secret()
        .hint("Les clés privées chiffrées ne sont pas prises en charge : fournissez une clé non chiffrée."),
    CredentialField::new(
        "ca_certificate",
        "Autorité de certification (PEM)",
        Textarea,
    )
    .hint("Seulement si le serveur utilise une autorité privée."),
];

const CUSTOM_AUTH_FIELDS: &[CredentialField] =
    &[CredentialField::new("json", "Définition (JSON)", Json)
        .required()
        .secret()
        .placeholder("{ \"headers\": { \"X-Token\": \"…\" }, \"qs\": {}, \"body\": {} }")];

const HMAC_FIELDS: &[CredentialField] = &[
    CredentialField::new("header_name", "En-tête de la signature", Text)
        .required()
        .default_value("X-Signature"),
    CredentialField::new("algorithm", "Algorithme", Select)
        .default_value("sha256")
        .options(&[
            option("sha256", "SHA-256"),
            option("sha512", "SHA-512"),
            option("sha1", "SHA-1"),
        ]),
    CredentialField::new("secret", "Secret partagé", Password)
        .required()
        .secret(),
];

const SMTP_FIELDS: &[CredentialField] = &[
    host("smtp.exemple.fr"),
    port("465"),
    CredentialField::new("security", "Sécurité", Select)
        .default_value("ssl")
        .options(&[
            option("ssl", "SSL/TLS"),
            option("starttls", "STARTTLS"),
            option("none", "Aucune"),
        ]),
    user(),
    password(),
];

const IMAP_FIELDS: &[CredentialField] = &[
    host("imap.exemple.fr"),
    port("993"),
    CredentialField::new("ssl", "SSL/TLS", Select)
        .default_value("yes")
        .options(&[option("yes", "Activé"), option("no", "Désactivé")]),
    user().required(),
    password().required(),
];

const SSH_FIELDS: &[CredentialField] = &[
    host("serveur.exemple.fr"),
    port("22"),
    user().required(),
    CredentialField::new("method", "Méthode", Select)
        .default_value("private-key")
        .options(&[
            option("private-key", "Clé privée"),
            option("password", "Mot de passe"),
        ]),
    password().required().show_when("method", &["password"]),
    CredentialField::new("private_key", "Clé privée", Textarea)
        .required()
        .secret()
        .placeholder("-----BEGIN OPENSSH PRIVATE KEY-----")
        .show_when("method", &["private-key"]),
    CredentialField::new("passphrase", "Phrase de passe", Password)
        .secret()
        .show_when("method", &["private-key"]),
];

const FTP_FIELDS: &[CredentialField] = &[
    CredentialField::new("protocol", "Protocole", Select)
        .default_value("sftp")
        .options(&[
            option("sftp", "SFTP"),
            option("ftps", "FTPS"),
            option("ftp", "FTP (non chiffré)"),
        ]),
    host("fichiers.exemple.fr"),
    port("22"),
    user().required(),
    password().required(),
];

const SQL_FIELDS: &[CredentialField] = &[
    CredentialField::new("engine", "Moteur", Select)
        .default_value("postgres")
        .options(&[
            option("postgres", "PostgreSQL"),
            option("mysql", "MySQL"),
            option("mariadb", "MariaDB"),
            option("mssql", "SQL Server"),
        ]),
    host("db.exemple.fr"),
    port("5432"),
    CredentialField::new("database", "Base", Text).required(),
    user().required(),
    password().required(),
    CredentialField::new("ssl", "SSL", Select)
        .default_value("require")
        .options(&[
            option("require", "Obligatoire"),
            option("prefer", "Si disponible"),
            option("disable", "Désactivé"),
        ]),
];

const ODOO_API_FIELDS: &[CredentialField] = &[
    CredentialField::new("base_url", "URL de l’instance", Text)
        .required()
        .placeholder("https://mon-entreprise.odoo.com"),
    CredentialField::new("database", "Base de données", Text).required(),
    CredentialField::new("username", "Utilisateur", Text).required(),
    CredentialField::new("api_key", "Clé d’API", Password)
        .required()
        .secret(),
];

use CredentialCategory::{Email, Http, Other, Servers, Webhook};
use CredentialUsage::{Database, Ftp, Imap, Odoo, Smtp, Ssh};

const NO_ORIGIN_CHOICE: &[&str] = &[];
const SUPPLIED_OR_GENERATED: &[&str] = &["supplied", "generated"];

const KINDS: &[CredentialKind] = &[
    CredentialKind {
        kind: "http_basic",
        label: "Basic Auth",
        category: Http,
        description: "Utilisateur et mot de passe envoyés dans l’en-tête Authorization.",
        usages: &[CredentialUsage::Http, CredentialUsage::Webhook],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: HTTP_BASIC_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "bearer_token",
        label: "Bearer",
        category: Http,
        description: "Un jeton envoyé sous la forme « Authorization: Bearer … ».",
        usages: &[CredentialUsage::Http, CredentialUsage::Webhook],
        origin_choices: SUPPLIED_OR_GENERATED,
        fields: BEARER_TOKEN_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "http_header",
        label: "Clé d’API dans un en-tête",
        category: Http,
        description: "Une clé transmise dans un en-tête HTTP de votre choix.",
        usages: &[CredentialUsage::Http, CredentialUsage::Webhook],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: HTTP_HEADER_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "api_key_query",
        label: "Clé d’API dans l’URL",
        category: Http,
        description: "Une clé ajoutée en paramètre de requête à chaque appel.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: API_KEY_QUERY_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "digest",
        label: "Digest Auth",
        category: Http,
        description: "Utilisateur et mot de passe échangés par défi HTTP Digest.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: DIGEST_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "oauth2_authorization_code",
        label: "OAuth2",
        category: Http,
        description: "Connexion déléguée : un utilisateur autorise Mestier à agir sur son compte.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: OAUTH2_AUTHORIZATION_CODE_FIELDS,
        executable: true,
        connectable: true,
    },
    CredentialKind {
        kind: "oauth2_client_credentials",
        label: "Client Credentials",
        category: Http,
        description: "OAuth2 de service à service : Mestier obtient un jeton avec son propre client.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: OAUTH2_CLIENT_CREDENTIALS_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "oauth1",
        label: "OAuth1",
        category: Http,
        description: "Ancienne génération d’OAuth, encore utilisée par certaines API.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: OAUTH1_FIELDS,
        executable: true,
        connectable: true,
    },
    CredentialKind {
        kind: "jwt",
        label: "JWT",
        category: Http,
        description: "Mestier signe un jeton JWT à chaque appel sortant.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: JWT_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "mtls",
        label: "Certificat client",
        category: Http,
        description: "Authentification mutuelle TLS (mTLS) par certificat.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: MTLS_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "custom_auth",
        label: "Authentification personnalisée",
        category: Http,
        description: "En-têtes, paramètres d’URL ou champs de corps à ajouter, décrits en JSON.",
        usages: &[CredentialUsage::Http],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: CUSTOM_AUTH_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "hmac",
        label: "Signature HMAC",
        category: Webhook,
        description: "Vérifie que les webhooks reçus sont signés avec un secret partagé.",
        usages: &[CredentialUsage::Webhook],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: HMAC_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "smtp",
        label: "SMTP",
        category: Email,
        description: "Envoi d’e-mails depuis votre serveur de messagerie.",
        usages: &[Smtp],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: SMTP_FIELDS,
        executable: true,
        connectable: false,
    },
    CredentialKind {
        kind: "imap",
        label: "IMAP",
        category: Email,
        description: "Lecture d’une boîte de réception.",
        usages: &[Imap],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: IMAP_FIELDS,
        executable: false,
        connectable: false,
    },
    CredentialKind {
        kind: "ssh",
        label: "SSH",
        category: Servers,
        description: "Accès à un serveur par mot de passe ou par clé privée.",
        usages: &[Ssh],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: SSH_FIELDS,
        executable: false,
        connectable: false,
    },
    CredentialKind {
        kind: "ftp",
        label: "FTP / SFTP",
        category: Servers,
        description: "Dépôt et récupération de fichiers sur un serveur distant.",
        usages: &[Ftp],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: FTP_FIELDS,
        executable: false,
        connectable: false,
    },
    CredentialKind {
        kind: "sql",
        label: "Base de données SQL",
        category: Servers,
        description: "PostgreSQL, MySQL, MariaDB ou SQL Server.",
        usages: &[Database],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: SQL_FIELDS,
        executable: false,
        connectable: false,
    },
    CredentialKind {
        kind: "odoo_api",
        label: "Clé d’API Odoo",
        category: Other,
        description: "Accès à une instance Odoo par clé d’API, utilisé par les actions Odoo.",
        usages: &[Odoo],
        origin_choices: NO_ORIGIN_CHOICE,
        fields: ODOO_API_FIELDS,
        executable: true,
        connectable: false,
    },
];

pub const HTTP_CREDENTIAL_KINDS: &[&str] = &[
    "bearer_token",
    "http_basic",
    "http_header",
    "api_key_query",
    "digest",
    "oauth2_authorization_code",
    "oauth2_client_credentials",
    "oauth1",
    "jwt",
    "mtls",
    "custom_auth",
];

pub const WEBHOOK_CREDENTIAL_KINDS: &[&str] =
    &["http_header", "http_basic", "bearer_token", "hmac"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SecretGuard {
    pub destinations: &'static [&'static str],
    pub secrets: &'static [&'static str],
}

pub const PROVIDER_URL_FIELDS: &[&str] = &[
    "authorization_url",
    "token_url",
    "userinfo_url",
    "request_token_url",
    "access_token_url",
];

pub fn secret_guard(kind: &str) -> Option<SecretGuard> {
    match kind {
        "oauth2_authorization_code" => Some(SecretGuard {
            destinations: &["authorization_url", "token_url", "userinfo_url"],
            secrets: &["client_secret"],
        }),
        "oauth2_client_credentials" => Some(SecretGuard {
            destinations: &["token_url"],
            secrets: &["client_secret"],
        }),
        "oauth1" => Some(SecretGuard {
            destinations: &[
                "request_token_url",
                "authorization_url",
                "access_token_url",
                "signature_method",
            ],
            secrets: &["consumer_secret"],
        }),
        "odoo_api" => Some(SecretGuard {
            destinations: &["base_url"],
            secrets: &["api_key"],
        }),
        _ => None,
    }
}

pub fn credential_kinds() -> &'static [CredentialKind] {
    KINDS
}

pub fn credential_kind(kind: &str) -> Option<&'static CredentialKind> {
    KINDS.iter().find(|candidate| candidate.kind == kind)
}

#[cfg(test)]
mod tests {
    use super::CredentialFieldKind::Boolean;
    use super::*;

    fn kind(code: &str) -> &'static CredentialKind {
        credential_kind(code).unwrap_or_else(|| panic!("{code} is a known kind"))
    }

    fn names(kind: &CredentialKind) -> Vec<&'static str> {
        kind.fields.iter().map(|field| field.name).collect()
    }

    #[test]
    fn the_catalogue_holds_the_seventeen_prototype_kinds_and_odoo() {
        let codes: Vec<&str> = credential_kinds().iter().map(|kind| kind.kind).collect();

        assert_eq!(
            codes,
            vec![
                "http_basic",
                "bearer_token",
                "http_header",
                "api_key_query",
                "digest",
                "oauth2_authorization_code",
                "oauth2_client_credentials",
                "oauth1",
                "jwt",
                "mtls",
                "custom_auth",
                "hmac",
                "smtp",
                "imap",
                "ssh",
                "ftp",
                "sql",
                "odoo_api",
            ]
        );
    }

    #[test]
    fn a_kind_code_is_unique() {
        let mut codes: Vec<&str> = credential_kinds().iter().map(|kind| kind.kind).collect();
        codes.sort_unstable();
        codes.dedup();

        assert_eq!(codes.len(), credential_kinds().len());
    }

    #[test]
    fn every_label_description_and_hint_is_french_with_a_typographic_apostrophe() {
        for kind in credential_kinds() {
            let mut texts = vec![kind.label, kind.description];
            for field in kind.fields {
                texts.push(field.label);
                texts.extend(field.hint);
                texts.extend(field.placeholder.filter(|text| !text.starts_with("http")));
                texts.extend(field.options.iter().map(|option| option.label));
            }
            for text in texts {
                assert!(!text.contains('\''), "`{text}` uses a straight apostrophe");
            }
        }
    }

    #[test]
    fn the_catalogue_carries_no_english_label_of_the_old_schemes() {
        let labels: Vec<&str> = credential_kinds()
            .iter()
            .flat_map(|kind| kind.fields.iter().map(|field| field.label))
            .collect();

        for english in [
            "Username",
            "Password",
            "Header name",
            "Header value",
            "Token",
        ] {
            assert!(!labels.contains(&english), "{english}");
        }
    }

    #[test]
    fn the_four_original_kinds_keep_their_field_names() {
        assert_eq!(
            names(kind("http_header")),
            vec!["header_name", "header_value"]
        );
        assert_eq!(names(kind("http_basic")), vec!["username", "password"]);
        assert_eq!(names(kind("bearer_token")), vec!["token"]);
        assert_eq!(
            names(kind("odoo_api")),
            vec!["base_url", "database", "username", "api_key"]
        );
    }

    #[test]
    fn only_the_documented_fields_are_secret() {
        let expected: &[(&str, &[&str])] = &[
            ("http_header", &["header_value"]),
            ("http_basic", &["password"]),
            ("bearer_token", &["token"]),
            ("odoo_api", &["api_key"]),
            ("api_key_query", &["param_value"]),
            ("digest", &["password"]),
            ("oauth2_authorization_code", &["client_secret"]),
            ("oauth2_client_credentials", &["client_secret"]),
            ("oauth1", &["consumer_secret"]),
            ("jwt", &["key"]),
            ("mtls", &["private_key", "passphrase"]),
            ("custom_auth", &["json"]),
            ("hmac", &["secret"]),
            ("smtp", &["password"]),
            ("imap", &["password"]),
            ("ssh", &["password", "private_key", "passphrase"]),
            ("ftp", &["password"]),
            ("sql", &["password"]),
        ];

        for (code, secrets) in expected {
            assert_eq!(kind(code).secret_names(), *secrets, "kind `{code}`");
        }
    }

    #[test]
    fn a_secret_field_is_never_a_select_or_a_boolean() {
        for kind in credential_kinds() {
            for field in kind.fields.iter().filter(|field| field.secret) {
                assert!(
                    !matches!(field.kind, Select | Boolean),
                    "{}.{}",
                    kind.kind,
                    field.name
                );
            }
        }
    }

    #[test]
    fn a_select_field_lists_its_options_and_its_default_is_one_of_them() {
        for kind in credential_kinds() {
            for field in kind.fields {
                if field.kind == Select {
                    assert!(!field.options.is_empty(), "{}.{}", kind.kind, field.name);
                    if let Some(default) = field.default {
                        assert!(
                            field.options.iter().any(|option| option.value == default),
                            "{}.{}",
                            kind.kind,
                            field.name
                        );
                    }
                } else {
                    assert!(field.options.is_empty(), "{}.{}", kind.kind, field.name);
                }
            }
        }
    }

    #[test]
    fn a_conditional_field_depends_on_a_select_of_the_same_kind() {
        for kind in credential_kinds() {
            for field in kind.fields {
                let Some(rule) = field.show_when else {
                    continue;
                };
                let controller = kind
                    .field(rule.field)
                    .unwrap_or_else(|| panic!("{}.{} depends on nothing", kind.kind, field.name));
                assert_eq!(controller.kind, Select);
                for value in rule.one_of {
                    assert!(
                        controller
                            .options
                            .iter()
                            .any(|option| option.value == *value),
                        "{}.{} waits for an option that does not exist",
                        kind.kind,
                        field.name
                    );
                }
            }
        }
    }

    #[test]
    fn ssh_shows_the_password_or_the_key_by_method() {
        let ssh = kind("ssh");

        assert_eq!(
            ssh.field("password").unwrap().show_when,
            Some(ShowWhen {
                field: "method",
                one_of: &["password"]
            })
        );
        assert_eq!(
            ssh.field("private_key").unwrap().show_when,
            Some(ShowWhen {
                field: "method",
                one_of: &["private-key"]
            })
        );
        assert_eq!(ssh.field("method").unwrap().default, Some("private-key"));
    }

    #[test]
    fn the_prototype_defaults_placeholders_and_hints_are_carried() {
        assert_eq!(
            kind("http_header").field("header_name").unwrap().default,
            Some("X-API-Key")
        );
        assert_eq!(
            kind("api_key_query").field("param_name").unwrap().default,
            Some("api_key")
        );
        assert_eq!(
            kind("hmac").field("header_name").unwrap().default,
            Some("X-Signature")
        );
        assert_eq!(kind("smtp").field("port").unwrap().default, Some("465"));
        assert_eq!(kind("imap").field("port").unwrap().default, Some("993"));
        assert_eq!(kind("ssh").field("port").unwrap().default, Some("22"));
        assert_eq!(kind("sql").field("port").unwrap().default, Some("5432"));
        assert_eq!(
            kind("smtp").field("host").unwrap().placeholder,
            Some("smtp.exemple.fr")
        );
        assert_eq!(
            kind("oauth2_client_credentials")
                .field("scopes")
                .unwrap()
                .hint,
            Some("Séparés par des espaces.")
        );
        assert_eq!(
            kind("mtls").field("ca_certificate").unwrap().hint,
            Some("Seulement si le serveur utilise une autorité privée.")
        );
    }

    #[test]
    fn the_categories_follow_the_prototype() {
        let category = |code: &str| kind(code).category.code();

        assert_eq!(category("http_basic"), "http");
        assert_eq!(category("oauth2_authorization_code"), "http");
        assert_eq!(category("hmac"), "webhook");
        assert_eq!(category("smtp"), "email");
        assert_eq!(category("imap"), "email");
        assert_eq!(category("ssh"), "servers");
        assert_eq!(category("sql"), "servers");
        assert_eq!(category("odoo_api"), "other");
        assert_eq!(Http.label(), "Requêtes HTTP");
        assert_eq!(Webhook.label(), "Webhooks entrants");
        assert_eq!(Email.label(), "E-mail");
        assert_eq!(Servers.label(), "Serveurs et bases de données");
    }

    #[test]
    fn only_the_kinds_without_an_executor_are_marked_so() {
        let not_executable: Vec<&str> = credential_kinds()
            .iter()
            .filter(|kind| !kind.executable)
            .map(|kind| kind.kind)
            .collect();

        assert_eq!(not_executable, vec!["imap", "ssh", "ftp", "sql"]);
        assert!(kind("smtp").executable);
    }

    #[test]
    fn only_the_oauth_kinds_with_a_dance_can_be_connected() {
        let connectable: Vec<&str> = credential_kinds()
            .iter()
            .filter(|kind| kind.connectable)
            .map(|kind| kind.kind)
            .collect();

        assert_eq!(connectable, vec!["oauth2_authorization_code", "oauth1"]);
    }

    #[test]
    fn the_usages_say_which_nodes_exist() {
        assert!(CredentialUsage::Http.has_node());
        assert!(CredentialUsage::Webhook.has_node());
        assert!(CredentialUsage::Smtp.has_node());
        assert!(!CredentialUsage::Imap.has_node());
        assert_eq!(CredentialUsage::Smtp.label(), "E-mail SMTP");
        assert_eq!(CredentialUsage::Imap.label(), "Lecture d’e-mails");
    }

    #[test]
    fn the_webhook_kinds_are_the_ones_the_kinds_say_they_serve() {
        for code in WEBHOOK_CREDENTIAL_KINDS {
            assert!(kind(code).accepts_usage(CredentialUsage::Webhook), "{code}");
        }
        let serving: Vec<&str> = credential_kinds()
            .iter()
            .filter(|kind| kind.accepts_usage(CredentialUsage::Webhook))
            .map(|kind| kind.kind)
            .collect();
        assert_eq!(serving.len(), WEBHOOK_CREDENTIAL_KINDS.len());
    }

    #[test]
    fn the_http_kinds_are_the_ones_the_kinds_say_they_serve() {
        for code in HTTP_CREDENTIAL_KINDS {
            assert!(kind(code).accepts_usage(CredentialUsage::Http), "{code}");
        }
        let serving: Vec<&str> = credential_kinds()
            .iter()
            .filter(|kind| kind.accepts_usage(CredentialUsage::Http))
            .map(|kind| kind.kind)
            .collect();
        assert_eq!(serving.len(), HTTP_CREDENTIAL_KINDS.len());
    }

    #[test]
    fn a_generated_origin_is_only_offered_for_a_bearer_token() {
        let offering: Vec<&str> = credential_kinds()
            .iter()
            .filter(|kind| !kind.origin_choices.is_empty())
            .map(|kind| kind.kind)
            .collect();

        assert_eq!(offering, vec!["bearer_token"]);
    }

    #[test]
    fn an_unknown_kind_reads_back_as_none() {
        assert!(credential_kind("not_a_real_kind").is_none());
    }
}
