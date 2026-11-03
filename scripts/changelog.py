#!/usr/bin/env python3
"""Parse, freeze and check the two player-facing release histories."""
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
import re
import subprocess
import sys

FILES = {"stable": "CHANGES.md", "beta": "CHANGES.beta.md"}
SECTIONS = ("Add", "Modify", "Fixes")
EMPTY = "Cette version ne comporte aucun changement visible pour le joueur."
VERSION = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)(?:-beta\.[1-9][0-9]*)?"


@dataclass
class Entry:
    label: str
    sections: dict[str, str]

    def has_changes(self):
        return any(self.sections.values())

    def render(self):
        empty = f"\n{EMPTY}\n" if self.label != "PENDING" and not self.has_changes() else ""
        return f"## {self.label}\n{empty}" + "".join(
            f"\n### {name}\n" + (f"\n{self.sections[name]}\n" if self.sections[name] else "")
            for name in SECTIONS
        )


@dataclass
class History:
    title: str
    entries: list[Entry]

    def render(self):
        return self.title + "\n\n" + "\n".join(entry.render() for entry in self.entries)

    def freeze(self, version, timestamp):
        self.entries[0].label = f"{version} — {timestamp}"
        self.entries.insert(0, pending())


def pending():
    return Entry("PENDING", dict.fromkeys(SECTIONS, ""))


def parse(text, channel):
    lines = text.splitlines()
    if not lines or not re.fullmatch(r"# .+", lines[0]):
        raise ValueError("Le fichier doit commencer par un titre de niveau 1.")
    starts = [i for i, line in enumerate(lines) if line.startswith("## ")]
    if not starts or any(line.strip() for line in lines[1:starts[0]]):
        raise ValueError("PENDING doit suivre le titre.")
    entries = []
    seen = set()
    for position, start in enumerate(starts):
        label = lines[start][3:]
        if position == 0:
            if label != "PENDING":
                raise ValueError("La première entrée doit être PENDING.")
        else:
            match = re.fullmatch(rf"({VERSION}) — (\d{{4}}-\d{{2}}-\d{{2}}T\d{{2}}:\d{{2}}:\d{{2}}Z)", label)
            if not match or ("-beta." in match[1]) != (channel == "beta"):
                raise ValueError(f"Version ou date invalide pour {channel} : {label}")
            datetime.strptime(match[2], "%Y-%m-%dT%H:%M:%SZ")
            if match[1] in seen:
                raise ValueError("Version dupliquée.")
            seen.add(match[1])
        end = starts[position + 1] if position + 1 < len(starts) else len(lines)
        body = lines[start + 1:end]
        headings = [i for i, line in enumerate(body) if line.startswith("#")]
        if [body[i] for i in headings] != [f"### {name}" for name in SECTIONS]:
            raise ValueError("Chaque entrée exige Add, Modify et Fixes, dans cet ordre.")
        intro = "\n".join(body[:headings[0]]).strip()
        if intro and (position == 0 or intro != EMPTY):
            raise ValueError("Texte inattendu avant les rubriques.")
        sections = {}
        for index, name in enumerate(SECTIONS):
            stop = headings[index + 1] if index + 1 < len(headings) else len(body)
            content = "\n".join(body[headings[index] + 1:stop]).strip()
            if content and not content.startswith("- "):
                raise ValueError("Les notes doivent être rédigées sous forme de liste.")
            sections[name] = content
        entry = Entry(label, sections)
        if intro and entry.has_changes():
            raise ValueError("La mention d'absence de changement contredit les notes.")
        if position and not entry.has_changes() and intro != EMPTY:
            raise ValueError("Une version vide exige la mention standard.")
        entries.append(entry)
    return History(lines[0], entries)


def git(*args):
    return subprocess.check_output(["git", *args], text=True, encoding="utf-8")


def exemptions(message):
    result = set()
    for line in message.splitlines():
        if line.strip().lower().startswith("changelog-exempt:"):
            match = re.fullmatch(r"Changelog-Exempt: (stable|beta) — (\S.*)", line.strip())
            if not match or not match[2].strip():
                raise ValueError("Exemption attendue : Changelog-Exempt: stable|beta — motif")
            result.add(match[1])
    return result


def check_commit(message):
    exempt = exemptions(message)
    has_head = subprocess.run(["git", "rev-parse", "--verify", "HEAD"], capture_output=True).returncode == 0
    for channel, filename in FILES.items():
        staged = parse(git("show", f":{filename}"), channel)
        if channel in exempt:
            continue
        previous = parse(git("show", f"HEAD:{filename}"), channel) if has_head and subprocess.run(
            ["git", "cat-file", "-e", f"HEAD:{filename}"], capture_output=True
        ).returncode == 0 else None
        if previous and staged.entries[0].sections == previous.entries[0].sections:
            raise ValueError(f"Mettre à jour le PENDING de {filename}, ou justifier une exemption {channel}.")
        if not previous and not staged.entries[0].has_changes():
            raise ValueError(f"Le premier PENDING de {filename} est vide ; justifier une exemption {channel}.")


def timestamp():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def main():
    try:
        if len(sys.argv) == 3 and sys.argv[1] == "check-commit":
            check_commit(Path(sys.argv[2]).read_text(encoding="utf-8"))
        elif len(sys.argv) == 2 and sys.argv[1] == "validate":
            for channel, filename in FILES.items():
                parse(Path(filename).read_text(encoding="utf-8"), channel)
        else:
            raise ValueError("Usage: changelog.py validate | check-commit <message-file>")
    except (ValueError, subprocess.CalledProcessError, OSError) as cause:
        print(f"Changelog : {cause}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
