#!/bin/bash
# macOS App Bundle Packaging Script for Sol UI
#
# Usage:
#   ./scripts/package-macos.sh [--release] [--sign IDENTITY] [--notarize]
#
# Options:
#   --release     Build in release mode (default: debug)
#   --sign ID     Code sign with the specified identity
#   --notarize    Submit for Apple notarization (requires --sign)
#   --dmg         Create DMG installer
#   --help        Show this help message

set -e

# Configuration
APP_NAME="Sol UI"
BUNDLE_ID="com.sol-ui.app"
EXECUTABLE_NAME="sol-ui"
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# Paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESOURCES_DIR="$PROJECT_ROOT/resources/macos"
BUILD_DIR="$PROJECT_ROOT/target"
OUTPUT_DIR="$PROJECT_ROOT/target/package"

# Defaults
BUILD_MODE="debug"
SIGN_IDENTITY=""
DO_NOTARIZE=false
CREATE_DMG=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --sign)
            SIGN_IDENTITY="$2"
            shift 2
            ;;
        --notarize)
            DO_NOTARIZE=true
            shift
            ;;
        --dmg)
            CREATE_DMG=true
            shift
            ;;
        --help)
            head -20 "$0" | tail -15
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "=== Sol UI macOS Packager ==="
echo "Version: $VERSION"
echo "Build mode: $BUILD_MODE"
echo ""

# Build the application
echo "Building $EXECUTABLE_NAME..."
if [ "$BUILD_MODE" = "release" ]; then
    cargo build --release
    BINARY_PATH="$BUILD_DIR/release/$EXECUTABLE_NAME"
else
    cargo build
    BINARY_PATH="$BUILD_DIR/debug/$EXECUTABLE_NAME"
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# App bundle structure
APP_BUNDLE="$OUTPUT_DIR/$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_APP_DIR="$CONTENTS_DIR/Resources"

echo "Creating app bundle at $APP_BUNDLE..."

# Clean previous bundle
rm -rf "$APP_BUNDLE"

# Create directory structure
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_APP_DIR"

# Copy executable
cp "$BINARY_PATH" "$MACOS_DIR/$EXECUTABLE_NAME"
chmod +x "$MACOS_DIR/$EXECUTABLE_NAME"

# Copy Info.plist
cp "$RESOURCES_DIR/Info.plist" "$CONTENTS_DIR/Info.plist"

# Update version in Info.plist
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$CONTENTS_DIR/Info.plist" 2>/dev/null || true

# Copy app icon if it exists
if [ -f "$RESOURCES_DIR/AppIcon.icns" ]; then
    cp "$RESOURCES_DIR/AppIcon.icns" "$RESOURCES_APP_DIR/AppIcon.icns"
    echo "App icon copied"
else
    echo "Warning: No app icon found at $RESOURCES_DIR/AppIcon.icns"
fi

# Create PkgInfo
echo -n "APPL????" > "$CONTENTS_DIR/PkgInfo"

echo "App bundle created successfully!"

# Code signing
if [ -n "$SIGN_IDENTITY" ]; then
    echo ""
    echo "Code signing with identity: $SIGN_IDENTITY"

    codesign --force --deep --sign "$SIGN_IDENTITY" \
        --entitlements "$RESOURCES_DIR/entitlements.plist" \
        --options runtime \
        "$APP_BUNDLE"

    # Verify signature
    codesign --verify --verbose "$APP_BUNDLE"
    echo "Code signing complete!"

    # Notarization
    if [ "$DO_NOTARIZE" = true ]; then
        echo ""
        echo "Submitting for notarization..."

        # Create zip for notarization
        NOTARIZE_ZIP="$OUTPUT_DIR/$APP_NAME-notarize.zip"
        ditto -c -k --keepParent "$APP_BUNDLE" "$NOTARIZE_ZIP"

        echo "Submit the zip file to Apple for notarization:"
        echo "  xcrun notarytool submit $NOTARIZE_ZIP --keychain-profile 'AC_PASSWORD' --wait"
        echo ""
        echo "After notarization completes, staple the ticket:"
        echo "  xcrun stapler staple '$APP_BUNDLE'"
    fi
else
    echo ""
    echo "Ad-hoc signing for local development..."
    codesign --force --deep --sign - "$APP_BUNDLE"
    echo "Ad-hoc signing complete!"
fi

# Create DMG
if [ "$CREATE_DMG" = true ]; then
    echo ""
    echo "Creating DMG installer..."

    DMG_NAME="$APP_NAME-$VERSION.dmg"
    DMG_PATH="$OUTPUT_DIR/$DMG_NAME"

    # Remove existing DMG
    rm -f "$DMG_PATH"

    # Create temporary directory for DMG contents
    DMG_TEMP="$OUTPUT_DIR/dmg-temp"
    rm -rf "$DMG_TEMP"
    mkdir -p "$DMG_TEMP"

    # Copy app to temp directory
    cp -R "$APP_BUNDLE" "$DMG_TEMP/"

    # Create Applications symlink
    ln -s /Applications "$DMG_TEMP/Applications"

    # Create DMG
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$DMG_TEMP" \
        -ov -format UDZO \
        "$DMG_PATH"

    # Clean up
    rm -rf "$DMG_TEMP"

    echo "DMG created: $DMG_PATH"

    # Sign DMG if we have an identity
    if [ -n "$SIGN_IDENTITY" ]; then
        codesign --force --sign "$SIGN_IDENTITY" "$DMG_PATH"
        echo "DMG signed!"
    fi
fi

echo ""
echo "=== Packaging Complete ==="
echo "App bundle: $APP_BUNDLE"
[ "$CREATE_DMG" = true ] && echo "DMG: $OUTPUT_DIR/$APP_NAME-$VERSION.dmg"
echo ""
echo "To run the app:"
echo "  open '$APP_BUNDLE'"
echo ""
echo "To verify the signature:"
echo "  codesign --verify --verbose '$APP_BUNDLE'"
