#!/usr/bin/env bash
# Famulus Games – FFI-Bindings erzeugen (v0.2.1).
# Baut den Kern als statische Bibliothek und generiert daraus
# die Swift-Bindings (UniFFI) nach swift-app/Generated.
# Läuft automatisch in build-app.sh; hier einzeln aufrufbar.

set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."

echo "==> Baue Kern (Release, staticlib)..."
cargo build --release -p famulus-games-core

LIB="target/release/libfamulus_games.a"
OUT="swift-app/Generated"
if [ ! -f "$LIB" ]; then
    echo "Fehler: $LIB wurde nicht gebaut." >&2
    exit 1
fi

mkdir -p "$OUT"
echo "==> Erzeuge Swift-Bindings via uniffi-bindgen..."
# generate erzeugt famulus_games.swift, famulus_gamesFFI.h und die
# famulus_gamesFFI.modulemap in einem Rutsch.
./target/release/uniffi-bindgen generate \
    -l swift \
    -o "$OUT" \
    src/ffi.udl \
    --crate famulus_games

echo "Fertig: $OUT/"
ls "$OUT"
