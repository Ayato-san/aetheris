#!/usr/bin/env python3
"""Prepare a release, or recover its immutable notes and source from a tag."""
import os
from pathlib import Path
import re
import subprocess
import tempfile

from changelog import FILES, VERSION, git, parse, pending, timestamp


def prepare(base, channel, resume_tag=""):
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", base):
        raise ValueError("Expected major.minor.patch without leading zeroes")
    if channel not in FILES:
        raise ValueError("Unknown release channel")
    filename = FILES[channel]
    if resume_tag:
        if not re.fullmatch(rf"v{VERSION}", resume_tag):
            raise ValueError("Invalid resume tag")
        version = resume_tag[1:]
        if version.split("-")[0] != base or ("-beta." in version) != (channel == "beta"):
            raise ValueError("Resume tag does not match the requested version/channel")
        history = parse(git("show", f"{resume_tag}:{filename}"), channel)
        matches = [entry for entry in history.entries[1:] if entry.label.split(" — ")[0] == version]
        if len(matches) != 1:
            raise ValueError("The tag must contain frozen notes for its version")
        for name in ("Cargo.toml", "Cargo.lock"):
            text = git("show", f"{resume_tag}:{name}")
            pattern = r'^version = "([^"]+)"' if name.endswith("toml") else r'\[\[package\]\]\nname = "aetheris"\nversion = "([^"]+)"'
            match = re.search(pattern, text, re.M)
            if not match or match[1] != version:
                raise ValueError(f"Tagged {name} does not match {resume_tag}")
        notes = matches[0].render()
        # Build and sign exactly the tagged source, even when main moved on.
        subprocess.run(["git", "checkout", "--detach", resume_tag], check=True)
        return version, notes

    version = base
    if channel == "beta":
        tags = git("tag", "--list").splitlines()
        counters = [int(match[1]) for tag in tags if (match := re.fullmatch(rf"v{re.escape(base)}-beta\.([1-9][0-9]*)", tag))]
        version += f"-beta.{max(counters, default=0) + 1}"
    if subprocess.run(["git", "show-ref", "--verify", "--quiet", f"refs/tags/v{version}"]).returncode == 0:
        raise ValueError("Release tag already exists; use resume_tag to resume publication")
    # Validate all inputs before writing anything.
    histories = {name: parse(Path(path).read_text(encoding="utf-8"), name) for name, path in FILES.items()}
    if any(entry.label.split(" — ")[0] == version for entry in histories[channel].entries[1:]):
        raise ValueError("Version already frozen in the history")
    cargo_updates = {}
    for name, pattern, replacement in (
        ("Cargo.toml", r'^version = "[^"]+"$', f'version = "{version}"'),
        ("Cargo.lock", r'(\[\[package\]\]\nname = "aetheris"\nversion = ")[^"]+', rf'\g<1>{version}'),
    ):
        updated, count = re.subn(pattern, replacement, Path(name).read_text(encoding="utf-8"), count=1, flags=re.M)
        if count != 1:
            raise ValueError(f"Missing package version in {name}")
        cargo_updates[name] = updated
    histories[channel].freeze(version, timestamp())
    notes = histories[channel].entries[1].render()
    if channel == "stable":
        histories["beta"].entries[0] = pending()
        Path(FILES["beta"]).write_text(histories["beta"].render(), encoding="utf-8")
    Path(filename).write_text(histories[channel].render(), encoding="utf-8")
    for name, text in cargo_updates.items():
        Path(name).write_text(text, encoding="utf-8")
    return version, notes


def main():
    version, notes = prepare(os.environ["RELEASE_VERSION"], os.environ["RELEASE_CHANNEL"], os.environ.get("RELEASE_RESUME_TAG", ""))
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", prefix="aetheris-notes-", dir=os.environ.get("RUNNER_TEMP"), encoding="utf-8", delete=False) as output:
        output.write(notes)
        notes_path = output.name
    with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
        output.write(f"version={version}\nnotes_path={notes_path}\n")


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as cause:
        raise SystemExit(str(cause))
