// Famulus Games – Zustandsmodell v0.2.3.
// Ruft den Rust-Kern über die UniFFI-Bindings (Generated/).
// Alle FFI-Aufrufe laufen auf Hintergrund-Tasks, damit die UI
// nie am Datei-I/O oder an curl hängt.

import SwiftUI
import Observation

/// Quellen-Filter der Kopfzeile.
enum QuellenFilter: String, CaseIterable, Identifiable {
    case alle = "Alle"
    case steam = "Steam"
    case gog = "GOG"
    var id: String { rawValue }
}

struct ToastInfo: Equatable {
    var meldung: String
    var fehler: Bool
}

@MainActor
@Observable
final class BibliothekStore {
    var spiele: [Spiel] = []
    var suche = ""
    var filter: QuellenFilter = .alle
    var toast: ToastInfo?

    let version = appVersion()
    private var toastAufgabe: Task<Void, Never>?

    /// Gefilterte Bibliothek (Quelle + Suchtext).
    var sichtbar: [Spiel] {
        spiele.filter { spiel in
            let passtQuelle: Bool
            switch filter {
            case .alle:  passtQuelle = true
            case .steam: passtQuelle = spiel.quelle == "Steam"
            case .gog:   passtQuelle = spiel.quelle == "GOG"
            }
            guard passtQuelle else { return false }
            return suche.isEmpty
                || spiel.titel.localizedLowercase.contains(suche.localizedLowercase)
        }
    }

    /// Bibliothek frisch vom Kern einlesen (Hintergrund).
    func laden() {
        // `Task` erbt den MainActor dieser Funktion – die Zuweisung
        // unten ist also sicher; die FFI-Arbeit läuft abgekoppelt.
        Task {
            let neu = await Task.detached { sammeleSpiele() }.value
            spiele = neu
        }
    }

    /// Spiel starten; meldet Erfolg/Fehler als Toast.
    func starten(_ spiel: Spiel) async {
        // Steam ohne laufende Instanz? Der Kern startet Steam dann mit –
        // das erklären wir kurz, statt den Nutzer zu überraschen.
        let steamWarAus = spiel.quelle == "Steam" && !steamLaeuft()
        do {
            _ = try await Task.detached {
                try starteSpiel(quelle: spiel.quelle, id: spiel.id, pfad: spiel.pfad)
            }.value
            toastAnzeigen(steamWarAus
                ? "\(spiel.titel): Steam wird mitgestartet"
                : "\(spiel.titel) gestartet")
        } catch {
            toastAnzeigen("Start fehlgeschlagen: \(fehlerText(error))", fehler: true)
        }
    }

    func toastAnzeigen(_ meldung: String, fehler: Bool = false) {
        toastAufgabe?.cancel()
        toast = ToastInfo(meldung: meldung, fehler: fehler)
        toastAufgabe = Task { [weak self] in
            try? await Task.sleep(nanoseconds: 2_200_000_000)
            if !Task.isCancelled { self?.toast = nil }
        }
    }

    /// Fehlermeldungen aus der FFI sind Enum-Fälle – auspacken.
    private func fehlerText(_ error: Error) -> String {
        if case Fehler.Nachricht(let meldung) = error { return meldung }
        return error.localizedDescription
    }
}
