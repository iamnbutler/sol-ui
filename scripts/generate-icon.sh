#!/bin/bash
# Generate a placeholder app icon for Sol UI
#
# This creates a simple icon using ImageMagick or sips.
# For production, replace with a proper designed icon.
#
# Usage:
#   ./scripts/generate-icon.sh [input.png]
#
# If no input is provided, creates a placeholder gradient icon.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/resources/macos"
ICONSET_DIR="$OUTPUT_DIR/AppIcon.iconset"
OUTPUT_ICNS="$OUTPUT_DIR/AppIcon.icns"

# Icon sizes required for macOS
SIZES=(16 32 64 128 256 512 1024)

echo "=== Sol UI Icon Generator ==="

# Create iconset directory
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

INPUT_IMAGE="$1"

if [ -z "$INPUT_IMAGE" ]; then
    echo "No input image provided, generating placeholder..."

    # Check if ImageMagick is available
    if command -v magick &> /dev/null; then
        echo "Using ImageMagick to generate placeholder icon..."

        # Generate a gradient icon with "Sol" text
        magick -size 1024x1024 \
            -define gradient:angle=135 \
            gradient:'#4A90D9-#7B68EE' \
            -gravity center \
            -font Helvetica-Bold \
            -pointsize 400 \
            -fill white \
            -annotate 0 'S' \
            "$ICONSET_DIR/icon_512x512@2x.png"

        INPUT_IMAGE="$ICONSET_DIR/icon_512x512@2x.png"
    else
        echo "ImageMagick not found. Please install it or provide a 1024x1024 PNG:"
        echo "  brew install imagemagick"
        echo ""
        echo "Or run with an existing image:"
        echo "  ./scripts/generate-icon.sh /path/to/icon.png"
        exit 1
    fi
fi

# Verify input image exists
if [ ! -f "$INPUT_IMAGE" ]; then
    echo "Error: Input image not found: $INPUT_IMAGE"
    exit 1
fi

echo "Generating icon sizes from: $INPUT_IMAGE"

# Generate all required sizes
for SIZE in "${SIZES[@]}"; do
    # Standard resolution
    sips -z $SIZE $SIZE "$INPUT_IMAGE" --out "$ICONSET_DIR/icon_${SIZE}x${SIZE}.png" 2>/dev/null

    # Retina resolution (2x)
    RETINA_SIZE=$((SIZE * 2))
    if [ $RETINA_SIZE -le 1024 ]; then
        sips -z $RETINA_SIZE $RETINA_SIZE "$INPUT_IMAGE" --out "$ICONSET_DIR/icon_${SIZE}x${SIZE}@2x.png" 2>/dev/null
    fi
done

# Special case: 512@2x is 1024
cp "$INPUT_IMAGE" "$ICONSET_DIR/icon_512x512@2x.png" 2>/dev/null || true
sips -z 1024 1024 "$ICONSET_DIR/icon_512x512@2x.png" --out "$ICONSET_DIR/icon_512x512@2x.png" 2>/dev/null

echo "Converting to icns..."
iconutil -c icns "$ICONSET_DIR" -o "$OUTPUT_ICNS"

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo ""
echo "=== Icon Generated ==="
echo "Output: $OUTPUT_ICNS"
echo ""
echo "Note: This is a placeholder icon. For production, replace with a"
echo "properly designed icon and re-run the packaging script."
