// Famulus Games v0.1.0 – Grafische Oberfläche (Tauri 2), dieselbe
// Bauweise wie das Famulus-GUI. Startet die Bibliothek und bietet
// das Fenster an. Version kommt aus Cargo.toml (env!).
//
// Kein Konsolenfenster unter Windows. Auf Linux wirkungslos.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("famulus-games {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    famulus_games_gui::run();
}
