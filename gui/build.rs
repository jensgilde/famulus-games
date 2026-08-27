fn main() {
    println!("cargo:rerun-if-changed=../ui/index.html");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities/default.json");
    println!("cargo:rerun-if-changed=icons/icon.png");
    tauri_build::build()
}
