use std::fs;
use std::path::Path;

use crate::finders::{find_bver_toml, find_cargo_toml, find_package_json, find_pyproject_toml};
use crate::schema::Config;

pub fn load_config() -> Option<Config> {
    load_from_bver_toml()
        .or_else(load_from_pyproject_toml)
        .or_else(load_from_package_json)
        .or_else(load_from_cargo_toml)
}

fn load_from_bver_toml() -> Option<Config> {
    let path = find_bver_toml()?;
    load_toml_config(&path)
}

fn load_from_pyproject_toml() -> Option<Config> {
    let path = find_pyproject_toml()?;
    let content = fs::read_to_string(&path).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    let bver_config = value.get("tool")?.get("bver")?;
    let mut config: Config = toml::Value::try_into(bver_config.clone()).ok()?;

    if config.current_version.is_none() {
        config.current_version = value
            .get("project")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .map(String::from);
    }

    Some(config)
}

fn load_from_package_json() -> Option<Config> {
    let path = find_package_json()?;
    let content = fs::read_to_string(&path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let bver_config = value.get("bver")?;
    let mut config: Config = serde_json::from_value(bver_config.clone()).ok()?;

    if config.current_version.is_none() {
        config.current_version = value
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);
    }

    Some(config)
}

fn load_from_cargo_toml() -> Option<Config> {
    let path = find_cargo_toml()?;
    let content = fs::read_to_string(&path).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    let bver_config = value.get("package")?.get("metadata")?.get("bver")?;
    let mut config: Config = toml::Value::try_into(bver_config.clone()).ok()?;

    if config.current_version.is_none() {
        config.current_version = value
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .map(String::from);
    }

    Some(config)
}

fn load_toml_config(path: &Path) -> Option<Config> {
    let content = fs::read_to_string(path).ok()?;
    toml::from_str(&content).ok()
}
