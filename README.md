# Brave Browser Debloater

A cross-platform Rust tool that generates Brave browser debloat configurations for Windows, macOS, and Linux. Supports both normal and nightly versions of Brave browser.

## Features

- **Cross-platform support**: Windows (Registry), macOS (defaults/plist), Linux (JSON policies)
- **Brave version support**: Both normal and nightly versions
- **Comprehensive debloating**: Disables telemetry, ads, rewards, VPN, wallet, and other privacy-invasive features
- **Extension management**: Force-installs essential privacy extensions

## Usage

```bash
# Build the tool
cargo build --release

# Generate Windows registry file for normal Brave
./target/release/brave-debloater --platform windows --version normal

# Generate macOS script for Brave Nightly
./target/release/brave-debloater --platform mac-os --version nightly

# Generate Linux JSON for normal Brave
./target/release/brave-debloater --platform linux --version normal

# Generate user preferences files for dashboard and search customization
./target/release/brave-debloater --platform linux --preferences

# Use custom config and extensions files with custom output directory
./target/release/brave-debloater --platform windows --config my-config.json --extensions my-extensions.json --output my-output

# Use the author's personal balanced config
./target/release/brave-debloater --platform linux --config configs/balanced.json

# Use minimal debloating
./target/release/brave-debloater --platform windows --config configs/minimal.json

# Generate both policies and user preferences with custom preferences config
./target/release/brave-debloater --platform linux --preferences --preferences-config my-preferences.json

# Default behavior uses privacy-focused config
./target/release/brave-debloater --platform linux
```

## Command Line Options

- `--platform`: Target platform (`windows`, `mac-os`, `linux`)
- `--version`: Brave version (`normal`, `nightly`)
- `--config`: Configuration file path (default: `configs/privacy-focused.json`)
- `--extensions`: Extensions configuration file path (default: `extensions.json`)
- `--output`: Output directory (default: `output`)
- `--preferences`: Generate user preferences files for dashboard and search customization
- `--preferences-config`: Preferences configuration file (default: `preferences.json`)

## Installation Instructions

### Windows
1. Run the generated `.reg` file as Administrator
2. Or manually import using Registry Editor

### macOS
1. Make the script executable: `chmod +x output/brave_debloat_macos.sh`
2. Run with sudo: `sudo ./output/brave_debloat_macos.sh`

### Linux
1. Follow instructions in the generated `_install.txt` file
2. Copy the JSON file to the appropriate Brave policies directory

## Configuration

The tool uses two types of configuration files:

### Config Variants (`configs/` folder)

Choose from three pre-configured privacy levels:

**🔒 Privacy-Focused (Default)** - `configs/privacy-focused.json`
- Maximum privacy settings
- Disables ALL Brave features, IPFS, Tor, telemetry
- Most restrictive permissions

**⚖️ Balanced** - `configs/balanced.json`  
- **Author's personal setup** - battle-tested configuration
- Disables all Brave bloat while keeping essential functionality
- Uses Chrome's exact default permission settings
- Enables guest mode, sign-in, and sync for usability

**🎯 Minimal** - `configs/minimal.json`
- Only removes core Brave bloatware
- Keeps standard browser functionality

### extensions.json
Contains privacy-focused extensions to force-install:

- **uBlock Origin**: Ad blocker and privacy protection
- **SponsorBlock**: Skip YouTube sponsor segments  
- **Privacy Badger**: Block trackers and protect privacy

### preferences.json
Contains user preferences for dashboard and search customization:

- **Search engines**: Configure default search engine (Brave, DuckDuckGo, SearXNG, etc.)
- **Dashboard settings**: Customize new tab page (show clock, hide widgets, etc.)
- **Experimental features**: Enable advanced ad-blocking and other experimental features

See `configs/README.md` for detailed comparison of variants.

## Generated Files

The tool generates the following files in the output directory:

### Policy Files

**For Normal Brave:**
- `brave_debloat.reg` (Windows Registry)
- `brave_debloat_macos.sh` (macOS Script)
- `brave_debloat_linux.json` + `brave_debloat_linux_install.txt` (Linux)

**For Brave Nightly:**
- `brave_nightly_debloat.reg` (Windows Registry)
- `brave_nightly_debloat_macos.sh` (macOS Script)  
- `brave_nightly_debloat_linux.json` + `brave_nightly_debloat_linux_install.txt` (Linux)

### User Preferences Files (when using --preferences flag)

**For Normal Brave:**
- `brave_user_preferences.json` + `brave_user_preferences_install.sh/.bat` 

**For Brave Nightly:**
- `brave_nightly_user_preferences.json` + `brave_nightly_user_preferences_install.sh/.bat`

User preferences files modify Brave's `Preferences` and `Local State` files directly to customize the dashboard, search engine, and enable experimental features. These work alongside the policy files for comprehensive debloating.

## License

This project is licensed under the MIT License - see the LICENSE file for details.