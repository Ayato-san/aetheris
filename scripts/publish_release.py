#!/usr/bin/env python3
"""Idempotently upload a complete ZIP to a draft, then make it public."""
import json
import os
from pathlib import Path
import subprocess


def publish(repository, version, channel, notes_path, archive):
    tag = f"v{version}"
    result = subprocess.run(
        ["gh", "api", f"repos/{repository}/releases/tags/{tag}"],
        capture_output=True, text=True, encoding="utf-8",
    )
    try:
        release = json.loads(result.stdout)
    except ValueError:
        raise ValueError("Cannot determine whether the GitHub release exists") from None
    if result.returncode:
        if str(release.get("status")) != "404":
            raise ValueError("Cannot query the GitHub release: " + result.stderr)
        flags = ["--prerelease"] if channel == "beta" else []
        subprocess.run(["gh", "release", "create", tag, "--repo", repository, "--draft", "--verify-tag", "--title", tag, "--notes-file", notes_path, *flags], check=True)
    elif not release["draft"]:
        # A successful publication is already complete. Never replace public assets.
        if (release.get("body") or "").strip() != Path(notes_path).read_text(encoding="utf-8").strip():
            raise ValueError("Published notes differ from the tagged notes")
        if not any(asset["name"] == Path(archive).name for asset in release.get("assets", [])):
            raise ValueError("Published release is missing its ZIP; repair it explicitly")
        return
    subprocess.run(["gh", "release", "upload", tag, archive, "--repo", repository, "--clobber"], check=True)
    subprocess.run(["gh", "release", "edit", tag, "--repo", repository, "--notes-file", notes_path, "--draft=false"], check=True)


if __name__ == "__main__":
    try:
        publish(os.environ["AETHERIS_REPOSITORY"], os.environ["RELEASE_VERSION"], os.environ["RELEASE_CHANNEL"], os.environ["RELEASE_NOTES"], "aetheris-windows-x86_64.zip")
    except (ValueError, OSError, subprocess.CalledProcessError) as cause:
        raise SystemExit(str(cause))
