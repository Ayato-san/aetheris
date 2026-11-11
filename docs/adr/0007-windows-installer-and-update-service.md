# Windows installer, update and capture services

Status: accepted; design validated on 3 November 2026, implemented from
9 November 2026 and amended on 11 November 2026. Detailed behavior is recorded in
[the installer design](../windows-installer.md).

Aetheris provides an Inno Setup installer for all Windows users, defaulting
to `C:\Program Files\Aetheris`, while retaining the release ZIP. The existing
GitHub Actions workflow invokes the official compiler directly on a Windows
runner, without a third-party installer Action.

To retain the protected shared installation directory while avoiding elevation
prompts for subsequent updates, the installer installs a Windows update
service using its initial administrator authorization. The service applies
authenticated Aetheris updates and is removed during uninstallation; the
application itself continues to run without administrator privileges. This
adds a privileged component to maintain instead of requesting elevation for
each update or moving the programs into a per-user directory.

Personal data, including `Engine.baseline.ini`, lives per Windows user in
`%LOCALAPPDATA%\Aetheris` and survives uninstallation. Uninstallation leaves
the game's `Engine.ini` unchanged. For installer-managed installations, this
supersedes the executable-adjacent baseline location in ADR 0001 and the
non-elevating update limitation in ADR 0004; other update integrity and recovery
guarantees remain applicable. The update channel is shared by all users of the
managed installation, and updates wait until all application instances have
closed without forcibly terminating them. Manual ZIP installations remain
independent of the service and update directly only when their directory is
writable.

## Privileged service boundaries

`AetherisUpdate` runs as LocalSystem from a protected bootstrap and accepts fixed
SCM controls for its registered installation. It downloads and authenticates
release packages itself; user requests cannot supply a destination, executable
or URL. The protected installation and its parent paths must not be replaceable
by a standard user. The new signed package's worker applies the update, retaining
the migration history needed when intermediate releases are skipped.

A separate LocalSystem service, `AetherisCapture`, runs the signed installed
helper for native Winsock acquisition under [ADR 0003](0003-debug-combat-capture.md).
The installer configures automatic startup, failure recovery and an inbound TCP
firewall rule restricted to that installed executable. Players neither elevate
the GUI nor configure firewall permissions manually. Separating acquisition from
the GUI supports long-running capture without giving the interface administrator
rights. Automatic recording with the GUI closed is not implemented by this
decision; the current debug interface still starts and stops each capture.

Capture IPC authenticates Windows endpoint ownership and the installed client
image. Client identity includes its SID and Windows session; game selection uses
the session and permits a different SID, so UAC can launch AION with a separate
administrator account. Other Windows sessions are excluded. Storage, decoding
and personal SQLite migrations remain with the original user; neither service
accepts a personal database path or opens it with SYSTEM privileges. Database
owners retain schema backups and preserve post-update data during rollback.

## Replacement, repair and recovery

Shared instance leases and an exclusive update lease coordinate the installer,
companion, GUI and update service across Windows sessions. A same-version
installer repairs the installation; newer installers upgrade it and older
installers are refused. Programs are staged and replaced through the helper's
durable transaction journal, preserving signed backups and recovery on failure.

Cooperative locks alone proved insufficient: a process can release its lease
while Windows still maps its executable, causing access-denied errors during
repair or update. Before replacement, restoration or journal recovery, the helper
queries Restart Manager and waits up to 30 seconds for actual program users to
exit. The installer includes its bootstrap and staging executables in this check.
It reports remaining blockers by name and PID without forcibly closing them.
Atomic replacements retry transient Windows errors 5, 32 and 33 for at most five
seconds while retaining the old file until replacement succeeds. Persistent
failures preserve recovery state and report both the original and rollback errors
if rollback also fails. A managed worker passes its detailed failure back through
the update service instead of reducing it to a generic exit-status message.

A launch attempted during replacement activates the existing progress window
and exits promptly. Blocking dialogs in the installed application or companion
would keep their own images mapped without an instance lease, so they are avoided
on this path. This amends the companion launch behavior in
[ADR 0004](0004-companion-crash-diagnostics-and-updates.md).

Before replacing the installed helper, the updater stops the capture service and
attempts to restart it after success or a recoverable failure. The installer also
attempts a restart when a repair is aborted. The new authenticated worker
configures and starts capture even when the existing update bootstrap predates
that service; relying on the old bootstrap would omit service installation.
Firewall configuration always targets the installed helper, never a temporary
worker. Capture startup waits for the running state, and client diagnostics
distinguish an absent service from a stopped service with a recorded startup error.
Uninstallation removes both services and the capture firewall rule while retaining
personal data.

## Validation limits

Native Windows tests cover executable images remaining mapped after cooperative
lease release, same-version repair, version upgrades, bounded sharing retries and
prompt exit on an update-time launch. Capture tests cover IPC authentication and
game discovery with a simulated different client SID in the same Windows session.
They do not replace a complete installer exercise with actual separate
administrator credentials, multiple logged-in sessions, interrupted updates,
both directions of live game traffic and uninstallation. The remaining system
checks are maintained in [the installer guide](../windows-installer.md).
