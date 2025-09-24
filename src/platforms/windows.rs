use std::fs;
use std::path::Path;

use crate::cli::BraveVersion;
use crate::config::{Config, Extension, ConfigValue};
use crate::error::DebloaterError;
use crate::platforms::{PlatformGenerator, get_brave_registry_path, get_version_suffix};
use crate::preferences::{PreferencesInputConfig, get_default_search_provider, get_default_dashboard_config, get_default_experimental_features};

pub struct WindowsGenerator;

impl PlatformGenerator for WindowsGenerator {
    fn generate_unified_script(&self, config: &Config, extensions: &[Extension], version: &BraveVersion, output_dir: &str, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError> {
        let filename = match version {
            BraveVersion::Normal => "brave_debloat.bat",
            BraveVersion::Nightly => "brave_nightly_debloat.bat",
        };
        
        let version_suffix = get_version_suffix(version);
        
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
        add_brave_process_check(&mut content);
        
        // Generate registry entries
        add_registry_policies(&mut content, config, extensions, version)?;
        
        // Add user preferences modification
        add_user_preferences_modification(&mut content, version_suffix, preferences_config)?;
        
        content.push_str("echo Configuration complete!\n");
        content.push_str("echo Please restart Brave browser for changes to take effect.\n");
        content.push_str("pause\n");
        
        let output_path = Path::new(output_dir).join(filename);
        fs::write(output_path, content)?;
        
        Ok(())
    }
}

fn add_brave_process_check(content: &mut String) {
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
}

fn add_registry_policies(content: &mut String, config: &Config, extensions: &[Extension], version: &BraveVersion) -> Result<(), DebloaterError> {
    content.push_str("echo Applying Brave policies via registry...\n");
    
    let registry_path = get_brave_registry_path(version);
    
    for (key, value) in config {
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
    if !extensions.is_empty() {
        for (i, ext) in extensions.iter().enumerate() {
            content.push_str(&format!("reg add \"HKEY_LOCAL_MACHINE\\{}\\ExtensionInstallForcelist\" /v \"{}\" /t REG_SZ /d \"{}\" /f >nul 2>&1\n", registry_path, i + 1, ext.id));
        }
    }
    
    content.push_str("echo Registry policies applied successfully!\n\n");
    
    Ok(())
}

fn add_user_preferences_modification(content: &mut String, version_suffix: &str, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError> {
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
    add_windows_preferences_powershell(content, preferences_config)?;
    
    Ok(())
}

fn add_windows_preferences_powershell(content: &mut String, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError> {
    let search_provider = get_default_search_provider(preferences_config);
    let dashboard_config = get_default_dashboard_config(preferences_config);
    let experimental_features = get_default_experimental_features(preferences_config);

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
    add_dashboard_settings_powershell(content, &dashboard_config);
    
    content.push_str("$prefs.brave.stats.enabled = $false\n");
    content.push_str("$prefs.brave.today.should_show_brave_today_widget = $false\n");
    
    // Save preferences
    content.push_str("$prefs | ConvertTo-Json -Depth 10 | Set-Content $prefsPath -Encoding UTF8\n");
    
    // Handle Local State file
    add_local_state_powershell(content, &experimental_features);
    
    content.push_str("\"\n");
    content.push_str("echo User preferences applied successfully!\n\n");
    
    Ok(())
}

fn add_dashboard_settings_powershell(content: &mut String, dashboard_config: &crate::preferences::NewTabPage) {
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
}

fn add_local_state_powershell(content: &mut String, experimental_features: &[String]) {
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
}