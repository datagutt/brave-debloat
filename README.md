# Brave Browser Debloater

A cross-platform Rust tool that generates comprehensive Brave browser debloat scripts for Windows, macOS, and Linux. Each script handles both system policies and user preferences in a single executable file. Supports both normal and nightly versions of Brave browser.

## Features

- **Unified Scripts**: Single executable files that handle both system policies and user preferences
- **Cross-platform support**: Windows (Registry + PowerShell), macOS (plist + shell), Linux (JSON policies + shell)
- **Brave version support**: Both normal and nightly versions
- **Comprehensive debloating**: Disables telemetry, ads, rewards, VPN, wallet, and other privacy-invasive features
- **Extension management**: Force-installs essential privacy extensions
- **User preferences**: Customizes dashboard, search engine, and experimental features
- **Smart detection**: Auto-detects Flatpak installations on Linux

## Usage

```bash
# Build the tool
cargo build --release

# Generate unified Windows script for normal Brave
./target/release/brave-debloater --platform windows --version normal

# Generate unified macOS script for Brave Nightly
./target/release/brave-debloater --platform mac-os --version nightly

# Generate unified Linux script for normal Brave
./target/release/brave-debloater --platform linux --version normal

# Use custom config and extensions files with custom output directory
./target/release/brave-debloater --platform windows --config my-config.json --extensions my-extensions.json --output my-output

# Use the author's personal balanced config
./target/release/brave-debloater --platform linux --config configs/balanced.json

# Use minimal debloating
./target/release/brave-debloater --platform windows --config configs/minimal.json

# Use custom preferences config for dashboard and search customization
./target/release/brave-debloater --platform linux --preferences-config my-preferences.json

# Default behavior uses privacy-focused config
./target/release/brave-debloater --platform linux
```

## Command Line Options

- `--platform`: Target platform (`windows`, `mac-os`, `linux`)
- `--version`: Brave version (`normal`, `nightly`)
- `--config`: Configuration file path (default: `configs/privacy-focused.json`)
- `--extensions`: Extensions configuration file path (default: `extensions.json`)
- `--output`: Output directory (default: `output`)
- `--preferences-config`: Preferences configuration file (default: `preferences.json`)

## Installation Instructions

### Windows
1. Right-click the generated `.bat` file and select "Run as Administrator"
2. The script will automatically apply registry policies and modify user preferences

### macOS
1. Make the script executable: `chmod +x output/brave_debloat_macos.sh`
2. Run with sudo for complete configuration: `sudo ./output/brave_debloat_macos.sh`
3. Or run without sudo to apply only user preferences

### Linux
1. Make the script executable: `chmod +x output/brave_debloat_linux.sh`
2. Run with sudo for complete configuration: `sudo ./output/brave_debloat_linux.sh`
3. Or run without sudo to apply only user preferences

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

The tool generates unified scripts that handle both system policies and user preferences:

**For Normal Brave:**
- `brave_debloat.bat` (Windows - Registry + PowerShell)
- `brave_debloat_macos.sh` (macOS - plist + shell + jq)
- `brave_debloat_linux.sh` (Linux - JSON policies + shell + jq)

**For Brave Nightly:**
- `brave_nightly_debloat.bat` (Windows)
- `brave_nightly_debloat_macos.sh` (macOS)  
- `brave_nightly_debloat_linux.sh` (Linux)

Each script performs the following actions:
1. **System Policies**: Applies organization-level policies (requires admin/sudo)
2. **User Preferences**: Modifies user's Preferences and Local State files directly
3. **Dashboard Customization**: Removes widgets, customizes new tab page
4. **Search Engine**: Configures default search provider
5. **Experimental Features**: Enables advanced ad-blocking and other features
6. **Safety Features**: Backs up existing files, checks for running processes

## License

This project is licensed under the MIT License - see the LICENSE file for details.