#!/usr/bin/env bash
# Famulus Games – native SwiftUI-App bauen + installieren (v0.2.5).
# Atomarer Swap nach /Applications wie bei der Tauri-Variante,
# aber für die native Swift-Hülle (swift-app/).
#
# Ablauf:
#   1. Rust-Kern als staticlib bauen
#   2. Swift-Bindings erzeugen (UniFFI)
#   3. Xcode-Projekt generieren (xcodegen)
#   4. App bauen (xcodebuild Release)
#   5. signieren, atomar installieren, starten
#
# WICHTIG – stabile Codesignatur für macOS-Ordnerfreigaben (TCC):
#   macOS merkt sich Ordner-Freigaben anhand der Codesignatur-Identität.
#   Bei AD-HOC-Signatur ist die Identität der reine Code-Hash (cdhash) –
#   und der ändert sich mit JEDEM Build. Folge: macOS hält jede neue
#   Version für ein "anderes Programm" und vergisst alle erteilten
#   Freigaben; der Nutzer muss jedes Mal neu freigeben.
#   Abhilfe: mit dem echten Entwickler-Zertifikat signieren. Dann ist die
#   Identität (Zertifikat + Bundle-ID) über Builds hinweg stabil und die
#   Freigaben bleiben dauerhaft erhalten.

set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH:/opt/homebrew/bin"
cd "$(dirname "$0")/.."

echo "==> 1/5 Kern + Bindings bauen..."
./scripts/build-ffi.sh

echo "==> 2/5 Xcode-Projekt generieren..."
(cd swift-app && xcodegen generate)

echo "==> 3/5 App bauen (Release)..."
(cd swift-app && xcodebuild \
    -project FamulusGames.xcodeproj \
    -scheme FamulusGames \
    -configuration Release \
    -derivedDataPath build CODE_SIGNING_ALLOWED=NO >/dev/null)

BUNDLE="swift-app/build/Build/Products/Release/Famulus Games.app"
DEST="/Applications/Famulus Games.app"
BACKUP="/Applications/.Famulus Games.app.vorherige-version"

if [ ! -d "$BUNDLE" ]; then
    echo "Fehler: $BUNDLE wurde nicht gebaut." >&2
    exit 1
fi

echo "==> 4/5 Signiere und installiere atomar nach $DEST..."
# Signatur mit dem echten Entwickler-Zertifikat (stabile Identität für
# macOS-Ordnerfreigaben). Fallback auf Ad-hoc nur wenn kein Zertifikat
# vorhanden – dann gehen Freigaben bei jedem Build verloren.
SIGN_IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
    | grep -o '"Apple Development: [^"]*"' | head -1 | tr -d '"')

if [ -n "$SIGN_IDENTITY" ]; then
    echo "Signatur-Identität: $SIGN_IDENTITY"
    codesign --force --deep --sign "$SIGN_IDENTITY" "$BUNDLE" 2>&1 \
        | grep -v "replacing existing signature" || true
else
    echo "WARNUNG: Kein Entwickler-Zertifikat gefunden – Ad-hoc-Signatur."
    echo "         Ordner-Freigaben gehen bei jedem Build verloren!"
    codesign --force --deep --sign - "$BUNDLE" >/dev/null 2>&1 || true
fi

# Quarantäne-Attribut entfernen, falls vorhanden (lokal gebaut → unnötig)
xattr -dr com.apple.quarantine "$BUNDLE" 2>/dev/null || true

rm -rf "$BACKUP"
if [ -d "$DEST" ]; then
    mv "$DEST" "$BACKUP"
fi
mv "$BUNDLE" "$DEST"
rm -rf "$BACKUP"

echo "==> 5/5 Starte Famulus Games..."
open -a "Famulus Games"
echo "Fertig."
