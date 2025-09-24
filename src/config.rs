use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::DebloaterError;
use crate::preferences::PreferencesInputConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    String(String),
    Number(i32),
    StringArray(Vec<String>),
}

pub type Config = HashMap<String, ConfigValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Extension {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtensionsConfig {
    pub extensions: Vec<Extension>,
}

pub fn load_config(config_path: &str) -> Result<Config, DebloaterError> {
    if !Path::new(config_path).exists() {
        return Err(DebloaterError::ConfigNotFound(config_path.to_string()));
    }
    
    let content = fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn load_extensions(extensions_path: &str) -> Result<Vec<Extension>, DebloaterError> {
    if !Path::new(extensions_path).exists() {
        return Err(DebloaterError::ConfigNotFound(extensions_path.to_string()));
    }
    
    let content = fs::read_to_string(extensions_path)?;
    let extensions_config: ExtensionsConfig = serde_json::from_str(&content)?;
    Ok(extensions_config.extensions)
}

pub fn load_preferences_config(preferences_path: &str) -> Result<Option<PreferencesInputConfig>, DebloaterError> {
    if !Path::new(preferences_path).exists() {
        return Ok(None); // Optional file
    }
    
    let content = fs::read_to_string(preferences_path)?;
    let prefs_config: PreferencesInputConfig = serde_json::from_str(&content)?;
    Ok(Some(prefs_config))
}