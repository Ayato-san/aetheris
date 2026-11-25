#!/usr/bin/env python3
"""Prepare an isolated CUE4Parse build and read-only catalogue icon requests."""
import argparse
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess
import xml.etree.ElementTree as ET

CUE4PARSE_REVISION = "4d2301a8eb129f593f1c76ff82612400801f1a94"


def prepare(source: Path, database: Path, work: Path) -> None:
    revision = subprocess.check_output(
        ["git", "-C", str(source), "rev-parse", "HEAD"], text=True
    ).strip()
    if revision != CUE4PARSE_REVISION:
        raise ValueError(f"Expected CUE4Parse revision {CUE4PARSE_REVISION}")
    subprocess.run([
        "git", "-C", str(source), "diff", "--exit-code", "HEAD", "--",
        "CUE4Parse", "CUE4Parse-Conversion", "Directory.Build.props", "Directory.Packages.props",
    ], check=True, stdout=subprocess.DEVNULL)
    destination = work / "CUE4Parse"
    if source == destination or source in destination.parents or destination in source.parents:
        raise ValueError("Use a work directory outside the CUE4Parse checkout")
    for name in ("CUE4Parse", "CUE4Parse-Conversion"):
        shutil.copytree(
            source / name, destination / name, dirs_exist_ok=True,
            ignore=shutil.ignore_patterns("bin", "obj"),
        )
    for name in ("Directory.Build.props", "Directory.Packages.props", "LICENSE", "NOTICE"):
        shutil.copy2(source / name, destination / name)
    # Pin patched dependencies in the isolated copy; never alter the source checkout.
    packages = destination / "Directory.Packages.props"
    tree = ET.parse(packages)
    group = ET.SubElement(tree.getroot(), "ItemGroup")
    ET.SubElement(group, "PackageVersion", Include="Microsoft.Bcl.Memory", Version="10.0.15")
    tree.write(packages, encoding="unicode")
    props = destination / "Directory.Build.props"
    tree = ET.parse(props)
    group = ET.SubElement(tree.getroot(), "PropertyGroup")
    ET.SubElement(group, "CentralPackageTransitivePinningEnabled").text = "true"
    tree.write(props, encoding="unicode")
    with sqlite3.connect(database.as_uri() + "?mode=ro", uri=True) as db:
        db.execute("PRAGMA query_only=ON")
        if db.execute("PRAGMA application_id").fetchone()[0] != 0x41454954:
            raise ValueError("Expected an Aetheris catalogue")
        references = [row[0] for row in db.execute(
            "SELECT reference FROM catalogue_icons ORDER BY reference LIMIT 100001"
        )]
        build = db.execute(
            "SELECT v.steam_build FROM catalogue_manifest m "
            "JOIN client_version v ON v.id=m.build_id WHERE m.id=1"
        ).fetchone()[0]
    if len(references) > 100000:
        raise ValueError("Too many icon references")
    (work / "references.json").write_text(json.dumps(references, ensure_ascii=False), encoding="utf-8")
    (work / "preparation.json").write_text(json.dumps({
        "steam_build": build, "references": len(references),
        "cue4parse_revision": revision,
        "dependency_overrides": {"Microsoft.Bcl.Memory": "10.0.15"},
    }, indent=2), encoding="utf-8")
    print(f"Prepared {len(references)} icon references for build {build} in {work}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cue4parse", required=True, type=Path)
    parser.add_argument("--database", required=True, type=Path)
    parser.add_argument("--work", type=Path, default=Path("target/game-icons"))
    args = parser.parse_args()
    prepare(args.cue4parse.resolve(), args.database.resolve(), args.work.resolve())
