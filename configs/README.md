# Configuration Variants

This folder contains different configuration variants for various privacy preferences and use cases.

## Available Configurations

### 🔒 privacy-focused.json (Default)
**Maximum privacy and security settings**

- Disables ALL Brave-specific features (Rewards, VPN, Wallet, News, Talk, AI Chat, Playlist, IPFS)
- Disables Tor functionality 
- Blocks all telemetry, analytics, and reporting
- Restrictive default permissions for location, notifications, sensors, etc.
- Disables background mode and sync
- Most privacy-invasive features turned off

**Best for:** Users who prioritize maximum privacy and don't mind some inconvenience.

### ⚖️ balanced.json
**Author's personal configuration** - *This is the setup I personally use*

- Disables ALL Brave-specific features and IPFS/Tor (same as privacy-focused)
- Disables privacy-invasive features (autofill, password manager, speedreader, wayback machine)
- Disables background mode and built-in DNS client
- **Key differences from privacy-focused:**
  - Uses Chrome's exact default permission settings (Ask for geo/notifications/sensors, Allow fonts, Block serial)
  - Enables guest mode for temporary browsing
  - Allows browser sign-in (setting 1 vs 0)
  - Keeps sync enabled (but no forced sync URL)
  - Disables alternate error pages for privacy
- WebRTC uses default policy with multiple routes enabled

**Best for:** Users who want a battle-tested configuration that prioritizes privacy while maintaining essential browser functionality.

### 🎯 minimal.json
**Only disables the most problematic features**

- Disables ALL Brave-specific features and IPFS/Tor (same as others)
- Disables core telemetry and reporting
- Keeps most browser functionality intact
- Minimal changes to default behavior

**Best for:** Users who want to remove Brave bloatware but keep standard browser functionality.

## Usage

```bash
# Use a specific config variant
./target/release/brave-debloater --platform linux --config configs/balanced.json

# Use the default (privacy-focused)
./target/release/brave-debloater --platform linux
```

## Key Principles

1. **Brave-specific features are ALWAYS disabled** in all variants (Rewards, VPN, Wallet, News, Talk, AI Chat, Playlist)
2. **IPFS and Tor are ALWAYS disabled** across all variants for security
3. **Telemetry and analytics are ALWAYS disabled** in all variants
4. Only convenience features vary between configurations

## Customization

You can create your own variant by copying one of these files and modifying it to suit your needs. The structure follows standard Chromium policy format.