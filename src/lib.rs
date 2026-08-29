// Famulus Games – Kern-Bibliothek v0.2.0.
// Liest die Spiele-Bibliotheken von Steam und Heroic (GOG) ein und
// vereinigt sie zu einer Liste. Nur lesen, nichts anfassen – das
// Das Starten macht bridge::starte_spiel (über die UniFFI-Schicht).

pub mod bridge;
pub mod gog;
pub mod steam;
pub mod steamcmd;

// UniFFI-Scaffolding muss im Crate-Root expandiert werden
// (erzeugt crate::UniFfiTag). Die UDL-Funktionen finden sich
// über die Re-Exporte unten; sammele_spiele() liegt hier direkt.
uniffi::include_scaffolding!("ffi");
pub use bridge::{app_version, format_groesse, hole_cover_datei, starte_spiel, steam_laeuft, Fehler};

/// Ein Spiel, egal aus welcher Quelle.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Spiel {
    /// Quell-ID: Steam-AppID oder GOG app_name.
    pub id: String,
    /// Anzeigetitel.
    pub titel: String,
    /// Quelle: "Steam" oder "GOG".
    pub quelle: String,
    /// Installationspfad (bei Steam der Spieleordner).
    pub pfad: String,
    /// Lokales Cover (absoluter Pfad), leer wenn keins gefunden.
    pub cover: String,
    /// Cover-URL (GOG), leer wenn keine.
    pub cover_url: String,
    /// Zuletzt gespielt als Unix-Zeitstempel (0 = unbekannt).
    pub zuletzt_gespielt: u64,
    /// Installierte Größe in Bytes (0 = unbekannt).
    pub groesse_bytes: u64,
}

/// Vereint alle gefundenen Spiele. Die Quellen sind unabhängig:
/// fällt eine aus (Heroic deinstalliert, Steam fehlt), liefern die
/// anderen trotzdem.
pub fn sammele_spiele() -> Vec<Spiel> {
    let mut spiele = Vec::new();
    if let Ok(s) = steam::liese_steam_spiele() {
        spiele.extend(s);
    }
    if let Ok(s) = gog::liese_gog_spiele() {
        spiele.extend(s);
    }
    // Alphabetisch – die Bibliothek soll ruhig stehen.
    spiele.sort_by_key(|a| a.titel.to_lowercase());
    spiele
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sortiert_nach_titel_ignoriert_grossschreibung() {
        let mut v = vec![
            Spiel { id: "1".into(), titel: "zelda".into(), quelle: "Steam".into(), pfad: String::new(), cover: String::new(), cover_url: String::new(), zuletzt_gespielt: 0, groesse_bytes: 0 },
            Spiel { id: "2".into(), titel: "Asteroids".into(), quelle: "GOG".into(), pfad: String::new(), cover: String::new(), cover_url: String::new(), zuletzt_gespielt: 0, groesse_bytes: 0 },
        ];
        v.sort_by(|a, b| a.titel.to_lowercase().cmp(&b.titel.to_lowercase()));
        assert_eq!(v[0].titel, "Asteroids");
    }

    #[test]
    fn sammeln_liefert_auf_fremdmaschine_leer_statt_absturz() {
        // Auf einer Maschine ohne Steam/Heroic darf das nicht paniken –
        // leere Liste ist ein gültiges Ergebnis.
        let _ = sammele_spiele();
    }
}
