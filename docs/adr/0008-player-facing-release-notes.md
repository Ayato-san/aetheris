# Player-facing release notes by channel

Status: accepted; implemented on 3 November 2026. The implementation
specification, local hook installation and verification commands are in
[the release notes design](../changes.md).

Aetheris will maintain separate beta and stable release histories on `main`.
Beta notes describe each release's changes; stable notes are an explicitly
maintained summary relative to the previous stable release. This editorial
work avoids exposing players to repeated beta adjustments or fixes for bugs
that never reached stable, which automatic concatenation or commit-derived
notes would include.

Commits will update the relevant pending notes or provide a justified exemption
per channel, checked by a local commit hook. Release tooling will freeze the
appropriate notes with the version and UTC timestamp, publish them to GitHub,
and support resuming a failed publication without freezing another version.
The future patch notes page will consume these histories in a separate task.
