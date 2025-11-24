#!/bin/bash

set -e

echo "Uninstalling Discord RPC plugin for DeaDBeeF..."

# Detect OS
OS="$(uname -s)"

case "${OS}" in
Linux*)
  echo "Detected Linux"
  PLUGIN_DIR="$HOME/.local/lib/deadbeef"
  EXTENSION="so"
  ;;
Darwin*)
  echo "Detected macOS"
  PLUGIN_DIR="$HOME/Library/Application Support/Deadbeef/Plugins"
  EXTENSION="dylib"
  ;;
*)
  echo "Unsupported OS: ${OS}"
  exit 1
  ;;
esac

PLUGIN_FILE="$PLUGIN_DIR/discordrpc.$EXTENSION"

# Check if plugin exists
if [ ! -f "$PLUGIN_FILE" ]; then
  echo "Plugin not found at: $PLUGIN_FILE"
  echo "Nothing to uninstall."
  exit 0
fi

# Remove the plugin
echo "Removing plugin: $PLUGIN_FILE"
rm "$PLUGIN_FILE"

if [ $? -eq 0 ]; then
  echo "Uninstallation complete!"
  echo "Plugin removed from: $PLUGIN_FILE"
  echo ""
  echo "Please restart DeaDBeeF for changes to take effect."
else
  echo "Failed to remove plugin."
  exit 1
fi
