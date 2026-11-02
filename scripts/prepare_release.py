#!/usr/bin/env python3
"""Resolve a serialized beta counter and update both Cargo version records."""
import os
from pathlib import Path
import re
import subprocess

base = os.environ["RELEASE_VERSION"]
channel = os.environ["RELEASE_CHANNEL"]
if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", base):
    raise SystemExit("Expected major.minor.patch without leading zeroes")
version = base
if channel == "beta":
    tags = subprocess.check_output(["git", "tag", "--list"], text=True).splitlines()
    counters = [int(match[1]) for tag in tags if (match := re.fullmatch(rf"v{re.escape(base)}-beta\.([1-9][0-9]*)", tag))]
    version += f"-beta.{max(counters, default=0) + 1}"
elif channel != "stable":
    raise SystemExit("Unknown release channel")
if subprocess.run(["git", "show-ref", "--verify", "--quiet", f"refs/tags/v{version}"]).returncode == 0:
    raise SystemExit("Release tag already exists")
manifest = Path("Cargo.toml")
manifest.write_text(re.sub(r'^version = "[^"]+"$', f'version = "{version}"', manifest.read_text(), count=1, flags=re.M))
lock = Path("Cargo.lock")
lock.write_text(re.sub(r'(\[\[package\]\]\nname = "aetheris"\nversion = ")[^"]+', rf'\g<1>{version}', lock.read_text(), count=1))
with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
    output.write(f"version={version}\n")
