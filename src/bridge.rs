// Famulus Games – UniFFI-Brücke v0.2.2.
// Dünne FFI-Schicht für die native Swift-Hülle (swift-app/).
// Enthält auch die Logik, die vorher im Tauri-GUI wohnte
// (Cover-Cache, Spielstart), damit der Kern sie allen Hüllen
// anbieten kann – Tauri eingeschlossen.

use crate::Spiel;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

/// Vereinte Bibliothek (Steam + GOG), alphabetisch sortiert.
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

/// Startet ein Spiel.
///
/// Steam über das URL-Schema (`steam://rungameid/{id}`), nachdem der
/// Client bei Bedarf lautlos hochgefahren wurde (siehe
/// `stelle_steam_still_sicher`) - Famulus Games soll die einzige UI
/// bleiben, die man zu Gesicht bekommt.
/// GOG-Spiele werden direkt als Binary gestartet (mit BepInEx-Env
/// wenn vorhanden) – `open` wird nicht verwendet, weil LaunchServices
/// Umgebungsvariablen ignoriert und BepInEx-Doorstop nicht lädt.
pub fn starte_spiel(quelle: String, id: String, pfad: String) -> Result<String, Fehler> {
    if quelle == "Steam" {
        stelle_steam_still_sicher();
        let status = std::process::Command::new("open")
            .arg(format!("steam://rungameid/{id}"))
            .status();
        return match status {
            Ok(s) if s.success() => Ok("gestartet".into()),
            Ok(s) => Err(fehler(format!("Start fehlgeschlagen (Code {s})"))),
            Err(e) => Err(fehler(format!("Start fehlgeschlagen: {e}"))),
        };
    }
    // GOG-Spiele: Binary + BepInEx-Setup
    starte_gog_spiel(&gog_start_pfad(&pfad))
}

/// Startet Steam lautlos im Hintergrund, falls es nicht schon läuft.
///
/// Ohne das würde `open steam://rungameid/...` unten Steam ganz normal
/// hochfahren - mit sichtbarem Hauptfenster/Bibliothek. `-silent` ist
/// Valves eigenes Client-Flag dafür (Windows/Mac/Linux identisch): Steam
/// startet nur den Hintergrunddienst, ohne je ein Fenster zu öffnen.
///
/// Der Poll danach ist nötig, weil `steam_laeuft()` nur den Prozess sieht,
/// nicht ob sein URL-Handler (IPC) schon bereit ist - schickt man
/// `rungameid` zu früh los, während Steam noch hochfährt, geht der
/// Spielstart ins Leere. 20×500ms sind grosszügig über der üblichen
/// Startzeit; läuft Steam danach immer noch nicht, versucht `open`
/// trotzdem sein Glück - dann eben mit Steams normalem (sichtbarem)
/// Auto-Start-Verhalten als Fallback.
fn stelle_steam_still_sicher() {
    if steam_laeuft() {
        return;
    }
    let _ = std::process::Command::new("open")
        .args(["-a", "Steam", "--args", "-silent"])
        .status();
    for _ in 0..20 {
        if steam_laeuft() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

/// Startet ein GOG-Spiel als Binary (nicht `open`).
/// Setzt BepInEx-Env-Vars wenn `libdoorstop.dylib` existiert.
fn starte_gog_spiel(app_pfad: &str) -> Result<String, Fehler> {
    let app_path = Path::new(app_pfad);

    // Binary-Pfad aus Info.plist oder fallback Binary-Name
    let binary_path = gog_binary_pfad(app_path)?;

    // Prüfe auf BepInEx (libdoorstop.dylib + run_bepinex.sh)
    let macos_dir = app_path.join("Contents/MacOS");
    let doorstop_dylib = macos_dir.join("libdoorstop.dylib");
    let hat_bepinex = doorstop_dylib.exists();

    let mut cmd = std::process::Command::new(&binary_path);

    if hat_bepinex {
        // BepInEx-Env-Vars setzen (analog zu run_bepinex.sh)
        let target_assembly = macos_dir.join("BepInEx/core/BepInEx.Preloader.dll");
        cmd.env("DOORSTOP_ENABLED", "1");
        if target_assembly.exists() {
            cmd.env("DOORSTOP_TARGET_ASSEMBLY", &target_assembly);
        }
        // DYLD_INSERT_LIBRARIES mit absolutem Pfad – das ist der
        // entscheidende Unterschied zum kaputten run_bepinex.sh,
        // das nur den Dateinamen setzt und dann von `arch -e`
        // zerschossen wird (DYLD_LIBRARY_PATH geht verloren).
        cmd.env("DYLD_INSERT_LIBRARIES", &doorstop_dylib);
        cmd.env("DYLD_LIBRARY_PATH", &macos_dir);
        // Apple Silicon: Binary als arm64 starten (kein Rosetta)
        if cfg!(target_arch = "aarch64") {
            cmd.env("ARCHPREFERENCE", "arm64,x86_64");
        }
    }

    // Binary aus dem MacOS-Ordner starten (wg. relativen Pfaden)
    if macos_dir.exists() {
        cmd.current_dir(&macos_dir);
    }

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => Ok("gestartet".into()),
        Ok(s) => Err(fehler(format!("Start fehlgeschlagen (Code {s})"))),
        Err(e) => Err(fehler(format!("Start fehlgeschlagen: {e}"))),
    }
}

/// Findet die ausführbare Binary im `.app`-Bundle.
/// 1. Ließt `CFBundleExecutable` aus `Info.plist`.
/// 2. Fallback: Ordnername ohne `.app` (oder `Contents/MacOS/<name>`).
/// 3. Fallback: `Contents/MacOS/<irgendeine Binary>`.
fn gog_binary_pfad(app_path: &Path) -> Result<PathBuf, Fehler> {
    let info_plist = app_path.join("Contents/Info.plist");
    if info_plist.exists() {
        // plist mit `defaults read` parsen
        let output = std::process::Command::new("defaults")
            .arg("read")
            .arg(app_path.join("Contents/Info"))
            .arg("CFBundleExecutable")
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !name.is_empty() {
                    let binary = app_path.join("Contents/MacOS").join(&name);
                    if binary.exists() {
                        return Ok(binary);
                    }
                }
            }
        }
    }
    // Fallback: Ordnername ohne .app in Contents/MacOS/
    if let Some(name) = app_path.file_stem() {
        let binary = app_path.join("Contents/MacOS").join(name);
        if binary.exists() {
            return Ok(binary);
        }
    }
    // Fallback: erste Binary in Contents/MacOS/
    let macos_dir = app_path.join("Contents/MacOS");
    if let Ok(rd) = std::fs::read_dir(&macos_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_file() && !p.extension().map(|x| x == "dylib" || x == "sh" || x == "plist").unwrap_or(false) {
                // Prüfe ob es eine Mach-O Binary ist
                if let Ok(meta) = std::fs::metadata(&p) {
                    if meta.len() > 1000 && meta.permissions().mode() & 0o111 != 0 {
                        return Ok(p);
                    }
                }
            }
        }
    }
    Err(fehler(format!(
        "Keine Binary in {} gefunden",
        app_path.display()
    )))
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
    // Byteweise Prozent-Kodierung: alles außer den unreservierten
    // Zeichen (A-Z, a-z, 0-9, -, ., _, ~) wird als %xx kodiert.
    // Das schließt Leerzeichen, »#«, »%«, »&«, »?« usw. ein.
    // Der file://-Präfix bleibt unkodiert (er ist kein Pfadteil).
    let mut url = String::from("file://");
    for &byte in pfad.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                url.push(byte as char);
            }
            b'/' => url.push('/'),
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

/// HTTP-GET mit curl (keine externen Abhängigkeiten).
fn http_get(url: &str) -> Result<Vec<u8>, String> {
    let output = std::process::Command::new("curl")
        .arg("-sS")
        .arg("-L") // folge Redirects (max 8)
        .arg("--max-time")
        .arg("20")
        .arg(url)
        .output()
        .map_err(|e| format!("curl fehlgeschlagen: {e}"))?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("curl: {stderr}"))
    }
}

// ---------------------------------------------------------------- GOG-Pfad

/// Sucht im GOG-Ordner die tatsächliche `.app`.
///
/// Cyberpunk 2077 liegt als `Cyberpunk 2077/Cyberpunk 2077/Cyberpunk2077.app`
/// – also zwei Ebenen tiefer. DREDGE.app und Stardew Valley.app liegen
/// direkt im GOG-Ordner. Diese Funktion normalisiert den Pfad auf die
/// `.app`, die die Binary enthält.
fn gog_start_pfad(pfad: &str) -> String {
    let root = std::path::Path::new(pfad);
    if root.is_dir() {
        // Direkt eine .app?
        if root.extension().map(|x| x == "app").unwrap_or(false) {
            return pfad.to_string();
        }
        // Eine Ebene tiefer: .app im Ordner
        if let Ok(rd) = std::fs::read_dir(root) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() && p.extension().map(|x| x == "app").unwrap_or(false) {
                    return p.to_string_lossy().to_string();
                }
            }
        }
        // zwei Ebenen tiefer (z.B. Cyberpunk 2077/Cyberpunk 2077/)
        if let Ok(rd) = std::fs::read_dir(root) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Ok(sub) = std::fs::read_dir(&p) {
                        for x in sub.flatten() {
                            let sp = x.path();
                            if sp.is_dir() && sp.extension().map(|y| y == "app").unwrap_or(false) {
                                return sp.to_string_lossy().to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    pfad.to_string()
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
            "file:///Users/x/Games/No%20Man%27s%20Sky.jpg"
        );
    }

    /// '#' unmaskiert würde WebKit als Fragment-Anfang lesen und den Rest
    /// der URL abschneiden - genau der Bug, den die neue Kodierung behebt.
    #[test]
    fn datei_url_maskiert_raute_und_prozent() {
        assert_eq!(
            datei_url("/Users/x/Games/Cyberpunk #2077 100%.jpg"),
            "file:///Users/x/Games/Cyberpunk%20%232077%20100%25.jpg"
        );
    }

    #[test]
    fn hole_cover_liefert_fehler_bei_leerer_url() {
        let r = hole_cover_datei("unbekannt".into(), String::new(), String::new());
        assert!(r.is_err());
    }
    #[test]
    fn gog_start_pfad_findet_eingebettete_app() {
        let dir = std::env::temp_dir().join("famulus-start-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Cyberpunk 2077/Cyberpunk 2077/Cyberpunk2077.app/Contents")).unwrap();
        std::fs::write(dir.join("Cyberpunk 2077/Cyberpunk 2077/Cyberpunk2077.app/Contents/Info.plist"), "x").unwrap();
        let start = gog_start_pfad(&dir.join("Cyberpunk 2077").to_string_lossy());
        assert!(start.ends_with("Cyberpunk2077.app"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gog_start_pfad_laesst_app_unveraendert() {
        assert_eq!(gog_start_pfad("/Users/x/Games/GOG/DREDGE.app"), "/Users/x/Games/GOG/DREDGE.app");
    }

    #[test]
    fn gog_binary_pfad_liest_info_plist() {
        let dir = std::env::temp_dir().join("famulus-binary-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Contents/MacOS")).unwrap();
        std::fs::write(dir.join("Contents/Info.plist"), "<?xml version=\"1.0\"?><plist><dict><key>CFBundleExecutable</key><string>DREDGE</string></dict></plist>").unwrap();
        std::fs::write(dir.join("Contents/MacOS/DREDGE"), "#!/bin/sh\necho ok").unwrap();
        // Machbar machen
        std::fs::set_permissions(dir.join("Contents/MacOS/DREDGE"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        let binary = gog_binary_pfad(&dir).unwrap();
        assert!(binary.ends_with("Contents/MacOS/DREDGE"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gog_binary_pfad_fallback_ordnername() {
        let dir = std::env::temp_dir().join("famulus-binary-test2");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Contents/MacOS")).unwrap();
        std::fs::write(dir.join("Contents/MacOS/famulus-binary-test2"), "data").unwrap();
        std::fs::set_permissions(dir.join("Contents/MacOS/famulus-binary-test2"), std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
        // Leere Info.plist
        std::fs::write(dir.join("Contents/Info.plist"), "<?xml version=\"1.0\"?><plist><dict/></plist>").unwrap();
        let binary = gog_binary_pfad(&dir).unwrap();
        assert!(binary.ends_with("Contents/MacOS/famulus-binary-test2"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}