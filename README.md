# Famulus Games

Vereinte Spiele-Bibliothek und Launcher – liest **Steam** und
**Heroic (GOG)** ein, zeigt alles in einem Fenster, startet per Klick.
Teil des [[Famulus]]-Ökosystems: gleiche Sprache (Rust), gleiche
Oberflächen-DNA (Tauri 2, dunkles Terminal-Design, gelbe Akzentfarbe).

**MVP-Bewusstsein:** Nur Starten, kein Installieren/Deinstallieren
(laut Beschluss: Heroic wird ersetzt, nicht überlagert – Installation
bleibt bis auf Weiteres seine Sache).

## Was es kann (v0.1.0)

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
./scripts/install-mac.sh   # baut Release + signiert + nach /Applications + startet
```

oder nur bauen:

```bash
cargo tauri build --bundles app -c gui/tauri.conf.json
```

Tests: `cargo test -p famulus-games-core` (10 Tests, 0 Warnungen).

## Struktur

```
src/            Kern-Bibliothek (Steam- und Heroic-Leser, Typ Spiel)
gui/            Tauri-2-App (Kommandos: laden, starten, Cover)
ui/index.html   Oberfläche (wie Famulus: ein HTML, kein JS-Build)
scripts/        install-mac.sh
```

## Bewusst nicht (Roadmap-Kandidaten)

- Installieren/Deinstallieren/Aktualisieren (Heroic-Ersatz, Phase 2)
- Spielzeit-Tracking, Cloud-Saves, Wunschliste
- Linux/Windows – Mac zuerst, Struktur lässt Platz
