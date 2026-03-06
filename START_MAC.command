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

QUARANTINE_REMOVED=false

if xattr -dr com.apple.quarantine "$APP_PATH" 2>/dev/null; then
  QUARANTINE_REMOVED=true
  echo "Quarantine attribute removed."
else
  echo "Standard removal failed (likely permission issue)."
  echo "Trying elevated Gatekeeper workaround..."

  ELEVATED_CMD="xattr -dr com.apple.quarantine \"$APP_PATH\"; spctl --add --label ScriptRunner \"$APP_PATH\""
  if osascript -e "do shell script \"$ELEVATED_CMD\" with administrator privileges" >/dev/null 2>&1; then
    QUARANTINE_REMOVED=true
    echo "Quarantine removed with admin privileges and app added to Gatekeeper exceptions."
  else
    echo "Could not remove quarantine automatically."
    echo "Run this manually in Terminal:"
    echo "sudo xattr -dr com.apple.quarantine \"$APP_PATH\""
    echo "sudo spctl --add --label ScriptRunner \"$APP_PATH\""
  fi
fi

if [ "$QUARANTINE_REMOVED" = true ]; then
  echo "Verifying Gatekeeper status..."
  spctl -a -vv "$APP_PATH" || true
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