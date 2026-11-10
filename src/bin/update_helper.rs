#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    if let Err(error) = aetheris::companion::managed::entry() {
        if matches!(
            std::env::args().nth(1).as_deref(),
            Some("--managed-worker" | "--capture-service")
        ) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        aetheris::companion::runtime::report(&format!(
            "Le helper de mise à jour ne peut pas poursuivre : {error}"
        ));
        std::process::exit(1);
    }
}
