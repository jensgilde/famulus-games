// Famulus Games – SteamCMD-Backend v0.3.0.
// Headless-Downloads/Updates über SteamCMD, im exakten Layout des
// Steam-Clients. Die Strategie ist am 2026-08-28 mit App 480
// empirisch verifiziert (siehe PLAN-SteamCMD-Backend.md):
//
// 1. SteamCMD schreibt Spieldateien direkt ins force_install_dir.
//    Deshalb setzen wir force_install_dir auf den echten Spielordner
//    <steamapps>/common/<installdir> – die Dateien landen dann genau
//    dort, wo der Client sie erwartet.
// 2. Das Manifest schreibt SteamCMD nach <ziel>/steamapps/. Wir
//    kopieren das Client-Manifest vorher dorthin (damit Updates
//    inkrementell sind – „already up to date" beim zweiten Lauf)
//    und bewegen es nachher zurück an den Client-Platz.
// 3. SteamCMD setzt LastPlayed im Manifest auf 0 zurück – wir
//    sichern den Wert vorher und stellen ihn nachher wieder her.
// 4. Login: einmalig interaktiv (Passwort + Steam-Guard-Code über
//    stdin). SteamCMD cached die Session danach selbst.
//
// SteamCMD liest die Anmeldedaten des Clients NICHT
// („Cached credentials not found", verifiziert) – eigener Login nötig.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

use crate::bridge::Fehler;

// ---------------------------------------------------------------- Pfade

/// Kandidaten für das SteamCMD-Programm (Homebrew ARM/Intel).
const STEAMCMD_PFADE: [&str; 2] = ["/opt/homebrew/bin/steamcmd", "/usr/local/bin/steamcmd"];

/// Das SteamCMD-Programm, falls installiert.
pub fn steamcmd_programm() -> Option<PathBuf> {
    STEAMCMD_PFADE.iter().map(PathBuf::from).find(|p| p.exists())
}

/// Das SteamCMD-Stammverzeichnis (STEAMROOT des Wrapper-Skripts):
/// der Ordner, in dem SteamCMD seine eigene config/loginusers.vdf
/// anlegt. Der Homebrew-Symlink zeigt ins Caskroom; zwei Ebenen
/// darüber liegt MacOS/, das steamcmd.sh als STEAMROOT nutzt.
fn steamcmd_wurzel() -> Option<PathBuf> {
    let link = steamcmd_programm()?;
    let ziel = std::fs::read_link(&link).unwrap_or(link);
    let version_dir = ziel.parent()?.parent()?;
    Some(version_dir.join("MacOS"))
}

// ---------------------------------------------------------------- Login-Zustand

/// Ist der Nutzer bei SteamCMD angemeldet? Prüft die loginusers.vdf,
/// die SteamCMD nach erfolgreichem Login in seinen STEAMROOT schreibt.
pub fn ist_angemeldet(benutzer: &str) -> bool {
    let Some(wurzel) = steamcmd_wurzel() else {
        return false;
    };
    let datei = wurzel.join("config/loginusers.vdf");
    let Ok(text) = std::fs::read_to_string(&datei) else {
        return false;
    };
    // Account-Name im VDF, mit etwas Toleranz für Groß/Klein.
    let name = benutzer.to_lowercase();
    text.lines().any(|z| {
        let z = z.trim().to_lowercase();
        z.contains(&format!("\"{name}\"")) && z.contains("accountname")
    })
}

// ---------------------------------------------------------------- Fortschritt

/// Zustand eines laufenden (oder abgeschlossenen) Downloads.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadStand {
    pub appid: String,
    /// "wartet", "laedt", "fertig", "fehler"
    pub phase: String,
    pub prozent: f64,
    pub meldung: String,
}

struct AktiverDownload {
    stand: DownloadStand,
    kind_pid: Option<u32>,
}

static AKTIV: Mutex<Option<AktiverDownload>> = Mutex::new(None);

/// Aktueller Download-Stand (None, wenn nie einer lief).
pub fn download_stand() -> Option<DownloadStand> {
    AKTIV.lock().ok()?.as_ref().map(|a| a.stand.clone())
}

/// Laufenden Download abbrechen (tötet den SteamCMD-Prozess).
pub fn download_stoppen() {
    let pid = AKTIV
        .lock()
        .ok()
        .and_then(|mut a| a.as_mut().and_then(|d| d.kind_pid.take()));
    if let Some(pid) = pid {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    if let Ok(mut a) = AKTIV.lock() {
        if let Some(d) = a.as_mut() {
            d.stand.phase = "fehler".into();
            d.stand.meldung = "Abgebrochen".into();
        }
    }
}

fn stand_phase(appid: &str, phase: &str, prozent: f64, meldung: &str) {
    let neu = DownloadStand {
        appid: appid.to_string(),
        phase: phase.to_string(),
        prozent,
        meldung: meldung.to_string(),
    };
    match AKTIV.lock() {
        Ok(mut a) if a.is_some() => {
            if let Some(d) = a.as_mut() {
                d.stand = neu;
            }
        }
        Ok(mut a) => {
            *a = Some(AktiverDownload { stand: neu, kind_pid: None });
        }
        Err(_) => {}
    }
}

// ---------------------------------------------------------------- Manifest-Helfer

/// Holt den LastPlayed-Wert aus einem Manifest-Text (None wenn keiner).
fn last_played_aus_manifest(text: &str) -> Option<String> {
    for zeile in text.lines() {
        let z = zeile.trim();
        if let Some(rest) = z.strip_prefix("\"LastPlayed\"") {
            let rest = rest.trim().trim_start_matches('"');
            if let Some(ende) = rest.find('"') {
                return Some(rest[..ende].to_string());
            }
        }
    }
    None
}

/// Setzt LastPlayed im Manifest-Text auf den gewünschten Wert.
fn last_played_setzen(text: &str, wert: &str) -> String {
    let mut erg = String::with_capacity(text.len());
    for zeile in text.lines() {
        if zeile.trim().starts_with("\"LastPlayed\"") {
            let einrueckung: String = zeile.chars().take_while(|c| *c == '\t').collect();
            erg.push_str(&format!("{einrueckung}\"LastPlayed\"\t\t\"{wert}\""));
        } else {
            erg.push_str(zeile);
        }
        erg.push('\n');
    }
    erg
}

// ---------------------------------------------------------------- Ausgabe-Parsing

/// Zerlegt eine SteamCMD-Ausgabezeile. Liefert (phase, prozent, meldung).
fn zeile_deuten(zeile: &str) -> Option<(String, f64, String)> {
    let z = zeile.trim();
    if let Some(i) = z.find("progress:") {
        let rest = z[i + "progress:".len()..].trim_start();
        let prozent: f64 = rest
            .split(|c: char| c == ' ' || c == '(')
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0.0);
        let phase = if z.contains("downloading") { "laedt" } else { "wartet" };
        return Some((phase.to_string(), prozent, z.to_string()));
    }
    if z.contains("Success!") {
        let meldung = if z.contains("already up to date") {
            "Bereits aktuell".to_string()
        } else {
            "Fertig".to_string()
        };
        return Some(("fertig".into(), 100.0, meldung));
    }
    if z.starts_with("ERROR") || z.contains("ERROR (") || z.contains("ERROR!") {
        return Some(("fehler".into(), 0.0, z.to_string()));
    }
    None
}

// ---------------------------------------------------------------- Update/Download

/// Startet ein Headless-Update (oder eine Erstinstallation) für ein
/// Steam-Spiel. `pfad` ist der Spielordner <steamapps>/common/<installdir>.
/// Läuft im Hintergrund; Fortschritt über download_stand().
pub fn starte_update(appid: String, pfad: String, benutzer: String) -> Result<(), Fehler> {
    let programm = steamcmd_programm()
        .ok_or_else(|| fehler("SteamCMD nicht gefunden (brew install --cask steamcmd)"))?;

    if !ist_angemeldet(&benutzer) {
        return Err(fehler("SteamCMD: nicht angemeldet – erst unten rechts anmelden"));
    }

    let spielordner = PathBuf::from(&pfad);
    let steamapps = spielordner
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| fehler("Pfad liegt nicht in steamapps/common/..."))?
        .to_path_buf();
    let client_manifest = steamapps.join(format!("appmanifest_{appid}.acf"));

    // Manifest-Zielpfade im SteamCMD-Bereich (inkrementelle Updates).
    let ziel_steamapps = spielordner.join("steamapps");
    let ziel_manifest = ziel_steamapps.join(format!("appmanifest_{appid}.acf"));

    // Slot atomar reservieren, bevor irgendetwas Dateisystem-seitig
    // passiert: Prüfen ("läuft schon einer?") und Setzen mussten früher
    // aus zwei getrennten Lock-Aufrufen bestehen, weil das Setzen erst im
    // gespawnten Thread passierte. Zwei fast gleichzeitige Aufrufe konnten
    // beide den "frei"-Zustand sehen und zwei SteamCMD-Prozesse parallel
    // lostreten. Jetzt: eine einzige Lock-Haltung für Check + Reservierung.
    {
        let mut aktiv = AKTIV
            .lock()
            .map_err(|_| fehler("Interner Zustand nicht verfügbar"))?;
        let belegt = aktiv
            .as_ref()
            .map(|d| matches!(d.stand.phase.as_str(), "wartet" | "laedt"))
            .unwrap_or(false);
        if belegt {
            return Err(fehler("Es läuft bereits ein Download"));
        }
        *aktiv = Some(AktiverDownload {
            stand: DownloadStand {
                appid: appid.clone(),
                phase: "wartet".into(),
                prozent: 0.0,
                meldung: "SteamCMD startet…".into(),
            },
            kind_pid: None,
        });
    }

    // Manifest in den SteamCMD-Bereich kopieren. Schlägt das fehl, muss
    // die Reservierung von oben wieder freigegeben werden - sonst bliebe
    // der Slot dauerhaft "belegt", ohne dass je ein Prozess läuft.
    if client_manifest.exists() {
        if let Err(e) = std::fs::create_dir_all(&ziel_steamapps)
            .map_err(|e| fehler(format!("SteamCMD-Ordner: {e}")))
            .and_then(|_| {
                std::fs::copy(&client_manifest, &ziel_manifest)
                    .map(|_| ())
                    .map_err(|e| fehler(format!("Manifest-Kopie: {e}")))
            })
        {
            if let Ok(mut aktiv) = AKTIV.lock() {
                *aktiv = None;
            }
            return Err(e);
        }
    }

    // LastPlayed sichern (SteamCMD setzt es auf 0 zurück).
    let gesichertes_last_played = std::fs::read_to_string(&client_manifest)
        .ok()
        .and_then(|t| last_played_aus_manifest(&t))
        .filter(|w| w != "0");

    let appid_clone = appid.clone();
    std::thread::spawn(move || {
        let mut kind = match Command::new(&programm)
            .arg("+force_install_dir")
            .arg(&spielordner)
            .arg("+login")
            .arg(&benutzer)
            .arg("+app_update")
            .arg(&appid_clone)
            .arg("+quit")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(k) => k,
            Err(e) => {
                stand_phase(&appid_clone, "fehler", 0.0, &format!("Start fehlgeschlagen: {e}"));
                aufraeumen(&ziel_manifest, &client_manifest, gesichertes_last_played.as_deref());
                return;
            }
        };

        if let Ok(mut aktiv) = AKTIV.lock() {
            if let Some(d) = aktiv.as_mut() {
                d.kind_pid = Some(kind.id());
            }
        }

        let stdout = kind.stdout.take().expect("stdout ist gepipet");
        for zeile in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Some((phase, prozent, meldung)) = zeile_deuten(&zeile) {
                stand_phase(&appid_clone, &phase, prozent, &meldung);
                if phase == "fertig" || phase == "fehler" {
                    break;
                }
            }
        }
        let _ = kind.wait();

        // Wenn die Schleife ohne Endstatus endete (Prozess z. B. getötet):
        let offen = AKTIV
            .lock()
            .map(|a| {
                a.as_ref()
                    .map(|d| matches!(d.stand.phase.as_str(), "wartet" | "laedt"))
                    .unwrap_or(false)
            })
            .unwrap_or(false);
        if offen {
            stand_phase(&appid_clone, "fehler", 0.0, "SteamCMD endete ohne Ergebnis");
        }

        aufraeumen(&ziel_manifest, &client_manifest, gesichertes_last_played.as_deref());
    });

    Ok(())
}

/// Manifest zurück an den Client-Platz bewegen und LastPlayed retten.
fn aufraeumen(ziel_manifest: &Path, client_manifest: &Path, last_played: Option<&str>) {
    if ziel_manifest.exists() {
        // LastPlayed wiederherstellen, bevor das Manifest zurückwandert.
        if let Some(wert) = last_played {
            if let Ok(text) = std::fs::read_to_string(ziel_manifest) {
                if last_played_aus_manifest(&text).as_deref() != Some(wert) {
                    let _ = std::fs::write(ziel_manifest, last_played_setzen(&text, wert));
                }
            }
        }
        // Verschieben; auf Fremddateisystemen (externe Bibliothek)
        // kann rename scheitern – dann kopieren + löschen.
        if std::fs::rename(ziel_manifest, client_manifest).is_err() {
            if std::fs::copy(ziel_manifest, client_manifest).is_ok() {
                let _ = std::fs::remove_file(ziel_manifest);
            }
        }
        // Leeren SteamCMD-Hilfsordner entfernen (geht nur, wenn leer).
        if let Some(dir) = ziel_manifest.parent() {
            let _ = std::fs::remove_dir(dir);
        }
    }
}

// ---------------------------------------------------------------- Anmeldung

/// Einmalige Anmeldung: Passwort (+ Steam-Guard-Code) über stdin.
/// Das Passwort steht so nie in der Prozessliste. 60 s hartes Limit.
pub fn anmelden(benutzer: String, passwort: String, guard_code: String) -> Result<String, Fehler> {
    let programm = steamcmd_programm()
        .ok_or_else(|| fehler("SteamCMD nicht gefunden (brew install --cask steamcmd)"))?;

    let mut kind = Command::new(&programm)
        .arg("+login")
        .arg(&benutzer)
        .arg("+quit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| fehler(format!("SteamCMD startete nicht: {e}")))?;

    // Passwort und Guard-Code in den stdin-Puffer schreiben. SteamCMD
    // liest sie genau dann, wenn es danach fragt – die Reihenfolge im
    // Puffer passt. Danach wird stdin geschlossen.
    {
        let stdin = kind.stdin.take().expect("stdin ist gepipet");
        let mut schreiber = std::io::BufWriter::new(stdin);
        let _ = writeln!(schreiber, "{passwort}");
        if !guard_code.is_empty() {
            let _ = writeln!(schreiber, "{guard_code}");
        }
        let _ = schreiber.flush();
    }

    // Ausgabe sammeln, mit hartem Timeout (Netz kann hängen).
    let stdout = kind.stdout.take().expect("stdout ist gepipet");
    let leser = std::thread::spawn(move || {
        let mut puffer = String::new();
        for zeile in BufReader::new(stdout).lines().map_while(Result::ok) {
            puffer.push_str(&zeile);
            puffer.push('\n');
        }
        puffer
    });

    let ausgabe = {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        while !leser.is_finished() {
            if std::time::Instant::now() > deadline {
                let _ = kind.kill();
                let _ = kind.wait();
                return Err(fehler("Zeitüberschreitung bei der Anmeldung"));
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        leser.join().unwrap_or_default()
    };
    let _ = kind.wait();

    // Deutung der Ausgabe – erst die klaren Fehler, dann der Erfolg.
    if ausgabe.contains("Invalid Password") {
        return Err(fehler("Passwort falsch"));
    }
    if ausgabe.contains("Invalid Steam Guard code")
        || ausgabe.contains("Two-factor code mismatch")
        || ausgabe.contains("already used")
    {
        return Err(fehler("Steam-Guard-Code falsch oder abgelaufen"));
    }
    if let Some(zeile) = ausgabe.lines().find(|z| z.contains("ERROR")) {
        return Err(fehler(zeile.trim().to_string()));
    }
    if ist_angemeldet(&benutzer) {
        Ok("Angemeldet – SteamCMD kann jetzt herunterladen".into())
    } else {
        Err(fehler("Anmeldung lief durch, aber kein Login gespeichert – bitte erneut versuchen"))
    }
}

fn fehler(meldung: impl Into<String>) -> Fehler {
    Fehler::Nachricht { meldung: meldung.into() }
}

// ---------------------------------------------------------------- Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeile_deuten_versteht_fortschritt() {
        let z = " Update state (0x5) downloading, progress: 42.7 (1234 / 5678)";
        let (phase, prozent, _) = zeile_deuten(z).unwrap();
        assert_eq!(phase, "laedt");
        assert!((prozent - 42.7).abs() < 0.01);
    }

    #[test]
    fn zeile_deuten_versteht_erfolg() {
        let (phase, prozent, meldung) =
            zeile_deuten("Success! App '480' fully installed.").unwrap();
        assert_eq!(phase, "fertig");
        assert_eq!(prozent, 100.0);
        assert_eq!(meldung, "Fertig");

        let (_, _, meldung) = zeile_deuten("Success! App '480' already up to date.").unwrap();
        assert_eq!(meldung, "Bereits aktuell");
    }

    #[test]
    fn zeile_deuten_versteht_fehler() {
        assert_eq!(zeile_deuten("ERROR! Failed to install").unwrap().0, "fehler");
        assert_eq!(zeile_deuten("ERROR (Invalid Password)").unwrap().0, "fehler");
    }

    #[test]
    fn zeile_deuten_ignoriert_nebensaechliches() {
        assert!(zeile_deuten("Loading Steam API...OK").is_none());
        assert!(zeile_deuten("Connecting anonymously to Steam Public...OK").is_none());
    }

    #[test]
    fn last_played_rundlauf() {
        let manifest =
            "\"AppState\"\n{\n\t\"appid\"\t\t\"480\"\n\t\"LastPlayed\"\t\t\"1787819314\"\n}\n";
        assert_eq!(
            last_played_aus_manifest(manifest),
            Some("1787819314".to_string())
        );
        let geaendert = last_played_setzen(manifest, "1787819314");
        assert!(geaendert.contains("\"LastPlayed\"\t\t\"1787819314\""));
    }

    #[test]
    fn last_played_setzen_erhaelt_format() {
        let manifest = "\"AppState\"\n{\n\t\"LastPlayed\"\t\t\"0\"\n}\n";
        let erg = last_played_setzen(manifest, "123");
        assert_eq!(erg, "\"AppState\"\n{\n\t\"LastPlayed\"\t\t\"123\"\n}\n");
    }

    #[test]
    fn last_played_fehlt_ohne_feld() {
        assert_eq!(last_played_aus_manifest("\"AppState\"\n{\n}\n"), None);
    }

    #[test]
    fn steamcmd_programm_findet_sich_auf_diesem_mac() {
        // Läuft auf Jens' Maschine – dort ist SteamCMD installiert.
        assert!(steamcmd_programm().is_some());
    }
}
