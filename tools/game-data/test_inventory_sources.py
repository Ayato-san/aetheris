import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import inventory_sources as inventory


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

    def probe(self, name, rows):
        path = self.root / (name + ".json")
        path.write_text(json.dumps({"rows": rows}))
        return path

    def test_reference_checks_keep_types_and_report_missing_or_ambiguous_values(self):
        probes = {"Source": self.probe("Source", [{"Targets": [{"Value": 12}, {"Value": 99}, {"Value": 0}, "12"]}]),
                  "Target": self.probe("Target", [{"ID": {"Value": 12}}, {"ID": {"Value": 12}}])}
        rule = {"source": "Source", "field": "Targets", "target": "Target", "target_field": "ID"}
        result = inventory.check_relations([rule], probes)[0]
        self.assertEqual((result["references"], result["resolved"], result["missing"], result["ambiguous"]), (3, 1, 2, 1))
        self.assertEqual(result["state"], "partial")

    def test_group_relations_allow_variants_without_promoting_a_hypothesis(self):
        probes = {"Source": self.probe("Source", [{"Type": "Skill", "Value": "12"}, {"Type": "Stat", "Value": "99"}]),
                  "Target": self.probe("Target", [{"ID": 12, "Class": "A"}, {"ID": 12, "Class": "B"}])}
        rule = {"source": "Source", "field": "Value", "target": "Target", "target_field": "ID", "encoding": "decimal_id",
                "where": {"Type": "Skill"}, "target_where": {"Class": "A"}, "candidate": True}
        result = inventory.check_relations([rule], probes)[0]
        self.assertEqual((result["references"], result["resolved"], result["ambiguous"]), (1, 1, 0))
        self.assertEqual(result["state"], "uninterpreted")
        rule.pop("target_where")
        rule["group"] = True
        result = inventory.check_relations([rule], probes)[0]
        self.assertEqual(result["ambiguous"], 0)

    def test_absent_tables_and_empty_references_do_not_become_verified_links(self):
        rule = {"source": "Source", "field": "Target", "target": "Target", "target_field": "ID"}
        probes = {"Source": self.probe("Source", [{"Target": {"Value": 0}}])}
        self.assertEqual(inventory.check_relations([rule], probes)[0]["state"], "not_researched")
        probes["Target"] = self.probe("Target", [{"ID": 1}])
        self.assertEqual(inventory.check_relations([rule], probes)[0]["state"], "not_found")

    def test_map_keys_are_not_localization_keys_and_no_translation_is_invented(self):
        probe = {"rows": [{"ID": 1, "Desc": {"Key": "name"}, "Stats": {"Entries": [{"Key": "EStat::HP", "Value": 10}]}}],
                 "column_hashes": [], "table_version": 13}
        result = inventory.summarize(probe, {"name": "Name"}, {})
        self.assertEqual((result["text_keys"], result["english_available"], result["localized_available"]), (1, 1, 0))
        self.assertIsNone(result["examples"][0]["texts"][0]["localized"])

    def test_wrong_probe_provenance_is_rejected_even_with_an_existing_output(self):
        mapping = self.root / "mapping"
        source = self.root / "Source.bin"
        mapping.write_bytes(b"mapping")
        source.write_bytes(b"source")
        output = self.probe("output", [])
        def run(*args, **kwargs):
            output.write_text(json.dumps({"mapping_sha256": "wrong", "payload_sha256": inventory.sha256(source), "row_struct": "Row"}))
            return SimpleNamespace(returncode=0)
        with patch.object(inventory.subprocess, "run", side_effect=run):
            with self.assertRaisesRegex(ValueError, "source or mapping changed"):
                inventory.probe_table(Path("probe"), mapping, source, "Row", output, inventory.sha256(mapping))

    def test_source_mutation_preserves_the_previous_inventory_report(self):
        tables = self.root / "tables"
        tables.mkdir()
        source = tables / "Source.bin"
        source.write_bytes(b"initial")
        for name, value in [("profile", {"aliases": {}, "families": {}, "relations": []}), ("en", {"entries": {}}), ("loc", {"entries": {}})]:
            (self.root / name).write_text(json.dumps(value))
        for name in ("mapping", "manifest"):
            (self.root / name).write_bytes(b"initial")
        output = self.root / "output"
        output.mkdir()
        previous = output / "inventory.json"
        previous.write_text('{"previous":true}')
        args = SimpleNamespace(profile=self.root / "profile", mapping=self.root / "mapping", english=self.root / "en",
                               localized=self.root / "loc", manifest=self.root / "manifest", executable=Path("probe"), tables=tables,
                               output=output, resources=None, decryption_report=None, sprites=None)
        def probe(*args):
            source.write_bytes(b"changed")
            return {"rows": [], "column_hashes": [], "table_version": 13}
        with patch.object(inventory.subprocess, "run", return_value=SimpleNamespace(stdout="{}")), patch.object(inventory, "probe_table", side_effect=probe):
            with self.assertRaisesRegex(ValueError, "inputs changed"):
                inventory.inventory(args)
        self.assertEqual(previous.read_text(), '{"previous":true}')

    def test_nested_map_payloads_are_found_without_claiming_to_decode_them(self):
        directory = self.root / "Map" / "Dungeon" / "Example"
        directory.mkdir(parents=True)
        (directory / "MapData.bin").write_bytes(b"unknown binary")
        result = inventory.resource_inventory(self.root)["map_binary"]
        self.assertEqual((result["files"], result["state"], result["json_decoded"]), (1, "uninterpreted", 0))
        self.assertEqual(result["examples"][0]["file"], "Dungeon/Example/MapData.bin")

    def test_map_events_report_missing_map_ids_without_inventing_positions(self):
        directory = self.root / "MapEvent"
        directory.mkdir()
        for name, value in [("known", 12), ("missing", 99)]:
            (directory / (name + ".bin")).write_text(json.dumps({"MapID": value, "Stages": []}))
        result = inventory.resource_inventory(self.root, {inventory.identity(12)})["map_events"]
        self.assertEqual(result["json_decoded"], 2)
        self.assertEqual(result["state"], "uninterpreted")
        self.assertEqual(result["map_links"]["state"], "partial")
        self.assertEqual((result["map_links"]["references"], result["map_links"]["resolved"]), (2, 1))
        self.assertEqual(result["map_links"]["missing_examples"][0]["map_id"], 99)


if __name__ == "__main__":
    unittest.main()
