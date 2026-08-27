#!/usr/bin/env bash
# Famulus Games installieren (Weiterleitung).
# Die Tauri-Variante wurde durch die native SwiftUI-Huelle ersetzt;
# das eigentliche Bauen + Installieren macht build-app.sh.
set -euo pipefail
exec "$(dirname "$0")/build-app.sh" "$@"
