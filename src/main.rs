#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod ui;

fn main() {
    use aetheris::companion::client::{self, Event};
    match client::attach() {
        Ok(true) => {}
        Ok(false) => {
            if let Err(error) = client::delegate() {
                aetheris::companion::runtime::report(&format!(
                    "Impossible de lancer le compagnon : {error}"
                ));
            }
            return;
        }
        Err(error) => {
            aetheris::companion::runtime::report(&format!(
                "Connexion au compagnon impossible : {error}"
            ));
            return;
        }
    }
    // Keep the shared installation lease in the GUI as well: a companion crash
    // must never let the service update while this process is still running.
    let _managed_installation = if aetheris::companion::managed::current_is_managed() {
        match std::env::current_exe()
            .map_err(|e| e.into())
            .and_then(|p| aetheris::companion::InstallationLease::acquire(p.parent().unwrap()))
        {
            Ok(lease) => Some(lease),
            Err(error) => {
                aetheris::companion::runtime::report(&format!(
                    "Installation indisponible : {error}"
                ));
                return;
            }
        }
    } else {
        None
    };
    let _instance = match aetheris::companion::data_directory()
        .and_then(|data| aetheris::companion::Lease::acquire(&data.join("application.lock")))
    {
        Ok(lease) => lease,
        Err(error) => {
            aetheris::companion::runtime::report(&format!(
                "Application déjà ouverte ou accès impossible : {error}"
            ));
            return;
        }
    };
    if let Ok(executable) = std::env::current_exe()
        && let Some(root) = executable.parent()
        && (!cfg!(debug_assertions) || root.join(aetheris::companion::manifest::MANIFEST).exists())
        && aetheris::companion::manifest::public_key()
            .and_then(|key| aetheris::companion::manifest::verify(root, &key))
            .is_err()
    {
        eprintln!("Intégrité locale non établie ; démarrage refusé.");
        return;
    }
    client::send(Event::Progress("Initialisation de GPUI".into()));
    ui::run();
    client::finish_shutdown();
}
