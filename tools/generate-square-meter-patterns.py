#!/usr/bin/env python3
"""Generate the CLDR extension from a pinned Unicode archive, without network."""
import hashlib
import json
from pathlib import Path
import sys
import tarfile

SHA256 = "702cec5d8caa9c1c323188eab426262003f45d1a58fc7ad073be4c7128ee8137"
archive = Path(sys.argv[1])
if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
    raise ValueError("Expected cldr-units-full-48.0.0.tgz")
output = Path(__file__).resolve().parents[1] / "src/internationalization"
patterns = []
locales = {}
with tarfile.open(archive) as source:
    for member in source:
        if not member.name.endswith("/units.json"):
            continue
        tag = member.name.split("/")[-2]
        units = json.load(source.extractfile(member))["main"][tag]["units"]
        widths = []
        for width in ("long", "short", "narrow"):
            area = units[width]["area-square-meter"]
            pattern = area.get("perUnitPattern")
            if pattern is None:
                name = area.get("unitPattern-count-one", area["unitPattern-count-other"])
                pattern = units[width]["per"]["compoundUnitPattern"].replace(
                    "{1}", name.replace("{0}", "").strip()
                )
            if pattern.count("{0}") != 1:
                raise ValueError(f"Invalid pattern: {tag} {width}")
            widths.append(pattern)
        if widths not in patterns:
            patterns.append(widths)
        locales[tag.lower()] = patterns.index(widths)
    (output / "UNICODE-LICENSE.txt").write_bytes(source.extractfile("package/LICENSE").read())
(output / "square_meter.json").write_text(
    json.dumps({"patterns": patterns, "locales": dict(sorted(locales.items()))},
               ensure_ascii=False, separators=(",", ":")) + "\n"
)
