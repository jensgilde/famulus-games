// Famulus Games – Fußzeile v0.2.5.
// Links Zähler, rechts „Aktualisieren“ + Version.

import SwiftUI

struct Fußzeile: View {
    @Bindable var store: BibliothekStore

    var body: some View {
        HStack {
            Text("\(store.sichtbar.count) von \(store.spiele.count) Spielen")
            Spacer()
            Button {
                store.laden()
            } label: {
                Image(systemName: "arrow.clockwise")
                    .font(.system(size: 12))
            }
            .buttonStyle(.plain)
            .foregroundStyle(Marke.textLeise)
            Text("Famulus Games v\(store.version)")
        }
        .font(.system(size: 11))
        .foregroundStyle(Marke.textLeise)
        .padding(.horizontal, 16).padding(.vertical, 8)
        .background(Marke.fußFläche)
    }
}
