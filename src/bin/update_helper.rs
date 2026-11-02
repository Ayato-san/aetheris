#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    if let Err(error) = aetheris::companion::runtime::run_helper() {
        aetheris::companion::runtime::report(&format!(
            "Le helper de mise à jour ne peut pas poursuivre : {error}"
        ));
    }
}
