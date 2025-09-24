pub mod windows;
pub mod macos;
pub mod linux;

use crate::cli::BraveVersion;
use crate::config::{Config, Extension};
use crate::error::DebloaterError;
use crate::preferences::PreferencesInputConfig;

pub trait PlatformGenerator {
    fn generate_unified_script(&self, config: &Config, extensions: &[Extension], version: &BraveVersion, output_dir: &str, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError>;
}

pub fn get_brave_registry_path(version: &BraveVersion) -> &'static str {
    match version {
        BraveVersion::Normal => "SOFTWARE\\Policies\\BraveSoftware\\Brave",
        BraveVersion::Nightly => "SOFTWARE\\Policies\\BraveSoftware\\Brave-Nightly",
    }
}

pub fn get_macos_bundle_id(version: &BraveVersion) -> &'static str {
    match version {
        BraveVersion::Normal => "com.brave.Browser",
        BraveVersion::Nightly => "com.brave.Browser.nightly",
    }
}

pub fn get_linux_policy_path(version: &BraveVersion) -> &'static str {
    match version {
        BraveVersion::Normal => "/etc/brave/policies/managed/brave.json",
        BraveVersion::Nightly => "/etc/brave-nightly/policies/managed/brave.json",
    }
}

pub fn get_version_suffix(version: &BraveVersion) -> &'static str {
    match version {
        BraveVersion::Normal => "Brave-Browser",
        BraveVersion::Nightly => "Brave-Browser-Nightly",
    }
}