use std::fs;

use crate::cli::{Platform, BraveVersion};
use crate::config::{Config, Extension};
use crate::error::DebloaterError;
use crate::platforms::{PlatformGenerator, windows::WindowsGenerator, macos::MacOSGenerator, linux::LinuxGenerator};
use crate::preferences::PreferencesInputConfig;

pub struct DebloaterGenerator {
    config: Config,
    extensions: Vec<Extension>,
    platform: Platform,
    version: BraveVersion,
    output_dir: String,
    preferences_config: Option<PreferencesInputConfig>,
}

impl DebloaterGenerator {
    pub fn new(
        config: Config,
        extensions: Vec<Extension>,
        platform: Platform,
        version: BraveVersion,
        output_dir: String,
        preferences_config: Option<PreferencesInputConfig>,
    ) -> Self {
        Self {
            config,
            extensions,
            platform,
            version,
            output_dir,
            preferences_config,
        }
    }

    pub fn generate(&self) -> Result<(), DebloaterError> {
        fs::create_dir_all(&self.output_dir)?;
        
        let generator: Box<dyn PlatformGenerator> = match self.platform {
            Platform::Windows => Box::new(WindowsGenerator),
            Platform::MacOS => Box::new(MacOSGenerator),
            Platform::Linux => Box::new(LinuxGenerator),
        };

        generator.generate_unified_script(
            &self.config,
            &self.extensions,
            &self.version,
            &self.output_dir,
            self.preferences_config.as_ref(),
        )
    }
}