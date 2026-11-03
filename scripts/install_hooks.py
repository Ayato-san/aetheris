#!/usr/bin/env python3
"""Opt in to the versioned local commit hook without replacing other hooks."""
import subprocess

current = subprocess.run(
    ["git", "config", "--get", "core.hooksPath"], capture_output=True, text=True
).stdout.strip()
if current and current != ".githooks":
    raise SystemExit(f"core.hooksPath utilise déjà {current!r}. Intégrer le contrôle manuellement.")
subprocess.run(["git", "config", "core.hooksPath", ".githooks"], check=True)
print("Hook commit-msg installé (Python 3 requis).")
