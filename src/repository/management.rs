use crate::error::{FlasherError, Result};
use chrono::{SecondsFormat, Utc};
use url::Url;

use super::repository::{delete_local, find_official, load_local, save_local, set_default, Local};

pub const DEFAULT_CHANNEL: &str = "official";

pub fn current() -> Result<Option<Local>> {
    let repositories = load_local()?;

    Ok(repositories
        .into_iter()
        .find(|repository| repository.is_default))
}

pub fn add(uri: &str, name: Option<&str>, is_default: bool) -> Result<String> {
    let url = validate_url(uri)?;
    let repo_name = suggested_name(name, &url)?;

    let remote = find_official(url.as_str())?;
    let (is_official, published_date) = match remote {
        Some(entry) => (true, Some(entry.published_date)),
        None => (false, None),
    };

    let published_date = published_date
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));

    let repository = Local {
        name: repo_name,
        url: url.to_string(),
        official: is_official,
        is_default,
        published_date,
    };

    save_local(&repository)?;

    Ok(repository.name)
}

pub fn available() -> Result<Vec<Local>> {
    load_local()
}

pub fn select(number: usize) -> Result<String> {
    set_default(number)
}

pub fn delete(number: usize) -> Result<String> {
    delete_local(number)
}

fn suggested_name(name: Option<&str>, url: &Url) -> Result<String> {
    match name {
        Some(value) if !value.trim().is_empty() => Ok(value.trim().to_owned()),
        _ => {
            let domain = url.domain().ok_or_else(|| {
                FlasherError::message("repository URL must contain a domain name")
            })?;

            Ok(domain.to_owned())
        }
    }
}

fn validate_url(address: &str) -> Result<Url> {
    let url =
        Url::parse(address.trim()).map_err(|_| FlasherError::message("invalid repository URL"))?;

    if url.scheme() != "https" {
        return Err(FlasherError::message("repository URL must use HTTPS"));
    }

    if !url.username().is_empty() || url.password().is_some() {
        return Err(FlasherError::message(
            "repository URL must not contain credentials",
        ));
    }

    Ok(url)
}
