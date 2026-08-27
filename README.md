# Famulus Games

Vereinte Spiele-Bibliothek und Launcher – liest **Steam** und
**Heroic (GOG)** ein, zeigt alles in einem Fenster, startet per Klick.
Teil des [[Famulus]]-Ökosystems: gleiche Sprache (Rust), gleiche
Oberflächen-DNA (dunkles Terminal-Design, gelbe Akzentfarbe #FFC53D).

Seit v0.2.0 ist die Oberfläche eine **native SwiftUI-App** statt Tauri –
der Rust-Kern wird über UniFFI statisch in die App eingebunden.

**MVP-Bewusstsein:** Nur Starten, kein Installieren/Deinstallieren
(laut Beschluss: Heroic wird ersetzt, nicht überlagert – Installation
bleibt bis auf Weiteres seine Sache).

## Was es kann

- Steam: liest `libraryfolders.vdf` + `appmanifest_*.acf`
  (alle Bibliotheken, nur vollständig installierte Spiele,
  StateFlags = 4), Cover aus `librarycache/<appid>`.
- Heroic/GOG: liest `gog_store/installed.json` (wer installiert ist,
  wohin, wie groß) + `store_cache/gog_library.json` (Titel, Cover-URL).
  Cover werden einmalig in den eigenen Cache geladen.
- Starten: Steam über `steam://rungameid/<appid>` (Steam selbst
  übernimmt), GOG-Spiele direkt über `open <pfad>`.
- Suche, Quellen-Filter (Alle/Steam/GOG), Neu-einlesen-Knopf.

## Bau & Installation

```bash
./scripts/build-app.sh   # Kern + Bindings + SwiftUI-App → /Applications → starten
```

Einzelschritte:

```bash
./scripts/build-ffi.sh                       # Rust-Kern + Swift-Bindings (UniFFI)
cd swift-app && xcodegen generate            # Xcode-Projekt aus project.yml
cargo test -p famulus-games-core             # Kern-Tests
```

## Struktur

```
src/            Kern-Bibliothek (Steam- und Heroic-Leser, Typ Spiel)
src/bridge.rs   UniFFI-Brücke: Cover-Download, Spielstart, Formatierung
src/ffi.udl     FFI-Vertrag zwischen Rust und Swift
swift-app/      Native SwiftUI-Hülle (XcodeGen: project.yml)
swift-app/Generated/  Von UniFFI erzeugte Swift-Bindings
scripts/        build-ffi.sh, build-app.sh, install-mac.sh
```

## Bewusst nicht (Roadmap-Kandidaten)

- Installieren/Deinstallieren/Aktualisieren (Heroic-Ersatz, Phase 2)
- Spielzeit-Tracking, Cloud-Saves, Wunschliste
- Linux/Windows – Mac zuerst, Struktur lässt Platz
