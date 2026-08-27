// Famulus Games – UniFFI-Bindgen-Einstiegspunkt v0.2.0.
// Wird per `cargo run --bin uniffi-bindgen -- …` aufgerufen und
// erzeugt aus der kompilierten Bibliothek (libfamulus_games.a)
// die Swift-Bindings. scripts/build-ffi.sh kapselt den Aufruf.
fn main() {
    uniffi::uniffi_bindgen_main()
}
