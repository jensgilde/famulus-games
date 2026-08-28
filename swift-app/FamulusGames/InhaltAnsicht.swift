// Famulus Games – Hauptansicht v0.2.5.
// Aufbau wie der Tauri-Vorgänger: Kopfzeile (Marke + Suche + Filter),
// Bibliotheks-Grid in der Mitte, Fußzeile mit Zähler und Version.

import SwiftUI

struct InhaltAnsicht: View {
    @State private var store = BibliothekStore()

    var body: some View {
        VStack(spacing: 0) {
            Kopfzeile(store: store)
            Divider().overlay(Marke.rand)
            bibliothek
            Divider().overlay(Marke.rand)
            Fußzeile(store: store)
        }
        .background(Marke.hintergrund)
        .preferredColorScheme(.dark)
        .overlay(alignment: .bottom) { toast }
        .task { store.laden() }
    }

    // ── Mitte: Grid oder Leerzustand ──
    @ViewBuilder
    private var bibliothek: some View {
        let sichtbar = store.sichtbar
        if sichtbar.isEmpty {
            leerzustand
        } else {
            ScrollView {
                LazyVGrid(
                    columns: [GridItem(.adaptive(minimum: 170, maximum: 240), spacing: 18)],
                    spacing: 18
                ) {
                    ForEach(sichtbar, id: \.id) { spiel in
                        SpielKarte(store: store, spiel: spiel)
                    }
                }
                .padding(20)
                .padding(.horizontal, 16)
            }

        }
    }

    private var leerzustand: some View {
        VStack(spacing: 8) {
            if store.spiele.isEmpty {
                Text("Keine Spiele gefunden").foregroundStyle(Marke.akzent)
                    .font(.system(size: 22, weight: .bold))
                Text("Steam- und Heroic-Bibliotheken werden beim Start gelesen.")
            } else {
                Text("Nichts passt zu dieser Auswahl")
            }
        }
        .foregroundStyle(Marke.textLeise)
        .font(.system(size: 13))
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // ── Toast-Meldung unten Mitte ──
    @ViewBuilder
    private var toast: some View {
        if let toast = store.toast {
            Text(toast.meldung)
                .font(.system(size: 12))
                .foregroundStyle(toast.fehler ? Marke.gefahr : Marke.text)
                .padding(.horizontal, 16).padding(.vertical, 7)
                .background(RoundedRectangle(cornerRadius: 6).fill(Marke.hover))
                .overlay(RoundedRectangle(cornerRadius: 6)
                    .stroke(toast.fehler ? Marke.gefahr : Marke.randHover, lineWidth: 1))
                .padding(.bottom, 46)
                .transition(.opacity)
                .animation(.easeInOut(duration: 0.2), value: store.toast)
        }
    }
}
