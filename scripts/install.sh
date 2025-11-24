#!/bin/bash

set -e

echo "Building Discord RPC plugin for DeaDBeeF..."
cargo build --release

# Detect OS
OS="$(uname -s)"

case "${OS}" in
Linux*)
  echo "Detected Linux"
  PLUGIN_DIR="$HOME/.local/lib/deadbeef"
  EXTENSION="so"
  # Find the compiled .so file
  PLUGIN_FILE=$(find target/release -maxdepth 1 -name "*.so" -type f | head -n 1)
  ;;
Darwin*)
  echo "Detected macOS"
  PLUGIN_DIR="$HOME/Library/Application Support/Deadbeef/Plugins"
  EXTENSION="dylib"
  # Find the compiled .dylib file
  PLUGIN_FILE=$(find target/release -maxdepth 1 -name "*.dylib" -type f | head -n 1)
  ;;
*)
  echo "Unsupported OS: ${OS}"
  exit 1
  ;;
esac

if [ -z "$PLUGIN_FILE" ]; then
  echo "Error: Could not find compiled plugin file in target/release/"
  exit 1
fi

echo "Found plugin file: $PLUGIN_FILE"

# Create plugin directory if it doesn't exist
if [ ! -d "$PLUGIN_DIR" ]; then
  echo "Creating plugin directory: $PLUGIN_DIR"
  mkdir -p "$PLUGIN_DIR"
fi

# Copy and rename the plugin
TARGET_FILE="$PLUGIN_DIR/discordrpc.$EXTENSION"
echo "Installing plugin to: $TARGET_FILE"
cp "$PLUGIN_FILE" "$TARGET_FILE"

echo "Installation complete!"
echo "Plugin installed to: $TARGET_FILE"
echo ""
echo "Please restart DeaDBeeF to load the plugin."
