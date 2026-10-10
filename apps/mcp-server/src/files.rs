use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Url, redirect::Policy};
use rmcp::schemars;
use serde::Deserialize;

use crate::api::Fault;

pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_BASE64_BYTES: usize = 5 * 1024 * 1024;
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);

pub const DEFAULT_HOSTS: [&str; 3] = ["oaiusercontent.com", "openai.com", "chatgpt.com"];

#[derive(Debug, Clone)]
pub struct FilePolicy {
    hosts: Vec<String>,
    insecure_hosts: Vec<String>,
}

impl Default for FilePolicy {
    fn default() -> Self {
        Self::new(DEFAULT_HOSTS.iter().map(|host| (*host).to_owned()))
    }
}

impl FilePolicy {
    pub fn new(hosts: impl IntoIterator<Item = String>) -> Self {
        Self {
            hosts: hosts
                .into_iter()
                .map(|host| host.trim().trim_start_matches('.').to_lowercase())
                .filter(|host| !host.is_empty())
                .collect(),
            insecure_hosts: Vec::new(),
        }
    }

    pub fn allow_http_for(mut self, host: impl Into<String>) -> Self {
        self.insecure_hosts.push(host.into().to_lowercase());
        self
    }

    fn host_allowed(&self, host: &str) -> bool {
        self.hosts
            .iter()
            .any(|suffix| host == suffix || host.ends_with(&format!(".{suffix}")))
    }

    fn check(&self, raw: &str) -> Result<Url, Fault> {
        let url = Url::parse(raw).map_err(|_| Fault::new("adresse du fichier invalide"))?;
        let host = url.host_str().unwrap_or_default().to_lowercase();
        let insecure_ok = url.scheme() == "http" && self.insecure_hosts.contains(&host);
        if url.scheme() != "https" && !insecure_ok {
            tracing::warn!(%host, scheme = url.scheme(), "file download refused: not https");
            return Err(Fault::new(
                "téléchargement refusé : l'adresse du fichier doit être en https",
            ));
        }
        if !self.host_allowed(&host) {
            tracing::warn!(%host, "file download refused: host not allowed");
            return Err(Fault::new(format!(
                "téléchargement refusé : l'hôte « {host} » n'est pas autorisé"
            )));
        }
        Ok(url)
    }
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct FileParam {
    #[schemars(description = "Temporary download URL of the attached file.")]
    pub download_url: String,
    #[schemars(description = "Identifier of the attached file.")]
    pub file_id: Option<String>,
    #[schemars(description = "MIME type declared by the client. Ignored: the type is detected.")]
    pub mime_type: Option<String>,
    #[schemars(description = "Original file name. Informational.")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Photo,
    Plan,
    SupplyPhoto,
    SupplyDatasheet,
    /// The DOE's cover photo, replacing any previous one.
    CoverPhoto,
}

impl Target {
    fn accepts_pdf(self) -> bool {
        matches!(self, Self::Plan | Self::SupplyDatasheet)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Detected {
    Jpeg,
    Png,
    Webp,
    Pdf,
}

impl Detected {
    pub fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
            Self::Pdf => "application/pdf",
        }
    }
}

pub fn sniff(bytes: &[u8]) -> Option<Detected> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(Detected::Jpeg)
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(Detected::Png)
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(Detected::Webp)
    } else if bytes.starts_with(b"%PDF-") {
        Some(Detected::Pdf)
    } else {
        None
    }
}

pub fn check_type(bytes: &[u8], target: Target) -> Result<Detected, Fault> {
    match sniff(bytes) {
        Some(Detected::Pdf) if !target.accepts_pdf() => Err(Fault::new(
            "fichier refusé : un PDF n'est accepté que pour un plan ou une fiche technique",
        )),
        Some(kind) => Ok(kind),
        None => Err(Fault::new(
            "fichier refusé : seuls JPEG, PNG et WebP sont acceptés (et PDF pour un plan ou une fiche technique)",
        )),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Source<'a> {
    Param(&'a FileParam),
    Base64(&'a str),
}

impl<'a> Source<'a> {
    pub fn pick(
        file: Option<&'a FileParam>,
        content_base64: Option<&'a str>,
    ) -> Result<Self, Fault> {
        match (file, content_base64) {
            (Some(file), None) => Ok(Self::Param(file)),
            (None, Some(content)) => Ok(Self::Base64(content)),
            (None, None) => Err(Fault::new(
                "aucun fichier reçu : joins le fichier à ta demande (paramètre `file`, ou `content_base64` pour un client sans pièce jointe)",
            )),
            (Some(_), Some(_)) => Err(Fault::new(
                "fournis soit `file`, soit `content_base64`, pas les deux",
            )),
        }
    }
}

#[derive(Clone)]
pub struct FileFetcher {
    http: reqwest::Client,
    policy: FilePolicy,
}

impl FileFetcher {
    pub fn new(policy: FilePolicy) -> Self {
        let http = reqwest::Client::builder()
            .redirect(Policy::none())
            .timeout(DOWNLOAD_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self { http, policy }
    }

    pub async fn acquire(&self, source: Source<'_>, target: Target) -> Result<Upload, Fault> {
        let bytes = match source {
            Source::Param(file) => self.download(&file.download_url).await?,
            Source::Base64(content) => decode_base64(content)?,
        };
        let kind = check_type(&bytes, target)?;
        Ok(Upload { bytes, kind })
    }

    async fn download(&self, raw: &str) -> Result<Vec<u8>, Fault> {
        let url = self.policy.check(raw)?;
        let mut response = self.http.get(url).send().await.map_err(|error| {
            tracing::warn!(%error, "file download failed");
            Fault::new("téléchargement du fichier impossible")
        })?;
        let status = response.status();
        if status.is_redirection() {
            return Err(Fault::new(
                "téléchargement refusé : l'adresse du fichier redirige ailleurs",
            ));
        }
        if !status.is_success() {
            return Err(Fault::new(format!(
                "téléchargement du fichier impossible (erreur {})",
                status.as_u16()
            )));
        }
        let too_big = || Fault::new("fichier trop volumineux (plus de 10 Mo)");
        if response
            .content_length()
            .is_some_and(|length| length > MAX_FILE_BYTES as u64)
        {
            return Err(too_big());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            tracing::warn!(%error, "file download interrupted");
            Fault::new("téléchargement du fichier interrompu")
        })? {
            if bytes.len() + chunk.len() > MAX_FILE_BYTES {
                return Err(too_big());
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
}

pub struct Upload {
    pub bytes: Vec<u8>,
    pub kind: Detected,
}

fn decode_base64(content: &str) -> Result<Vec<u8>, Fault> {
    let too_big = || Fault::new("fichier trop volumineux (plus de 5 Mo en base64)");
    if content.len() > MAX_BASE64_BYTES.div_ceil(3) * 4 + 4 {
        return Err(too_big());
    }
    let bytes = STANDARD
        .decode(content.trim())
        .map_err(|_| Fault::new("contenu base64 invalide"))?;
    if bytes.len() > MAX_BASE64_BYTES {
        return Err(too_big());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_are_recognised() {
        assert_eq!(sniff(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(Detected::Jpeg));
        assert_eq!(
            sniff(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0]),
            Some(Detected::Png)
        );
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(Detected::Webp));
        assert_eq!(sniff(b"%PDF-1.7"), Some(Detected::Pdf));
        assert_eq!(sniff(b"<html>"), None);
        assert_eq!(sniff(b""), None);
    }

    #[test]
    fn host_suffix_matching_needs_a_dot_boundary() {
        let policy = FilePolicy::default();
        assert!(policy.host_allowed("files.oaiusercontent.com"));
        assert!(policy.host_allowed("openai.com"));
        assert!(!policy.host_allowed("evilopenai.com"));
        assert!(!policy.host_allowed("openai.com.evil.test"));
    }

    #[test]
    fn a_pdf_is_not_a_photo() {
        assert!(check_type(b"%PDF-1.4", Target::Photo).is_err());
        assert!(check_type(b"%PDF-1.4", Target::SupplyPhoto).is_err());
        assert!(check_type(b"%PDF-1.4", Target::CoverPhoto).is_err());
        assert!(check_type(b"%PDF-1.4", Target::Plan).is_ok());
        assert!(check_type(b"%PDF-1.4", Target::SupplyDatasheet).is_ok());
    }
}
