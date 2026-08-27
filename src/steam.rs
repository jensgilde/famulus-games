// Steam-Leser: findet alle Steam-Bibliotheken (libraryfolders.vdf
// kann mehrere Ordner kennen), parst die appmanifest_*.acf-Dateien
// (Valves VDF-Format: "key" "wert", kein XML) und sucht lokale
// Cover in appcache/librarycache/<appid>/.
//
// Geprüft auf Jens' Mac: eine Bibliothek unter
// ~/Library/Application Support/Steam, No Man's Sky (275850) mit
// Capsule-Cover im Cache.

use crate::Spiel;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

fn steam_basis() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("Library/Application Support/Steam")
}

/// Liest libraryfolders.vdf und liefert alle steamapps-Ordner.
fn bibliotheken() -> Vec<PathBuf> {
    let basis = steam_basis();
    let vdf = basis.join("steamapps/libraryfolders.vdf");
    let mut ordner: Vec<PathBuf> = Vec::new();
    if let Ok(text) = std::fs::read_to_string(&vdf) {
        for zeile in text.lines() {
            // Zeilen wie: "path"		"/Users/.../Steam"
            if let Some(wert) = wert_von(zeile, "\"path\"") {
                let steamapps = PathBuf::from(wert).join("steamapps");
                if !ordner.contains(&steamapps) {
                    ordner.push(steamapps);
                }
            }
        }
    }
    if ordner.is_empty() {
        // Fallback: der Standard-Ordner selbst.
        ordner.push(basis.join("steamapps"));
    }
    ordner
}

/// Sehr kleiner VDF-Parser: sucht ein Schlüssel-Wert-Paar in einer
/// Zeile der Form  "key"<Trennzeichen>"wert"  und liefert den Wert
/// ohne Anführungszeichen.
fn wert_von(zeile: &str, schluessel: &str) -> Option<String> {
    let zeile = zeile.trim();
    if !zeile.starts_with(schluessel) {
        return None;
    }
    let rest = zeile[schluessel.len()..].trim_start();
    let rest = rest.strip_prefix('"')?;
    let ende = rest.find('"')?;
    Some(rest[..ende].to_string())
}

/// Parst eine einzelne appmanifest-Datei zu einem Spiel.
/// Liefert None, wenn Pflichtfelder fehlen oder das Spiel nicht
/// vollständig installiert ist (StateFlags != 4).
fn parse_manifest(inhalt: &str, steamapps_ordner: &std::path::Path) -> Option<Spiel> {
    let mut felder: HashMap<String, String> = HashMap::new();
    for zeile in inhalt.lines() {
        let z = zeile.trim();
        for key in ["appid", "name", "installdir", "LastPlayed", "SizeOnDisk", "StateFlags"] {
            if let Some(v) = wert_von(z, &format!("\"{key}\"")) {
                felder.insert(key.to_string(), v);
            }
        }
    }
    // Halbfertige Downloads haben kein Recht, in der Bibliothek zu stehen.
    if felder.get("StateFlags").map(|s| s.as_str()).unwrap_or("4") != "4" {
        return None;
    }
    let appid = felder.get("appid")?.clone();
    let titel = felder.get("name")?.clone();
    let installdir = felder.get("installdir")?.clone();

    let spiel_pfad = steamapps_ordner.join("common").join(&installdir);

    // Cover: librarycache/<appid>/<hash>/library_capsule.jpg (300×450).
    let cover = suche_capsule(&appid);

    Some(Spiel {
        id: appid,
        titel,
        quelle: "Steam".into(),
        pfad: spiel_pfad.to_string_lossy().to_string(),
        cover,
        cover_url: String::new(),
        zuletzt_gespielt: felder.get("LastPlayed").and_then(|s| s.parse().ok()).unwrap_or(0),
        groesse_bytes: felder.get("SizeOnDisk").and_then(|s| s.parse().ok()).unwrap_or(0),
    })
}

/// Sucht das Capsule-Cover einer AppID im Steam-Cache.
fn suche_capsule(appid: &str) -> String {
    let verzeichnis = steam_basis().join("appcache/librarycache").join(appid);
    let Ok(eintraege) = std::fs::read_dir(&verzeichnis) else {
        return String::new();
    };
    for eintrag in eintraege.flatten() {
        if !eintrag.path().is_dir() {
            continue;
        }
        let capsule = eintrag.path().join("library_capsule.jpg");
        if capsule.exists() {
            return capsule.to_string_lossy().to_string();
        }
    }
    String::new()
}

/// Alle Steam-Spiele aller Bibliotheken.
pub fn liese_steam_spiele() -> Result<Vec<Spiel>> {
    let mut alle = Vec::new();
    for steamapps in bibliotheken() {
        let Ok(eintraege) = std::fs::read_dir(&steamapps) else { continue };
        for eintrag in eintraege.flatten() {
            let name = eintrag.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }
            let Ok(inhalt) = std::fs::read_to_string(eintrag.path()) else { continue };
            if let Some(spiel) = parse_manifest(&inhalt, &steamapps) {
                alle.push(spiel);
            }
        }
    }
    Ok(alle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wert_von_findet_wert_mit_tabs() {
        let z = "\t\"name\"\t\t\"No Man's Sky\"";
        assert_eq!(wert_von(z, "\"name\""), Some("No Man's Sky".into()));
    }

    #[test]
    fn wert_von_ignoriert_falschen_schluessel() {
        let z = "\"appid\"\t\t\"275850\"";
        assert_eq!(wert_von(z, "\"name\""), None);
    }

    #[test]
    fn parse_manifest_liefert_spiel() {
        let inhalt = "\"AppState\"\n{\n\t\"appid\"\t\t\"275850\"\n\t\"name\"\t\t\"Testspiel\"\n\t\"StateFlags\"\t\t\"4\"\n\t\"installdir\"\t\t\"Testspiel\"\n\t\"LastPlayed\"\t\t\"1787819314\"\n\t\"SizeOnDisk\"\t\t\"12345\"\n}";
        let s = parse_manifest(inhalt, std::path::Path::new("/tmp/steamapps")).unwrap();
        assert_eq!(s.id, "275850");
        assert_eq!(s.titel, "Testspiel");
        assert_eq!(s.zuletzt_gespielt, 1787819314);
        assert_eq!(s.groesse_bytes, 12345);
        assert!(s.pfad.ends_with("common/Testspiel"));
    }

    #[test]
    fn parse_manifest_verwirft_unfertige() {
        let inhalt = "\"AppState\"\n{\n\t\"appid\"\t\t\"1\"\n\t\"name\"\t\t\"Halbfertig\"\n\t\"StateFlags\"\t\t\"2\"\n\t\"installdir\"\t\t\"x\"\n}";
        assert!(parse_manifest(inhalt, std::path::Path::new("/tmp")).is_none());
    }

    #[test]
    fn parse_manifest_verwirft_leere() {
        assert!(parse_manifest("", std::path::Path::new("/tmp")).is_none());
    }
}
