# macOS App Packaging Guide

This document describes how to build and package Sol UI applications as native macOS app bundles.

## Quick Start

```bash
# Build and package (debug mode, ad-hoc signed)
./scripts/package-macos.sh

# Build release and create DMG
./scripts/package-macos.sh --release --dmg

# Run the packaged app
open target/package/Sol\ UI.app
```

## Directory Structure

```
resources/macos/
├── Info.plist          # App metadata and configuration
├── entitlements.plist  # Code signing entitlements
└── AppIcon.icns        # App icon (generate or provide your own)

scripts/
├── package-macos.sh    # Main packaging script
└── generate-icon.sh    # Icon generation helper
```

## Packaging Options

### Basic Packaging

```bash
# Debug build with ad-hoc signing (for local development)
./scripts/package-macos.sh

# Release build (optimized)
./scripts/package-macos.sh --release
```

### Code Signing

For distribution outside the App Store, you'll need an Apple Developer ID:

```bash
# Sign with Developer ID
./scripts/package-macos.sh --release --sign "Developer ID Application: Your Name (TEAM_ID)"
```

Find your signing identity:
```bash
security find-identity -v -p codesigning
```

### Notarization

Apple requires notarization for apps distributed outside the App Store (macOS 10.15+):

```bash
# Sign and prepare for notarization
./scripts/package-macos.sh --release --sign "Developer ID Application: Your Name" --notarize

# The script will output instructions for notarization
```

Manual notarization steps:
```bash
# 1. Store credentials (one time)
xcrun notarytool store-credentials "AC_PASSWORD" \
    --apple-id "your@email.com" \
    --team-id "TEAM_ID" \
    --password "app-specific-password"

# 2. Submit for notarization
xcrun notarytool submit target/package/Sol\ UI-notarize.zip \
    --keychain-profile "AC_PASSWORD" \
    --wait

# 3. Staple the notarization ticket
xcrun stapler staple "target/package/Sol UI.app"
```

### Creating a DMG Installer

```bash
# Create DMG with Applications folder symlink
./scripts/package-macos.sh --release --dmg

# With signing
./scripts/package-macos.sh --release --sign "Developer ID Application: Your Name" --dmg
```

## App Icon

### Using a Custom Icon

Provide a 1024x1024 PNG image:

```bash
./scripts/generate-icon.sh /path/to/your/icon.png
```

### Generating a Placeholder

If you have ImageMagick installed:

```bash
./scripts/generate-icon.sh  # Creates a simple placeholder
```

Install ImageMagick if needed:
```bash
brew install imagemagick
```

## Info.plist Configuration

Key fields in `resources/macos/Info.plist`:

| Key | Description |
|-----|-------------|
| `CFBundleDisplayName` | App name shown to users |
| `CFBundleIdentifier` | Unique bundle ID (reverse DNS) |
| `CFBundleVersion` | Build number (increments with each build) |
| `CFBundleShortVersionString` | Version shown to users |
| `LSMinimumSystemVersion` | Minimum macOS version required |
| `NSHighResolutionCapable` | Enable Retina display support |

## Entitlements

The `entitlements.plist` includes:

- `com.apple.security.cs.allow-jit` - Required for Metal shader compilation
- `com.apple.security.cs.allow-unsigned-executable-memory` - For dynamic code
- `com.apple.security.cs.disable-library-validation` - Development convenience

For production, review and minimize entitlements based on your app's requirements.

## Troubleshooting

### "App is damaged and can't be opened"

This usually means the app isn't properly signed or notarized:

```bash
# Remove quarantine attribute for local testing
xattr -cr "target/package/Sol UI.app"

# Or properly sign the app
./scripts/package-macos.sh --sign "Developer ID Application: Your Name"
```

### Verify Code Signature

```bash
codesign --verify --verbose "target/package/Sol UI.app"
spctl --assess --verbose "target/package/Sol UI.app"
```

### Check Entitlements

```bash
codesign -d --entitlements - "target/package/Sol UI.app"
```

## Customizing for Your App

To package a different binary or customize the bundle:

1. Edit `scripts/package-macos.sh`:
   - Change `APP_NAME`, `BUNDLE_ID`, `EXECUTABLE_NAME`

2. Edit `resources/macos/Info.plist`:
   - Update bundle identifier, display name, etc.

3. Replace `resources/macos/AppIcon.icns` with your app's icon

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
- name: Package macOS App
  run: |
    ./scripts/package-macos.sh --release --dmg

- name: Upload DMG
  uses: actions/upload-artifact@v3
  with:
    name: macos-app
    path: target/package/*.dmg
```

For signed/notarized builds in CI, use secrets to store credentials securely.
