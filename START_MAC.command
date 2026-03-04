#!/bin/bash

set -u

APP_NAME="ScriptRunner.app"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

APP_PATH=""
if [ -d "/Applications/${APP_NAME}" ]; then
  APP_PATH="/Applications/${APP_NAME}"
elif [ -d "$HOME/Applications/${APP_NAME}" ]; then
  APP_PATH="$HOME/Applications/${APP_NAME}"
elif [ -d "${SCRIPT_DIR}/${APP_NAME}" ]; then
  APP_PATH="${SCRIPT_DIR}/${APP_NAME}"
fi

echo "========================================"
echo " ScriptRunner macOS Starter"
echo "========================================"

echo ""
if [ -z "$APP_PATH" ]; then
  echo "ScriptRunner.app not found."
  echo ""
  echo "1) Open the DMG file"
  echo "2) Drag ScriptRunner.app to Applications"
  echo "3) Run this START_MAC.command again"
  echo ""
  read -r -p "Press Enter to close..."
  exit 1
fi

echo "App found at: $APP_PATH"
echo "Removing quarantine attribute (Gatekeeper workaround)..."

if xattr -dr com.apple.quarantine "$APP_PATH"; then
  echo "Quarantine attribute removed."
else
  echo "Could not remove quarantine attribute automatically."
  echo "You may need to run this script with elevated permissions."
fi

echo ""
echo "Launching ScriptRunner..."
if open "$APP_PATH"; then
  echo "Done."
else
  echo "Failed to launch app. Try opening it manually from Applications."
fi

echo ""
read -r -p "Press Enter to close..."