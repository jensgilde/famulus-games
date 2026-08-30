// Famulus Games – Spielkarte v0.2.5.
// Cover (2:3) + Titelzeile + Hover-Overlay „Spielen“.
// Das Cover lädt der Rust-Kern (Cache/Download) – die Datei-URL
// wird asynchron in ein Bild verwandelt.

import SwiftUI

struct SpielKarte: View {
    @Bindable var store: BibliothekStore
    let spiel: Spiel

    @State private var coverBild: CGImage?
    @State private var fährt = false
    @State private var hover = false

    var body: some View {
        VStack(spacing: 0) {
            coverFläche
            info
        }
        .background(Marke.fläche)
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(hover ? Marke.akzent : Marke.rand, lineWidth: 1)
        )
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .overlay(spieldOverlay)
        .onHover { hover = $0 }
        .onTapGesture { spielen() }
        .task { await coverLaden() }
        .animation(.easeInOut(duration: 0.15), value: hover)
    }

    // ── Cover: Bild oder Initial-Buchstabe ──
    @ViewBuilder
    private var coverFläche: some View {
        ZStack {
            Marke.flächeDunkel
            if let bild = coverBild {
                Image(decorative: bild, scale: 1)
                    .resizable()
                    .aspectRatio(contentMode: .fill)
            } else {
                Text(ersterBuchstabe)
                    .font(.system(size: 42, weight: .bold))
                    .foregroundStyle(Marke.textHauch)
            }
        }
        .aspectRatio(2/3, contentMode: .fit)
        .clipped()
    }

    private var ersterBuchstabe: String {
        spiel.titel.first.map { String($0).uppercased() } ?? "?"
    }

    // ── Titelzeile mit Quell-Badge ──
    private var info: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(spiel.titel)
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(Marke.text)
                .lineLimit(1)
                .truncationMode(.tail)
            HStack {
                badge
                Spacer()
                if spiel.groesseBytes > 0 {
                    Text(formatGroesse(bytes: spiel.groesseBytes))
                        .font(.system(size: 10))
                        .foregroundStyle(Marke.textLeise)
                }
            }
        }
        .padding(.horizontal, 10).padding(.vertical, 8)
        .overlay(alignment: .top) {
            Rectangle().fill(Marke.rand).frame(height: 1)
        }
    }

    private var badge: some View {
        Text(spiel.quelle)
            .font(.system(size: 10, weight: .semibold))
            .textCase(.uppercase)
            .tracking(0.8)
            .foregroundStyle(spiel.quelle == "GOG" ? Marke.gogViolett : Marke.steamBlau)
    }

    // ── Hover-Overlay „Spielen“ ──
    @ViewBuilder
    private var spieldOverlay: some View {
        ZStack {
            if hover || fährt {
                Color.black.opacity(0.55)
                Text(fährt ? "Startet…" : "Spielen")
                    .font(.system(size: 13, weight: .bold))
                    .foregroundStyle(Marke.grundFläche)
                    .padding(.horizontal, 20).padding(.vertical, 8)
                    .background(RoundedRectangle(cornerRadius: 6)
                        .fill(fährt ? Marke.textLeise : Marke.akzent))
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .allowsHitTesting(false)
    }

    private func spielen() {
        guard !fährt else { return }
        fährt = true
        Task {
            await store.starten(spiel)
            try? await Task.sleep(nanoseconds: 1_500_000_000)
            fährt = false
        }
    }

    private func coverLaden() async {
        guard coverBild == nil else { return }
        let kopie = spiel
        let urlText = await Task.detached { () -> String? in
            try? holeCoverDatei(
                spielId: kopie.id, cover: kopie.cover, coverUrl: kopie.coverUrl)
        }.value
        guard let urlText, let url = URL(string: urlText) else { return }
        // Cover ist lokal (file://) – per Data laden statt AsyncImage,
        // das file:-URLs nicht zuverlässig behandelt.
        let bild = await Task.detached { () -> CGImage? in
            guard let daten = try? Data(contentsOf: url) else { return nil }
            guard let src = CGImageSourceCreateWithData(daten as CFData, nil) else { return nil }
            return CGImageSourceCreateImageAtIndex(src, 0, nil)
        }.value
        if let bild { coverBild = bild }
    }
}
