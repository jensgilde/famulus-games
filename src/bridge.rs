// Famulus Games – UniFFI-Brücke v0.2.1.
// Dünne FFI-Schicht für die native Swift-Hülle (swift-app/).
// Enthält auch die Logik, die vorher im Tauri-GUI wohnte
// (Cover-Cache, Spielstart), damit der Kern sie allen Hüllen
// anbieten kann – Tauri eingeschlossen.

use crate::Spiel;
use std::path::PathBuf;

// Die UniFFI-Scaffolding wird in lib.rs eingebunden (Crate-Root).

// ---------------------------------------------------------------- Fehler

/// Fehlertyp für die FFI-Grenze: einfache Meldung, kein anyhow.
#[derive(Debug)]
pub enum Fehler {
    Nachricht { meldung: String },
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fehler::Nachricht { meldung } => write!(f, "{meldung}"),
        }
    }
}

impl std::error::Error for Fehler {}

fn fehler(meldung: impl Into<String>) -> Fehler {
    Fehler::Nachricht {
        meldung: meldung.into(),
    }
}

// ---------------------------------------------------------------- Bibliothek

/// Vereinte Bibliothek (Steam + Heroic/GOG), alphabetisch sortiert.
pub fn sammele_spiele() -> Vec<Spiel> {
    crate::sammele_spiele()
}

/// App-Version, kommt aus Cargo.toml (env!) – Präferenz Jens.
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Läuft Steam gerade?
pub fn steam_laeuft() -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg("steam_osx")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Menschliche Größenangabe („1,5 GB“) – macOS-übliche Einheiten (1000er).
pub fn format_groesse(bytes: u64) -> String {
    const EINHEITEN: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut wert = bytes as f64;
    let mut i = 0;
    while wert >= 1000.0 && i < EINHEITEN.len() - 1 {
        wert /= 1000.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", wert, EINHEITEN[i])
    }
}

// ---------------------------------------------------------------- Spielestart

/// Startet ein Spiel. Steam über das URL-Schema (Steam macht den Rest),
/// GOG/Heroic direkt über `open <pfad>`.
pub fn starte_spiel(quelle: String, id: String, pfad: String) -> Result<String, Fehler> {
    let status = if quelle == "Steam" {
        std::process::Command::new("open")
            .arg(format!("steam://rungameid/{id}"))
            .status()
    } else {
        std::process::Command::new("open").arg(&pfad).status()
    };
    match status {
        Ok(s) if s.success() => Ok("gestartet".into()),
        Ok(s) => Err(fehler(format!("Start fehlgeschlagen (Code {s})"))),
        Err(e) => Err(fehler(format!("Start fehlgeschlagen: {e}"))),
    }
}

// ---------------------------------------------------------------- Cover

/// Eigener Cache-Ordner – getrennt vom Famulus-Kern.
fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let dir = home.join("Library/Application Support/famulus-games/covers");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Liefert eine file://-URL zum Cover. Reihenfolge:
/// 1. lokales Steam-Cover (Existenz geprüft),
/// 2. eigener Cache (alle Endungen),
/// 3. Download von cover_url in den Cache (max. 8 Redirects, 20 s).
pub fn hole_cover_datei(spiel_id: String, cover: String, cover_url: String) -> Result<String, Fehler> {
    // 1. Lokales Cover (Steam: librarycache). Existenz prüfen – der
    //    Pfad kann veraltet sein, wenn das Spiel verschoben wurde.
    if !cover.is_empty() && std::path::Path::new(&cover).exists() {
        return Ok(datei_url(&cover));
    }
    // 2. Cache.
    for alt in ["jpg", "jpeg", "png", "webp"] {
        let kandidat = cache_dir().join(format!("{spiel_id}.{alt}"));
        if kandidat.exists() {
            return Ok(datei_url(&kandidat.to_string_lossy()));
        }
    }
    // 3. Download.
    if cover_url.is_empty() {
        return Err(fehler("kein Cover vorhanden"));
    }
    let pfad_teil = cover_url.split('?').next().unwrap_or("");
    let endung = pfad_teil
        .rsplit('.')
        .next()
        .filter(|e| matches!(*e, "jpg" | "jpeg" | "png" | "webp"))
        .unwrap_or("jpg");
    let ziel = cache_dir().join(format!("{spiel_id}.{endung}"));
    let bytes = http_get(&cover_url).map_err(|e| fehler(format!("Download fehlgeschlagen: {e}")))?;
    std::fs::write(&ziel, &bytes).map_err(|e| fehler(format!("Cache-Schreibfehler: {e}")))?;
    Ok(datei_url(&ziel.to_string_lossy()))
}

fn datei_url(pfad: &str) -> String {
    // Cover-Pfade enthalten hier praktisch keine Sonderzeichen;
    // Leerzeichen müssen aber auch in file:-URLs maskiert werden.
    format!("file://{}", pfad.replace(' ', "%20"))
}

/// Lädt eine URL per curl herunter (auf macOS immer vorhanden).
/// Folgt Redirects (-L), 20 s Timeout, HTTPS-fähig.
fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let ausgabe = std::process::Command::new("curl")
        .args(["--max-time", "20", "-fsSL"])
        .arg(url)
        .output()
        .map_err(|e| format!("curl nicht ausführbar: {e}"))?;
    if ausgabe.status.success() && !ausgabe.stdout.is_empty() {
        Ok(ausgabe.stdout)
    } else {
        let fehler_text = String::from_utf8_lossy(&ausgabe.stderr);
        Err(format!("HTTP-Fehler: {}", fehler_text.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_groesse_typische_werte() {
        assert_eq!(format_groesse(0), "0 B");
        assert_eq!(format_groesse(999), "999 B");
        assert_eq!(format_groesse(1_500_000), "1.5 MB");
        assert!(format_groesse(2_500_000_000).starts_with("2.5"));
    }

    #[test]
    fn datei_url_maskiert_leerzeichen() {
        assert_eq!(
            datei_url("/Users/x/Games/No Man's Sky.jpg"),
            "file:///Users/x/Games/No%20Man's%20Sky.jpg"
        );
    }

    #[test]
    fn hole_cover_liefert_fehler_bei_leerer_url() {
        let r = hole_cover_datei("unbekannt".into(), String::new(), String::new());
        assert!(r.is_err());
    }
}
