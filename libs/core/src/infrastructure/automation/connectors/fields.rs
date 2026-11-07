use serde_json::{Map, Value};
use uuid::Uuid;

pub(super) fn text<'a>(config: &'a Map<String, Value>, name: &str) -> Result<&'a str, String> {
    config
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("`{name}` must resolve to a string"))
}

pub(super) fn filled<'a>(
    config: &'a Map<String, Value>,
    name: &str,
    max_chars: usize,
) -> Result<&'a str, String> {
    let value = text(config, name)?;
    if value.trim().is_empty() {
        return Err(format!("`{name}` must not be blank"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("`{name}` holds at most {max_chars} characters"));
    }
    Ok(value)
}

pub(super) fn uuid(config: &Map<String, Value>, name: &str) -> Result<Uuid, String> {
    Uuid::parse_str(text(config, name)?.trim()).map_err(|_| format!("`{name}` must be a UUID"))
}

pub(super) fn optional_text<'a>(
    config: &'a Map<String, Value>,
    name: &str,
) -> Result<Option<&'a str>, String> {
    match config.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if value.trim().is_empty() => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.trim())),
        Some(_) => Err(format!("`{name}` must resolve to a string")),
    }
}

pub(super) fn optional_whole_number(
    config: &Map<String, Value>,
    name: &str,
) -> Result<Option<u64>, String> {
    let invalid = || format!("`{name}` must be a whole number");
    match config.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(number)) => number.as_u64().map(Some).ok_or_else(invalid),
        Some(Value::String(value)) if value.trim().is_empty() => Ok(None),
        Some(Value::String(value)) => value.trim().parse().map(Some).map_err(|_| invalid()),
        Some(_) => Err(invalid()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn config(name: &str, value: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert(name.to_owned(), value);
        config
    }

    #[test]
    fn a_text_must_be_a_string() {
        assert_eq!(text(&config("a", json!("x")), "a"), Ok("x"));
        assert_eq!(
            text(&config("a", json!(3)), "a"),
            Err("`a` must resolve to a string".to_owned())
        );
        assert_eq!(
            text(&Map::new(), "a"),
            Err("`a` must resolve to a string".to_owned())
        );
    }

    #[test]
    fn a_filled_text_is_neither_blank_nor_too_long() {
        assert_eq!(filled(&config("a", json!("ok")), "a", 2), Ok("ok"));
        assert_eq!(
            filled(&config("a", json!("  ")), "a", 10),
            Err("`a` must not be blank".to_owned())
        );
        assert_eq!(
            filled(&config("a", json!("abc")), "a", 2),
            Err("`a` holds at most 2 characters".to_owned())
        );
    }

    #[test]
    fn a_uuid_is_parsed_without_echoing_the_value() {
        let id = Uuid::from_u128(7);
        assert_eq!(uuid(&config("a", json!(id.to_string())), "a"), Ok(id));
        let error = uuid(&config("a", json!("do-not-leak")), "a").unwrap_err();
        assert_eq!(error, "`a` must be a UUID");
    }

    #[test]
    fn an_optional_text_is_none_when_absent_null_or_blank() {
        for value in [json!(null), json!(""), json!("  ")] {
            assert_eq!(optional_text(&config("a", value), "a"), Ok(None));
        }
        assert_eq!(optional_text(&Map::new(), "a"), Ok(None));
        assert_eq!(
            optional_text(&config("a", json!(" x ")), "a"),
            Ok(Some("x"))
        );
        assert!(optional_text(&config("a", json!(3)), "a").is_err());
    }

    #[test]
    fn an_optional_whole_number_reads_a_number_or_a_numeric_string() {
        for value in [json!(null), json!(""), json!(" ")] {
            assert_eq!(optional_whole_number(&config("a", value), "a"), Ok(None));
        }
        assert_eq!(optional_whole_number(&Map::new(), "a"), Ok(None));
        assert_eq!(
            optional_whole_number(&config("a", json!(7)), "a"),
            Ok(Some(7))
        );
        assert_eq!(
            optional_whole_number(&config("a", json!(" 7 ")), "a"),
            Ok(Some(7))
        );
        for refused in [json!(-1), json!(1.5), json!("x"), json!([1]), json!(true)] {
            assert_eq!(
                optional_whole_number(&config("a", refused), "a"),
                Err("`a` must be a whole number".to_owned())
            );
        }
    }
}
