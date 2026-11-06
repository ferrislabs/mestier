use reqwest::Url;

pub fn normalized_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if trimmed.contains(['{', '}']) {
        return Err("the base url cannot hold an unresolved server variable".to_owned());
    }
    let url =
        Url::parse(trimmed).map_err(|error| format!("the base url is not a valid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("the base url must use http or https".to_owned());
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err("the base url needs a host".to_owned());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("the base url cannot carry credentials".to_owned());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("the base url cannot carry a query or a fragment".to_owned());
    }
    Ok(trimmed.to_owned())
}

pub fn cleaned_source_url(raw: &str) -> Result<String, String> {
    let mut url = Url::parse(raw.trim())
        .map_err(|error| format!("the source url is not a valid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("the source url must use http or https".to_owned());
    }
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}
