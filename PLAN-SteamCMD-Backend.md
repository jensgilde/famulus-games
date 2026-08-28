# Plan: Famulus Games als SteamCMD-Backend (Option A)

Stand: 2026-08-28
Ziel: Steam-Downloads/Updates über SteamCMD headless, Steam-Client nur für
Cloud-Save-Sync, Heroic durch gogdl/GOG-API ersetzen. Nur Mac-Spiele, kein Wine.

## Ist-Zustand (verifiziert)

| Komponente | Status |
|---|---|
| SteamCMD | installiert (Homebrew cask) |
| Steam-Client | installiert, startet bei Bedarf |
| Heroic | installiert, liefert GOG-Bibliothek |
| gogdl / legendary | nicht installiert |
| Famulus Games v0.2.5 | Phoenix-Style, liest Steam-Manifeste + Heroic-Cache |
| Launch-Logik | Steam: steam://rungameid, GOG: open <pfad> |

## Phase 1 – SteamCMD-Backend für Steam

Ziel: Steam-Bibliothek nicht mehr über Steam-Manifeste lesen, sondern
über SteamCMD. Downloads/Updates laufen headless.

Aufgaben:
1. SteamCMD-Login speichern (einmalig; Steam-Guard-Code beim ersten
   Login interaktiv nötig, danach cached SteamCMD die Session)
2. Bibliothek lesen: bleibt wie heute (Manifest-Parsing in src/steam.rs
   funktioniert ohne Client – App-IDs kommen aus den acf-Dateien)
3. Download/Update headless: `steamcmd +login <user> +app_update <appid> validate +quit`
   in einen eigenen steamapps-Ordner (eigener Steam-Pfad via +set_steam_cmd)
4. Fortschritt anzeigen (SteamCMD schreibt Prozent auf stdout, parsen)
5. Launch bleibt: steam://rungameid (Client startet nur dafür kurz)

Offene Fragen (verifizieren, bevor gebaut wird):
- SteamCMD auf macOS: cached es den Login wirklich ohne den Client?
  (Das ist ein Test mit deinem Account – nicht aus der Doku belegbar.)
- Speichert SteamCMD die Spiele im selben steamapps-Ordner wie der
  Client, damit Launch und Manifest-Parsing weiter funktionieren?

Aufwand: 1–2 Tage

## Phase 2 – Heroic entfernen, GOG direkt

Ziel: GOG-Bibliothek nicht mehr über Heroic-Cache lesen, sondern direkt
über gogdl oder GOG-API.

Aufgaben:
1. gogdl installieren (pip oder GitHub-Release)
2. GOG-Login: `gogdl login <email>` (speichert Token)
3. Bibliothek auflisten: `gogdl list --platform osx`
4. Download: `gogdl download <id> --platform osx`
5. Launch: open <pfad> (bleibt)
6. Heroic-App entfernen, Heroic-Cache löschen

Risiko: gogdl ist ein Community-Tool, API kann brechen. Alternative:
GOG Galaxy API direkt (aufwendiger).

Aufwand: 2–3 Tage

## Phase 3 – Cloud-Save-Sync über Steam-Client

Ziel: Nach dem Spielen den Steam-Client kurz starten, damit Saves syncen.

Aufgaben:
1. Beim Start von Famulus Games: `open -g -a Steam` (unsichtbar, ohne Fenster)
   – der Client synchronisiert dann die Saves im Hintergrund
2. Optional: Nach dem Spielende erneut kurz anstoßen
3. Steam-Client so konfigurieren, dass er sich nicht in den Vordergrund drängt
   (Steam → Einstellungen → Interface: „beim Start minimieren", kein Autostart-Fenster)

Offene Frage (verifizieren, bevor gebaut wird):
- Wann genau synchronisiert der Steam-Client Cloud-Saves? Beim Client-Start,
  beim Spielstart oder nach dem Beenden des Spiels? Das entscheidet, ob
  „kurz beim Hochfahren starten" reicht oder ob wir nach jedem Spielende
  anstoßen müssen.

Risiko: Steam-Client muss installiert bleiben. „Client komplett weg" ist
technisch nicht möglich mit Cloud-Saves (Valve-Entscheidung).

Aufwand: 1 Tag

## Phase 4 – Vereinheitlichte Oberfläche

Ziel: Eine Bibliothek für Steam + GOG, ein UI.

Aufgaben:
1. Bibliothek zusammenführen (Steam + GOG)
2. Download-Manager (Progress, Abbrechen)
3. Update-Check (SteamCMD app_status, gogdl list)
4. Cloud-Save-Status anzeigen

Aufwand: 3–5 Tage

## Gesamt

| Phase | Aufwand | Risiko |
|---|---|---|
| 1. SteamCMD-Backend | 1–2 Tage | Steam Guard blockiert headless-Login |
| 2. GOG direkt (gogdl) | 2–3 Tage | Community-Tool, API kann brechen |
| 3. Cloud-Save-Sync | 1 Tag | Steam-Client muss bleiben |
| 4. Vereinheitlichte UI | 3–5 Tage | – |
| **Gesamt** | **7–11 Tage** | – |

## Entscheidungspunkte (vor Umsetzung)

1. **SteamCMD-Login testen**: Funktioniert headless-Login mit deinem
   Steam-Account? (Steam Guard kann blockieren)
2. **gogdl vs. GOG-API**: Community-Tool oder offizielle API?
3. **Heroic entfernen**: Vorher Backup der GOG-Installationen?

## Nächster Schritt

Phase 1 starten: SteamCMD-Login testen mit deinem Account.
Dazu brauche ich: Steam-Benutzername + Steam-Guard-Setup (2FA?).
