use serde::{Deserialize, Serialize};

use reqwest::blocking::Client;
use std::{
    fs,
    io::{ErrorKind, Write},
    time::Duration,
};
use tempfile::NamedTempFile;
use url::Url;

use crate::error::{FlasherError, Result};

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/ialopezg/feros-targets/main/repository.toml";

#[derive(Debug, Deserialize)]
pub struct Registry {
    pub schema_version: u32,
    pub repositories: Vec<Remote>,
}

#[derive(Debug, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub published_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Local {
    pub name: String,
    pub url: String,
    pub official: bool,
    pub is_default: bool,
    pub published_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LocalStore {
    repositories: Vec<Local>,
}

pub(super) fn save_local(repository: &Local) -> Result<()> {
    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| FlasherError::message("cannot locate executable directory"))?
        .join("config");

    fs::create_dir_all(&directory)?;
    let path = directory.join("repositories.toml");

    let mut store: LocalStore = match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).map_err(|error| {
            FlasherError::message(format!("invalid local repository store: {error}"))
        })?,
        Err(error) if error.kind() == ErrorKind::NotFound => LocalStore {
            repositories: Vec::new(),
        },
        Err(error) => return Err(error.into()),
    };

    for existing in &store.repositories {
        if existing.url == repository.url || existing.name.eq_ignore_ascii_case(&repository.name) {
            return Err(FlasherError::message(
                "repository URL or name is already registered",
            ));
        }
    }

    if repository.is_default {
        for existing in &mut store.repositories {
            existing.is_default = false;
        }
    }

    store.repositories.push(Local {
        name: repository.name.clone(),
        url: repository.url.clone(),
        official: repository.official,
        is_default: repository.is_default,
        published_date: repository.published_date.clone(),
    });

    let content = toml::to_string_pretty(&store).map_err(|error| {
        FlasherError::message(format!("cannot serialize repositories: {error}"))
    })?;

    let mut temporary = NamedTempFile::new_in(&directory)?;
    temporary.write_all(content.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(&path)
        .map_err(|error| FlasherError::message(format!("cannot save repositories: {error}")))?;

    Ok(())
}

pub(super) fn load_local() -> Result<Vec<Local>> {
    let executable = std::env::current_exe()?;
    let path = executable
        .parent()
        .ok_or_else(|| FlasherError::message("cannot locate executable directory"))?
        .join("config")
        .join("repositories.toml");

    match fs::read_to_string(path) {
        Ok(content) => {
            let store: LocalStore = toml::from_str(&content).map_err(|error| {
                FlasherError::message(format!("invalid local repository store: {error}"))
            })?;

            Ok(store.repositories)
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn set_default(number: usize) -> Result<String> {
    let mut repositories = load_local()?;

    if number == 0 || number > repositories.len() {
        return Err(FlasherError::message(
            "repository number is out of range; run \
             'flasher channel --list' and select the desired repository",
        ));
    }

    let selected = number - 1;

    for (index, repository) in repositories.iter_mut().enumerate() {
        repository.is_default = index == selected;
    }

    let name = repositories[selected].name.clone();
    let store = LocalStore { repositories };

    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| FlasherError::message("cannot locate executable directory"))?
        .join("config");

    let path = directory.join("repositories.toml");

    let content = toml::to_string_pretty(&store).map_err(|error| {
        FlasherError::message(format!("cannot serialize repositories: {error}"))
    })?;

    let mut temporary = NamedTempFile::new_in(&directory)?;
    temporary.write_all(content.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(&path)
        .map_err(|error| FlasherError::message(format!("cannot save repositories: {error}")))?;

    Ok(name)
}

pub fn find_official(repo_url: &str) -> Result<Option<Remote>> {
    let candidate =
        Url::parse(repo_url.trim()).map_err(|_| FlasherError::message("invalid repository URL"))?;

    let registry = load_registry()?;

    for entry in registry.repositories {
        let registered = Url::parse(&entry.url)
            .map_err(|_| FlasherError::message("invalid URL in official repository registry"))?;

        if registered == candidate {
            return Ok(Some(entry));
        }
    }

    Ok(None)
}

fn load_registry() -> Result<Registry> {
    let client = Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| FlasherError::message(format!("cannot create HTTP client: {error}")))?;

    let response = client
        .get(REGISTRY_URL)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|error| FlasherError::message(format!("cannot download registry: {error}")))?;

    let content = response
        .text()
        .map_err(|error| FlasherError::message(format!("cannot read registry: {error}")))?;

    let registry: Registry = toml::from_str(&content)
        .map_err(|error| FlasherError::message(format!("invalid registry TOML: {error}")))?;

    if registry.schema_version != 1 {
        return Err(FlasherError::message(
            "unsupported repository registry version",
        ));
    }

    Ok(registry)
}

pub(super) fn delete_local(number: usize) -> Result<String> {
    let mut repositories = load_local()?;

    if number == 0 || number > repositories.len() {
        return Err(FlasherError::message(
            "repository number is out of range; run \
             'flasher channel --list' and select the repository to delete",
        ));
    }

    let index = number - 1;

    if repositories[index].is_default {
        return Err(FlasherError::message(
            "cannot delete the default repository; select another one first",
        ));
    }

    let removed = repositories.remove(index);
    let store = LocalStore { repositories };

    let executable = std::env::current_exe()?;
    let directory = executable
        .parent()
        .ok_or_else(|| FlasherError::message("cannot locate executable directory"))?
        .join("config");

    let path = directory.join("repositories.toml");

    let content = toml::to_string_pretty(&store).map_err(|error| {
        FlasherError::message(format!("cannot serialize repositories: {error}"))
    })?;

    let mut temporary = NamedTempFile::new_in(&directory)?;
    temporary.write_all(content.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(&path)
        .map_err(|error| FlasherError::message(format!("cannot save repositories: {error}")))?;

    Ok(removed.name)
}
