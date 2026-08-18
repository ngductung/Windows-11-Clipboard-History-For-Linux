#!/usr/bin/env bash
set -euo pipefail

UUID="win11-clipboard-history@gustavosett.dev"
SCHEMA="org.gnome.shell.extensions.win11-clipboard-history"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../gnome-extension/$UUID" && pwd)"
DEST_DIR="$HOME/.local/share/gnome-shell/extensions/$UUID"
APP_COMMAND="${1:-win11-clipboard-history}"

if ! command -v glib-compile-schemas >/dev/null 2>&1; then
  echo "glib-compile-schemas not found. Install libglib2.0-bin / glib2 tools first." >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
cp -R "$SRC_DIR"/. "$DEST_DIR"/
glib-compile-schemas "$DEST_DIR/schemas"

GSETTINGS_SCHEMA_DIR="$DEST_DIR/schemas" gsettings set "$SCHEMA" command "$APP_COMMAND"

if [ "${SKIP_GNOME_KEYBINDING_FIX:-0}" != "1" ] && command -v gsettings >/dev/null 2>&1; then
  if gsettings get org.gnome.shell.keybindings toggle-message-tray 2>/dev/null | grep -qi "<Super>v"; then
    gsettings set org.gnome.shell.keybindings toggle-message-tray "['<Super><Shift>v']"
  fi

  OLD_SHORTCUT_SCHEMA="org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/win11-clipboard-history/"
  if gsettings writable "$OLD_SHORTCUT_SCHEMA" binding >/dev/null 2>&1; then
    gsettings set "$OLD_SHORTCUT_SCHEMA" binding "''" || true
  fi
fi

if command -v gnome-extensions >/dev/null 2>&1; then
  gnome-extensions enable "$UUID" || {
    echo "Extension installed, but GNOME Shell may need a logout/login before it can be enabled."
  }
else
  echo "gnome-extensions command not found. Enable $UUID from the Extensions app after logging in again."
fi

echo "Installed $UUID"
echo "On Wayland, log out and back in if Super+V does not trigger immediately."
