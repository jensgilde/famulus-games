#!/usr/bin/env bash
# Baut Famulus Games und installiert es nach /Applications.
# Bewusst einfacher als install-mac.sh vom Kern-Famulus: kein
# laufender Dienst, der im Weg stehen koennte – atomarer Swap reicht.

set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."

echo "==> Baue Famulus Games (Release, signiert)..."
cargo tauri build --bundles app -c gui/tauri.conf.json

BUNDLE="target/release/bundle/macos/Famulus Games.app"
DEST="/Applications/Famulus Games.app"
BACKUP="/Applications/.Famulus Games.app.vorherige-version"

if [ ! -d "$BUNDLE" ]; then
    echo "Fehler: $BUNDLE wurde nicht gebaut." >&2
    exit 1
fi

echo "==> Pruefe Signatur des frischen Builds..."
codesign --verify --deep --strict --verbose=4 "$BUNDLE"

echo "==> Installiere atomar nach $DEST..."
rm -rf "$BACKUP"
if [ -d "$DEST" ]; then
    mv "$DEST" "$BACKUP"
fi
mv "$BUNDLE" "$DEST"
rm -rf "$BACKUP"

echo "==> Pruefe installiertes Bundle..."
codesign --verify --deep --strict --verbose=4 "$DEST"

echo "==> Starte Famulus Games..."
open -a "Famulus Games"

echo "Fertig."
