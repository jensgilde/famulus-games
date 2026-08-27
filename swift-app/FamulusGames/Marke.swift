// Famulus Games – Design-Tokens v0.2.0.
// Dieselben Werte wie ui/index.html (Tauri-Vorgänger): dunkles
// Terminal-Design, Famulus-Gelb. Magenta ist seit 2026-08-27
// dauerhaft raus.

import SwiftUI

enum Marke {
    // Akzent: Famulus-Gelb
    static let akzent      = Color(red: 1.000, green: 0.773, blue: 0.239) // #FFC53D
    static let akzentHover = Color(red: 1.000, green: 0.820, blue: 0.400) // #FFD166
    static let akzentGetönt = akzent.opacity(0.12)

    // Flächen
    static let hintergrund  = Color.black                                  // #000000
    static let fläche       = Color(red: 0.031, green: 0.031, blue: 0.031) // #080808
    static let flächeDunkel = Color(red: 0.051, green: 0.051, blue: 0.051) // #0d0d0d
    static let eingabe      = Color(red: 0.039, green: 0.039, blue: 0.039) // #0a0a0a
    static let hover        = Color(red: 0.078, green: 0.078, blue: 0.078) // #141414

    // Text
    static let text          = Color(red: 0.878, green: 0.878, blue: 0.878) // #e0e0e0
    static let textSekundär  = Color(red: 0.627, green: 0.627, blue: 0.627) // #a0a0a0
    static let textLeise     = Color(red: 0.376, green: 0.376, blue: 0.376) // #606060
    static let textHauch     = Color(red: 0.227, green: 0.227, blue: 0.227) // #3a3a3a

    // Ränder & Signale
    static let rand      = Color(red: 0.102, green: 0.102, blue: 0.102)     // #1a1a1a
    static let randHover = Color(red: 0.165, green: 0.165, blue: 0.165)     // #2a2a2a
    static let gefahr    = Color(red: 1.000, green: 0.267, blue: 0.267)     // #ff4444

    // Quellen-Farben (wie in der Tauri-Oberfläche)
    static let steamBlau   = Color(red: 0.400, green: 0.753, blue: 0.957)   // #66c0f4
    static let gogViolett  = Color(red: 0.690, green: 0.478, blue: 1.000)   // #b07aff
}
