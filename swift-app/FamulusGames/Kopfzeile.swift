// Famulus Games – Kopfzeile v0.2.4.
// Marke links, Suchfeld in der Mitte, Quell-Filter rechts.

import SwiftUI

struct Kopfzeile: View {
    @Bindable var store: BibliothekStore

    var body: some View {
        HStack(spacing: 12) {
            marke
            suchfeld
                .frame(maxWidth: 340)
            Spacer(minLength: 8)
            filterChips
        }
        .padding(.horizontal, 16).padding(.vertical, 12)
        .background(Marke.kopfFläche)
    }

    private var marke: some View {
        HStack(spacing: 0) {
            Text("Famulus ")
            Text("Games").foregroundStyle(Marke.akzent)
        }
        .font(.system(size: 15, weight: .bold))
        .foregroundStyle(Marke.text)
    }

    private var suchfeld: some View {
        HStack(spacing: 6) {
            Image(systemName: "magnifyingglass")
                .font(.system(size: 12)).foregroundStyle(Marke.textLeise)
            TextField("", text: $store.suche, prompt:
                Text("Suchen…").foregroundStyle(Marke.textLeise))
                .textFieldStyle(.plain)
                .font(.system(size: 13))
                .foregroundStyle(Marke.text)
        }
        .padding(.horizontal, 10).padding(.vertical, 6)
        .background(RoundedRectangle(cornerRadius: 6).fill(Marke.eingabe))
        .overlay(RoundedRectangle(cornerRadius: 6)
            .stroke(Marke.rand, lineWidth: 1))
    }

    private var filterChips: some View {
        HStack(spacing: 6) {
            ForEach(QuellenFilter.allCases) { f in
                chip(f)
            }
        }
    }

    private func chip(_ f: QuellenFilter) -> some View {
        let aktiv = store.filter == f
        return Button {
            store.filter = f
        } label: {
            Text(f.rawValue)
                .font(.system(size: 12))
                .foregroundStyle(aktiv ? Marke.akzent : Marke.textLeise)
                .padding(.horizontal, 12).padding(.vertical, 4)
                .background(Capsule().fill(aktiv ? Marke.akzentGetönt : .clear))
                .overlay(Capsule().stroke(aktiv ? Marke.akzent : Marke.rand, lineWidth: 1))
        }
        .buttonStyle(.plain)
    }
}
