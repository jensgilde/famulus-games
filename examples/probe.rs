// Famulus Games – Diagnose-Werkzeug v0.2.2.
// Zeigt, was der Kern liefert, ohne dass die GUI laufen muss:
//   cargo run --example probe
// Nützlich, wenn die App weniger Spiele zeigt als erwartet –
// so sieht man sofort, ob der Kern sie überhaupt findet.

fn main() {
    let spiele = famulus_games::sammele_spiele();
    println!("Kern liefert {} Spiele:", spiele.len());
    for s in &spiele {
        println!(
            "  [{}] {} ({}) pfad={} cover={} url={} groesse={}",
            s.quelle, s.titel, s.id, s.pfad, s.cover, s.cover_url, s.groesse_bytes
        );
    }
}
