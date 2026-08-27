// Famulus-Games Bibliothek. Tauri 2; main.rs ruft nur run() auf.
// Die GUI macht drei Dinge: Bibliothek sammeln, Spiele starten,
// Cover laden (lokal lesen oder aus dem GOG-CDN in den eigenen
// Cache herunterladen). Alle Bilder gehen als Data-URLs ins
// Frontend – kein Asset-Protokoll nötig, keine Extrapflichten.

use famulus_games::Spiel;
use std::io::Read;
use std::path::PathBuf;

/// Eigener Cache-Ordner – getrennt vom Famulus-Kern, damit die
/// Produkte einander nicht in die Quere kommen.
fn cache_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let dir = home.join("Library/Application Support/famulus-games/covers");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn als_data_url(bytes: &[u8], endung: &str) -> String {
    use base64::Engine as _;
    let typ = match endung.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        _ => "image/jpeg",
    };
    format!(
        "data:{};base64,{}",
        typ,
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

fn endung_von(pfad: &str) -> String {
    std::path::Path::new(pfad)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_else(|| "jpg".into())
}

#[tauri::command]
fn bibliothek_laden() -> Vec<Spiel> {
    famulus_games::sammele_spiele()
}

#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Startet ein Spiel. Steam über das URL-Schema (Steam selbst macht
/// dann alles Weitere), GOG/Heroic-Spiele direkt über `open <app>`.
#[tauri::command]
fn starte_spiel(quelle: String, id: String, pfad: String) -> Result<String, String> {
    let status = if quelle == "Steam" {
        std::process::Command::new("open")
            .arg(format!("steam://rungameid/{id}"))
            .status()
    } else {
        std::process::Command::new("open").arg(&pfad).status()
    };
    match status {
        Ok(s) if s.success() => Ok("gestartet".into()),
        Ok(s) => Err(format!("Start fehlgeschlagen (Code {s})")),
        Err(e) => Err(format!("Start fehlgeschlagen: {e}")),
    }
}

/// Liest ein lokales Cover und liefert es als Data-URL.
#[tauri::command]
fn cover_lokal(pfad: String) -> Result<String, String> {
    let mut datei = std::fs::File::open(&pfad).map_err(|e| format!("lesbar? {e}"))?;
    let mut bytes = Vec::new();
    datei
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Fehler beim Lesen: {e}"))?;
    Ok(als_data_url(&bytes, &endung_von(&pfad)))
}

/// Lädt ein Cover aus dem Netz in den lokalen Cache und liefert es
/// als Data-URL. Ist die Datei schon im Cache, kein zweiter
/// Download.
#[tauri::command]
fn hole_cover(spiel_id: String, url: String) -> Result<String, String> {
    if url.is_empty() {
        return Err("keine Cover-URL".into());
    }

    // Schon im Cache? (alle Endungen durchprobieren)
    for alt in ["jpg", "jpeg", "png", "webp"] {
        let kandidat = cache_dir().join(format!("{spiel_id}.{alt}"));
        if kandidat.exists() {
            return cover_lokal(kandidat.to_string_lossy().to_string());
        }
    }

    // Endung aus der URL raten (vor dem ? abschneiden).
    let pfad_teil = url.split('?').next().unwrap_or("");
    let endung = pfad_teil
        .rsplit('.')
        .next()
        .filter(|e| matches!(*e, "jpg" | "jpeg" | "png" | "webp"))
        .unwrap_or("jpg");
    let ziel = cache_dir().join(format!("{spiel_id}.{endung}"));

    let status = std::process::Command::new("curl")
        .args(["--max-time", "20", "-fsSL", "-o"])
        .arg(&ziel)
        .arg(&url)
        .status()
        .map_err(|e| format!("curl nicht ausführbar: {e}"))?;

    if status.success() && ziel.exists() {
        cover_lokal(ziel.to_string_lossy().to_string())
    } else {
        let _ = std::fs::remove_file(&ziel); // Halbe Dateien weg.
        Err("Download fehlgeschlagen".into())
    }
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            bibliothek_laden,
            starte_spiel,
            hole_cover,
            cover_lokal,
            app_version
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Start von Famulus Games");
}
