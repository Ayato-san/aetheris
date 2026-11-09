# Windows installer and update service

Status: accepted; design validated on 3 November 2026. Implementation deferred
to a later session. Detailed behavior is recorded in
[the installer design](../windows-installer.md).

Aetheris will provide an Inno Setup installer for all Windows users, defaulting
to `C:\Program Files\Aetheris`, while retaining the release ZIP. The existing
GitHub Actions workflow will invoke the official compiler directly on a Windows
runner, without a third-party installer Action.

To retain the protected shared installation directory while avoiding elevation
prompts for subsequent updates, the installer will install a Windows update
service using its initial administrator authorization. The service will apply
authenticated Aetheris updates and be removed during uninstallation; the
application itself will continue to run without administrator privileges. This
adds a privileged component to maintain instead of requesting elevation for
each update or moving the programs into a per-user directory.

Personal data, including `Engine.baseline.ini`, will live per Windows user in
`%LOCALAPPDATA%\Aetheris` and survive uninstallation. Uninstallation will leave
the game's `Engine.ini` unchanged. For installer-managed installations, this
supersedes the executable-adjacent baseline location in ADR 0001 and the
non-elevating update limitation in ADR 0004; other update integrity and recovery
guarantees remain applicable. The update channel is shared by all users of the
managed installation, and updates wait until all application instances have
closed without forcibly terminating them. Manual ZIP installations remain
independent of the service and update directly only when their directory is
writable.
