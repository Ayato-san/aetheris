//! Complete a managed program replacement with the capture service provisioned.
use super::Result;

pub(super) trait Service {
    fn exists(&mut self) -> Result<bool>;
    fn stop(&mut self) -> Result<()>;
    fn configure(&mut self) -> Result<()>;
    fn start(&mut self) -> Result<()>;
}

pub(super) fn replace<T>(
    service: &mut impl Service,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let existed = service.exists()?;
    if existed {
        service.stop()?;
    }
    match operation() {
        Ok(value) => {
            // The previous update bootstrap may predate capture entirely. The
            // newly authenticated worker must perform this migration itself.
            service.configure()?;
            service.start()?;
            Ok(value)
        }
        Err(error) => {
            if existed {
                let _ = service.start();
            }
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    struct Host {
        installed: bool,
        running: bool,
        configuration_fails: bool,
        events: Rc<RefCell<Vec<&'static str>>>,
    }
    impl Service for Host {
        fn exists(&mut self) -> Result<bool> {
            Ok(self.installed)
        }
        fn stop(&mut self) -> Result<()> {
            self.events.borrow_mut().push("stop");
            self.running = false;
            Ok(())
        }
        fn configure(&mut self) -> Result<()> {
            self.events.borrow_mut().push("configure");
            if self.configuration_fails {
                return Err("configuration refused".into());
            }
            self.installed = true;
            Ok(())
        }
        fn start(&mut self) -> Result<()> {
            assert!(self.installed);
            self.events.borrow_mut().push("start");
            self.running = true;
            Ok(())
        }
    }
    fn host(installed: bool) -> Host {
        Host {
            installed,
            running: installed,
            configuration_fails: false,
            events: Rc::new(RefCell::new(Vec::new())),
        }
    }

    #[test]
    fn update_from_a_pre_capture_installation_creates_and_starts_the_service() {
        let mut service = host(false);
        let events = service.events.clone();
        replace(&mut service, || {
            events.borrow_mut().push("apply signed programs");
            Ok(())
        })
        .unwrap();
        assert!(service.installed && service.running);
        assert_eq!(
            *events.borrow(),
            ["apply signed programs", "configure", "start"]
        );
    }

    #[test]
    fn existing_capture_releases_programs_before_replacement_and_restarts() {
        let mut service = host(true);
        let events = service.events.clone();
        replace(&mut service, || {
            events.borrow_mut().push("apply signed programs");
            Ok(())
        })
        .unwrap();
        assert!(service.running);
        assert_eq!(
            *events.borrow(),
            ["stop", "apply signed programs", "configure", "start"]
        );
    }

    #[test]
    fn failed_replacement_restores_only_a_preexisting_service() {
        for existed in [false, true] {
            let mut service = host(existed);
            let error =
                replace(&mut service, || Err::<(), _>("replacement failed".into())).unwrap_err();
            assert_eq!(error.to_string(), "replacement failed");
            assert_eq!(service.installed, existed);
            assert_eq!(service.running, existed);
            assert!(!service.events.borrow().contains(&"configure"));
        }
    }

    #[test]
    fn failed_provisioning_cannot_be_reported_as_a_successful_update() {
        let mut service = host(false);
        service.configuration_fails = true;
        assert!(replace(&mut service, || Ok(())).is_err());
        assert!(!service.running);
    }
}
