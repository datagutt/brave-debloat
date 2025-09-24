use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DebloaterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config file not found: {0}")]
    ConfigNotFound(String),
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum BraveVersion {
    Normal,
    Nightly,
}

#[derive(Parser, Debug)]
#[command(name = "brave-debloater")]
#[command(about = "A tool to generate Brave browser debloat configurations for different platforms")]
pub struct Args {
    #[arg(short, long, value_enum)]
    platform: Platform,
    
    #[arg(short, long, value_enum, default_value = "normal")]
    version: BraveVersion,
    
    #[arg(short, long, default_value = "configs/privacy-focused.json")]
    config: String,
    
    #[arg(short, long, default_value = "extensions.json")]
    extensions: String,
    
    #[arg(short, long, default_value = "output")]
    output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    String(String),
    Number(i32),
    StringArray(Vec<String>),
}

type Config = HashMap<String, ConfigValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Extension {
    id: String,
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExtensionsConfig {
    extensions: Vec<Extension>,
}

struct DebloaterGenerator {
    config: Config,
    extensions: Vec<Extension>,
    platform: Platform,
    version: BraveVersion,
    output_dir: String,
}

impl DebloaterGenerator {
    fn new(config: Config, extensions: Vec<Extension>, platform: Platform, version: BraveVersion, output_dir: String) -> Self {
        Self {
            config,
            extensions,
            platform,
            version,
            output_dir,
        }
    }

    fn generate(&self) -> Result<(), DebloaterError> {
        fs::create_dir_all(&self.output_dir)?;
        
        match self.platform {
            Platform::Windows => self.generate_windows_registry(),
            Platform::MacOS => self.generate_macos_script(),
            Platform::Linux => self.generate_linux_json(),
        }
    }

    fn get_brave_registry_path(&self) -> &str {
        match self.version {
            BraveVersion::Normal => "SOFTWARE\\Policies\\BraveSoftware\\Brave",
            BraveVersion::Nightly => "SOFTWARE\\Policies\\BraveSoftware\\Brave-Nightly",
        }
    }

    fn get_macos_bundle_id(&self) -> &str {
        match self.version {
            BraveVersion::Normal => "com.brave.Browser",
            BraveVersion::Nightly => "com.brave.Browser.nightly",
        }
    }

    fn get_linux_policy_path(&self) -> &str {
        match self.version {
            BraveVersion::Normal => "/etc/brave/policies/managed/brave.json",
            BraveVersion::Nightly => "/etc/brave-nightly/policies/managed/brave.json",
        }
    }

    fn generate_windows_registry(&self) -> Result<(), DebloaterError> {
        let mut content = String::new();
        content.push_str("Windows Registry Editor Version 5.00\n\n");
        
        let registry_path = self.get_brave_registry_path();
        content.push_str(&format!("[HKEY_LOCAL_MACHINE\\{}]\n", registry_path));
        
        for (key, value) in &self.config {
            if key == "ExtensionInstallForcelist" {
                continue; // Handle separately
            }
            
            let reg_value = match value {
                ConfigValue::Bool(b) => format!("\"{}\"=dword:{:08x}\n", key, if *b { 1 } else { 0 }),
                ConfigValue::String(s) => format!("\"{}\"=\"{}\"\n", key, s),
                ConfigValue::Number(n) => format!("\"{}\"=dword:{:08x}\n", key, n),
                ConfigValue::StringArray(_) => continue, // Skip arrays for main section
            };
            content.push_str(&reg_value);
        }
        
        // Handle ExtensionInstallForcelist
        if !self.extensions.is_empty() {
            content.push_str(&format!("\n[HKEY_LOCAL_MACHINE\\{}\\ExtensionInstallForcelist]\n", registry_path));
            for (i, ext) in self.extensions.iter().enumerate() {
                content.push_str(&format!("\"{}\"=\"{}\"\n", i + 1, ext.id));
            }
        }

        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat.reg",
            BraveVersion::Nightly => "brave_nightly_debloat.reg",
        };
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(output_path, content)?;
        
        Ok(())
    }

    fn generate_macos_script(&self) -> Result<(), DebloaterError> {
        let mut content = String::new();
        content.push_str("#!/bin/zsh\n");
        content.push_str("# Brave Browser Debloater Script for macOS\n");
        content.push_str("# This script configures Brave Browser policies using the 'defaults' command\n\n");
        
        let bundle_id = self.get_macos_bundle_id();
        
        for (key, value) in &self.config {
            if key == "ExtensionInstallForcelist" || key == "ReportAppInventory" || key == "ReportWebsiteTelemetry" {
                continue; // Skip arrays for defaults commands
            }
            
            let defaults_cmd = match value {
                ConfigValue::Bool(b) => format!("defaults write {} {} -bool {}\n", bundle_id, key, b),
                ConfigValue::String(s) => format!("defaults write {} {} -string \"{}\"\n", bundle_id, key, s),
                ConfigValue::Number(n) => format!("defaults write {} {} -int {}\n", bundle_id, key, n),
                ConfigValue::StringArray(_) => continue,
            };
            content.push_str(&defaults_cmd);
        }
        
        content.push_str("\n# Restart cfprefsd to apply changes immediately\n");
        content.push_str("sudo killall cfprefsd\n\n");
        
        // Create managed preferences plist
        content.push_str("# Create managed preferences directory and plist file (overwrite if exists)\n");
        content.push_str("sudo mkdir -p /Library/Managed\\ Preferences\n");
        content.push_str(&format!("cat << 'EOF' | sudo tee /Library/Managed\\ Preferences/{}.plist > /dev/null\n", bundle_id));
        content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        content.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
        content.push_str("<plist version=\"1.0\">\n<dict>\n");
        
        for (key, value) in &self.config {
            if key == "ExtensionInstallForcelist" || key == "ReportAppInventory" || key == "ReportWebsiteTelemetry" {
                continue;
            }
            
            content.push_str(&format!("    <key>{}</key>\n", key));
            match value {
                ConfigValue::Bool(b) => content.push_str(&format!("    <{}/>", if *b { "true" } else { "false" })),
                ConfigValue::String(s) => content.push_str(&format!("    <string>{}</string>", s)),
                ConfigValue::Number(n) => content.push_str(&format!("    <integer>{}</integer>", n)),
                ConfigValue::StringArray(_) => continue,
            }
            content.push_str("\n");
        }
        
        content.push_str("</dict>\n</plist>\nEOF\n\n");
        content.push_str(&format!("sudo chmod 644 /Library/Managed\\ Preferences/{}.plist\n", bundle_id));
        
        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat_macos.sh",
            BraveVersion::Nightly => "brave_nightly_debloat_macos.sh",
        };
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(output_path, content)?;
        
        Ok(())
    }

    fn generate_linux_json(&self) -> Result<(), DebloaterError> {
        let mut final_config = self.config.clone();
        
        // Add ExtensionInstallForcelist if extensions exist
        if !self.extensions.is_empty() {
            let extension_ids: Vec<String> = self.extensions.iter().map(|e| e.id.clone()).collect();
            final_config.insert("ExtensionInstallForcelist".to_string(), ConfigValue::StringArray(extension_ids));
        }
        
        let config_json = serde_json::to_string_pretty(&final_config)?;
        
        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat_linux.json",
            BraveVersion::Nightly => "brave_nightly_debloat_linux.json",
        };
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(&output_path, config_json)?;
        
        // Create installation instructions
        let policy_path = self.get_linux_policy_path();
        let instructions = format!(
            "# Linux Installation Instructions\n\
            # Copy this JSON file to: {}\n\
            # Create the directory if it doesn't exist:\n\
            sudo mkdir -p $(dirname {})\n\
            sudo cp {} {}\n\
            sudo chmod 644 {}\n",
            policy_path,
            policy_path,
            output_path.display(),
            policy_path,
            policy_path
        );
        
        let instructions_file = match self.version {
            BraveVersion::Normal => "brave_debloat_linux_install.txt",
            BraveVersion::Nightly => "brave_nightly_debloat_linux_install.txt",
        };
        
        let instructions_path = Path::new(&self.output_dir).join(instructions_file);
        fs::write(instructions_path, instructions)?;
        
        Ok(())
    }
}

fn load_config(config_path: &str) -> Result<Config, DebloaterError> {
    if !Path::new(config_path).exists() {
        return Err(DebloaterError::ConfigNotFound(config_path.to_string()));
    }
    
    let content = fs::read_to_string(config_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

fn load_extensions(extensions_path: &str) -> Result<Vec<Extension>, DebloaterError> {
    if !Path::new(extensions_path).exists() {
        return Err(DebloaterError::ConfigNotFound(extensions_path.to_string()));
    }
    
    let content = fs::read_to_string(extensions_path)?;
    let extensions_config: ExtensionsConfig = serde_json::from_str(&content)?;
    Ok(extensions_config.extensions)
}

fn main() -> Result<(), DebloaterError> {
    let args = Args::parse();
    
    println!("Loading configuration from: {}", args.config);
    let config = load_config(&args.config)?;
    
    println!("Loading extensions from: {}", args.extensions);
    let extensions = load_extensions(&args.extensions)?;
    
    let extension_names: Vec<String> = extensions.iter().map(|e| e.name.clone()).collect();
    println!("Loaded {} extensions: {}", 
             extensions.len(),
             extension_names.join(", "));
    
    println!("Generating {} configuration for Brave {:?}...", 
             match args.platform {
                 Platform::Windows => "Windows Registry",
                 Platform::MacOS => "macOS Script",
                 Platform::Linux => "Linux JSON",
             },
             args.version);
    
    let output_dir = args.output.clone();
    let generator = DebloaterGenerator::new(config, extensions, args.platform, args.version, args.output);
    generator.generate()?;
    
    println!("Configuration files generated successfully in: {}", output_dir);
    
    Ok(())
}