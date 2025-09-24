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
    
    #[arg(long, help = "Generate user preferences files")]
    preferences: bool,
    
    #[arg(long, default_value = "preferences.json", help = "Preferences configuration file")]
    preferences_config: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchProvider {
    keyword: String,
    name: String,
    search_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTabPage {
    show_clock: Option<bool>,
    show_background_image: Option<bool>,
    show_stats: Option<bool>,
    show_shortcuts: Option<bool>,
    show_branded_background_image: Option<bool>,
    show_cards: Option<bool>,
    show_search_widget: Option<bool>,
    show_brave_news: Option<bool>,
    show_together: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BraveStats {
    enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BraveToday {
    should_show_brave_today_widget: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BravePreferences {
    new_tab_page: Option<NewTabPage>,
    stats: Option<BraveStats>,
    today: Option<BraveToday>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserPreferences {
    enabled_labs_experiments: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPreferences {
    default_search_provider_data: Option<SearchProvider>,
    brave: Option<BravePreferences>,
    browser: Option<BrowserPreferences>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalState {
    browser: Option<BrowserPreferences>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreferencesInputConfig {
    search_engines: Vec<SearchProvider>,
    dashboard: NewTabPage,
    experimental_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreferencesConfig {
    preferences: Option<UserPreferences>,
    local_state: Option<LocalState>,
}

struct DebloaterGenerator {
    config: Config,
    extensions: Vec<Extension>,
    platform: Platform,
    version: BraveVersion,
    output_dir: String,
    generate_preferences: bool,
    preferences_config: Option<PreferencesInputConfig>,
}

impl DebloaterGenerator {
    fn new(config: Config, extensions: Vec<Extension>, platform: Platform, version: BraveVersion, output_dir: String, generate_preferences: bool, preferences_config: Option<PreferencesInputConfig>) -> Self {
        Self {
            config,
            extensions,
            platform,
            version,
            output_dir,
            generate_preferences,
            preferences_config,
        }
    }

    fn generate(&self) -> Result<(), DebloaterError> {
        fs::create_dir_all(&self.output_dir)?;
        
        match self.platform {
            Platform::Windows => self.generate_windows_registry(),
            Platform::MacOS => self.generate_macos_script(),
            Platform::Linux => self.generate_linux_json(),
        }?;
        
        if self.generate_preferences {
            self.generate_user_preferences()?;
        }
        
        Ok(())
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

    fn get_preferences_path(&self) -> Vec<String> {
        match self.platform {
            Platform::Windows => {
                let version_suffix = match self.version {
                    BraveVersion::Normal => "Brave-Browser",
                    BraveVersion::Nightly => "Brave-Browser-Nightly",
                };
                vec![
                    format!("%USERPROFILE%\\AppData\\Local\\BraveSoftware\\{}\\User Data\\Default\\Preferences", version_suffix),
                    format!("%USERPROFILE%\\AppData\\Local\\BraveSoftware\\{}\\User Data\\Local State", version_suffix),
                ]
            }
            Platform::MacOS => {
                let version_suffix = match self.version {
                    BraveVersion::Normal => "Brave-Browser",
                    BraveVersion::Nightly => "Brave-Browser-Nightly",
                };
                vec![
                    format!("~/Library/Application Support/BraveSoftware/{}/Default/Preferences", version_suffix),
                    format!("~/Library/Application Support/BraveSoftware/{}/Local State", version_suffix),
                ]
            }
            Platform::Linux => {
                let version_suffix = match self.version {
                    BraveVersion::Normal => "Brave-Browser",
                    BraveVersion::Nightly => "Brave-Browser-Nightly",
                };
                vec![
                    format!("~/.config/BraveSoftware/{}/Default/Preferences", version_suffix),
                    format!("~/.config/BraveSoftware/{}/Local State", version_suffix),
                ]
            }
        }
    }

    fn generate_user_preferences(&self) -> Result<(), DebloaterError> {
        let prefs_config = self.preferences_config.as_ref();
        
        // Use config if provided, otherwise use defaults
        let search_provider = prefs_config
            .and_then(|p| p.search_engines.first())
            .cloned()
            .unwrap_or_else(|| SearchProvider {
                keyword: "brave".to_string(),
                name: "Brave Search".to_string(),
                search_url: "https://search.brave.com/search?q={searchTerms}".to_string(),
            });

        let dashboard_config = prefs_config
            .map(|p| p.dashboard.clone())
            .unwrap_or_else(|| NewTabPage {
                show_clock: Some(true),
                show_background_image: Some(false),
                show_stats: Some(false),
                show_shortcuts: Some(false),
                show_branded_background_image: Some(false),
                show_cards: Some(false),
                show_search_widget: Some(false),
                show_brave_news: Some(false),
                show_together: Some(false),
            });

        let experimental_features = prefs_config
            .map(|p| p.experimental_features.clone())
            .unwrap_or_else(|| vec!["brave-adblock-experimental-list-default@1".to_string()]);

        // Generate user preferences for dashboard customization
        let preferences = UserPreferences {
            default_search_provider_data: Some(search_provider),
            brave: Some(BravePreferences {
                new_tab_page: Some(dashboard_config),
                stats: Some(BraveStats {
                    enabled: Some(false),
                }),
                today: Some(BraveToday {
                    should_show_brave_today_widget: Some(false),
                }),
            }),
            browser: None,
        };

        // Generate Local State for experimental features
        let local_state = LocalState {
            browser: Some(BrowserPreferences {
                enabled_labs_experiments: Some(experimental_features),
            }),
        };

        let prefs_config = PreferencesConfig {
            preferences: Some(preferences),
            local_state: Some(local_state),
        };

        let preferences_json = serde_json::to_string_pretty(&prefs_config)?;
        
        let filename = match self.version {
            BraveVersion::Normal => "brave_user_preferences.json",
            BraveVersion::Nightly => "brave_nightly_user_preferences.json",
        };
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(&output_path, preferences_json)?;

        // Generate installation instructions
        self.generate_preferences_instructions()?;

        Ok(())
    }

    fn generate_preferences_instructions(&self) -> Result<(), DebloaterError> {
        let paths = self.get_preferences_path();
        let prefs_path = &paths[0];
        let local_state_path = &paths[1];

        let filename = match self.version {
            BraveVersion::Normal => "brave_user_preferences.json",
            BraveVersion::Nightly => "brave_nightly_user_preferences.json",
        };

        let instructions = match self.platform {
            Platform::Windows => {
                format!(
                    "# User Preferences Installation Instructions for Windows\n\
                    # \n\
                    # IMPORTANT: Close Brave browser completely before applying these changes!\n\
                    # \n\
                    # 1. Navigate to your Brave user data directory:\n\
                    #    {}\n\
                    # \n\
                    # 2. If the Preferences file exists, back it up:\n\
                    #    copy \"Preferences\" \"Preferences.backup\"\n\
                    # \n\
                    # 3. Extract the 'preferences' section from {} and overwrite/merge with your Preferences file\n\
                    # \n\
                    # 4. For Local State, navigate to:\n\
                    #    {}\n\
                    # \n\
                    # 5. Extract the 'local_state' section and merge with your Local State file\n\
                    # \n\
                    # 6. Start Brave browser to see the changes\n\
                    # \n\
                    # Note: The JSON structure in {} shows the exact format needed.\n\
                    # You can manually merge these settings or use a JSON editor.",
                    prefs_path, filename, local_state_path, filename
                )
            }
            Platform::MacOS => {
                format!(
                    "#!/bin/bash\n\
                    # User Preferences Installation Instructions for macOS\n\
                    # \n\
                    # IMPORTANT: Close Brave browser completely before applying these changes!\n\
                    # \n\
                    echo \"Applying Brave user preferences...\"\n\
                    \n\
                    PREFS_PATH=\"{}\"\n\
                    LOCAL_STATE_PATH=\"{}\"\n\
                    \n\
                    # Create directories if they don't exist\n\
                    mkdir -p \"$(dirname \"$PREFS_PATH\")\"\n\
                    mkdir -p \"$(dirname \"$LOCAL_STATE_PATH\")\"\n\
                    \n\
                    # Backup existing files\n\
                    [ -f \"$PREFS_PATH\" ] && cp \"$PREFS_PATH\" \"$PREFS_PATH.backup\"\n\
                    [ -f \"$LOCAL_STATE_PATH\" ] && cp \"$LOCAL_STATE_PATH\" \"$LOCAL_STATE_PATH.backup\"\n\
                    \n\
                    echo \"Please manually merge the 'preferences' and 'local_state' sections from {}\"\n\
                    echo \"with your existing Preferences and Local State files.\"\n\
                    echo \"Preferences file: $PREFS_PATH\"\n\
                    echo \"Local State file: $LOCAL_STATE_PATH\"\n\
                    \n\
                    echo \"Configuration complete. Start Brave to see changes.\"",
                    prefs_path, local_state_path, filename
                )
            }
            Platform::Linux => {
                format!(
                    "#!/bin/bash\n\
                    # User Preferences Installation Instructions for Linux\n\
                    # \n\
                    # IMPORTANT: Close Brave browser completely before applying these changes!\n\
                    # \n\
                    echo \"Applying Brave user preferences...\"\n\
                    \n\
                    PREFS_PATH=\"{}\"\n\
                    LOCAL_STATE_PATH=\"{}\"\n\
                    \n\
                    # Create directories if they don't exist\n\
                    mkdir -p \"$(dirname \"$PREFS_PATH\")\"\n\
                    mkdir -p \"$(dirname \"$LOCAL_STATE_PATH\")\"\n\
                    \n\
                    # Backup existing files\n\
                    [ -f \"$PREFS_PATH\" ] && cp \"$PREFS_PATH\" \"$PREFS_PATH.backup\"\n\
                    [ -f \"$LOCAL_STATE_PATH\" ] && cp \"$LOCAL_STATE_PATH\" \"$LOCAL_STATE_PATH.backup\"\n\
                    \n\
                    echo \"Please manually merge the 'preferences' and 'local_state' sections from {}\"\n\
                    echo \"with your existing Preferences and Local State files using jq or a JSON editor.\"\n\
                    echo \"\"\n\
                    echo \"Example using jq (if installed):\"\n\
                    echo \"# For Preferences file:\"\n\
                    echo \"jq -s '.[0] * .[1].preferences' \\\"$PREFS_PATH\\\" {} > temp_prefs.json && mv temp_prefs.json \\\"$PREFS_PATH\\\"\"\n\
                    echo \"\"\n\
                    echo \"# For Local State file:\"\n\
                    echo \"jq -s '.[0] * .[1].local_state' \\\"$LOCAL_STATE_PATH\\\" {} > temp_local.json && mv temp_local.json \\\"$LOCAL_STATE_PATH\\\"\"\n\
                    echo \"\"\n\
                    echo \"Preferences file: $PREFS_PATH\"\n\
                    echo \"Local State file: $LOCAL_STATE_PATH\"\n\
                    echo \"\"\n\
                    echo \"Configuration complete. Start Brave to see changes.\"",
                    prefs_path, local_state_path, filename, filename, filename
                )
            }
        };

        let instructions_filename = match self.version {
            BraveVersion::Normal => match self.platform {
                Platform::Windows => "brave_user_preferences_install.bat",
                _ => "brave_user_preferences_install.sh",
            },
            BraveVersion::Nightly => match self.platform {
                Platform::Windows => "brave_nightly_user_preferences_install.bat",
                _ => "brave_nightly_user_preferences_install.sh",
            },
        };

        let instructions_path = Path::new(&self.output_dir).join(instructions_filename);
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

fn load_preferences_config(preferences_path: &str) -> Result<Option<PreferencesInputConfig>, DebloaterError> {
    if !Path::new(preferences_path).exists() {
        return Ok(None); // Optional file
    }
    
    let content = fs::read_to_string(preferences_path)?;
    let prefs_config: PreferencesInputConfig = serde_json::from_str(&content)?;
    Ok(Some(prefs_config))
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

    // Load preferences config if preferences generation is requested
    let preferences_config = if args.preferences {
        println!("Loading preferences from: {}", args.preferences_config);
        load_preferences_config(&args.preferences_config)?
    } else {
        None
    };

    if args.preferences {
        if preferences_config.is_some() {
            println!("Loaded preferences configuration");
        } else {
            println!("Using default preferences configuration");
        }
    }
    
    println!("Generating {} configuration for Brave {:?}...", 
             match args.platform {
                 Platform::Windows => "Windows Registry",
                 Platform::MacOS => "macOS Script",
                 Platform::Linux => "Linux JSON",
             },
             args.version);
    
    let output_dir = args.output.clone();
    let generator = DebloaterGenerator::new(config, extensions, args.platform, args.version, args.output, args.preferences, preferences_config);
    generator.generate()?;
    
    println!("Configuration files generated successfully in: {}", output_dir);
    
    Ok(())
}