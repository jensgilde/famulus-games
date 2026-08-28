// Famulus Games – Einstiegspunkt der nativen SwiftUI-Hülle v0.2.5.
// Dieselbe Marken-DNA wie der Famulus-Kern: dunkles Terminal-Design,
// Orange-Akzent (#F86E27), Monospace. Die Logik liegt im
// Rust-Kern (libfamulus_games.a via UniFFI).

import SwiftUI

@main
struct FamulusGamesApp: App {
    var body: some Scene {
        WindowGroup("Famulus Games") {
            InhaltAnsicht()
                .frame(minWidth: 760, minHeight: 480)
        }
        .defaultSize(width: 1100, height: 720)
    }
}
