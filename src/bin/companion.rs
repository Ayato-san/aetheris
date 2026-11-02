#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    if let Err(error) = aetheris::companion::runtime::run() {
        aetheris::companion::runtime::report(&format!(
            "Le compagnon ne peut pas poursuivre : {error}"
        ));
    }
}
