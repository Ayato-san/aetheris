//! Wait for real Windows image/file users, beyond cooperative installation locks.
use super::Result;
use std::path::Path;

#[cfg(windows)]
mod windows;

pub(super) fn wait(root: &Path, progress: impl Fn(&str)) -> Result<()> {
    #[cfg(windows)]
    {
        windows::wait(root, progress, std::time::Duration::from_secs(30))
    }
    #[cfg(not(windows))]
    {
        let _ = (root, progress);
        Ok(())
    }
}

pub(super) fn replace(from: &Path, to: &Path) -> Result<()> {
    #[cfg(windows)]
    {
        windows::replace(from, to)
    }
    #[cfg(not(windows))]
    {
        super::durable_rename(from, to)
    }
}

#[cfg(windows)]
pub(super) fn wait_for_installer(root: &Path) -> Result<()> {
    windows::wait_for_installer(root)
}

#[cfg(all(windows, test))]
pub(super) mod tests {
    use super::*;
    use std::{
        io::Write,
        os::windows::process::CommandExt,
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };

    pub(in crate::companion) struct Fixture {
        pub child: Child,
    }
    impl Fixture {
        pub fn prepare(root: &Path, name: &str) {
            std::fs::copy(std::env::current_exe().unwrap(), root.join(name)).unwrap();
        }
        pub fn spawn(root: &Path, name: &str) -> Self {
            Self::spawn_mode(root, name, "mapped")
        }
        fn spawn_mode(root: &Path, name: &str, mode: &str) -> Self {
            let child = Command::new(root.join(name))
                .args([
                    "--exact",
                    "companion::program_files::tests::mapped_executable_fixture",
                    "--ignored",
                    "--nocapture",
                ])
                .env("AETHERIS_PROGRAM_LOCK_FIXTURE", root)
                .env("AETHERIS_PROGRAM_LOCK_MODE", mode)
                .env("LOCALAPPDATA", root.join("fixture-profile"))
                .creation_flags(0x08000000)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let mut fixture = Self { child };
            let deadline = Instant::now() + Duration::from_secs(10);
            while !root.join("fixture-ready").exists() {
                if let Some(exit) = fixture.child.try_wait().unwrap() {
                    assert!(
                        exit.success() && root.join("fixture-ready").exists(),
                        "fixture exited before acknowledgement"
                    );
                }
                assert!(Instant::now() < deadline, "fixture startup timed out");
                std::thread::sleep(Duration::from_millis(10));
            }
            fixture
        }
        pub fn release(&mut self) {
            self.child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(b"exit\n")
                .unwrap();
            assert!(self.child.wait().unwrap().success());
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // Only this test-owned child is terminated when an assertion fails.
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
            }
            let _ = self.child.wait();
        }
    }

    #[test]
    #[ignore = "child-process fixture invoked only by executable replacement tests"]
    fn mapped_executable_fixture() {
        let Some(root) = std::env::var_os("AETHERIS_PROGRAM_LOCK_FIXTURE") else {
            return;
        };
        let root = std::path::PathBuf::from(root);
        if std::env::var("AETHERIS_PROGRAM_LOCK_MODE").as_deref() == Ok("blocked-launch") {
            std::fs::write(root.join("fixture-ready"), b"ready").unwrap();
            assert!(crate::companion::runtime::installation_is_updating(&root));
            return;
        }
        // Reproduce the exact shutdown gap: the cooperative lease is released,
        // yet the process and its executable mapping remain alive.
        let lease = crate::companion::Lease::acquire(&root.join("cooperative.lock")).unwrap();
        drop(lease);
        std::fs::write(root.join("fixture-ready"), b"ready").unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    }

    #[test]
    fn launching_during_an_update_exits_without_holding_a_modal_dialog() {
        let directory = crate::companion::tests::Directory::new();
        Fixture::prepare(&directory.0, "aetheris-companion.exe");
        let _update = crate::companion::InstallationLease::acquire_update(&directory.0).unwrap();
        let mut child =
            Fixture::spawn_mode(&directory.0, "aetheris-companion.exe", "blocked-launch");
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(exit) = child.child.try_wait().unwrap() {
                assert!(exit.success());
                break;
            }
            assert!(
                Instant::now() < deadline,
                "companion is still mapping its executable during update"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
