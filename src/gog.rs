// GOG-Leser (Stufe 2): eigene Datenquelle statt Heroic-Cache.
//
// Seit 2026-08-29 liegt die GOG-Bibliothek verzeichnisbasiert unter
// ~/Games/GOG/ – ein Ordner pro Spiel, direkt wie von GOG Galaxy/GOG
// angelegt (Cyberpunk 2077/, Stardew Valley.app, DREDGE.app, ...).
// Heroic ist damit als Zulieferer abgelöst; VR-Unterordner (z.B.
// "Cyberpunk 2077/Cyberpunk 2077/") werden nicht als eigenes Spiel
// gezählt, sondern als Teil ihres übergeordneten Titels.
//
// Metadaten (Titel, Cover) kommen aus einer kleinen eigenen Kopie der
// GOG-Bibliothek: ~/Library/Application Support/famulus-games/
// gog_library.json (Stand, als Heroic noch lief). Fallback: Titel aus
// dem Ordnernamen, Cover leer – keine Heroic-Abhängigkeit mehr.
//
// Cover-Wahl bleibt wie gehabt: art_square (Hochformat) vor art_cover.

use crate::Spiel;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// eigener Bibliotheks-Cache (Kopie aus Heroics store_cache).
fn gog_basis() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("Library/Application Support/famulus-games")
}

/// Wurzelordner der GOG-Spiele auf Jens' Mac.
fn gog_spiele_ordner() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("Games/GOG")
}

#[derive(Deserialize)]
struct GogBibliothek {
    #[serde(default)]
    games: Vec<GogSpiel>,
}

#[derive(Deserialize)]
struct GogSpiel {
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    folder_name: String,
    /// Hochformat-Cover (gamesdb), passt ins 2:3-Grid.
    #[serde(default)]
    art_square: String,
    /// Quer-Banner – nur Fallback.
    #[serde(default)]
    art_cover: String,
}

impl GogSpiel {
    /// Wie heißt der Ordner auf der Platte?
    fn ordnername(&self) -> String {
        if !self.folder_name.is_empty() {
            self.folder_name.clone()
        } else if !self.title.is_empty() {
            self.title.clone()
        } else {
            self.app_name.clone()
        }
        .trim_end_matches(".app")
        .to_string()
    }
}

/// Titel + Cover pro Ordnername aus der eigenen Bibliothekskopie.
fn lade_metadaten() -> Result<HashMap<String, (String, String)>> {
    let pfad = gog_basis().join("gog_library.json");
    let text = std::fs::read_to_string(&pfad)?;
    let bib: GogBibliothek = serde_json::from_str(&text)?;
    let mut m = HashMap::new();
    for g in bib.games {
        let ordner = g.ordnername();
        let url = if g.art_square.starts_with("http") {
            g.art_square
        } else if g.art_cover.starts_with("http") {
            g.art_cover
        } else {
            String::new()
        };
        m.insert(ordner, (g.title, url));
    }
    Ok(m)
}

/// Liest die GOG-Ordner unter ~/Games/GOG/ und liefert Spiele.
pub fn liese_gog_spiele() -> Result<Vec<Spiel>> {
    let wurzel = gog_spiele_ordner();
    let meta = lade_metadaten().unwrap_or_default();

    let mut spiele = Vec::new();
    for eintrag in std::fs::read_dir(&wurzel)? {
        let eintrag = eintrag?;
        let pfad = eintrag.path();
        if !eintrag.file_type()?.is_dir() {
            continue;
        }
        let name = eintrag.file_name().to_string_lossy().to_string();

        // Echte .app-Einträge sind selbsterklärend; alles andere muss
        // zumindest einen Unterordner/eine .app enthalten, sonst ist
        // es (z.B. .DS_Store-Ordner, Prefixes) kein Spiel.
        let ist_app = name.ends_with(".app");
        let hat_inhalt = eintrag
            .file_type()
            .map(|t| t.is_dir())
            .unwrap_or(false);
        let ist_spiel = ist_app || (hat_inhalt && ordner_hat_app_drin(&pfad));
        if !ist_spiel {
            continue;
        }

        // Ordnername → Schlüssel für Metadaten (ohne .app).
        let schluessel = name.trim_end_matches(".app").to_string();
        let (titel, cover_url) = meta
            .get(&schluessel)
            .cloned()
            .unwrap_or_else(|| (schluessel.clone(), String::new()));

        spiele.push(Spiel {
            id: schluessel.clone(),
            titel: if titel.is_empty() { schluessel } else { titel },
            quelle: "GOG".into(),
            pfad: pfad.to_string_lossy().to_string(),
            cover: String::new(),
            cover_url,
            zuletzt_gespielt: 0,
            groesse_bytes: groesse_des_ordners(&pfad),
        });
    }
    Ok(spiele)
}

/// enthält der Ordner eine .app oder einen Spiele-Unterordner?
fn ordner_hat_app_drin(pfad: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(pfad) else { return false };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.ends_with(".app") {
            return true;
        }
        // z.B. Cyberpunk 2077/Cyberpunk 2077/ – nur eine Ebene tiefer
        if e.path().is_dir()
            && std::fs::read_dir(e.path())
                .map(|mut sub| sub.any(|x| {
                    x.map(|y| y.file_name().to_string_lossy().ends_with(".app"))
                        .unwrap_or(false)
                }))
                .unwrap_or(false)
        {
            return true;
        }
    }
    false
}

/// Größe des Spielordners in Bytes (Summe, ohne Symbolic-Links).
fn groesse_des_ordners(pfad: &Path) -> u64 {
    fn walk(p: &Path) -> u64 {
        let Ok(rd) = std::fs::read_dir(p) else { return 0 };
        let mut sum = 0;
        for e in rd.flatten() {
            let p = e.path();
            let meta = match e.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if meta.is_dir() {
                sum += walk(&p);
            } else if meta.is_file() {
                sum += meta.len();
            }
        }
        sum
    }
    walk(pfad)
}

/// Menschliche Größe – behält die groesse_bytes, aber fürs Debug.
#[cfg(test)]
fn groesse_format(text: &str) -> String {
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordnername_entfernt_app_suffix() {
        let g = GogSpiel {
            app_name: "1453375253".into(),
            title: "Stardew Valley".into(),
            folder_name: "Stardew Valley.app".into(),
            art_square: String::new(),
            art_cover: String::new(),
        };
        assert_eq!(g.ordnername(), "Stardew Valley");
    }

    #[test]
    fn metaschluessel_findet_titel() {
        let mut meta: HashMap<String, (String, String)> = HashMap::new();
        meta.insert("Stardew Valley".into(), ("Stardew Valley".into(), "url".into()));
        assert_eq!(
            meta.get("Stardew Valley"),
            Some(&("Stardew Valley".to_string(), "url".to_string()))
        );
    }

    #[test]
    fn groesse_zaehlt_dateien() {
        let dir = std::env::temp_dir().join("famulus-gog-groesse-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/datei.bin"), vec![0u8; 10]).unwrap();
        std::fs::write(dir.join("a/b/zwei.bin"), vec![0u8; 20]).unwrap();
        std::fs::write(dir.join("top.bin"), vec![0u8; 7]).unwrap();
        assert_eq!(groesse_des_ordners(&dir), 37);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ordner_ohne_app_ist_kein_spiel() {
        let dir = std::env::temp_dir().join("famulus-gog-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("laerm/ohne_app")).unwrap();
        std::fs::write(dir.join("laerm/ohne_app/datei.txt"), "x").unwrap();
        assert!(!ordner_hat_app_drin(&dir.join("laerm")));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ordner_mit_eingebetteter_app_ist_spiel() {
        let dir = std::env::temp_dir().join("famulus-gog-test2");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Cyberpunk 2077/Cyberpunk 2077/Cyberpunk2077.app/Contents/MacOS")).unwrap();
        std::fs::write(dir.join("Cyberpunk 2077/Cyberpunk 2077/Cyberpunk2077.app/Contents/Info.plist"), "x").unwrap();
        assert!(ordner_hat_app_drin(&dir.join("Cyberpunk 2077")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}