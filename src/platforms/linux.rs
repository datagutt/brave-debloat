use std::fs;
use std::path::Path;

use crate::cli::BraveVersion;
use crate::config::{Config, Extension, ConfigValue};
use crate::error::DebloaterError;
use crate::platforms::{PlatformGenerator, get_linux_policy_path, get_version_suffix};
use crate::preferences::{PreferencesInputConfig, get_default_search_provider, get_default_dashboard_config, get_default_experimental_features};

pub struct LinuxGenerator;

impl PlatformGenerator for LinuxGenerator {
    fn generate_unified_script(&self, config: &Config, extensions: &[Extension], version: &BraveVersion, output_dir: &str, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError> {
        let filename = match version {
            BraveVersion::Normal => "brave_debloat_linux.sh",
            BraveVersion::Nightly => "brave_nightly_debloat_linux.sh",
        };
        
        let version_suffix = get_version_suffix(version);
        let policy_path = get_linux_policy_path(version);
        
        let mut content = String::new();
        content.push_str("#!/bin/bash\n");
        content.push_str("# Unified Brave Browser Debloater Script for Linux\n");
        content.push_str("# This script applies both policies and user preferences\n");
        content.push_str("# Run with sudo for system-wide changes\n\n");
        
        add_color_definitions(&mut content);
        add_header(&mut content);
        add_sudo_check(&mut content);
        add_brave_process_check(&mut content);
        add_system_policies(&mut content, config, extensions, policy_path)?;
        add_user_preferences(&mut content, version_suffix, preferences_config)?;
        
        content.push_str("echo -e \"${GREEN}Configuration complete!${NC}\"\n");
        content.push_str("echo -e \"${GREEN}Please restart Brave browser for changes to take effect.${NC}\"\n");
        
        let output_path = Path::new(output_dir).join(filename);
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
}

fn add_color_definitions(content: &mut String) {
    content.push_str("# Colors for output\n");
    content.push_str("RED='\\033[0;31m'\n");
    content.push_str("GREEN='\\033[0;32m'\n");
    content.push_str("YELLOW='\\033[1;33m'\n");
    content.push_str("NC='\\033[0m' # No Color\n\n");
}

fn add_header(content: &mut String) {
    content.push_str("echo -e \"${GREEN}Brave Browser Debloater for Linux${NC}\"\n");
    content.push_str("echo -e \"${GREEN}===================================${NC}\"\n");
    content.push_str("echo\n\n");
}

fn add_sudo_check(content: &mut String) {
    content.push_str("if [ \"$EUID\" -ne 0 ]; then\n");
    content.push_str("    echo -e \"${YELLOW}Note: Running without sudo. System policies will be skipped.${NC}\"\n");
    content.push_str("    echo -e \"${YELLOW}Run with sudo for complete configuration.${NC}\"\n");
    content.push_str("    SKIP_POLICIES=1\n");
    content.push_str("else\n");
    content.push_str("    SKIP_POLICIES=0\n");
    content.push_str("fi\n");
    content.push_str("echo\n\n");
}

fn add_brave_process_check(content: &mut String) {
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
}

fn add_system_policies(content: &mut String, config: &Config, extensions: &[Extension], policy_path: &str) -> Result<(), DebloaterError> {
    content.push_str("if [ \"$SKIP_POLICIES\" -eq 0 ]; then\n");
    content.push_str("    echo -e \"${GREEN}Applying system policies...${NC}\"\n");
    content.push_str(&format!("    mkdir -p \"$(dirname '{}')\"\n", policy_path));
    
    // Create the JSON policy file
    content.push_str(&format!("    cat << 'EOF' > '{}'\n", policy_path));
    
    // Generate JSON content
    let mut final_config = config.clone();
    if !extensions.is_empty() {
        let extension_ids: Vec<String> = extensions.iter().map(|e| e.id.clone()).collect();
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
    
    Ok(())
}

fn add_user_preferences(content: &mut String, version_suffix: &str, preferences_config: Option<&PreferencesInputConfig>) -> Result<(), DebloaterError> {
    let search_provider = get_default_search_provider(preferences_config);
    let dashboard_config = get_default_dashboard_config(preferences_config);
    let experimental_features = get_default_experimental_features(preferences_config);

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
    
    add_jq_installation_check(content);
    add_preferences_modification(content, &search_provider, &dashboard_config);
    add_local_state_modification(content, &experimental_features);
    
    content.push_str("echo -e \"${GREEN}User preferences applied successfully!${NC}\"\n");
    content.push_str("echo\n\n");
    
    Ok(())
}

fn add_jq_installation_check(content: &mut String) {
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
}

fn add_preferences_modification(content: &mut String, search_provider: &crate::preferences::SearchProvider, dashboard_config: &crate::preferences::NewTabPage) {
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
    add_dashboard_settings_jq(content, dashboard_config);
    
    content.push_str("  .brave.stats.enabled = false |\n");
    content.push_str("  .brave.today.should_show_brave_today_widget = false\n");
    content.push_str("')\n\n");
    
    content.push_str("echo \"$PREFS_JSON\" > \"$PREFS_FILE\"\n\n");
}

fn add_dashboard_settings_jq(content: &mut String, dashboard_config: &crate::preferences::NewTabPage) {
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
}

fn add_local_state_modification(content: &mut String, experimental_features: &[String]) {
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
}