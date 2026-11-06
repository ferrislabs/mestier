use serde_json::{Map, Value};
use thiserror::Error;

pub const SMTP_SEND_KIND: &str = "smtp.send";
pub const MAX_RECIPIENTS: usize = 50;
pub const MAX_SUBJECT_CHARS: usize = 255;
pub const MAX_BODY_BYTES: usize = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SmtpFault {
    #[error("`{0}` must resolve to a string or a list of strings")]
    NotText(&'static str),
    #[error("`{0}` must not contain a line break")]
    LineBreak(&'static str),
    #[error("`to` names no recipient")]
    NoRecipient,
    #[error("an e-mail goes to at most {MAX_RECIPIENTS} recipients, `to` and `cc` together")]
    TooManyRecipients,
    #[error("`subject` must not be blank")]
    BlankSubject,
    #[error("`subject` holds at most {MAX_SUBJECT_CHARS} characters")]
    SubjectTooLong,
    #[error("`body` must not be blank")]
    BlankBody,
    #[error("`body` holds at most {MAX_BODY_BYTES} bytes")]
    BodyTooLarge,
    #[error("`format` must be text or html")]
    UnknownFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyFormat {
    Text,
    Html,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft<'a> {
    pub to: Vec<&'a str>,
    pub cc: Vec<&'a str>,
    pub subject: &'a str,
    pub body: &'a str,
    pub format: BodyFormat,
}

pub fn has_line_break(text: &str) -> bool {
    text.contains(['\r', '\n'])
}

fn addresses_in(list: &str) -> impl Iterator<Item = &str> {
    list.split([',', ';'])
        .map(str::trim)
        .filter(|address| !address.is_empty())
}

fn texts<'a>(value: &'a Value, field: &'static str) -> Result<Vec<&'a str>, SmtpFault> {
    match value {
        Value::String(text) => Ok(vec![text.as_str()]),
        Value::Array(items) => items
            .iter()
            .map(|item| item.as_str().ok_or(SmtpFault::NotText(field)))
            .collect(),
        _ => Err(SmtpFault::NotText(field)),
    }
}

fn recipients<'a>(
    config: &'a Map<String, Value>,
    field: &'static str,
) -> Result<Vec<&'a str>, SmtpFault> {
    let Some(value) = config.get(field).filter(|value| !value.is_null()) else {
        return Ok(Vec::new());
    };
    let lists = texts(value, field)?;
    if lists.iter().any(|list| has_line_break(list)) {
        return Err(SmtpFault::LineBreak(field));
    }
    Ok(lists.into_iter().flat_map(addresses_in).collect())
}

fn text<'a>(config: &'a Map<String, Value>, field: &'static str) -> Result<&'a str, SmtpFault> {
    config
        .get(field)
        .and_then(Value::as_str)
        .ok_or(SmtpFault::NotText(field))
}

fn format_of(config: &Map<String, Value>) -> Result<BodyFormat, SmtpFault> {
    match config.get("format") {
        None | Some(Value::Null) => Ok(BodyFormat::Text),
        Some(Value::String(format)) if format.trim().is_empty() => Ok(BodyFormat::Text),
        Some(Value::String(format)) if format == "text" => Ok(BodyFormat::Text),
        Some(Value::String(format)) if format == "html" => Ok(BodyFormat::Html),
        Some(_) => Err(SmtpFault::UnknownFormat),
    }
}

impl<'a> Draft<'a> {
    pub fn from_config(config: &'a Map<String, Value>) -> Result<Self, SmtpFault> {
        let to = recipients(config, "to")?;
        let cc = recipients(config, "cc")?;
        if to.is_empty() {
            return Err(SmtpFault::NoRecipient);
        }
        if to.len() + cc.len() > MAX_RECIPIENTS {
            return Err(SmtpFault::TooManyRecipients);
        }

        let subject = text(config, "subject")?;
        if has_line_break(subject) {
            return Err(SmtpFault::LineBreak("subject"));
        }
        if subject.trim().is_empty() {
            return Err(SmtpFault::BlankSubject);
        }
        if subject.chars().count() > MAX_SUBJECT_CHARS {
            return Err(SmtpFault::SubjectTooLong);
        }

        let body = text(config, "body")?;
        if body.trim().is_empty() {
            return Err(SmtpFault::BlankBody);
        }
        if body.len() > MAX_BODY_BYTES {
            return Err(SmtpFault::BodyTooLarge);
        }

        Ok(Self {
            to,
            cc,
            subject,
            body,
            format: format_of(config)?,
        })
    }

    pub fn recipients(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.to.iter().chain(self.cc.iter()).copied()
    }
}

fn literal<'a>(config: &'a Map<String, Value>, field: &str) -> Option<&'a str> {
    config
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.contains("{{"))
}

pub fn literal_faults(config: &Map<String, Value>) -> Vec<SmtpFault> {
    let mut faults = Vec::new();
    for field in ["to", "cc", "subject"] {
        if literal(config, field).is_some_and(has_line_break) {
            faults.push(SmtpFault::LineBreak(field));
        }
    }
    let counted: usize = ["to", "cc"]
        .iter()
        .filter_map(|field| literal(config, field))
        .map(|list| addresses_in(list).count())
        .sum();
    if counted > MAX_RECIPIENTS {
        faults.push(SmtpFault::TooManyRecipients);
    }
    faults
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn config(entries: &[(&str, Value)]) -> Map<String, Value> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect()
    }

    fn valid() -> Map<String, Value> {
        config(&[
            ("to", json!("claire@fontaine.fr")),
            ("subject", json!("Votre dossier")),
            ("body", json!("Bonjour")),
        ])
    }

    fn with(base: Map<String, Value>, field: &str, value: Value) -> Map<String, Value> {
        let mut base = base;
        base.insert(field.to_owned(), value);
        base
    }

    fn fault(config: &Map<String, Value>) -> SmtpFault {
        Draft::from_config(config).unwrap_err()
    }

    #[test]
    fn a_minimal_draft_is_a_text_mail_to_one_recipient() {
        let config = valid();

        let draft = Draft::from_config(&config).unwrap();

        assert_eq!(draft.to, ["claire@fontaine.fr"]);
        assert!(draft.cc.is_empty());
        assert_eq!(draft.format, BodyFormat::Text);
        assert_eq!(draft.subject, "Votre dossier");
    }

    #[test]
    fn recipients_come_from_a_string_split_on_commas_and_semicolons_or_from_a_list() {
        let from_string = with(valid(), "to", json!(" a@x.fr, b@x.fr ;c@x.fr,, "));
        let from_list = with(valid(), "to", json!(["a@x.fr", "b@x.fr, c@x.fr"]));

        assert_eq!(
            Draft::from_config(&from_string).unwrap().to,
            ["a@x.fr", "b@x.fr", "c@x.fr"]
        );
        assert_eq!(
            Draft::from_config(&from_list).unwrap().to,
            ["a@x.fr", "b@x.fr", "c@x.fr"]
        );
    }

    #[test]
    fn the_copy_recipients_count_with_the_main_ones() {
        let fifty = (0..50).map(|n| format!("u{n}@x.fr")).collect::<Vec<_>>();
        let within = with(valid(), "to", json!(fifty.join(",")));
        let over = with(within.clone(), "cc", json!("one-more@x.fr"));

        assert_eq!(Draft::from_config(&within).unwrap().to.len(), 50);
        assert_eq!(fault(&over), SmtpFault::TooManyRecipients);
    }

    #[test]
    fn a_line_break_in_an_address_or_the_subject_is_refused() {
        for field in ["to", "cc"] {
            for injected in [
                "a@x.fr\r\nBcc: spy@evil.fr",
                "a@x.fr\nBcc: spy@evil.fr",
                "a@x.fr\rX: y",
            ] {
                let config = with(valid(), field, json!(injected));
                let by_list = with(valid(), field, json!([injected]));

                assert_eq!(fault(&config), SmtpFault::LineBreak(field));
                assert!(matches!(fault(&by_list), SmtpFault::LineBreak(_)));
            }
        }
        let subject = with(valid(), "subject", json!("Hello\r\nBcc: spy@evil.fr"));
        assert_eq!(fault(&subject), SmtpFault::LineBreak("subject"));
    }

    #[test]
    fn the_message_never_echoes_the_value() {
        let config = with(valid(), "subject", json!("secret-token\nBcc: x"));

        let message = fault(&config).to_string();

        assert_eq!(message, "`subject` must not contain a line break");
    }

    #[test]
    fn a_draft_needs_a_recipient_a_subject_and_a_body() {
        assert_eq!(
            fault(&with(valid(), "to", json!(" , "))),
            SmtpFault::NoRecipient
        );
        let mut without_to = valid();
        without_to.remove("to");
        assert_eq!(fault(&without_to), SmtpFault::NoRecipient);
        assert_eq!(
            fault(&with(valid(), "subject", json!("  "))),
            SmtpFault::BlankSubject
        );
        assert_eq!(
            fault(&with(valid(), "body", json!(""))),
            SmtpFault::BlankBody
        );
        let mut without_body = valid();
        without_body.remove("body");
        assert_eq!(fault(&without_body), SmtpFault::NotText("body"));
        assert_eq!(
            fault(&with(valid(), "to", json!(3))),
            SmtpFault::NotText("to")
        );
        assert_eq!(
            fault(&with(valid(), "to", json!([1]))),
            SmtpFault::NotText("to")
        );
    }

    #[test]
    fn the_subject_and_the_body_are_bounded() {
        let subject_ok = with(valid(), "subject", json!("é".repeat(MAX_SUBJECT_CHARS)));
        let subject_long = with(valid(), "subject", json!("é".repeat(MAX_SUBJECT_CHARS + 1)));
        let body_ok = with(valid(), "body", json!("x".repeat(MAX_BODY_BYTES)));
        let body_big = with(valid(), "body", json!("x".repeat(MAX_BODY_BYTES + 1)));

        assert!(Draft::from_config(&subject_ok).is_ok());
        assert_eq!(fault(&subject_long), SmtpFault::SubjectTooLong);
        assert!(Draft::from_config(&body_ok).is_ok());
        assert_eq!(fault(&body_big), SmtpFault::BodyTooLarge);
    }

    #[test]
    fn the_format_is_text_or_html() {
        assert_eq!(
            Draft::from_config(&with(valid(), "format", json!("html")))
                .unwrap()
                .format,
            BodyFormat::Html
        );
        for text in [json!("text"), json!(""), json!(null)] {
            assert_eq!(
                Draft::from_config(&with(valid(), "format", text))
                    .unwrap()
                    .format,
                BodyFormat::Text
            );
        }
        assert_eq!(
            fault(&with(valid(), "format", json!("markdown"))),
            SmtpFault::UnknownFormat
        );
    }

    #[test]
    fn a_saved_graph_is_refused_for_a_literal_line_break_or_too_many_literal_recipients() {
        let injected = config(&[
            ("to", json!("a@x.fr\nBcc: b@x.fr")),
            ("cc", json!("c@x.fr\r")),
            ("subject", json!("s\n")),
        ]);
        let many = config(&[
            (
                "to",
                json!(
                    (0..30)
                        .map(|n| format!("u{n}@x.fr"))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ),
            (
                "cc",
                json!(
                    (0..21)
                        .map(|n| format!("v{n}@x.fr"))
                        .collect::<Vec<_>>()
                        .join(";")
                ),
            ),
        ]);

        assert_eq!(
            literal_faults(&injected),
            [
                SmtpFault::LineBreak("to"),
                SmtpFault::LineBreak("cc"),
                SmtpFault::LineBreak("subject"),
            ]
        );
        assert_eq!(literal_faults(&many), [SmtpFault::TooManyRecipients]);
        assert!(literal_faults(&valid()).is_empty());
    }

    #[test]
    fn a_value_with_an_expression_is_left_to_the_run() {
        let config = config(&[
            ("to", json!("{{ trigger.email }}\n")),
            ("subject", json!("{{ trigger.title }}\r\n")),
        ]);

        assert!(literal_faults(&config).is_empty());
    }
}
