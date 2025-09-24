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

    preferences_config: Option<PreferencesInputConfig>,
}

impl DebloaterGenerator {
    fn new(config: Config, extensions: Vec<Extension>, platform: Platform, version: BraveVersion, output_dir: String, preferences_config: Option<PreferencesInputConfig>) -> Self {
        Self {
            config,
            extensions,
            platform,
            version,
            output_dir,
            preferences_config,
        }
    }

    fn generate(&self) -> Result<(), DebloaterError> {
        fs::create_dir_all(&self.output_dir)?;
        
        match self.platform {
            Platform::Windows => self.generate_unified_windows_script(),
            Platform::MacOS => self.generate_unified_macos_script(),
            Platform::Linux => self.generate_unified_linux_script(),
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

    fn generate_unified_windows_script(&self) -> Result<(), DebloaterError> {
        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat.bat",
            BraveVersion::Nightly => "brave_nightly_debloat.bat",
        };
        
        let version_suffix = match self.version {
            BraveVersion::Normal => "Brave-Browser",
            BraveVersion::Nightly => "Brave-Browser-Nightly",
        };
        
        let mut content = String::new();
        content.push_str("@echo off\n");
        content.push_str("setlocal enabledelayedexpansion\n");
        content.push_str("REM Unified Brave Browser Debloater Script for Windows\n");
        content.push_str("REM This script applies both policies and user preferences\n");
        content.push_str("REM Run as Administrator\n\n");
        
        content.push_str("echo Brave Browser Debloater for Windows\n");
        content.push_str("echo ====================================\n\n");
        
        content.push_str("REM Check for admin rights\n");
        content.push_str("net session >nul 2>&1\n");
        content.push_str("if %errorlevel% neq 0 (\n");
        content.push_str("    echo This script must be run as Administrator!\n");
        content.push_str("    pause\n");
        content.push_str("    exit /b 1\n");
        content.push_str(")\n\n");
        
        // Check if Brave is running
        content.push_str("echo Checking if Brave is running...\n");
        content.push_str("tasklist /fi \"imagename eq brave.exe\" 2>nul | find /i \"brave.exe\" >nul\n");
        content.push_str("if not errorlevel 1 (\n");
        content.push_str("    echo WARNING: Brave browser is running!\n");
        content.push_str("    echo Please close Brave browser before running this script.\n");
        content.push_str("    echo Press any key to continue anyway, or Ctrl+C to exit.\n");
        content.push_str("    pause >nul\n");
        content.push_str("    taskkill /f /im brave.exe >nul 2>&1\n");
        content.push_str("    timeout /t 2 >nul\n");
        content.push_str(")\n\n");
        
        // Generate registry entries
        content.push_str("echo Applying Brave policies via registry...\n");
        
        let registry_path = self.get_brave_registry_path();
        
        for (key, value) in &self.config {
            if key == "ExtensionInstallForcelist" {
                continue; // Handle separately
            }
            
            let reg_value = match value {
                ConfigValue::Bool(b) => format!("reg add \"HKEY_LOCAL_MACHINE\\{}\" /v \"{}\" /t REG_DWORD /d {} /f >nul 2>&1\n", registry_path, key, if *b { 1 } else { 0 }),
                ConfigValue::String(s) => format!("reg add \"HKEY_LOCAL_MACHINE\\{}\" /v \"{}\" /t REG_SZ /d \"{}\" /f >nul 2>&1\n", registry_path, key, s),
                ConfigValue::Number(n) => format!("reg add \"HKEY_LOCAL_MACHINE\\{}\" /v \"{}\" /t REG_DWORD /d {} /f >nul 2>&1\n", registry_path, key, n),
                ConfigValue::StringArray(_) => continue,
            };
            content.push_str(&reg_value);
        }
        
        // Handle ExtensionInstallForcelist
        if !self.extensions.is_empty() {
            for (i, ext) in self.extensions.iter().enumerate() {
                content.push_str(&format!("reg add \"HKEY_LOCAL_MACHINE\\{}\\ExtensionInstallForcelist\" /v \"{}\" /t REG_SZ /d \"{}\" /f >nul 2>&1\n", registry_path, i + 1, ext.id));
            }
        }
        
        content.push_str("echo Registry policies applied successfully!\n\n");
        
        // Add user preferences modification
        content.push_str("echo Modifying user preferences...\n");
        content.push_str(&format!("set \"BRAVE_DATA=%USERPROFILE%\\AppData\\Local\\BraveSoftware\\{}\\User Data\"\n", version_suffix));
        content.push_str("set \"PREFS_FILE=%BRAVE_DATA%\\Default\\Preferences\"\n");
        content.push_str("set \"LOCAL_STATE=%BRAVE_DATA%\\Local State\"\n\n");
        
        // Create directories if they don't exist
        content.push_str("if not exist \"%BRAVE_DATA%\\Default\" mkdir \"%BRAVE_DATA%\\Default\"\n\n");
        
        // Backup existing files
        content.push_str("if exist \"%PREFS_FILE%\" copy \"%PREFS_FILE%\" \"%PREFS_FILE%.backup\" >nul 2>&1\n");
        content.push_str("if exist \"%LOCAL_STATE%\" copy \"%LOCAL_STATE%\" \"%LOCAL_STATE%.backup\" >nul 2>&1\n\n");
        
        // Generate the user preferences modification using PowerShell
        self.add_windows_preferences_powershell(&mut content)?;
        
        content.push_str("echo Configuration complete!\n");
        content.push_str("echo Please restart Brave browser for changes to take effect.\n");
        content.push_str("pause\n");
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(output_path, content)?;
        
        Ok(())
    }

    fn generate_unified_macos_script(&self) -> Result<(), DebloaterError> {
        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat_macos.sh",
            BraveVersion::Nightly => "brave_nightly_debloat_macos.sh",
        };
        
        let version_suffix = match self.version {
            BraveVersion::Normal => "Brave-Browser",
            BraveVersion::Nightly => "Brave-Browser-Nightly",
        };
        
        let mut content = String::new();
        content.push_str("#!/bin/zsh\n");
        content.push_str("# Unified Brave Browser Debloater Script for macOS\n");
        content.push_str("# This script applies both policies and user preferences\n");
        content.push_str("# Run with sudo for system-wide changes\n\n");
        
        content.push_str("# Colors for output\n");
        content.push_str("RED='\\033[0;31m'\n");
        content.push_str("GREEN='\\033[0;32m'\n");
        content.push_str("YELLOW='\\033[1;33m'\n");
        content.push_str("NC='\\033[0m' # No Color\n\n");
        
        content.push_str("echo -e \"${GREEN}Brave Browser Debloater for macOS${NC}\"\n");
        content.push_str("echo -e \"${GREEN}====================================${NC}\"\n");
        content.push_str("echo\n\n");
        
        // Check for sudo if needed for policies
        content.push_str("if [ \"$EUID\" -ne 0 ]; then\n");
        content.push_str("    echo -e \"${YELLOW}Note: Running without sudo. System policies will be skipped.${NC}\"\n");
        content.push_str("    echo -e \"${YELLOW}Run with sudo for complete configuration.${NC}\"\n");
        content.push_str("    SKIP_POLICIES=1\n");
        content.push_str("else\n");
        content.push_str("    SKIP_POLICIES=0\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Check if Brave is running
        content.push_str("echo \"Checking if Brave is running...\"\n");
        content.push_str("if pgrep -f \"Brave Browser\" > /dev/null; then\n");
        content.push_str("    echo -e \"${YELLOW}WARNING: Brave browser is running!${NC}\"\n");
        content.push_str("    echo \"Please close Brave browser before running this script.\"\n");
        content.push_str("    echo \"Press Enter to continue anyway, or Ctrl+C to exit.\"\n");
        content.push_str("    read\n");
        content.push_str("    pkill -f \"Brave Browser\" 2>/dev/null\n");
        content.push_str("    sleep 2\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Apply system policies if running as root
        content.push_str("if [ \"$SKIP_POLICIES\" -eq 0 ]; then\n");
        content.push_str("    echo -e \"${GREEN}Applying system policies...${NC}\"\n");
        
        let bundle_id = self.get_macos_bundle_id();
        
        // Create managed preferences plist
        content.push_str("    mkdir -p /Library/Managed\\ Preferences\n");
        content.push_str(&format!("    cat << 'EOF' > /Library/Managed\\ Preferences/{}.plist\n", bundle_id));
        content.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        content.push_str("<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n");
        content.push_str("<plist version=\"1.0\">\n<dict>\n");
        
        for (key, value) in &self.config {
            if key == "ExtensionInstallForcelist" {
                // Handle extension list
                content.push_str("    <key>ExtensionInstallForcelist</key>\n");
                content.push_str("    <array>\n");
                for ext in &self.extensions {
                    content.push_str(&format!("        <string>{}</string>\n", ext.id));
                }
                content.push_str("    </array>\n");
                continue;
            }
            if key == "ReportAppInventory" || key == "ReportWebsiteTelemetry" {
                continue;
            }
            
            content.push_str(&format!("    <key>{}</key>\n", key));
            match value {
                ConfigValue::Bool(b) => content.push_str(&format!("    <{}/>", if *b { "true" } else { "false" })),
                ConfigValue::String(s) => content.push_str(&format!("    <string>{}</string>", s)),
                ConfigValue::Number(n) => content.push_str(&format!("    <integer>{}</integer>", n)),
                ConfigValue::StringArray(arr) => {
                    content.push_str("    <array>\n");
                    for item in arr {
                        content.push_str(&format!("        <string>{}</string>\n", item));
                    }
                    content.push_str("    </array>");
                }
            }
            content.push_str("\n");
        }
        
        content.push_str("</dict>\n</plist>\nEOF\n");
        content.push_str(&format!("    chmod 644 /Library/Managed\\ Preferences/{}.plist\n", bundle_id));
        content.push_str("    echo -e \"${GREEN}System policies applied successfully!${NC}\"\n");
        content.push_str("else\n");
        content.push_str("    echo -e \"${YELLOW}Skipping system policies (not running as sudo)${NC}\"\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Add user preferences modification
        self.add_macos_preferences_section(&mut content, version_suffix)?;
        
        content.push_str("echo -e \"${GREEN}Configuration complete!${NC}\"\n");
        content.push_str("echo -e \"${GREEN}Please restart Brave browser for changes to take effect.${NC}\"\n");
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(output_path, content)?;
        
        Ok(())
    }

    fn generate_unified_linux_script(&self) -> Result<(), DebloaterError> {
        let filename = match self.version {
            BraveVersion::Normal => "brave_debloat_linux.sh",
            BraveVersion::Nightly => "brave_nightly_debloat_linux.sh",
        };
        
        let version_suffix = match self.version {
            BraveVersion::Normal => "Brave-Browser",
            BraveVersion::Nightly => "Brave-Browser-Nightly",
        };
        
        let policy_path = self.get_linux_policy_path();
        
        let mut content = String::new();
        content.push_str("#!/bin/bash\n");
        content.push_str("# Unified Brave Browser Debloater Script for Linux\n");
        content.push_str("# This script applies both policies and user preferences\n");
        content.push_str("# Run with sudo for system-wide changes\n\n");
        
        content.push_str("# Colors for output\n");
        content.push_str("RED='\\033[0;31m'\n");
        content.push_str("GREEN='\\033[0;32m'\n");
        content.push_str("YELLOW='\\033[1;33m'\n");
        content.push_str("NC='\\033[0m' # No Color\n\n");
        
        content.push_str("echo -e \"${GREEN}Brave Browser Debloater for Linux${NC}\"\n");
        content.push_str("echo -e \"${GREEN}===================================${NC}\"\n");
        content.push_str("echo\n\n");
        
        // Check for sudo if needed for policies
        content.push_str("if [ \"$EUID\" -ne 0 ]; then\n");
        content.push_str("    echo -e \"${YELLOW}Note: Running without sudo. System policies will be skipped.${NC}\"\n");
        content.push_str("    echo -e \"${YELLOW}Run with sudo for complete configuration.${NC}\"\n");
        content.push_str("    SKIP_POLICIES=1\n");
        content.push_str("else\n");
        content.push_str("    SKIP_POLICIES=0\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Check if Brave is running
        content.push_str("echo \"Checking if Brave is running...\"\n");
        content.push_str("if pgrep -f brave > /dev/null; then\n");
        content.push_str("    echo -e \"${YELLOW}WARNING: Brave browser is running!${NC}\"\n");
        content.push_str("    echo \"Please close Brave browser before running this script.\"\n");
        content.push_str("    echo \"Press Enter to continue anyway, or Ctrl+C to exit.\"\n");
        content.push_str("    read\n");
        content.push_str("    pkill -f brave 2>/dev/null\n");
        content.push_str("    sleep 2\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Apply system policies if running as root
        content.push_str("if [ \"$SKIP_POLICIES\" -eq 0 ]; then\n");
        content.push_str("    echo -e \"${GREEN}Applying system policies...${NC}\"\n");
        content.push_str(&format!("    mkdir -p \"$(dirname '{}')\"\n", policy_path));
        
        // Create the JSON policy file
        content.push_str(&format!("    cat << 'EOF' > '{}'\n", policy_path));
        
        // Generate JSON content
        let mut final_config = self.config.clone();
        if !self.extensions.is_empty() {
            let extension_ids: Vec<String> = self.extensions.iter().map(|e| e.id.clone()).collect();
            final_config.insert("ExtensionInstallForcelist".to_string(), ConfigValue::StringArray(extension_ids));
        }
        let config_json = serde_json::to_string_pretty(&final_config)?;
        content.push_str(&config_json);
        content.push_str("\nEOF\n");
        
        content.push_str(&format!("    chmod 644 '{}'\n", policy_path));
        content.push_str("    echo -e \"${GREEN}System policies applied successfully!${NC}\"\n");
        content.push_str("else\n");
        content.push_str("    echo -e \"${YELLOW}Skipping system policies (not running as sudo)${NC}\"\n");
        content.push_str("fi\n");
        content.push_str("echo\n\n");
        
        // Add user preferences modification
        self.add_linux_preferences_section(&mut content, version_suffix)?;
        
        content.push_str("echo -e \"${GREEN}Configuration complete!${NC}\"\n");
        content.push_str("echo -e \"${GREEN}Please restart Brave browser for changes to take effect.${NC}\"\n");
        
        let output_path = Path::new(&self.output_dir).join(filename);
        fs::write(&output_path, content)?;
        
        // Make the script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_path, perms)?;
        }
        
        Ok(())
    }



    fn add_windows_preferences_powershell(&self, content: &mut String) -> Result<(), DebloaterError> {
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

        // Create PowerShell script embedded in batch
        content.push_str("echo Modifying Preferences file...\n");
        content.push_str("powershell -ExecutionPolicy Bypass -Command \"\n");
        content.push_str("$prefsPath = '%PREFS_FILE%';\n");
        content.push_str("$localStatePath = '%LOCAL_STATE%';\n");
        content.push_str("if (Test-Path $prefsPath) {\n");
        content.push_str("    $prefs = Get-Content $prefsPath -Raw | ConvertFrom-Json\n");
        content.push_str("} else {\n");
        content.push_str("    $prefs = @{}\n");
        content.push_str("}\n");
        
        // Add search provider
        content.push_str("if (-not $prefs.default_search_provider_data) { $prefs.default_search_provider_data = @{} }\n");
        content.push_str(&format!("$prefs.default_search_provider_data.keyword = '{}'\n", search_provider.keyword));
        content.push_str(&format!("$prefs.default_search_provider_data.name = '{}'\n", search_provider.name));
        content.push_str(&format!("$prefs.default_search_provider_data.search_url = '{}'\n", search_provider.search_url));
        
        // Add brave preferences
        content.push_str("if (-not $prefs.brave) { $prefs.brave = @{} }\n");
        content.push_str("if (-not $prefs.brave.new_tab_page) { $prefs.brave.new_tab_page = @{} }\n");
        content.push_str("if (-not $prefs.brave.stats) { $prefs.brave.stats = @{} }\n");
        content.push_str("if (-not $prefs.brave.today) { $prefs.brave.today = @{} }\n");
        
        // Dashboard settings
        if let Some(show_clock) = dashboard_config.show_clock {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_clock = ${}\n", show_clock.to_string().to_lowercase()));
        }
        if let Some(show_bg) = dashboard_config.show_background_image {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_background_image = ${}\n", show_bg.to_string().to_lowercase()));
        }
        if let Some(show_stats) = dashboard_config.show_stats {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_stats = ${}\n", show_stats.to_string().to_lowercase()));
        }
        if let Some(show_shortcuts) = dashboard_config.show_shortcuts {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_shortcuts = ${}\n", show_shortcuts.to_string().to_lowercase()));
        }
        if let Some(show_branded) = dashboard_config.show_branded_background_image {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_branded_background_image = ${}\n", show_branded.to_string().to_lowercase()));
        }
        if let Some(show_cards) = dashboard_config.show_cards {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_cards = ${}\n", show_cards.to_string().to_lowercase()));
        }
        if let Some(show_search) = dashboard_config.show_search_widget {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_search_widget = ${}\n", show_search.to_string().to_lowercase()));
        }
        if let Some(show_news) = dashboard_config.show_brave_news {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_brave_news = ${}\n", show_news.to_string().to_lowercase()));
        }
        if let Some(show_together) = dashboard_config.show_together {
            content.push_str(&format!("$prefs.brave.new_tab_page.show_together = ${}\n", show_together.to_string().to_lowercase()));
        }
        
        content.push_str("$prefs.brave.stats.enabled = $false\n");
        content.push_str("$prefs.brave.today.should_show_brave_today_widget = $false\n");
        
        // Save preferences
        content.push_str("$prefs | ConvertTo-Json -Depth 10 | Set-Content $prefsPath -Encoding UTF8\n");
        
        // Handle Local State file
        content.push_str("if (Test-Path $localStatePath) {\n");
        content.push_str("    $localState = Get-Content $localStatePath -Raw | ConvertFrom-Json\n");
        content.push_str("} else {\n");
        content.push_str("    $localState = @{}\n");
        content.push_str("}\n");
        content.push_str("if (-not $localState.browser) { $localState.browser = @{} }\n");
        content.push_str("$localState.browser.enabled_labs_experiments = @(\n");
        for (i, feature) in experimental_features.iter().enumerate() {
            content.push_str(&format!("    '{}'", feature));
            if i < experimental_features.len() - 1 {
                content.push_str(",");
            }
            content.push_str("\n");
        }
        content.push_str(")\n");
        content.push_str("$localState | ConvertTo-Json -Depth 10 | Set-Content $localStatePath -Encoding UTF8\n");
        content.push_str("\"\n");
        content.push_str("echo User preferences applied successfully!\n\n");
        
        Ok(())
    }

    fn add_macos_preferences_section(&self, content: &mut String, version_suffix: &str) -> Result<(), DebloaterError> {
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

        content.push_str("echo -e \"${GREEN}Modifying user preferences...${NC}\"\n");
        content.push_str(&format!("BRAVE_DATA=\"$HOME/Library/Application Support/BraveSoftware/{}\"\n", version_suffix));
        content.push_str("PREFS_FILE=\"$BRAVE_DATA/Default/Preferences\"\n");
        content.push_str("LOCAL_STATE=\"$BRAVE_DATA/Local State\"\n\n");
        
        // Create directories if they don't exist
        content.push_str("mkdir -p \"$BRAVE_DATA/Default\"\n\n");
        
        // Backup existing files
        content.push_str("[ -f \"$PREFS_FILE\" ] && cp \"$PREFS_FILE\" \"$PREFS_FILE.backup\"\n");
        content.push_str("[ -f \"$LOCAL_STATE\" ] && cp \"$LOCAL_STATE\" \"$LOCAL_STATE.backup\"\n\n");
        
        // Check if jq is available
        content.push_str("if ! command -v jq &> /dev/null; then\n");
        content.push_str("    echo -e \"${YELLOW}jq not found. Installing via Homebrew...${NC}\"\n");
        content.push_str("    if command -v brew &> /dev/null; then\n");
        content.push_str("        brew install jq\n");
        content.push_str("    else\n");
        content.push_str("        echo -e \"${RED}Error: jq is required but not installed and Homebrew is not available${NC}\"\n");
        content.push_str("        echo \"Please install jq manually: https://stedolan.github.io/jq/download/\"\n");
        content.push_str("        exit 1\n");
        content.push_str("    fi\n");
        content.push_str("fi\n\n");
        
        // Modify preferences file
        content.push_str("# Create or modify Preferences file\n");
        content.push_str("if [ -f \"$PREFS_FILE\" ]; then\n");
        content.push_str("    PREFS_JSON=$(cat \"$PREFS_FILE\")\n");
        content.push_str("else\n");
        content.push_str("    PREFS_JSON='{}'\n");
        content.push_str("fi\n\n");
        
        // Update preferences using jq
        content.push_str("PREFS_JSON=$(echo \"$PREFS_JSON\" | jq '\n");
        
        // Search provider
        content.push_str("  .default_search_provider_data = {\n");
        content.push_str(&format!("    \"keyword\": \"{}\",\n", search_provider.keyword));
        content.push_str(&format!("    \"name\": \"{}\",\n", search_provider.name));
        content.push_str(&format!("    \"search_url\": \"{}\"\n", search_provider.search_url));
        content.push_str("  } |\n");
        
        // Brave preferences
        content.push_str("  .brave = (.brave // {}) |\n");
        content.push_str("  .brave.new_tab_page = (.brave.new_tab_page // {}) |\n");
        content.push_str("  .brave.stats = (.brave.stats // {}) |\n");
        content.push_str("  .brave.today = (.brave.today // {}) |\n");
        
        // Dashboard settings
        if let Some(show_clock) = dashboard_config.show_clock {
            content.push_str(&format!("  .brave.new_tab_page.show_clock = {} |\n", show_clock.to_string().to_lowercase()));
        }
        if let Some(show_bg) = dashboard_config.show_background_image {
            content.push_str(&format!("  .brave.new_tab_page.show_background_image = {} |\n", show_bg.to_string().to_lowercase()));
        }
        if let Some(show_stats) = dashboard_config.show_stats {
            content.push_str(&format!("  .brave.new_tab_page.show_stats = {} |\n", show_stats.to_string().to_lowercase()));
        }
        if let Some(show_shortcuts) = dashboard_config.show_shortcuts {
            content.push_str(&format!("  .brave.new_tab_page.show_shortcuts = {} |\n", show_shortcuts.to_string().to_lowercase()));
        }
        if let Some(show_branded) = dashboard_config.show_branded_background_image {
            content.push_str(&format!("  .brave.new_tab_page.show_branded_background_image = {} |\n", show_branded.to_string().to_lowercase()));
        }
        if let Some(show_cards) = dashboard_config.show_cards {
            content.push_str(&format!("  .brave.new_tab_page.show_cards = {} |\n", show_cards.to_string().to_lowercase()));
        }
        if let Some(show_search) = dashboard_config.show_search_widget {
            content.push_str(&format!("  .brave.new_tab_page.show_search_widget = {} |\n", show_search.to_string().to_lowercase()));
        }
        if let Some(show_news) = dashboard_config.show_brave_news {
            content.push_str(&format!("  .brave.new_tab_page.show_brave_news = {} |\n", show_news.to_string().to_lowercase()));
        }
        if let Some(show_together) = dashboard_config.show_together {
            content.push_str(&format!("  .brave.new_tab_page.show_together = {} |\n", show_together.to_string().to_lowercase()));
        }
        
        content.push_str("  .brave.stats.enabled = false |\n");
        content.push_str("  .brave.today.should_show_brave_today_widget = false\n");
        content.push_str("')\n\n");
        
        content.push_str("echo \"$PREFS_JSON\" > \"$PREFS_FILE\"\n\n");
        
        // Handle Local State file
        content.push_str("# Create or modify Local State file\n");
        content.push_str("if [ -f \"$LOCAL_STATE\" ]; then\n");
        content.push_str("    LOCAL_JSON=$(cat \"$LOCAL_STATE\")\n");
        content.push_str("else\n");
        content.push_str("    LOCAL_JSON='{}'\n");
        content.push_str("fi\n\n");
        
        content.push_str("LOCAL_JSON=$(echo \"$LOCAL_JSON\" | jq '\n");
        content.push_str("  .browser = (.browser // {}) |\n");
        content.push_str("  .browser.enabled_labs_experiments = [\n");
        for (i, feature) in experimental_features.iter().enumerate() {
            content.push_str(&format!("    \"{}\"", feature));
            if i < experimental_features.len() - 1 {
                content.push_str(",");
            }
            content.push_str("\n");
        }
        content.push_str("  ]\n");
        content.push_str("')\n\n");
        
        content.push_str("echo \"$LOCAL_JSON\" > \"$LOCAL_STATE\"\n");
        content.push_str("echo -e \"${GREEN}User preferences applied successfully!${NC}\"\n");
        content.push_str("echo\n\n");
        
        Ok(())
    }

    fn add_linux_preferences_section(&self, content: &mut String, version_suffix: &str) -> Result<(), DebloaterError> {
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

        content.push_str("echo -e \"${GREEN}Modifying user preferences...${NC}\"\n");
        
        // Check for Flatpak installation
        content.push_str("if command -v flatpak &> /dev/null && flatpak list | grep -q com.brave.Browser; then\n");
        content.push_str(&format!("    BRAVE_DATA=\"$HOME/.var/app/com.brave.Browser/config/BraveSoftware/{}\"\n", version_suffix));
        content.push_str("    echo \"Detected Flatpak Brave installation\"\n");
        content.push_str("else\n");
        content.push_str(&format!("    BRAVE_DATA=\"$HOME/.config/BraveSoftware/{}\"\n", version_suffix));
        content.push_str("    echo \"Detected native Brave installation\"\n");
        content.push_str("fi\n\n");
        
        content.push_str("PREFS_FILE=\"$BRAVE_DATA/Default/Preferences\"\n");
        content.push_str("LOCAL_STATE=\"$BRAVE_DATA/Local State\"\n\n");
        
        // Create directories if they don't exist
        content.push_str("mkdir -p \"$BRAVE_DATA/Default\"\n\n");
        
        // Backup existing files
        content.push_str("[ -f \"$PREFS_FILE\" ] && cp \"$PREFS_FILE\" \"$PREFS_FILE.backup\"\n");
        content.push_str("[ -f \"$LOCAL_STATE\" ] && cp \"$LOCAL_STATE\" \"$LOCAL_STATE.backup\"\n\n");
        
        // Check if jq is available
        content.push_str("if ! command -v jq &> /dev/null; then\n");
        content.push_str("    echo -e \"${YELLOW}jq not found. Attempting to install...${NC}\"\n");
        content.push_str("    if command -v apt-get &> /dev/null; then\n");
        content.push_str("        sudo apt-get update && sudo apt-get install -y jq\n");
        content.push_str("    elif command -v dnf &> /dev/null; then\n");
        content.push_str("        sudo dnf install -y jq\n");
        content.push_str("    elif command -v pacman &> /dev/null; then\n");
        content.push_str("        sudo pacman -S jq\n");
        content.push_str("    elif command -v zypper &> /dev/null; then\n");
        content.push_str("        sudo zypper install jq\n");
        content.push_str("    else\n");
        content.push_str("        echo -e \"${RED}Error: jq is required but could not be installed automatically${NC}\"\n");
        content.push_str("        echo \"Please install jq manually for your distribution\"\n");
        content.push_str("        exit 1\n");
        content.push_str("    fi\n");
        content.push_str("    if ! command -v jq &> /dev/null; then\n");
        content.push_str("        echo -e \"${RED}Error: jq installation failed${NC}\"\n");
        content.push_str("        exit 1\n");
        content.push_str("    fi\n");
        content.push_str("fi\n\n");
        
        // Modify preferences file
        content.push_str("# Create or modify Preferences file\n");
        content.push_str("if [ -f \"$PREFS_FILE\" ]; then\n");
        content.push_str("    PREFS_JSON=$(cat \"$PREFS_FILE\")\n");
        content.push_str("else\n");
        content.push_str("    PREFS_JSON='{}'\n");
        content.push_str("fi\n\n");
        
        // Update preferences using jq
        content.push_str("PREFS_JSON=$(echo \"$PREFS_JSON\" | jq '\n");
        
        // Search provider
        content.push_str("  .default_search_provider_data = {\n");
        content.push_str(&format!("    \"keyword\": \"{}\",\n", search_provider.keyword));
        content.push_str(&format!("    \"name\": \"{}\",\n", search_provider.name));
        content.push_str(&format!("    \"search_url\": \"{}\"\n", search_provider.search_url));
        content.push_str("  } |\n");
        
        // Brave preferences
        content.push_str("  .brave = (.brave // {}) |\n");
        content.push_str("  .brave.new_tab_page = (.brave.new_tab_page // {}) |\n");
        content.push_str("  .brave.stats = (.brave.stats // {}) |\n");
        content.push_str("  .brave.today = (.brave.today // {}) |\n");
        
        // Dashboard settings
        if let Some(show_clock) = dashboard_config.show_clock {
            content.push_str(&format!("  .brave.new_tab_page.show_clock = {} |\n", show_clock.to_string().to_lowercase()));
        }
        if let Some(show_bg) = dashboard_config.show_background_image {
            content.push_str(&format!("  .brave.new_tab_page.show_background_image = {} |\n", show_bg.to_string().to_lowercase()));
        }
        if let Some(show_stats) = dashboard_config.show_stats {
            content.push_str(&format!("  .brave.new_tab_page.show_stats = {} |\n", show_stats.to_string().to_lowercase()));
        }
        if let Some(show_shortcuts) = dashboard_config.show_shortcuts {
            content.push_str(&format!("  .brave.new_tab_page.show_shortcuts = {} |\n", show_shortcuts.to_string().to_lowercase()));
        }
        if let Some(show_branded) = dashboard_config.show_branded_background_image {
            content.push_str(&format!("  .brave.new_tab_page.show_branded_background_image = {} |\n", show_branded.to_string().to_lowercase()));
        }
        if let Some(show_cards) = dashboard_config.show_cards {
            content.push_str(&format!("  .brave.new_tab_page.show_cards = {} |\n", show_cards.to_string().to_lowercase()));
        }
        if let Some(show_search) = dashboard_config.show_search_widget {
            content.push_str(&format!("  .brave.new_tab_page.show_search_widget = {} |\n", show_search.to_string().to_lowercase()));
        }
        if let Some(show_news) = dashboard_config.show_brave_news {
            content.push_str(&format!("  .brave.new_tab_page.show_brave_news = {} |\n", show_news.to_string().to_lowercase()));
        }
        if let Some(show_together) = dashboard_config.show_together {
            content.push_str(&format!("  .brave.new_tab_page.show_together = {} |\n", show_together.to_string().to_lowercase()));
        }
        
        content.push_str("  .brave.stats.enabled = false |\n");
        content.push_str("  .brave.today.should_show_brave_today_widget = false\n");
        content.push_str("')\n\n");
        
        content.push_str("echo \"$PREFS_JSON\" > \"$PREFS_FILE\"\n\n");
        
        // Handle Local State file
        content.push_str("# Create or modify Local State file\n");
        content.push_str("if [ -f \"$LOCAL_STATE\" ]; then\n");
        content.push_str("    LOCAL_JSON=$(cat \"$LOCAL_STATE\")\n");
        content.push_str("else\n");
        content.push_str("    LOCAL_JSON='{}'\n");
        content.push_str("fi\n\n");
        
        content.push_str("LOCAL_JSON=$(echo \"$LOCAL_JSON\" | jq '\n");
        content.push_str("  .browser = (.browser // {}) |\n");
        content.push_str("  .browser.enabled_labs_experiments = [\n");
        for (i, feature) in experimental_features.iter().enumerate() {
            content.push_str(&format!("    \"{}\"", feature));
            if i < experimental_features.len() - 1 {
                content.push_str(",");
            }
            content.push_str("\n");
        }
        content.push_str("  ]\n");
        content.push_str("')\n\n");
        
        content.push_str("echo \"$LOCAL_JSON\" > \"$LOCAL_STATE\"\n");
        content.push_str("echo -e \"${GREEN}User preferences applied successfully!${NC}\"\n");
        content.push_str("echo\n\n");
        
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

    // Always load preferences config for unified scripts
    println!("Loading preferences from: {}", args.preferences_config);
    let preferences_config = load_preferences_config(&args.preferences_config)?;

    if preferences_config.is_some() {
        println!("Loaded preferences configuration");
    } else {
        println!("Using default preferences configuration");
    }
    
    println!("Generating unified {} script for Brave {:?}...", 
             match args.platform {
                 Platform::Windows => "Windows",
                 Platform::MacOS => "macOS",
                 Platform::Linux => "Linux",
             },
             args.version);
    
    let output_dir = args.output.clone();
    let generator = DebloaterGenerator::new(config, extensions, args.platform, args.version, args.output, preferences_config);
    generator.generate()?;
    
    println!("Configuration files generated successfully in: {}", output_dir);
    
    Ok(())
}