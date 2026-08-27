# Changelog

Alle nennenswerten Änderungen an Famulus Games werden hier dokumentiert.

Das Format basiert auf [Keep a Changelog](https://keepachangelog.com/de-DE/1.1.0/),
die Versionierung folgt [Semantischer Versionierung](https://semver.org/lang/de/).

## [0.2.3] – 2026-08-27

### Geändert
- **Gleicher Style wie der Famulus-Kern**: Die alte Schwarz/Gelb-
  Palette ist raus; Famulus Games übernimmt jetzt die warme Marken-
  DNA: Braun oben, durchgängiger Verlauf bis Schwarz unten, Creme-
  Text, Orange-Akzent `#F86E27` (Hover `#FF8A4A`). Kopfzeile braun
  und teiltransparent, Fußzeile fast schwarz – wie Toolbar/Statusbar
  im Kern. Alle Famulus-Produkte sollen den selben Style haben.

### Hinzugefügt
- **App-Icon**: „FG" in Menlo Bold und Orange auf Braun-Schwarz-
  Verlauf – gleiche DNA wie das Famulus-Icon („F", Gelb). Neuer
  Asset Catalog (`swift-app/FamulusGames/Assets.xcassets`),
  XcodeGen zieht ihn automatisch mit.

## [0.2.2] – 2026-08-27

### Behoben
- **GOG-Cover wurden im Grid stark abgeschnitten**: Der Heroic-Leser
  nahm `art_cover` – das ist ein Quer-Banner (1600×740, ~2.2:1), der
  im 2:3-Grid links und rechts massiv beschnitten wurde. Jetzt wird
  `art_square` bevorzugt (Hochformat, ~342×482, nah an 2:3);
  `art_cover` bleibt Fallback. Steam-Capsules (300×450) waren nie
  betroffen.
- Alten Cover-Cache geleert, damit die Quer-Banner nicht liegen
  bleiben (`~/Library/Application Support/famulus-games/covers/`).

### Hinzugefügt
- `examples/probe.rs`: Debug-Werkzeug, das zeigt, was der Kern
  liefert (`cargo run --example probe`) – Titel, Quelle, Pfade,
  Cover, Größe.

## [0.2.1] – 2026-08-27

### Entfernt
- Reste der Tauri-Variante (`gui/`, `ui/`) aus dem Repository
  entfernt; Archiv liegt als Zip in Google Drive unter
  `Meine Ablage/Famulus-Archive/famulus-games-tauri-archiv.zip`.

### Geändert
- `scripts/install-mac.sh`: leitet jetzt auf `build-app.sh` weiter
  (der dortige Tauri-Build-Aufruf war nach dem Aufräumen tot).
- `README.md`: Strukturblock ohne die gelöschten Ordner.

## [0.2.0] – 2026-08-27

### Neu
- **Native SwiftUI-Hülle (`swift-app/`)**: ersetzt die Tauri-2-App.
  Gleiche Optik (dunkles Terminal-Design, Famulus-Gelb `#FFC53D`),
  aber ohne WebView und ohne WebView-Runtime – die App ist jetzt ein
  natives macOS-Programm.
- **UniFFI-Brücke (`src/bridge.rs`)**: der Rust-Kern wird als statische
  Bibliothek gebaut und per UniFFI (v0.29) an Swift angebunden.
  Bindings werden per `scripts/build-ffi.sh` erzeugt.
- **Neue Build-Skripte**:
  - `scripts/build-ffi.sh` – Kern + Swift-Bindings erzeugen
  - `scripts/build-app.sh` – App bauen + signieren + atomar installieren

### Geändert
- `Cargo.toml`: `crate-type = ["lib", "staticlib"]` für den Kern
  (statisches Einbinden in Swift), `[build-dependencies]` mit `uniffi`.
- `src/lib.rs`: `uniffi::include_scaffolding!` + Re-Exporte für die
  UDL-Funktionen (UniFFI erwartet das Scaffolding im Crate-Root).
- `src/ffi.udl`: UDL-Datei liegt jetzt direkt in `src/` (UniFFI findet
  die `Cargo.toml` nur zwei Ordnerebenen über der Datei).
- `[profile.release]`: `strip = "none"` damit die FFI-Symbole in der
  statischen Bibliothek erhalten bleiben (sonst kann Swift nicht linken).
- `bridge.rs::http_get`: nutzt jetzt `curl -fsSL` statt eines eigenen
  TCP-Clients (auf macOS immer vorhanden, HTTPS-fähig, Redirects).

### Entfernt
- Tauri-2-Abhängigkeit für die GUI. Die `gui/`- und `ui/`-Ordner
  wurden archiviert (Zip in Google Drive: Meine Ablage/Famulus-Archive/
  famulus-games-tauri-archiv.zip) und anschließend gelöscht.
- `scripts/install-mac.sh` leitet jetzt auf `build-app.sh` weiter.

### Intern
- `src/bin/uniffi-bindgen.rs`: Einstiegspunkt für den UniFFI-Generator,
  wird beim Build der statischen Bibliothek mit erzeugt.
- Swift-Projekt per XcodeGen (`swift-app/project.yml`) – kein manuell
  gepflegtes `.xcodeproj` mehr.
- Sandbox und Hardened Runtime sind bewusst deaktiviert (App liest
  Steam-/Heroic-Ordner und startet Prozesse per `open`).

## [0.1.0] – 2026-08-27

### Neu
- Steam-Leser: `libraryfolders.vdf` + `appmanifest_*.acf`
  (alle Bibliotheken, nur installierte Spiele, StateFlags = 4).
- Heroic/GOG-Leser: `gog_store/installed.json` + `store_cache/gog_library.json`.
- Tauri-2-GUI: Bibliotheks-Grid mit Cover, Suchfeld, Quellen-Filter,
  Starten-Knopf, Toast-Meldungen.
- Starten: Steam über `steam://rungameid/<appid>`, GOG direkt über `open`.
