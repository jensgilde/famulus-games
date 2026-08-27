// Heroic/GOG-Leser: liest Heroics Bibliotheks-Cache
// (store_cache/gog_library.json) und die Installations-Datenbank
// (gog_store/installed.json) und verbindet beides: Metadaten +
// echter Installationspfad. Heroic selbst wird nicht gestartet und
// nicht verändert – wir lesen nur, was es geschrieben hat.
//
// Geprüft auf Jens' Mac: 29 GOG-Spiele im Cache, 2 installiert
// (DREDGE, Stardew Valley), Cover liegen als URLs im Cache. Die
// GUI lädt die URL-Cover in ihren eigenen Cache nach.

use crate::Spiel;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

fn heroic_pfad() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join("Library/Application Support/heroic")
}

#[derive(Deserialize)]
struct GogBibliothek {
    #[serde(default)]
    games: Vec<GogSpiel>,
}

#[derive(Deserialize)]
struct GogSpiel {
    app_name: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    art_cover: String,
}

#[derive(Deserialize)]
struct InstallierteListe {
    #[serde(default)]
    installed: Vec<Installiert>,
}

#[derive(Deserialize)]
struct Installiert {
    #[serde(rename = "appName")]
    app_name: String,
    install_path: String,
    #[serde(default)]
    install_size: String,
}

/// Alle installierten Heroic/GOG-Spiele. Titel und Cover-URL kommen
/// aus dem Bibliotheks-Cache; fehlt ein Titel dort (Cache veraltet),
/// springt der Ordnername ein.
pub fn liese_heroic_spiele() -> Result<Vec<Spiel>> {
    let basis = heroic_pfad();

    // 1. Installations-Datenbank: wer ist wirklich da, und wo.
    let installiert: InstallierteListe = {
        let pfad = basis.join("gog_store/installed.json");
        let text = std::fs::read_to_string(&pfad)?;
        serde_json::from_str(&text)?
    };

    // 2. Bibliotheks-Cache: Titel und Cover pro app_name.
    let mut cache: HashMap<String, (String, String)> = HashMap::new();
    if let Ok(text) = std::fs::read_to_string(basis.join("store_cache/gog_library.json")) {
        if let Ok(bib) = serde_json::from_str::<GogBibliothek>(&text) {
            for g in bib.games {
                cache.insert(g.app_name, (g.title, g.art_cover));
            }
        }
    }

    let mut spiele = Vec::new();
    for inst in &installiert.installed {
        let (titel, cover_url) = match cache.get(&inst.app_name) {
            Some((t, c)) => {
                let url = if c.starts_with("http") { c.clone() } else { String::new() };
                let titel = if t.is_empty() { ordnername(&inst.install_path) } else { t.clone() };
                (titel, url)
            }
            None => (ordnername(&inst.install_path), String::new()),
        };

        spiele.push(Spiel {
            id: inst.app_name.clone(),
            titel,
            quelle: "GOG".into(),
            pfad: inst.install_path.clone(),
            // Lokales Cover bleibt leer: die GUI lädt cover_url in
            // ihren eigenen Cache (hole_cover).
            cover: String::new(),
            cover_url,
            zuletzt_gespielt: 0, // MVP: eigene Spielzeit kommt später.
            groesse_bytes: groesse_aus_text(&inst.install_size),
        });
    }
    let _ = basis; // (basis wird oben schon verbraucht; ruhig lassen)
    Ok(spiele)
}

/// Wandelt Heroics Größen-Texte ("678.36 MiB", "1.5 GiB") in Bytes.
fn groesse_aus_text(text: &str) -> u64 {
    let teile: Vec<&str> = text.split_whitespace().collect();
    if teile.len() != 2 {
        return 0;
    }
    let Ok(zahl) = teile[0].parse::<f64>() else { return 0 };
    let faktor = match teile[1] {
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        "TiB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "B" => 1.0,
        _ => 0.0,
    };
    (zahl * faktor) as u64
}

/// Letzter Pfadbestandteil ohne .app – Fallback-Titel.
fn ordnername(pfad: &str) -> String {
    let name = std::path::Path::new(pfad)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| pfad.to_string());
    name.trim_end_matches(".app").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordnername_streift_app() {
        assert_eq!(ordnername("/Users/x/Games/DREDGE.app"), "DREDGE");
        assert_eq!(ordnername("/Users/x/Games/Stardew Valley.app"), "Stardew Valley");
    }

    #[test]
    fn groesse_aus_text_kann_mib() {
        assert_eq!(groesse_aus_text("678.36 MiB"), (678.36 * 1024.0 * 1024.0) as u64);
        assert_eq!(groesse_aus_text("1.5 GiB"), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
        assert_eq!(groesse_aus_text("kaputt"), 0);
        assert_eq!(groesse_aus_text(""), 0);
    }

    #[test]
    fn gog_installiert_json_ist_parsebar() {
        // Exakte Struktur von Jens' echter installed.json nachgestellt.
        let beispiel = r#"{"installed":[{"platform":"osx","executable":"","install_path":"/Users/jensgilde/Games/Heroic/DREDGE.app","install_size":"783.87 MiB","is_dlc":false,"version":"2993","appName":"1744110647","installedDLCs":[],"language":"en-US"}]}"#;
        let v: InstallierteListe = serde_json::from_str(beispiel).unwrap();
        assert_eq!(v.installed.len(), 1);
        assert_eq!(v.installed[0].app_name, "1744110647");
        assert!(v.installed[0].install_path.ends_with("DREDGE.app"));
    }
}
