#!/usr/bin/env python3
"""Maintainer inventory for #75; uses the Rust probe, never edits data.sqlite.

Raw probes and reports stay in a local output directory. A filename or a
successful structural decode is evidence to investigate, not a business model.
"""
import argparse
from collections import defaultdict
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read_json(path):
    if path.stat().st_size > 512 * 1024 * 1024:
        raise ValueError("JSON exceeds the inventory's 512 MiB input limit")
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=path.parent, encoding="utf-8", delete=False) as stream:
        temporary = Path(stream.name)
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    temporary.replace(path)


def values(value, path):
    """Traverse explicit fields/arrays; do not infer joins from arbitrary numbers."""
    if isinstance(value, list):
        for entry in value:
            yield from values(entry, path)
    elif path:
        if isinstance(value, dict) and path[0] in value:
            yield from values(value[path[0]], path[1:])
    else:
        if isinstance(value, dict):
            if set(value) == {"Value"}:
                yield from values(value["Value"], [])
        elif not isinstance(value, bool) and value not in (None, "", "None", 0):
            yield value


def identity(value):
    # Preserve type: an item technical name "123" is not source ID 123.
    return type(value).__name__, str(value)


def leaves(value, path=""):
    if isinstance(value, dict):
        for key, child in sorted(value.items()):
            yield from leaves(child, f"{path}.{key}" if path else key)
    elif isinstance(value, list):
        for child in value:
            yield from leaves(child, path + "[]")
    else:
        yield path, value


def translated(entries, key):
    return entries.get(key, entries.get(f"String_{key}_body"))


def text_references(value):
    if isinstance(value, dict):
        # Unreal map entries also have a Key, but are not localized FText.
        key = value.get("Key") if "Value" not in value else None
        if isinstance(key, str) and key not in ("", "None"):
            yield key
        for child in value.values():
            yield from text_references(child)
    elif isinstance(value, list):
        for child in value:
            yield from text_references(child)


def summarize(probe, english, localized):
    fields = set()
    text_keys = set()
    resources = defaultdict(set)
    examples = []
    for row in probe["rows"]:
        fields.update(row)
        row_texts = []
        for key in text_references(row):
            text_keys.add(key)
            en, loc = translated(english, key), translated(localized, key)
            if len(row_texts) < 3:
                row_texts.append({"key": key, "en": en[:240] if en else None,
                                  "localized": loc[:240] if loc else None,
                                  "sample_truncated": bool((en and len(en) > 240) or (loc and len(loc) > 240))})
        for path, value in leaves(row):
            field = path.rsplit(".", 1)[-1].lower()
            if isinstance(value, str) and value not in ("", "None") and any(t in field for t in ("icon", "image", "portrait", "texture", "resource", "assetpath", "costume")):
                resources[path].add(value)
        if len(examples) < 3 and (row_texts or not examples):
            examples.append({"identity_candidates": {key: row[key] for key in ("ID", "Name", "Group", "AchievementId", "DataId", "ItemId") if key in row}, "texts": row_texts})
    return {"rows": len(probe["rows"]), "columns": len(probe["column_hashes"]),
            "table_version": probe["table_version"], "fields": sorted(fields),
            "text_keys": len(text_keys),
            "english_available": sum(translated(english, key) is not None for key in text_keys),
            "localized_available": sum(translated(localized, key) is not None for key in text_keys),
            "examples": examples,
            "resource_candidates": {path: {"references": len(refs), "examples": sorted(refs)[:3], "state": "uninterpreted"} for path, refs in sorted(resources.items())}}


def probe_table(executable, mapping, source, structure, output, expected_mapping):
    expected_payload = sha256(source)
    command = [str(executable), "probe-table", str(mapping), structure, str(source), str(output)]
    result = subprocess.run(command, text=True, capture_output=True, check=False)
    if result.returncode:
        raise ValueError(result.stderr.strip() or f"probe exited {result.returncode}")
    probe = read_json(output)
    if (probe["mapping_sha256"], probe["payload_sha256"], probe["row_struct"]) != (expected_mapping, expected_payload, structure):
        raise ValueError("source or mapping changed while probing")
    return probe


def check_relations(rules, probes):
    indexes = {}
    results = []
    grouped = defaultdict(list)
    for rule in rules:
        grouped[rule["source"]].append(rule)
    for source, source_rules in sorted(grouped.items()):
        source_rows = read_json(probes[source])["rows"] if source in probes else None
        for rule in source_rules:
            result = dict(rule)
            target = rule["target"]
            if source_rows is None or target not in probes:
                result.update(state="not_researched", diagnostic="source or target not decoded")
                results.append(result)
                continue
            index_key = target, rule["target_field"], tuple(sorted(rule.get("target_where", {}).items()))
            if index_key not in indexes:
                target_rows = read_json(probes[target])["rows"]
                index = defaultdict(int)
                for row in target_rows:
                    if any(row.get(k) != v for k, v in rule.get("target_where", {}).items()):
                        continue
                    for value in values(row, rule["target_field"].split(".")):
                        index[identity(value)] += 1
                indexes[index_key] = index
            index = indexes[index_key]
            refs = resolved = ambiguous = 0
            missing = []
            examples = []
            for position, row in enumerate(source_rows):
                if any(row.get(k) != v for k, v in rule.get("where", {}).items()):
                    continue
                for value in values(row, rule["field"].split(".")):
                    if rule.get("encoding") == "decimal_id" and isinstance(value, str) and value.isdecimal():
                        value = int(value)
                    refs += 1
                    count = index.get(identity(value), 0)
                    resolved += count > 0
                    ambiguous += count > 1 and not rule.get("group", False)
                    sample = {"source_row": position, "value": value, "target_matches": count}
                    if len(examples) < 3:
                        examples.append(sample)
                    if count == 0 and len(missing) < 5:
                        missing.append(sample)
            result.update(state="uninterpreted" if rule.get("candidate") and refs else "verified" if refs and resolved == refs and not ambiguous else "partial" if refs else "not_found",
                          references=refs, resolved=resolved, missing=refs-resolved, ambiguous=ambiguous,
                          examples=examples, missing_examples=missing,
                          interpretation="exact field/key matching only; no inferred units, live availability or network IDs")
            results.append(result)
    return results


def resource_inventory(root, map_ids=None):
    result = {}
    for family, relative, parse_json in [("map_events", "MapEvent", True), ("map_hierarchy", "MapDataHierarchy", True), ("map_binary", "Map", False)]:
        directory = root / relative
        files = sorted(directory.rglob("*.bin")) if directory.is_dir() else []
        entry = {"path": relative, "files": len(files), "state": "not_found" if not files else "uninterpreted", "examples": []}
        decoded = failed = 0
        map_references = map_resolved = 0
        missing_maps = []
        for path in files:
            sample = {"file": path.relative_to(directory).as_posix(), "bytes": path.stat().st_size, "sha256": sha256(path)}
            if parse_json:
                try:
                    data = read_json(path)
                    sample["root_type"] = type(data).__name__
                    sample["root_fields"] = sorted(data)[:30] if isinstance(data, dict) else []
                    decoded += 1
                    if isinstance(data, dict) and data.get("MapID"):
                        map_references += 1
                        matches = map_ids is not None and identity(data["MapID"]) in map_ids
                        map_resolved += matches
                        if map_ids is not None and not matches and len(missing_maps) < 5:
                            missing_maps.append({"file": sample["file"], "map_id": data["MapID"]})
                except (ValueError, UnicodeError) as error:
                    failed += 1
                    sample["error"] = str(error)[:240]
            if len(entry["examples"]) < 3:
                entry["examples"].append(sample)
            if sample["sha256"] != sha256(path):
                raise ValueError(f"resource changed while probing: {path.name}")
        if files != (sorted(directory.rglob("*.bin")) if directory.is_dir() else []):
            raise ValueError(f"resource inventory changed: {relative}")
        entry.update(json_decoded=decoded, failed=failed)
        if map_references:
            entry["map_links"] = {"state": "not_researched" if map_ids is None else "verified" if map_references == map_resolved else "partial",
                                  "references": map_references, "resolved": map_resolved, "missing_examples": missing_maps}
        result[family] = entry
    result["unreal_images"] = {"state": "not_researched", "diagnostic": "resource fields are candidates; this inventory does not mount Unreal packages or assume matching texture filenames"}
    result["worldmap_and_navsignature"] = {"state": "not_researched", "diagnostic": "outside decrypted table scope; see extraction/decryption errors for unsupported encrypted families"}
    return result


def inventory(args):
    profile = read_json(args.profile)
    mapping_hash = sha256(args.mapping)
    english = read_json(args.english)["entries"]
    localized = read_json(args.localized)["entries"]
    manifest = subprocess.run([str(args.executable), "inspect-steam-manifest", str(args.manifest)], text=True, capture_output=True, check=True)
    report = {"profile_sha256": sha256(args.profile), "mapping_sha256": mapping_hash,
              "manifest": json.loads(manifest.stdout), "region": {"state": "unknown"},
              "localizations": {"english_sha256": sha256(args.english), "localized_sha256": sha256(args.localized)},
              "compatibility_verified": False, "tables": {}, "families": {}, "relations": []}
    probes = {}
    paths = sorted(args.tables.glob("*.bin"))
    if not paths:
        raise ValueError("no root table payloads found")
    inputs = {path: sha256(path) for path in [*paths, args.mapping, args.english, args.localized, args.manifest, args.profile]}
    for path in paths:
        name = path.stem
        structure = profile["aliases"].get(name, name + "TableItem")
        entry = {"path": path.name, "bytes": path.stat().st_size, "sha256": inputs[path], "candidate_structure": structure,
                 "families": [key for key, family in profile["families"].items() if name in family.get("tables", []) or any(name.startswith(p) for p in family.get("prefixes", []))]}
        try:
            output = args.output / "probes" / (name + ".json")
            output.parent.mkdir(parents=True, exist_ok=True)
            probe = probe_table(args.executable, args.mapping, path, structure, output, mapping_hash)
            entry.update(state="uninterpreted", structural_state="decoded", **summarize(probe, english, localized))
            probes[name] = output
        except (ValueError, OSError) as error:
            entry.update(state="failed", structural_state="failed", diagnostic=str(error))
        report["tables"][name] = entry
    report["relations"] = check_relations(profile.get("relations", []), probes)
    report["classification_labels"] = {}
    if "EnumString" in probes:
        for row in read_json(probes["EnumString"])["rows"]:
            if row.get("Enum") in profile.get("classification_enums", []):
                key = row.get("String", {}).get("Key")
                report["classification_labels"].setdefault(row["Enum"], []).append({
                    "code": row.get("Value"), "key": key,
                    "en": translated(english, key), "localized": translated(localized, key)})
    for key, family in profile["families"].items():
        names = sorted(name for name, table in report["tables"].items() if key in table["families"])
        report["families"][key] = {**family, "present": names,
            "decoded": sum(name in probes for name in names),
            "missing_candidates": [name for name in family.get("expected", []) if name not in report["tables"]],
            "semantic_state": "uninterpreted", "production_projection": family.get("production_projection", "not_researched")}
    map_ids = {identity(v) for row in read_json(probes["Map"])["rows"] for v in values(row, ["ID"])} if "Map" in probes else None
    report["resources"] = resource_inventory(args.resources, map_ids) if args.resources else {"state": "not_researched"}
    report["text_links"] = []
    for rule in profile.get("text_links", []):
        if rule["source"] not in probes:
            report["text_links"].append({**rule, "state": "not_researched"})
            continue
        refs = en_available = loc_available = 0
        missing = []
        for row in read_json(probes[rule["source"]])["rows"]:
            if any(row.get(k) != v for k, v in rule.get("where", {}).items()):
                continue
            for value in values(row, rule["field"].split(".")):
                key = rule["prefix"] + str(value) + rule.get("suffix", "")
                refs += 1
                en, loc = translated(english, key), translated(localized, key)
                en_available += en is not None
                loc_available += loc is not None
                if (en is None or loc is None) and len(missing) < 5:
                    missing.append(key)
        report["text_links"].append({**rule, "state": "verified" if refs and refs == en_available == loc_available else "partial" if refs else "not_found",
                                     "references": refs, "english_available": en_available, "localized_available": loc_available, "missing_examples": missing,
                                     "interpretation": "declared text-key construction only; no unit or value formula inferred"})
    if args.decryption_report:
        inputs[args.decryption_report] = sha256(args.decryption_report)
        decrypted = read_json(args.decryption_report)
        failures = [error for error in decrypted.get("errors", []) if "WorldMap" in error or "NavSignature" in error]
        report["resources"]["worldmap_and_navsignature"] = {"state": "failed" if failures else "not_found", "sha256": inputs[args.decryption_report], "errors": len(failures), "examples": failures[:4]}
    if args.sprites:
        inputs[args.sprites] = sha256(args.sprites)
        sprites = read_json(args.sprites)
        matches = sprites.get("mapping_sha256") == mapping_hash
        report["resources"]["ui_sprites"] = {"state": "failed" if not matches else "partial" if sprites.get("errors") else "verified", "sha256": inputs[args.sprites], "mapping_matches": matches, "sprites": len(sprites.get("sprites", [])), "aliases": len(sprites.get("aliases", [])), "errors": sprites.get("errors", [])[:5], "interpretation": "existing extraction report; not proof that every family-specific icon is resolved"}
    if mapping_hash != sha256(args.mapping) or paths != sorted(args.tables.glob("*.bin")) or any(sha256(path) != digest for path, digest in inputs.items()):
        raise ValueError("inventory inputs changed; previous report preserved")
    write_json(args.output / "inventory.json", report)
    print(json.dumps({"tables": len(paths), "decoded": len(probes), "failed": len(paths)-len(probes),
                      "families": len(report["families"]), "relations": len(report["relations"]),
                      "report": str(args.output / "inventory.json")}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("executable", "mapping", "tables", "english", "localized", "manifest", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--profile", type=Path, default=Path(__file__).with_name("catalogue-sources.json"))
    parser.add_argument("--resources", type=Path)
    parser.add_argument("--decryption-report", type=Path)
    parser.add_argument("--sprites", type=Path)
    args = parser.parse_args()
    args.executable = args.executable.resolve(strict=True)
    args.output = args.output.resolve()
    # This tool owns its report and raw probe directory, never the source files.
    for path in filter(None, (args.mapping, args.tables, args.english, args.localized, args.manifest, args.profile, args.resources, args.decryption_report, args.sprites)):
        if path.resolve() == args.output or args.output in path.resolve().parents:
            raise ValueError("output directory must not contain inventory inputs")
    inventory(args)


if __name__ == "__main__":
    main()
